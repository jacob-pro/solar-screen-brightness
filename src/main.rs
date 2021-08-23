#![windows_subsystem = "windows"]

#[macro_use]
extern crate validator_derive;

mod assets;
mod brightness;
mod config;
mod console;
mod controller;
mod tray;
mod tui;
#[cfg(target_os = "windows")]
mod wide;

use crate::config::Config;
use crate::controller::BrightnessController;
// use crate::wide::WideString;

// use solar_screen_brightness_windows_bindings::Windows::Win32::{
//     Foundation::{BOOL, HWND, PWSTR},
//     System::Diagnostics::Debug::{GetLastError, SetLastError, WIN32_ERROR},
//     System::Threading::CreateMutexW,
// };

fn main() {
    // if already_running() {
    //     panic!("Already running")
    // };
    let config = Config::load().ok().unwrap_or_default();
    let mut controller = BrightnessController::new(config);
    controller.start();
    tray::run(controller);
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
