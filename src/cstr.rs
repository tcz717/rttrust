// pub use cstr_core::CStr;

use cty::c_char;

#[allow(non_camel_case_types)]
pub type c_str = *const c_char;

const FIXED_STRING_SIZE: usize = 32;
type FixedStringData = [u8; FIXED_STRING_SIZE];
pub struct FixedString {
    data: FixedStringData,
    used: usize,
}

impl FixedString {
    pub fn new(data: FixedStringData, used: usize) -> Self {
        Self { data, used }
    }
    pub fn copy_from(s: &[u8]) -> Option<Self> {
        if s.len() > FIXED_STRING_SIZE - 1 {
            None
        } else {
            let mut data = [0; FIXED_STRING_SIZE];
            data[..s.len()].copy_from_slice(s);
            Some(Self::new(data, s.len()))
        }
    }

    pub fn iter_str<'a>(s: &str) -> impl Iterator<Item = FixedString> + '_ {
        let chunk_size = 31;
        s.as_bytes()
            .chunks(chunk_size)
            .map(|chunk| FixedString::copy_from(chunk).unwrap())
    }

    pub fn as_cstr(&self) -> *const c_char {
        self.data.as_ptr() as *const c_char
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.used]
    }

    pub fn len(self) -> usize {
        self.used
    }
}

#[cfg(test)]
mod tests {

    use super::FixedString;

    #[test]
    fn split() {
        let s = "abc".repeat(50);
        let n = FixedString::iter_str(s.as_str()).count();
        let cstr = FixedString::iter_str(s.as_str()).nth(1).unwrap();

        assert_eq!(n, s.len() / 32 + 1);
        assert_eq!(cstr.as_bytes()[..6], [b'b', b'c', b'a', b'b', b'c', b'a',]);
    }
}
