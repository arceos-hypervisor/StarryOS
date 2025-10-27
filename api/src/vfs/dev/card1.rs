use core::any::Any;

use axfs_ng_vfs::{DeviceId, NodeFlags, VfsError, VfsResult};
use starry_vm::VmMutPtr;

use crate::vfs::DeviceOps;

/// Device ID for /dev/dri/card1
pub const CARD1_SYSTEM_DEVICE_ID: DeviceId = DeviceId::new(255, 0);

pub struct Card1;

impl Card1 {
    /// Creates a new /dev/dri/card1 device.
    pub fn new() -> Self {
        warn!("dri: Creating new Card1 instance");
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
        warn!("card1: ioctl called cmd={:#x}, arg={:#x}", cmd, arg);
        
        // Handle common card1 heap ioctls
        match cmd {
            // For now, we just return success for all ioctls and zero the first u32
            // if arg is a user pointer, similar to rknpu implementation
            _ => {
                // Best-effort: if arg is a user pointer, zero the first u32 there so
                // user-space doesn't read uninitialized memory
                if arg != 0 {
                    // write a safe default (0) to the user pointer
                    // Use vm_write to safely write across the VM boundary.
                    if let Err(e) = (arg as *mut u32).vm_write(0u32) {
                        warn!("card1: ioctl vm_write failed: {:?}", e);
                        return Err(VfsError::InvalidInput);
                    }
                }
                Ok(0)
            }
        }
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