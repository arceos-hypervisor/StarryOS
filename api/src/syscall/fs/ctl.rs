use alloc::{ffi::CString, vec, vec::Vec};
use core::{
    ffi::{c_char, c_int},
    mem::offset_of,
    time::Duration,
};

use axerrno::{LinuxError, LinuxResult};
use axfs_ng::{FS_CONTEXT, FsContext};
use axfs_ng_vfs::{DeviceId, MetadataUpdate, NodePermission, NodeType, path::Path};
use axhal::time::wall_time;
use axtask::current;
use linux_raw_sys::{
    general::*,
    ioctl::{FIONBIO, TIOCGWINSZ},
};
use starry_core::{task::AsThread, vfs::Device as VfsDevice};
use starry_vm::{VmPtr, vm_write_slice};

use crate::{
    file::{Directory, FileLike, get_file_like, resolve_at, with_fs},
    mm::vm_load_string,
    time::TimeValueLike,
};

/// The ioctl() system call manipulates the underlying device parameters
/// of special files.
pub fn sys_ioctl(fd: i32, cmd: u32, arg: usize) -> LinuxResult<isize> {
    debug!("sys_ioctl <= fd: {}, cmd: {}, arg: {}", fd, cmd, arg);
    let f = get_file_like(fd)?;
    if cmd == FIONBIO {
        let val = (arg as *const u8).vm_read()?;
        if val != 0 && val != 1 {
            return Err(LinuxError::EINVAL);
        }
        f.set_nonblocking(val != 0)?;
        return Ok(0);
    }
    f.ioctl(cmd, arg)
        .map(|result| result as isize)
        .inspect_err(|err| {
            if *err == LinuxError::ENOTTY {
                // glibc likes to call TIOCGWINSZ on non-terminal files, just
                // ignore it
                if cmd == TIOCGWINSZ {
                    return;
                }
                warn!("Unsupported ioctl command: {cmd} for fd: {fd}");
            }
        })
}

pub fn sys_chdir(path: *const c_char) -> LinuxResult<isize> {
    let path = vm_load_string(path)?;
    debug!("sys_chdir <= path: {}", path);

    let mut fs = FS_CONTEXT.lock();
    let entry = fs.resolve(path)?;
    fs.set_current_dir(entry)?;
    Ok(0)
}

