use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::{GetConsoleWindow, FreeConsole, SetConsoleTitleW};
use winapi::um::winuser::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::ntdef::{NULL};
use crate::tui::run_tui;
use crate::tray::MessageSender;
use crate::wide::WideString;
use crate::assets::Assets;

pub struct Console {}

impl Console {

    pub fn create(tray: MessageSender) -> Self {
        unsafe {
            if AllocConsole() != TRUE {panic!("Error opening console")};
            let console_window = GetConsoleWindow();
            if console_window == NULL as HWND { panic!("Null console window") };
            let console_menu = GetSystemMenu(console_window, FALSE);
            EnableMenuItem(console_menu, SC_CLOSE as u32, MF_ENABLED | MF_GRAYED);
            SetConsoleTitleW("Solar Screen Brightness".to_wide().as_ptr());

            let mut asset = Assets::get("icon-256.png").expect("Icon missing").into_owned();
            let hicon = CreateIconFromResource(asset.as_mut_ptr(), asset.len() as u32, TRUE, 0x00030000);
            if hicon == NULL as HICON { panic!("Failed to create icon") }
            SendMessageW(console_window, WM_SETICON, ICON_BIG as WPARAM, hicon as LPARAM);
            SendMessageW(console_window, WM_SETICON, ICON_SMALL as WPARAM, hicon as LPARAM);
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
