use solar_screen_brightness_windows_bindings::windows::{self, HRESULT};
use solar_screen_brightness_windows_bindings::Windows::Win32::{
    Foundation::HWND,
    System::Diagnostics::Debug::SetLastError,
    UI::WindowsAndMessaging::{GetWindowLongPtrW, GWLP_USERDATA},
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
    let user_data = set_and_get_error(|| GetWindowLongPtrW(*hwnd, GWLP_USERDATA)).unwrap();
    if user_data == 0 {
        return None;
    }
    Some(&mut *(user_data as *mut T))
}

#[inline]
pub unsafe fn set_and_get_error<F, R>(mut f: F) -> windows::Result<R>
where
    F: FnMut() -> R,
{
    SetLastError(0);
    let result = f();
    HRESULT::from_thread().ok().map(|_| result)
}