pub fn sys_fchdir(dirfd: i32) -> LinuxResult<isize> {
    debug!("sys_fchdir <= dirfd: {}", dirfd);

    let entry = with_fs(dirfd, |fs| Ok(fs.current_dir().clone()))?;
    FS_CONTEXT.lock().set_current_dir(entry)?;
    Ok(0)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_mkdir(path: *const c_char, mode: u32) -> LinuxResult<isize> {
    sys_mkdirat(AT_FDCWD, path, mode)
}

pub fn sys_chroot(path: *const c_char) -> LinuxResult<isize> {
    let path = vm_load_string(path)?;
    debug!("sys_chroot <= path: {}", path);

    let mut fs = FS_CONTEXT.lock();
    let loc = fs.resolve(path)?;
    if loc.node_type() != NodeType::Directory {
        return Err(LinuxError::ENOTDIR);
    }
    *fs = FsContext::new(loc);
    Ok(0)
}

pub fn sys_mkdirat(dirfd: i32, path: *const c_char, mode: u32) -> LinuxResult<isize> {
    let path = vm_load_string(path)?;
    debug!(
        "sys_mkdirat <= dirfd: {}, path: {}, mode: {}",
        dirfd, path, mode
    );

    let mode = mode & !current().as_thread().proc_data.umask();
    let mode = NodePermission::from_bits_truncate(mode as u16);

    with_fs(dirfd, |fs| {
        fs.create_dir(path, mode)?;
        Ok(0)
    })
}

pub fn sys_mknodat(dirfd: i32, path: *const c_char, mode: u32, dev: u64) -> LinuxResult<isize> {
    let path = vm_load_string(path)?;
    debug!(
        "sys_mknodat <= dirfd: {}, path: {:?}, mode: {}, dev: {}",
        dirfd, path, mode, dev
    );

    // Split type and permission bits
    let ftype = mode & (S_IFMT as u32);
    let mut perm = (mode & !(S_IFMT as u32)) as u32;
    // apply umask like mkdir
    perm &= !current().as_thread().proc_data.umask();

    let node_type = match ftype {
        S_IFREG => NodeType::RegularFile,
        S_IFCHR => NodeType::CharacterDevice,
        S_IFBLK => NodeType::BlockDevice,
        S_IFIFO => NodeType::Fifo,
        S_IFSOCK => NodeType::Socket,
        _ => return Err(LinuxError::EINVAL),
    };

    with_fs(dirfd, |fs| {
        let (dir, name) = fs.resolve_nonexistent(Path::new(&path))?;
        let loc = dir.create(
            name,
            node_type,
            NodePermission::from_bits_truncate(perm as u16),
        )?;

        // If device node, set rdev
        if matches!(node_type, NodeType::CharacterDevice | NodeType::BlockDevice) {
            // Simple major/minor split: major in high bits, minor in low bits.
            let major = (dev >> 8) as u32;
            let minor = (dev & 0xff) as u32;
            // Try to set device id by downcasting the created entry to a Device
            // (this works for in-kernel SimpleFs device nodes).
            if let Ok(dev_node) = loc.entry().downcast::<VfsDevice>() {
                dev_node.set_device_id(DeviceId::new(major, minor));
            } else {
                // If downcast fails, we can't set rdev through MetadataUpdate
                // (not supported), so just ignore and continue.
                warn!("not a device node, cannot set rdev");
            }
        }

        Ok(0)
    })
}

// Directory buffer for getdents64 syscall
struct DirBuffer {
    buf: Vec<u8>,
    offset: usize,
}

impl DirBuffer {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0; len],
            offset: 0,
        }
    }

    fn remaining_space(&self) -> usize {
        self.buf.len().saturating_sub(self.offset)
    }

    fn write_entry(&mut self, d_ino: u64, d_off: i64, d_type: NodeType, name: &[u8]) -> bool {
        const NAME_OFFSET: usize = offset_of!(linux_dirent64, d_name);

        let len = NAME_OFFSET + name.len() + 1;
        // alignment
        let len = len.next_multiple_of(align_of::<linux_dirent64>());
        if self.remaining_space() < len {
            return false;
        }

        // FIXME: safety
        unsafe {
            let entry_ptr = self.buf.as_mut_ptr().add(self.offset);
            entry_ptr.cast::<linux_dirent64>().write(linux_dirent64 {
                d_ino,
                d_off,
                d_reclen: len as _,
                d_type: d_type as _,
                d_name: Default::default(),
            });

            let name_ptr = entry_ptr.add(NAME_OFFSET);
            name_ptr.copy_from_nonoverlapping(name.as_ptr(), name.len());
            name_ptr.add(name.len()).write(0);
        }

        self.offset += len;
        true
    }
}

pub fn sys_getdents64(fd: i32, buf: *mut u8, len: usize) -> LinuxResult<isize> {
    debug!("sys_getdents64 <= fd: {}, buf: {:?}, len: {}", fd, buf, len);

    let mut buffer = DirBuffer::new(len);

    let dir = Directory::from_fd(fd)?;
    let mut dir_offset = dir.offset.lock();

    let mut has_remaining = false;

    dir.inner()
        .read_dir(*dir_offset, &mut |name: &str, ino, node_type, offset| {
            has_remaining = true;
            if !buffer.write_entry(ino, offset as _, node_type, name.as_bytes()) {
                return false;
            }
            *dir_offset = offset;
            true
        })?;

    if has_remaining && buffer.offset == 0 {
        return Err(LinuxError::EINVAL);
    }

    vm_write_slice(buf, &buffer.buf)?;

    Ok(buffer.offset as _)
}

