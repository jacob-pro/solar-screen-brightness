use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::iter::once;

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for &str {
    fn to_wide(&self) -> Vec<u16> {
        OsStr::new(self).encode_wide().chain(once(0)).collect()
    }
}

pub fn wide_to_str(s: &[u16]) -> Result<String, OsString> {
    let end = s.iter().position(|&x| x == 0).unwrap();
    let truncated = &s[0..end];
    OsString::from_wide(truncated).into_string()
}

