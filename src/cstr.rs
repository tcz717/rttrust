use crate::ffi::RT_NAME_MAX;

use cty::c_char;

#[allow(non_camel_case_types)]
pub type c_str = *const c_char;
#[allow(non_camel_case_types)]
pub type c_str_mut = *mut c_char;

#[derive(Clone, Default)]
pub struct RtName {
    buf: [u8; RT_NAME_MAX as usize],
}

impl RtName {
    pub fn new(buf: [u8; RT_NAME_MAX as usize]) -> Self {
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
