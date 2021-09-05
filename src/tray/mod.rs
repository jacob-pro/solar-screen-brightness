#[cfg(not(windows))]
#[path = "qt.rs"]
mod tray_impl;

#[cfg(windows)]
#[path = "windows.rs"]
mod tray_impl;

use crate::controller::BrightnessController;
use crate::cursive::Cursive;

#[derive(Clone)]
pub struct TrayApplicationHandle(tray_impl::Handle);

impl TrayApplicationHandle {
    #[inline]
    pub fn close_console(&self, cursive: &mut Cursive) {
        log::info!("Sending close console message to tray");
        self.0.close_console(cursive);
    }

    #[inline]
    pub fn exit_application(&self) {
        log::info!("Sending exit application message to tray");
        self.0.exit_application();
    }
}

/// Blocking call, runs on this thread
pub fn run_tray_application(controller: BrightnessController, launch_console: bool) {
    log::info!("Launching tray application");
    tray_impl::run(controller, launch_console);
    log::info!("Tray application stopping");
}

pub fn show_console_in_another_process() {
    log::info!("Attempting to show the already running application");
    tray_impl::show_console_in_another_process();
}
