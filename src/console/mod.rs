#[cfg(not(target_os = "windows"))]
#[path = "unix.rs"]
mod console_impl;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod console_impl;

pub use console_impl::*;
