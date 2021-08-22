use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for &str {
    fn to_wide(&self) -> Vec<u16> {
        OsStr::new(self).encode_wide().chain(once(0)).collect()
    }
}

pub fn wchar_to_string(s: &[u16]) -> String {
    let end = s.iter().position(|&x| x == 0).unwrap_or(s.len());
    let truncated = &s[0..end];
    OsString::from_wide(truncated).to_string_lossy().into()
}
