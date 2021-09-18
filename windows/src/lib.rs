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
