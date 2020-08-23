use crate::{device::Device, Result};
use cty::{c_int, c_void};

/// cmd and arg pair used for `rt_xxx_control` APIs
pub trait Command {
    fn get_cmd(&self) -> c_int;
    fn get_arg(&mut self) -> *mut c_void;
}

/// Command can be used only for device
pub trait DeviceCommand: Command {
    type Return;
    fn exec(self, device: &mut Device) -> Result<Self::Return>;
}

#[derive(Debug, Clone, Copy)]
pub struct RawCommand {
    cmd: c_int,
    arg: *mut c_void,
}

impl RawCommand {
    pub fn new(cmd: c_int, arg: *mut c_void) -> Self {
        Self { cmd, arg }
    }
}

impl Command for RawCommand {
    fn get_cmd(&self) -> c_int {
        self.cmd
    }

    fn get_arg(&mut self) -> *mut c_void {
        self.arg
    }
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
