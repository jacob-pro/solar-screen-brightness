#[cfg(windows)]
#[path = "windows.rs"]
mod uninstall_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod uninstall_platform;

use crate::common::CONFIG_DIR;

pub fn uninstall() -> anyhow::Result<()> {
    log::info!("Starting uninstall");
    uninstall_platform::uninstall()?;
    crate::common::ensure_not_running();
    log::info!("Deleting folder {}", CONFIG_DIR.to_str().unwrap());
    std::fs::remove_dir_all(CONFIG_DIR.as_path())?;
    Ok(())
}
