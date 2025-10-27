use core::{any::Any, convert::TryFrom, mem};

use axfs_ng_vfs::{DeviceId, NodeFlags, VfsError, VfsResult};
use axhal::asm::user_copy;
use rknpu::{
    RknpuAction,
    ioctrl::{RknpuMemCreate, RknpuSubmit},
};
use starry_vm::VmMutPtr;

use crate::vfs::DeviceOps;

/// Device ID for /dev/rknpu (pick an unused major/minor)
pub const RKNPU_DEVICE_ID: DeviceId = DeviceId::new(251, 0);

const IOC_NRSHIFT: u32 = 0;
const IOC_NRBITS: u32 = 8;
const IOC_NRMASK: u32 = (1 << IOC_NRBITS) - 1;

/// Device ID for /dev/dri/card1
pub const CARD1_SYSTEM_DEVICE_ID: DeviceId = DeviceId::new(255, 0);

pub struct Card1;

impl Card1 {
    /// Creates a new /dev/dri/card1 device.
    pub fn new() -> Card1 {
        warn!("card1: new called");
        Self
    }
}

impl Default for Card1 {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceOps for Card1 {
    fn read_at(&self, _buf: &mut [u8], _offset: u64) -> VfsResult<usize> {
        warn!("dri: read_at called");
        // card1 heap devices are not meant to be read directly
        Err(VfsError::InvalidInput)
    }

    fn write_at(&self, _buf: &[u8], _offset: u64) -> VfsResult<usize> {
        warn!("dri: write_at called");
        // card1 heap devices are not meant to be written directly
        Err(VfsError::InvalidInput)
    }

    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        if arg == 0 {
            warn!("[rknpu]: ioctl received null arg pointer");
            return Err(VfsError::InvalidData);
        }
        let flag = arg as *mut RknpuUserAction;
        let flag_val = unsafe { &*flag }.flag();
        info!("flag_val is {:?}", flag_val);

        npu_power_on().expect("Failed to power on NPU.");

        if let Ok(op) = RknpuCmd::try_from(cmd) {
            match op {
                RknpuCmd::Action => {
                    info!("rknpu action ioctl");
                    let mut action_args = RknpuUserAction {
                        flags: RknpuAction::GetHwVersion,
                        _value: 0,
                    };

                    copy_from_user(
                        &mut action_args as *mut _ as *mut u8,
                        flag as *const u8,
                        mem::size_of::<RknpuUserAction>(),
                    )?;

                    if let Err(e) = with_npu(|rknpu_dev| {
                        rknpu_dev
                            .action(flag_val)
                            .map_err(|_| VfsError::InvalidData)
                    }) {
                        warn!("rknpu action ioctl failed: {:?}", e);
                    }

                    copy_to_user(
                        flag as *mut u8,
                        &action_args as *const _ as *const u8,
                        mem::size_of::<RknpuUserAction>(),
                    )?;
                }
                RknpuCmd::Submit => {
                    info!("rknpu submit ioctl");
                    let mut submit_args = RknpuSubmit::default();

                    copy_from_user(
                        &mut submit_args as *mut _ as *mut u8,
                        arg as *const u8,
                        mem::size_of::<RknpuSubmit>(),
                    )?;

                    if let Err(e) = with_npu(|rknpu_dev| {
                        rknpu_dev
                            .submit_ioctrl(&mut submit_args)
                            .map_err(|_| VfsError::InvalidData)
                    }) {
                        warn!("rknpu submit ioctl failed: {:?}", e);
                    }

                    copy_to_user(
                        arg as *mut u8,
                        &submit_args as *const _ as *const u8,
                        mem::size_of::<RknpuSubmit>(),
                    )?;
                }
                RknpuCmd::MemCreate => {
                    info!("rknpu mem_create ioctl");
                    let mut mem_create_args = RknpuMemCreate::default();

                    copy_from_user(
                        &mut mem_create_args as *mut _ as *mut u8,
                        arg as *const u8,
                        mem::size_of::<RknpuMemCreate>(),
                    )?;

                    if let Err(e) = with_npu(|rknpu_dev| {
                        rknpu_dev
                            .create(&mut mem_create_args)
                            .map_err(|_| VfsError::InvalidData)
                    }) {
                        warn!("rknpu mem_create ioctl failed: {:?}", e);
                    }

                    copy_to_user(
                        arg as *mut u8,
                        &mem_create_args as *const _ as *const u8,
                        mem::size_of::<RknpuMemCreate>(),
                    )?;
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
        info!("card1: as_any called - used for dynamic type checking");
        self
    }

    fn flags(&self) -> NodeFlags {
        info!("card1: flags called - returning NON_CACHEABLE flag");
        NodeFlags::NON_CACHEABLE
    }
}

pub fn npu() -> Result<rdrive::DeviceGuard<::rknpu::Rknpu>, VfsError> {
    rdrive::get_one()
        .ok_or(VfsError::NotFound)?
        .try_lock()
        .map_err(|_| VfsError::AddressInUse)
}

pub fn with_npu<F, R>(f: F) -> Result<R, VfsError>
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

fn copy_from_user(dst: *mut u8, src: *const u8, size: usize) -> Result<(), axio::Error> {
    let ret = unsafe { user_copy(dst, src, size) };

    if ret != 0 {
        warn!("[rknpu]: copy_to_user failed, ret={}", ret);
        return Err(VfsError::InvalidData);
    }
    Ok(())
}

fn copy_to_user(dst: *mut u8, src: *const u8, size: usize) -> Result<(), axio::Error> {
    let ret = unsafe { user_copy(dst, src, size) };

    if ret != 0 {
        warn!("[rknpu]: copy_to_user failed, ret={}", ret);
        return Err(VfsError::InvalidData);
    }
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
