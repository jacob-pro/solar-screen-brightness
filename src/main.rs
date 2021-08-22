#![windows_subsystem = "windows"]

#[macro_use]
extern crate validator_derive;

mod assets;
mod brightness;
mod config;
// mod console;
mod controller;
mod tray;
mod tui;
// mod wide;

use crate::config::Config;
use crate::controller::BrightnessController;
// use crate::wide::WideString;
use std::panic::PanicInfo;
use std::process::exit;

// use solar_screen_brightness_windows_bindings::Windows::Win32::{
//     Foundation::{BOOL, HWND, PWSTR},
//     System::Diagnostics::Debug::{GetLastError, SetLastError, WIN32_ERROR},
//     System::Threading::CreateMutexW,
//     UI::WindowsAndMessaging::{MessageBoxW, MB_ICONSTOP, MB_OK},
// };

fn main() {
    // std::panic::set_hook(Box::new(handle_panic));
    // if already_running() {
    //     panic!("Already running")
    // };
    let config = Config::load().ok().unwrap_or_default();
    let mut controller = BrightnessController::new(config);
    controller.start();
    tray::run(&controller);
}

// fn already_running() -> bool {
//     const ERROR_ALREADY_EXISTS: WIN32_ERROR = WIN32_ERROR(183);
//     unsafe {
//         let mut name = "solar-screen-brightness".to_wide();
//         SetLastError(0);
//         CreateMutexW(
//             std::ptr::null_mut(),
//             BOOL::from(true),
//             PWSTR(name.as_mut_ptr()),
//         );
//         return GetLastError() == ERROR_ALREADY_EXISTS;
//     }
// }

// The console is being used by Crossterm so any output won't be visible
// fn handle_panic(info: &PanicInfo) {
//     unsafe {
//         let mut title = "Fatal Error".to_wide();
//         let mut text = format!("{}", info).as_str().to_wide();
//         MessageBoxW(
//             HWND::NULL,
//             PWSTR(text.as_mut_ptr()),
//             PWSTR(title.as_mut_ptr()),
//             MB_OK | MB_ICONSTOP,
//         );
//         exit(1);
//     }
// }
