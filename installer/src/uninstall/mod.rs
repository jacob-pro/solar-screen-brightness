#[cfg(windows)]
#[path = "windows.rs"]
mod uninstall_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod uninstall_platform;
