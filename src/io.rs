//! Re-export [genio]

use core::fmt::Arguments;
use crate::Result;

pub use genio::*;

/// Wraper trait to extent genio's [Write](genio::Write) to be used by `write!` marco
pub trait WriteFmt: Write<WriteError = crate::RtError> {
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<()> {
        // Create a shim which translates a Write to a fmt::Write and saves
        // off I/O errors. instead of discarding them
        struct Adaptor<'a, T: ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<()>,
        }

        impl<T: Write<WriteError = crate::RtError> + ?Sized> core::fmt::Write for Adaptor<'_, T> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(core::fmt::Error)
                    }
                }
            }
        }

        let mut output = Adaptor {
            inner: self,
            error: Ok(()),
        };
        match core::fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => output.error,
        }
    }
}
