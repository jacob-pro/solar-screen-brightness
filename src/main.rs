#![windows_subsystem = "windows"]

#[macro_use]
extern crate validator_derive;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod ssc;

mod assets;
mod tray;
mod wide;
mod console;
mod tui;
mod config;
mod brightness;
mod monitor;

use crate::config::Config;
use crate::brightness::BrightnessMessage;
use std::panic::PanicInfo;
use winapi::um::winuser::{MessageBoxW, MB_OK, MB_ICONSTOP};
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::NULL;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::shared::minwindef::TRUE;
use crate::wide::WideString;
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use std::process::exit;

fn main() {
    std::panic::set_hook(Box::new(handle_panic));
    if already_running() { panic!("Already running") };
    let config = Config::load().unwrap_or(Config::default());
    let (sender, status) = brightness::run(config);
    tray::run(sender.clone(), status);
    sender.send(BrightnessMessage::Exit).unwrap();
}

fn already_running() -> bool {
    unsafe {
        let name = "solar-screen-brightness".to_wide();
        SetLastError(0);
        CreateMutexW(NULL as LPSECURITY_ATTRIBUTES, TRUE,  name.as_ptr());
        return GetLastError() == ERROR_ALREADY_EXISTS
    }
}

// The console is being used by Crossterm so any output won't be visible
fn handle_panic(info: &PanicInfo) {
    unsafe {
        let title = "Fatal Error".to_wide();
        let text = format!("{}", info).as_str().to_wide();
        MessageBoxW(NULL as HWND, text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONSTOP);
        exit(1);
    }
}
