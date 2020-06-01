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

use crate::tray::TrayApplication;
use crate::config::Config;
use crate::brightness::{start_loop, BrightnessLoopMessage};

fn main() {
    let config = Config::load().unwrap_or(Config::default());
    let brightness_loop = start_loop(config);
    let app = TrayApplication::create();
    app.run();
    brightness_loop.send(BrightnessLoopMessage::Exit).unwrap();
}
