#[cfg(windows)]
#[path = "windows.rs"]
mod uninstall_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod uninstall_platform;

use crate::install::CONFIG_DIR;
use std::path::Path;

pub fn uninstall() -> anyhow::Result<()> {
    log::info!("Starting uninstall");
    log::info!("Deleting folder {}", CONFIG_DIR.to_str().unwrap());
    std::fs::remove_dir_all(CONFIG_DIR.as_path())?;
    uninstall_platform::uninstall()?;
    Ok(())
}

pub fn remove_file_if_exists<P: AsRef<Path>>(p: P) -> std::io::Result<()> {
    if p.as_ref().is_file() {
        std::fs::remove_file(p)?
    }
    Ok(())
}
