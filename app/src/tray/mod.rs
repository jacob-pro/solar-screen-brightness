#[cfg(unix)]
#[path = "qt.rs"]
mod tray_impl;

#[cfg(windows)]
#[path = "windows.rs"]
mod tray_impl;

use crate::controller::BrightnessController;
use crate::cursive::Cursive;
use crate::lock::ApplicationLock;

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
pub fn run_tray_application(
    controller: BrightnessController,
    lock: ApplicationLock,
    launch_console: bool,
) {
    log::info!("Launching tray application");
    tray_impl::run(controller, lock, launch_console);
    log::info!("Tray application stopping");
}

#[cfg(windows)]
pub use tray_impl::show_console_in_owning_process;