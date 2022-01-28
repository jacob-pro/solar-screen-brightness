#[cfg(windows)]
#[path = "windows.rs"]
mod common_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod common_platform;

pub use common_platform::*;

use directories::BaseDirs;
use lazy_static::lazy_static;
use std::env::consts::EXE_EXTENSION;
use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "Solar Screen Brightness";
pub const BINARY_NAME: &str = "solar-screen-brightness";

lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = BaseDirs::new().unwrap().config_dir().join(APP_NAME);
    pub static ref BINARY_PATH: PathBuf =
        CONFIG_DIR.join(BINARY_NAME).with_extension(EXE_EXTENSION);
}

pub fn remove_file_if_exists<P: AsRef<Path>>(p: P) -> std::io::Result<()> {
    if p.as_ref().is_file() {
        log::info!("Removing file {}", p.as_ref().display());
        std::fs::remove_file(p)?
    }
    Ok(())
}
