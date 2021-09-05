#[cfg(not(windows))]
#[path = "bearlib.rs"]
mod console_impl;

#[cfg(windows)]
#[path = "windows.rs"]
mod console_impl;

use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;

pub struct Console(console_impl::Console);

impl Console {
    pub fn new(tray: TrayApplicationHandle, controller: BrightnessController) -> Self {
        Self(console_impl::Console::new(tray, controller))
    }

    pub fn show(&mut self) {
        log::info!("Showing console");
        self.0.show();
    }

    pub fn hide(&mut self) {
        log::info!("Hiding console");
        self.0.hide();
    }
}
