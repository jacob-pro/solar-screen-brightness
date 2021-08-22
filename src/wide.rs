use solar_screen_brightness_windows_bindings::Windows::Win32::Foundation::HWND;
use solar_screen_brightness_windows_bindings::Windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, GWLP_USERDATA,
};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for &str {
    fn to_wide(&self) -> Vec<u16> {
        OsStr::new(self).encode_wide().chain(once(0)).collect()
    }
}

#[inline]
pub fn loword(l: u32) -> u32 {
    l & 0xffff
}

pub unsafe fn get_user_data<T>(hwnd: &HWND) -> Option<&mut T> {
    let user_data = GetWindowLongPtrW(*hwnd, GWLP_USERDATA);
    if user_data == 0 {
        return None;
    }
    Some(&mut *(user_data as *mut T))
}
