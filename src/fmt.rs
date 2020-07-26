use crate::{ffi::rt_kputs, cstr::FixedString};
use core::fmt::Write;

pub struct Console;

#[macro_export]
macro_rules! kprintf {
    ($fmt:expr) => (unsafe {
        $crate::ffi::rt_kprintf($fmt.as_ptr())
    });
    ($fmt:expr, $($arg:tt)*) => (unsafe {
        $crate::ffi::rt_kprintf($fmt.as_ptr(), $($arg)*)
    });
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for cstr in FixedString::iter_str(s) {
            unsafe {
                rt_kputs(cstr.as_cstr());
            }
        }
        Ok(())
    }
}
