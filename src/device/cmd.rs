use crate::cmd::RawCommand;
use crate::cmd::Command;
use crate::{Result, ffi::*};
use core::mem::MaybeUninit;

cfg_if! {
    if #[cfg(test)] {
        use crate::mock::device::MockDevice as Device;
    } else {
        use crate::device::Device;
    }
}

/// Command can be used only for device
pub trait DeviceCommand: Command {
    type Return;
    fn exec(self, device: &mut Device) -> Result<Self::Return>;
}


impl DeviceCommand for RawCommand {
    type Return = RawCommand;

    fn exec(mut self, device: &mut Device) -> Result<Self::Return> {
        match device.control(&mut self) {
            Ok(_) => Ok(self),
            Err(err) => Err(err),
        }
    }
}

/// Get geometry information of block device
#[derive(Debug)]
pub struct GetGeometry {
    result: MaybeUninit<rt_device_blk_geometry>,
}

impl GetGeometry {
    pub fn new() -> Self {
        Self {
            result: MaybeUninit::uninit(),
        }
    }
}

impl Command for GetGeometry {
    #[inline]
    fn get_cmd(&self) -> cty::c_int {
        RT_DEVICE_CTRL_BLK_GETGEOME as cty::c_int
    }

    fn get_arg(&mut self) -> *mut cty::c_void {
        self.result.as_mut_ptr().cast()
    }
}

impl DeviceCommand for GetGeometry {
    type Return = rt_device_blk_geometry;

    fn exec(mut self, device: &mut Device) -> Result<Self::Return> {
        device
            .control(&mut self)
            .map(|_| unsafe { self.result.assume_init() })
    }
}
