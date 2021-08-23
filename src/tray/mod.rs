#[cfg_attr(not(target_os = "windows"), path = "qt.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod tray_impl;

pub use tray_impl::*;

pub trait TrayApplicationHandle {
    fn close_console(&self);
    fn exit_application(&self);
}
