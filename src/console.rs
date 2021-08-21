use crate::assets::Assets;
use crate::controller::StateRef;
use crate::tray::TrayMessageSender;
use crate::tui::run;
use crate::wide::WideString;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::*;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::{FreeConsole, GetConsoleWindow, SetConsoleTitleW};
use winapi::um::winuser::*;

pub struct Console {}

impl Console {
    pub fn create(tray: TrayMessageSender, state: StateRef) -> Self {
        unsafe {
            assert_eq!(AllocConsole(), TRUE);
            let console_window = GetConsoleWindow();
            assert_ne!(console_window, NULL as HWND);
            let console_menu = GetSystemMenu(console_window, FALSE);
            EnableMenuItem(console_menu, SC_CLOSE as u32, MF_ENABLED | MF_GRAYED);
            SetConsoleTitleW("Solar Screen Brightness".to_wide().as_ptr());

            let mut asset = Assets::get("icon-256.png")
                .expect("Icon missing")
                .into_owned();
            let hicon =
                CreateIconFromResource(asset.as_mut_ptr(), asset.len() as u32, TRUE, 0x00030000);
            assert_ne!(hicon, NULL as HICON);
            SendMessageW(
                console_window,
                WM_SETICON,
                ICON_BIG as WPARAM,
                hicon as LPARAM,
            );
            SendMessageW(
                console_window,
                WM_SETICON,
                ICON_SMALL as WPARAM,
                hicon as LPARAM,
            );
        }
        std::thread::spawn(move || {
            run(tray, state);
        });
        Console {}
    }

    pub fn show(&self) {
        unsafe {
            let console_window = GetConsoleWindow();
            assert_ne!(console_window, NULL as HWND);
            ShowWindow(console_window, SW_RESTORE);
            BringWindowToTop(console_window);
            SetForegroundWindow(console_window);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let console_window = GetConsoleWindow();
            assert_ne!(console_window, NULL as HWND);
            ShowWindow(console_window, SW_HIDE);
        }
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(FreeConsole(), TRUE);
        }
    }
}
