pub use cursive;

use windows::core::Error;
use windows::Win32::Foundation::{SetLastError, HANDLE, HWND};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, GetWindowThreadProcessId, MessageBoxW, ShowWindow, GWLP_USERDATA,
    MB_ICONSTOP, MB_OK, SW_HIDE,
};

/// Hides the current Console Window if and only if the Window belongs to the current process
pub fn hide_process_console_window() {
    unsafe {
        let console = GetConsoleWindow();
        if !HANDLE(console).is_invalid() {
            let mut console_pid = 0;
            GetWindowThreadProcessId(console, &mut console_pid);
            if console_pid == GetCurrentProcessId() {
                ShowWindow(console, SW_HIDE);
            }
        }
    }
}

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for &str {
    fn to_wide(&self) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(self).encode_wide().chain(once(0)).collect()
    }
}

#[inline]
pub fn loword(l: u32) -> u32 {
    l & 0xffff
}

pub trait WindowDataExtension {
    unsafe fn get_user_data<T>(&self) -> Option<&mut T>;
}

impl WindowDataExtension for HWND {
    unsafe fn get_user_data<T>(&self) -> Option<&mut T> {
        let user_data = set_and_get_error(|| GetWindowLongPtrW(self, GWLP_USERDATA)).unwrap();
        if user_data == 0 {
            return None;
        }
        Some(&mut *(user_data as *mut T))
    }
}

#[inline]
pub unsafe fn set_and_get_error<F, R>(mut f: F) -> windows::core::Result<R>
where
    F: FnMut() -> R,
{
    SetLastError(0);
    let result = f();
    let error = Error::from_win32();
    if error == Error::OK {
        Ok(result)
    } else {
        Err(error)
    }
}

/// Shows a MessageBox on Panic
pub fn wrap_panic_hook() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| unsafe {
        before(info);
        let title = "Fatal Error";
        let text = format!("{}", info);
        MessageBoxW(HWND::default(), text, title, MB_OK | MB_ICONSTOP);
    }));
}
