//! Detects system events such as:
//! - Monitor connect/disconnect events
//! - Messages from another process to show the main window

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        use self::linux as platform;
    } else if #[cfg(windows)] {
        pub mod windows;
        use self::windows as platform;
    } else {
        compile_error!("unsupported platform");
    }
}

pub use platform::EventWatcher;
