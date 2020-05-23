
pub trait StrExt {
    fn to_wide(&self) -> Vec<u16>;
}

impl StrExt for &str {
    fn to_wide(&self) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::iter::once;
        OsStr::new(self).encode_wide().chain(once(0)).collect()
    }
}
