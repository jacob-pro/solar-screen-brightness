#![windows_subsystem = "windows"]

#[macro_use]
extern crate validator_derive;

mod assets;
mod brightness;
mod config;
mod console;
mod controller;
mod monitor;
mod tray;
mod tui;
mod wide;

use crate::config::Config;
use crate::controller::BrightnessController;
use crate::wide::WideString;
use std::panic::PanicInfo;
use std::process::exit;
use winapi::shared::minwindef::TRUE;
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::winuser::{MessageBoxW, MB_ICONSTOP, MB_OK};

fn main() {
    std::panic::set_hook(Box::new(handle_panic));
    if already_running() {
        panic!("Already running")
    };
    let config = Config::load().ok().unwrap_or_default();
    let mut controller = BrightnessController::new(config);
    controller.start();
    tray::run(&controller);
}

fn already_running() -> bool {
    unsafe {
        let name = "solar-screen-brightness".to_wide();
        SetLastError(0);
        CreateMutexW(NULL as LPSECURITY_ATTRIBUTES, TRUE, name.as_ptr());
        return GetLastError() == ERROR_ALREADY_EXISTS;
    }
}

// The console is being used by Crossterm so any output won't be visible
fn handle_panic(info: &PanicInfo) {
    unsafe {
        let title = "Fatal Error".to_wide();
        let text = format!("{}", info).as_str().to_wide();
        MessageBoxW(
            NULL as HWND,
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONSTOP,
        );
        exit(1);
    }
}
