#[cfg(not(target_os = "windows"))]
#[path = "qt.rs"]
mod tray_impl;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod tray_impl;

pub use tray_impl::run;
pub use tray_impl::TrayApplicationHandle;
