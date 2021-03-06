//! # rttrust
//!
//! Rust wrapper for [rt-thread](https://github.com/RT-Thread/rt-thread/tree/master)

//! ### TODO
//! 1. communication
//!     1. rt_mailbox
//!     1. rt_messagequeue
//!     1. rt_signal
//!

#![cfg_attr(not(test), no_std)]

#![feature(clamp)]
#![feature(alloc_error_handler)]
#![cfg_attr(test, feature(proc_macro_hygiene))]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "alloc", macro_use)]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate cfg_if;

#[cfg(test)]
extern crate mockall;

#[cfg(all(not(test), feature = "alloc"))]
pub mod allocator;
#[cfg(feature = "alloc")]
pub mod callback;
pub mod cmd;
pub mod cstr;
pub mod device;
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod ffi;
pub mod fmt;
pub mod ipc;
pub mod object;
pub mod thread;
pub mod timer;

#[cfg(test)]
pub(crate) mod mock;

#[cfg(feature = "alloc")]
pub use alloc::{boxed::Box, rc::Rc, vec::Vec};

#[cfg(feature = "io")]
pub mod io;

#[cfg(all(not(test), not(feature = "custom-panic")))]
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
#[cfg_attr(not(test), panic_handler)]
fn panic(panic: &PanicInfo<'_>) -> ! {
    let mut writer = fmt::Console {};
    writeln!(writer, "{}", panic).ok();
    loop {}
}

#[cfg(all(not(test), feature = "alloc"))]
#[cfg_attr(not(test), alloc_error_handler)]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// RT-Thread error code definitions
#[derive(Debug)]
pub enum RtError {
    /// A generic error happens
    Error,
    /// Timed out
    TimeOut,
    /// The resource is full
    Full,
    /// The resource is empty
    Empty,
    /// No memory
    NoMem,
    /// No system
    NoSys,
    /// Busy
    Busy,
    /// IO error
    IO,
    /// Interrupted system call
    Intr,
    /// Invalid argument
    Inval,
    /// Unknown error
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
        F: FnOnce() -> R,
    {
        Self::from_code(err).map_or_else(|| Ok(ok()), |e| Err(e))
    }

    pub fn to_code(&self) -> rt_err_t {
        let code = match self {
            RtError::Error => ffi::RT_EOK,
            RtError::TimeOut => ffi::RT_ETIMEOUT,
            RtError::Full => ffi::RT_EFULL,
            RtError::Empty => ffi::RT_EEMPTY,
            RtError::NoMem => ffi::RT_ENOMEM,
            RtError::NoSys => ffi::RT_ENOSYS,
            RtError::Busy => ffi::RT_EBUSY,
            RtError::IO => ffi::RT_EIO,
            RtError::Intr => ffi::RT_EINTR,
            RtError::Inval => ffi::RT_EINVAL,
            RtError::Unknown => ffi::RT_EINVAL + 1,
        };
        -(code as rt_err_t)
    }
}
