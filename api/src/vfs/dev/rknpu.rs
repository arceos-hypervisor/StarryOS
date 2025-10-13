use core::{any::Any, convert::TryFrom};

use axfs_ng_vfs::{DeviceId, NodeFlags, VfsError, VfsResult};
use rknpu::RknpuAction;

use crate::vfs::DeviceOps;

/// Device ID for /dev/rknpu (pick an unused major/minor)
pub const RKNPU_DEVICE_ID: DeviceId = DeviceId::new(251, 0);

const IOC_NRSHIFT: u32 = 0;
const IOC_NRBITS: u32 = 8;
const IOC_NRMASK: u32 = (1 << IOC_NRBITS) - 1;

/// rknpu device (stub)
pub struct Rknpu;

impl DeviceOps for Rknpu {
    fn read_at(&self, _buf: &mut [u8], _offset: u64) -> VfsResult<usize> {
        info!(
            "rknpu: read_at called, offset={} len={}",
            _offset,
            _buf.len()
        );
        Ok(0)
    }

    fn write_at(&self, _buf: &[u8], _offset: u64) -> VfsResult<usize> {
        info!(
            "rknpu: write_at called, offset={} len={}",
            _offset,
            _buf.len()
        );
        Ok(0)
    }

    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        if arg == 0 {
            warn!("[rknpu]: ioctl received null arg pointer");
            return Err(VfsError::InvalidData);
        }
        let flag = arg as *mut RknpuUserAction;
        let flag_val = unsafe { &*flag }.flag();
        info!("flag is {:?}", unsafe { &*flag }.flag());

        npu_power_on().expect("Failed to power on NPU.");

        if let Ok(op) = RknpuCmd::try_from(cmd) {
            match op {
                RknpuCmd::Action => {
                    info!("Action");
                    let result = with_npu(|rknpu_dev| {
                        let r = rknpu_dev.action(flag_val);
                        r.map_err(|_| VfsError::InvalidData)
                    });
                    match result {
                        Ok(_) => info!("rknpu ioctl done."),
                        Err(e) => info!("rknpu ioctl failed: {:?}", e),
                    }
                }
                RknpuCmd::Submit => {
                    todo!()
                }
                RknpuCmd::MemCreate => {
                    todo!()
                }
                _ => {
                    warn!("not implemented yet");
                }
            }
        } else {
            warn!("Unknown RKNPU cmd: {:#x}", cmd);
            return Err(VfsError::BadIoctl);
        }

        npu_power_off().expect("Failed to power off NPU.");

        Ok(0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn flags(&self) -> NodeFlags {
        NodeFlags::NON_CACHEABLE | NodeFlags::STREAM
    }
}

fn npu() -> Result<rdrive::DeviceGuard<::rknpu::Rknpu>, VfsError> {
    rdrive::get_one()
        .ok_or(VfsError::NotFound)?
        .try_lock()
        .map_err(|_| VfsError::AddressInUse)
}

fn with_npu<F, R>(f: F) -> Result<R, VfsError>
where
    F: FnOnce(&mut ::rknpu::Rknpu) -> Result<R, VfsError>,
{
    let mut npu = npu()?;
    f(&mut npu)
}

// controlled in npu driver, return Ok(()) for stub
fn npu_power_on() -> Result<(), VfsError> {
    Ok(())
}

// controlled in npu driver, return Ok(()) for stub
fn npu_power_off() -> Result<(), VfsError> {
    Ok(())
}

#[derive(Debug, Copy, Clone)]
struct RknpuUserAction {
    flags: RknpuAction,
    _value: u32,
}

impl RknpuUserAction {
    fn flag(&self) -> RknpuAction {
        self.flags
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RknpuCmd {
    Action     = 0x00,
    Submit     = 0x01,
    MemCreate  = 0x02,
    MemMap     = 0x03,
    MemDestroy = 0x04,
    MemSync    = 0x05,
}

impl TryFrom<u32> for RknpuCmd {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match ioc_nr(value) {
            0x00 => Ok(RknpuCmd::Action),
            0x01 => Ok(RknpuCmd::Submit),
            0x02 => Ok(RknpuCmd::MemCreate),
            0x03 => Ok(RknpuCmd::MemMap),
            0x04 => Ok(RknpuCmd::MemDestroy),
            0x05 => Ok(RknpuCmd::MemSync),
            _ => Err(()),
        }
    }
}

#[inline(always)]
fn ioc_nr(cmd: u32) -> u32 {
    (cmd >> IOC_NRSHIFT) & IOC_NRMASK
}
