use core::any::Any;

use axfs_ng_vfs::{DeviceId, NodeFlags, VfsResult, VfsError};
use starry_vm::VmMutPtr;

use crate::vfs::DeviceOps;

/// Device ID for /dev/rknpu (pick an unused major/minor)
pub const RKNPU_DEVICE_ID: DeviceId = DeviceId::new(251, 0);

/// rknpu device (stub)
pub struct Rknpu;

impl DeviceOps for Rknpu {
    fn read_at(&self, _buf: &mut [u8], _offset: u64) -> VfsResult<usize> {
        info!("rknpu: read_at called, offset={} len={}", _offset, _buf.len());
        Ok(0)
    }

    fn write_at(&self, _buf: &[u8], _offset: u64) -> VfsResult<usize> {
        info!("rknpu: write_at called, offset={} len={}", _offset, _buf.len());
        Ok(0)
    }

    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        info!("rknpu: ioctl called cmd={} arg={:#x}", cmd, arg);
        // Best-effort: if arg is a user pointer, zero the first u32 there so
        // user-space doesn't read uninitialized memory (which produced
        // 0xffffffff in demos and led to crashes).
        if arg != 0 {
            // write a safe default (0) to the user pointer; many rknn ioctls
            // return a small struct whose first field is a u32 target/type.
            // Use vm_write to safely write across the VM boundary.
            if let Err(e) = (arg as *mut u32).vm_write(0u32) {
                warn!("rknpu: ioctl vm_write failed: {:?}", e);
                return Err(VfsError::InvalidInput);
            }
        }
        Ok(0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flags(&self) -> NodeFlags {
        NodeFlags::NON_CACHEABLE | NodeFlags::STREAM
    }
}
