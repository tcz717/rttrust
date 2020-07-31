use crate::ffi::rt_kputs;

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

const CONSOLE_BUF_SIZE: usize = 32;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut buf = [0; CONSOLE_BUF_SIZE];
        for chunk in s.as_bytes().chunks(CONSOLE_BUF_SIZE - 1) {
            buf[..chunk.len()].copy_from_slice(chunk);
            unsafe {
                rt_kputs(buf.as_ptr().cast());
            }
        }
        Ok(())
    }
}
