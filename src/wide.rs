use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for &str {
    fn to_wide(&self) -> Vec<u16> {
        OsStr::new(self).encode_wide().chain(once(0)).collect()
    }
}
