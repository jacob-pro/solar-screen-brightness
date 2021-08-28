#[cfg(not(target_os = "windows"))]
#[path = "unix.rs"]
mod console_impl;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod console_impl;

use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use console_impl::ConsoleImpl as Inner;

pub struct Console(Inner);

impl Console {
    pub fn new(tray: TrayApplicationHandle, controller: BrightnessController) -> Self {
        Self(Inner::new(tray, controller))
    }

    pub fn show(&mut self) {
        self.0.show();
    }

    pub fn hide(&self) {
        self.0.hide();
    }
}
