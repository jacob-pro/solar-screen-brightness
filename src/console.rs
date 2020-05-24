use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::{GetConsoleWindow, FreeConsole};
use winapi::shared::ntdef::{NULL};
use winapi::um::winuser::{GetSystemMenu, EnableMenuItem, SC_CLOSE, MF_ENABLED, MF_GRAYED, ShowWindow, BringWindowToTop, SetForegroundWindow, SW_RESTORE, SW_HIDE};
use winapi::shared::minwindef::{TRUE, FALSE};
use winapi::shared::windef::HWND;
use crate::tui::run_tui;
use crate::tray::MessageSender;

pub struct Console {}

impl Console {

    pub fn create(tray: MessageSender) -> Self {
        unsafe {
            if AllocConsole() != TRUE {panic!("Error opening console")};
            let console_window = GetConsoleWindow();
            if console_window == NULL as HWND { panic!("Null console window") };
            let console_menu = GetSystemMenu(console_window, FALSE);
            EnableMenuItem(console_menu, SC_CLOSE as u32, MF_ENABLED | MF_GRAYED);
        }
        std::thread::spawn(move || {
            run_tui(tray);
        });
        Console{}
    }

    pub fn show(&self) {
        unsafe {
            let console_window = GetConsoleWindow();
            if console_window == NULL as HWND { panic!("Null console window") };
            ShowWindow(console_window, SW_RESTORE);
            BringWindowToTop(console_window);
            SetForegroundWindow(console_window);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let console_window = GetConsoleWindow();
            if console_window == NULL as HWND { panic!("Null console window") };
            ShowWindow(console_window, SW_HIDE);
        }
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        unsafe {
            if FreeConsole() != TRUE { panic!("Error closing console") };
        }
    }
}
