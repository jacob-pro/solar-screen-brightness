#[cfg_attr(not(target_os = "windows"), path = "unix.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod console_impl;

pub use console_impl::*;