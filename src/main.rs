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

use crate::tray::TrayApplication;
use crate::config::Config;

fn main() {
    let config = Config::load().unwrap_or(Config::default());
    config.save();
    println!("{:?}", config);
    let app = TrayApplication::create();
    app.run();
}
