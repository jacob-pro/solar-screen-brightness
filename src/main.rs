#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;

mod assets;
mod shell_icon;
mod str_ext;

// use cursive::views::{Dialog, TextView};
// use cursive::Cursive;
use winapi::um::consoleapi::{AllocConsole, SetConsoleCtrlHandler};
use winapi::shared::minwindef::{DWORD, BOOL, TRUE, FALSE};
use winapi::shared::windef::HWND;
use winapi::um::wincon::{FreeConsole, GetConsoleWindow};
use crate::shell_icon::ShellIcon;
use std::sync::Mutex;
use winapi::shared::ntdef::NULL;
use winapi::um::winuser::{GetSystemMenu, EnableMenuItem, SC_CLOSE, MF_ENABLED, MF_GRAYED, ShowWindow, SW_RESTORE, BringWindowToTop, SetForegroundWindow};


fn main() {


    let icon = ShellIcon::create();
    icon.drive();

}

// When the console window is being closed
unsafe extern "system" fn ctrl_handler(_dw_ctrl_type: DWORD) -> BOOL {
    let mut visible = UI_VISIBLE.lock().unwrap();
    FreeConsole();
    *visible = false;
    TRUE
}

lazy_static! {
    static ref UI_VISIBLE: Mutex<bool> = Mutex::new(false);
}

pub fn display_ui() {

    let mut visible = UI_VISIBLE.lock().unwrap();
    if *visible {
        unsafe {
            let console_window = GetConsoleWindow();
            if console_window == NULL as HWND { panic!("Null console window") };
            ShowWindow(console_window, SW_RESTORE);
            BringWindowToTop(console_window);
            SetForegroundWindow(console_window);
        }
        return;
    }

    unsafe {
        if AllocConsole() != TRUE {panic!("Error opening console")};
        if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) != TRUE {panic!("Error setting handler")};
        let console_window = GetConsoleWindow();
        if console_window == NULL as HWND { panic!("Null console window") };
        let console_menu = GetSystemMenu(console_window, FALSE);
        EnableMenuItem(console_menu, SC_CLOSE as u32, MF_ENABLED | MF_GRAYED);
    }

    // std::thread::spawn(|| {
    //     // Creates the cursive root - required for every application.
    //     let mut siv = Cursive::crossterm().unwrap();
    //
    //     // Creates a dialog with a single "Quit" button
    //     siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
    //         .title("Cursive")
    //         .button("Quit", |s| s.quit()));
    //
    //
    //     // Starts the event loop.
    //     siv.run();
    // });

    *visible = true;
}
