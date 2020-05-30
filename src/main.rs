#![windows_subsystem = "windows"]

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

fn main() {
    let app = TrayApplication::create();
    app.run();
}
