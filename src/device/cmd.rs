use crate::cmd::{Command, DeviceCommand};
use crate::ffi::*;
use core::mem::MaybeUninit;

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

    fn exec(mut self, device: &mut super::Device) -> crate::Result<Self::Return> {
        device
            .control(&mut self)
            .map(|_| unsafe { self.result.assume_init() })
    }
}