/// create a link from new_path to old_path
/// old_path: old file path
/// new_path: new file path
/// flags: link flags
/// return value: return 0 when success, else return -1.
pub fn sys_linkat(
    old_dirfd: c_int,
    old_path: *const c_char,
    new_dirfd: c_int,
    new_path: *const c_char,
    flags: u32,
) -> LinuxResult<isize> {
    let old_path = old_path.nullable().map(vm_load_string).transpose()?;
    let new_path = vm_load_string(new_path)?;
    debug!(
        "sys_linkat <= old_dirfd: {}, old_path: {:?}, new_dirfd: {}, new_path: {}, flags: {}",
        old_dirfd, old_path, new_dirfd, new_path, flags
    );

    if flags != 0 {
        warn!("Unsupported flags: {flags}");
    }

    let old = resolve_at(old_dirfd, old_path.as_deref(), flags)?
        .into_file()
        .ok_or(LinuxError::EBADF)?;
    if old.is_dir() {
        return Err(LinuxError::EPERM);
    }
    let (new_dir, new_name) =
        with_fs(new_dirfd, |fs| fs.resolve_nonexistent(Path::new(&new_path)))?;

    new_dir.link(new_name, &old)?;
    Ok(0)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_link(old_path: *const c_char, new_path: *const c_char) -> LinuxResult<isize> {
    sys_linkat(AT_FDCWD, old_path, AT_FDCWD, new_path, 0)
}

/// remove link of specific file (can be used to delete file)
/// dir_fd: the directory of link to be removed
/// path: the name of link to be removed
/// flags: can be 0 or AT_REMOVEDIR
/// return 0 when success, else return -1
pub fn sys_unlinkat(dirfd: i32, path: *const c_char, flags: usize) -> LinuxResult<isize> {
    let path = vm_load_string(path)?;

    debug!(
        "sys_unlinkat <= dirfd: {}, path: {:?}, flags: {}",
        dirfd, path, flags
    );

    with_fs(dirfd, |fs| {
        if flags == AT_REMOVEDIR as _ {
            fs.remove_dir(path)?;
        } else {
            fs.remove_file(path)?;
        }
        Ok(0)
    })
}

#[cfg(target_arch = "x86_64")]
pub fn sys_rmdir(path: *const c_char) -> LinuxResult<isize> {
    sys_unlinkat(AT_FDCWD, path, AT_REMOVEDIR as _)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_unlink(path: *const c_char) -> LinuxResult<isize> {
    sys_unlinkat(AT_FDCWD, path, 0)
}

pub fn sys_getcwd(buf: *mut u8, size: isize) -> LinuxResult<isize> {
    let size: usize = size.try_into().map_err(|_| LinuxError::EFAULT)?;
    if buf.is_null() {
        return Ok(0);
    }

    let cwd = FS_CONTEXT.lock().current_dir().absolute_path()?;
    debug!("sys_getcwd => cwd: {}", cwd);

    let cwd = CString::new(cwd.as_str()).map_err(|_| LinuxError::EINVAL)?;
    let cwd = cwd.as_bytes_with_nul();

    if cwd.len() <= size {
        vm_write_slice(buf, cwd)?;
        // FIXME: it is said that this should return 0
        Ok(buf.as_ptr() as _)
    } else {
        Err(LinuxError::ERANGE)
    }
}

#[cfg(target_arch = "x86_64")]
pub fn sys_symlink(target: *const c_char, linkpath: *const c_char) -> LinuxResult<isize> {
    sys_symlinkat(target, AT_FDCWD, linkpath)
}

pub fn sys_symlinkat(
    target: *const c_char,
    new_dirfd: i32,
    linkpath: *const c_char,
) -> LinuxResult<isize> {
    let target = vm_load_string(target)?;
    let linkpath = vm_load_string(linkpath)?;
    debug!(
        "sys_symlinkat <= target: {:?}, new_dirfd: {}, linkpath: {:?}",
        target, new_dirfd, linkpath
    );

    with_fs(new_dirfd, |fs| {
        fs.symlink(target, linkpath)?;
        Ok(0)
    })
}

#[cfg(target_arch = "x86_64")]
pub fn sys_readlink(path: *const c_char, buf: *mut u8, size: usize) -> LinuxResult<isize> {
    sys_readlinkat(AT_FDCWD, path, buf, size)
}

pub fn sys_readlinkat(
    dirfd: i32,
    path: *const c_char,
    buf: *mut u8,
    size: usize,
) -> LinuxResult<isize> {
    let path = vm_load_string(path)?;

    debug!("sys_readlinkat <= dirfd: {}, path: {:?}", dirfd, path);

    with_fs(dirfd, |fs| {
        let entry = fs.resolve_no_follow(path)?;
        let link = entry.read_link()?;
        let read = size.min(link.len());
        vm_write_slice(buf, &link.as_bytes()[..read])?;
        Ok(read as isize)
    })
}

#[cfg(target_arch = "x86_64")]
pub fn sys_chown(path: *const c_char, uid: u32, gid: u32) -> LinuxResult<isize> {
    sys_fchownat(AT_FDCWD, path, uid, gid, 0)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_lchown(path: *const c_char, uid: u32, gid: u32) -> LinuxResult<isize> {
    use linux_raw_sys::general::AT_SYMLINK_NOFOLLOW;
    sys_fchownat(AT_FDCWD, path, uid, gid, AT_SYMLINK_NOFOLLOW)
}

pub fn sys_fchown(fd: i32, uid: i32, gid: i32) -> LinuxResult<isize> {
    sys_fchownat(fd, core::ptr::null(), uid, gid, AT_EMPTY_PATH)
}

pub fn sys_fchownat(
    dirfd: i32,
    path: *const c_char,
    uid: i32,
    gid: i32,
    flags: u32,
) -> LinuxResult<isize> {
    let path = path.nullable().map(vm_load_string).transpose()?;
    let loc = resolve_at(dirfd, path.as_deref(), flags)?
        .into_file()
        .ok_or(LinuxError::EBADF)?;
    let meta = loc.metadata()?;

    let mut mode = meta.mode;
    // chown always clears the setuid bits
    mode.remove(NodePermission::SET_UID);
    // chown also removes the setgid bits if group-executable
    if mode.contains(NodePermission::GROUP_EXEC) {
        mode.remove(NodePermission::SET_GID);
    }

    let uid = if uid == -1 { meta.uid } else { uid as _ };
    let gid = if gid == -1 { meta.gid } else { gid as _ };
    loc.update_metadata(MetadataUpdate {
        owner: Some((uid, gid)),
        mode: Some(mode),
        ..Default::default()
    })?;
    Ok(0)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_chmod(path: *const c_char, mode: u32) -> LinuxResult<isize> {
    sys_fchmodat(AT_FDCWD, path, mode, 0)
}

pub fn sys_fchmod(fd: i32, mode: u32) -> LinuxResult<isize> {
    sys_fchmodat(fd, core::ptr::null(), mode, AT_EMPTY_PATH)
}

pub fn sys_fchmodat(dirfd: i32, path: *const c_char, mode: u32, flags: u32) -> LinuxResult<isize> {
    let path = path.nullable().map(vm_load_string).transpose()?;
    resolve_at(dirfd, path.as_deref(), flags)?
        .into_file()
        .ok_or(LinuxError::EBADF)?
        .update_metadata(MetadataUpdate {
            mode: Some(NodePermission::from_bits_truncate(mode as u16)),
            ..Default::default()
        })?;
    Ok(0)
}

fn update_times(
    dirfd: i32,
    path: *const c_char,
    atime: Option<Duration>,
    mtime: Option<Duration>,
    flags: u32,
) -> LinuxResult<()> {
    let path = path.nullable().map(vm_load_string).transpose()?;
    resolve_at(dirfd, path.as_deref(), flags)?
        .into_file()
        .ok_or(LinuxError::EBADF)?
        .update_metadata(MetadataUpdate {
            atime,
            mtime,
            ..Default::default()
        })?;
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct utimbuf {
    actime: linux_raw_sys::general::__kernel_old_time_t,
    modtime: linux_raw_sys::general::__kernel_old_time_t,
}

#[cfg(target_arch = "x86_64")]
pub fn sys_utime(path: *const c_char, times: *const utimbuf) -> LinuxResult<isize> {
    let (atime, mtime) = if let Some(times) = times.nullable() {
        // FIXME: AnyBitPattern
        let times = unsafe { times.vm_read_uninit()?.assume_init() };
        (
            Duration::from_secs(times.actime as _),
            Duration::from_secs(times.modtime as _),
        )
    } else {
        let time = wall_time();
        (time, time)
    };
    update_times(AT_FDCWD, path, Some(atime), Some(mtime), 0)?;
    Ok(0)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_utimes(
    path: *const c_char,
    times: *const [linux_raw_sys::general::timeval; 2],
) -> LinuxResult<isize> {
    let (atime, mtime) = if let Some(times) = times.nullable() {
        // FIXME: AnyBitPattern
        let [atime, mtime] = unsafe { times.vm_read_uninit()?.assume_init() };
        (atime.try_into_time_value()?, mtime.try_into_time_value()?)
    } else {
        let time = wall_time();
        (time, time)
    };
    update_times(AT_FDCWD, path, Some(atime), Some(mtime), 0)?;
    Ok(0)
}

pub fn sys_utimensat(
    dirfd: i32,
    path: *const c_char,
    times: *const [timespec; 2],
    mut flags: u32,
) -> LinuxResult<isize> {
    if path.is_null() {
        flags |= AT_EMPTY_PATH;
    }
    fn utime_to_duration(time: &timespec) -> Option<LinuxResult<Duration>> {
        match time.tv_nsec {
            val if val == UTIME_OMIT as _ => None,
            val if val == UTIME_NOW as _ => Some(Ok(wall_time())),
            _ => Some(time.try_into_time_value()),
        }
    }

    let (atime, mtime) = if let Some(times) = times.nullable() {
        // FIXME: AnyBitPattern
        let [atime, mtime] = unsafe { times.vm_read_uninit()?.assume_init() };
        (
            utime_to_duration(&atime).transpose()?,
            utime_to_duration(&mtime).transpose()?,
        )
    } else {
        let time = wall_time();
        (Some(time), Some(time))
    };
    if atime.is_none() && mtime.is_none() {
        return Ok(0);
    }

    update_times(dirfd, path, atime, mtime, flags)?;
    Ok(0)
}

#[cfg(target_arch = "x86_64")]
pub fn sys_rename(old_path: *const c_char, new_path: *const c_char) -> LinuxResult<isize> {
    sys_renameat(AT_FDCWD, old_path, AT_FDCWD, new_path)
}

pub fn sys_renameat(
    old_dirfd: i32,
    old_path: *const c_char,
    new_dirfd: i32,
    new_path: *const c_char,
) -> LinuxResult<isize> {
    sys_renameat2(old_dirfd, old_path, new_dirfd, new_path, 0)
}

pub fn sys_renameat2(
    old_dirfd: i32,
    old_path: *const c_char,
    new_dirfd: i32,
    new_path: *const c_char,
    flags: u32,
) -> LinuxResult<isize> {
    let old_path = vm_load_string(old_path)?;
    let new_path = vm_load_string(new_path)?;
    debug!(
        "sys_renameat2 <= old_dirfd: {}, old_path: {:?}, new_dirfd: {}, new_path: {}, flags: {}",
        old_dirfd, old_path, new_dirfd, new_path, flags
    );

    let (old_dir, old_name) = with_fs(old_dirfd, |fs| fs.resolve_parent(Path::new(&old_path)))?;
    let (new_dir, new_name) =
        with_fs(new_dirfd, |fs| fs.resolve_nonexistent(Path::new(&new_path)))?;

    old_dir.rename(&old_name, &new_dir, new_name)?;
    Ok(0)
}

pub fn sys_sync() -> LinuxResult<isize> {
    warn!("dummy sys_sync");
    Ok(0)
}

pub fn sys_syncfs(_fd: i32) -> LinuxResult<isize> {
    warn!("dummy sys_syncfs");
    Ok(0)
}
