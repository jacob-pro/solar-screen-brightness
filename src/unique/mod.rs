//! Utility to ensure a single instance of SSB is running

use std::fmt::Debug;
use thiserror::Error;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        use self::linux as platform;
    } else if #[cfg(windows)] {
        mod windows;
        use self::windows as platform;
    } else {
        compile_error!("unsupported platform");
    }
}

pub use platform::ExistingInstance;
pub use platform::SsbUniqueInstance;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to check if Solar Screen Brightness is already running: {0}")]
    PlatformError(#[source] Box<dyn std::error::Error>),
    #[error("Solar Screen Brightness is already running")]
    AlreadyRunning(ExistingInstance),
}
