use crate::ffi::RT_NAME_MAX;

use core::str::from_utf8_unchecked;
use cty::c_char;

#[allow(non_camel_case_types)]
pub type c_str = *const c_char;
#[allow(non_camel_case_types)]
pub type c_str_mut = *mut c_char;

type NameArray = [u8; RT_NAME_MAX as usize];

#[derive(Clone, Default)]
pub struct RtName {
    buf: NameArray,
}

impl RtName {
    pub fn new(buf: NameArray) -> Self {
        Self { buf }
    }

    #[inline]
    pub fn as_array_str(&self) -> &[u8] {
        &self.buf
    }
    #[inline]
    pub fn as_mut_array_str(&mut self) -> &mut [u8] {
        &mut self.buf
    }
}

impl From<&str> for RtName {
    fn from(s: &str) -> Self {
        let mut buf = [0; RT_NAME_MAX as usize];
        let src = s.as_bytes();
        let len = s.len().clamp(0, RT_NAME_MAX as usize);
        buf[..len].copy_from_slice(&src[..len]);
        RtName { buf }
    }
}

impl Into<c_str> for RtName {
    #[inline]
    fn into(self) -> c_str {
        self.buf.as_ptr().cast()
    }
}

impl Into<c_str_mut> for RtName {
    #[inline]
    fn into(mut self) -> c_str_mut {
        self.buf.as_mut_ptr().cast()
    }
}

pub struct RtNameRef<'a> {
    name: &'a NameArray,
}

impl<'a> RtNameRef<'a> {
    pub fn new(name: &'a NameArray) -> Self {
        Self { name }
    }
}

impl<'a> From<&'a [u8; RT_NAME_MAX as usize]> for RtNameRef<'a> {
    #[inline]
    fn from(name: &'a [u8; RT_NAME_MAX as usize]) -> Self {
        Self { name }
    }
}

impl<'a> From<&'a [i8; RT_NAME_MAX as usize]> for RtNameRef<'a> {
    #[inline]
    fn from(name: &'a [i8; RT_NAME_MAX as usize]) -> Self {
        Self {
            name: unsafe { &*(name.as_ptr().cast() as *const [u8; RT_NAME_MAX as usize]) },
        }
    }
}

impl<'a> Into<&'a str> for RtNameRef<'a> {
    #[inline]
    fn into(self) -> &'a str {
        unsafe { from_utf8_unchecked(self.name) }
    }
}

impl<'a> AsRef<str> for RtNameRef<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        unsafe { from_utf8_unchecked(self.name) }
    }
}
