pub use cursive;
pub use windows;
windows::include_bindings!();

/// Hides the current Console Window if and only if the Window belongs to the current process
pub fn hide_process_console_window() {
    use Windows::Win32::{
        System::Console::GetConsoleWindow,
        System::Threading::GetCurrentProcessId,
        UI::WindowsAndMessaging::{GetWindowThreadProcessId, ShowWindow, SW_HIDE},
    };
    unsafe {
        let console = GetConsoleWindow();
        if !console.is_null() {
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

impl WindowDataExtension for Windows::Win32::Foundation::HWND {
    unsafe fn get_user_data<T>(&self) -> Option<&mut T> {
        use Windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWLP_USERDATA};
        let user_data = set_and_get_error(|| GetWindowLongPtrW(self, GWLP_USERDATA)).unwrap();
        if user_data == 0 {
            return None;
        }
        Some(&mut *(user_data as *mut T))
    }
}

#[inline]
pub unsafe fn set_and_get_error<F, R>(mut f: F) -> windows::Result<R>
where
    F: FnMut() -> R,
{
    use windows::HRESULT;
    use Windows::Win32::System::Diagnostics::Debug::SetLastError;
    SetLastError(0);
    let result = f();
    HRESULT::from_thread().ok().map(|_| result)
}

/// Shows a MessageBox on Panic
pub fn wrap_panic_hook() {
    use Windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONSTOP, MB_OK};
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| unsafe {
        before(info);
        let title = "Fatal Error";
        let text = format!("{}", info);
        MessageBoxW(
            Windows::Win32::Foundation::HWND::NULL,
            text,
            title,
            MB_OK | MB_ICONSTOP,
        );
    }));
}
