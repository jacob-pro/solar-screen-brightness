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

fn main() -> Result<(), &'static str> {
    check_for_duplicate()?;
    let config = Config::load().unwrap_or(Config::default());
    let (sender, status) = start_loop(config);
    let app = TrayApplication::create(sender.clone(), status);
    app.run();
    sender.send(BrightnessLoopMessage::Exit).unwrap();
    Ok(())
}

fn check_for_duplicate() -> Result<(), &'static str> {
    //Err("Already running")
    Ok(())
}
