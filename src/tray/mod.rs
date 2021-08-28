#[cfg(not(target_os = "windows"))]
#[path = "qt.rs"]
mod tray_impl;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod tray_impl;

pub use tray_impl::run;
use tray_impl::TrayApplicationHandleImpl as Inner;

#[derive(Clone)]
pub struct TrayApplicationHandle(Inner);

impl TrayApplicationHandle {
    pub fn close_console(&self) {
        self.0.close_console();
    }

    pub fn exit_application(&self) {
        self.0.exit_application();
    }
}
