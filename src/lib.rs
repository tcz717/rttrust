#![no_std]
#![feature(clamp)]

#![feature(alloc_error_handler)]
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod allocator;
pub mod cstr;
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod ffi;
pub mod fmt;
pub mod ipc;
pub mod thread;

#[cfg(feature = "alloc")]
pub use alloc::{boxed::Box, rc::Rc, vec::Vec};

#[cfg(not(test))]
use core::{fmt::Write, panic::PanicInfo};
use ffi::rt_err_t;

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::fmt::Console {};
        writer.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

#[cfg(all(not(test), not(feature = "custom-panic")))]
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    let mut writer = fmt::Console {};
    writeln!(writer, "{}", panic).ok();
    loop {}
}

#[cfg(all(not(test), feature = "alloc"))]
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[derive(Debug)]
pub enum RtError {
    Error,
    TimeOut,
    Full,
    Empty,
    NoMem,
    NoSys,
    Busy,
    IO,
    Intr,
    Inval,
    Unknown,
}

pub type Result<R> = core::result::Result<R, RtError>;

impl RtError {
    pub fn from_code(err: rt_err_t) -> Option<Self> {
        let err = err.abs() as u32;
        match err {
            ffi::RT_EOK => None,
            ffi::RT_ETIMEOUT => Some(RtError::TimeOut),
            ffi::RT_EFULL => Some(RtError::Full),
            ffi::RT_EEMPTY => Some(RtError::Empty),
            ffi::RT_ENOMEM => Some(RtError::NoMem),
            ffi::RT_ENOSYS => Some(RtError::NoSys),
            ffi::RT_EBUSY => Some(RtError::Busy),
            ffi::RT_EIO => Some(RtError::IO),
            ffi::RT_EINTR => Some(RtError::Intr),
            ffi::RT_EINVAL => Some(RtError::Inval),
            _ => Some(RtError::Unknown),
        }
    }
    pub fn from_code_none<R>(err: rt_err_t, ok: R) -> Result<R> {
        Self::from_code(err).map_or(Ok(ok), |e| Err(e))
    }
    pub fn from_code_none_then<F, R>(err: rt_err_t, ok: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        Self::from_code(err).map_or_else(ok, |e| Err(e))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
