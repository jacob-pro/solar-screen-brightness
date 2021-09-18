#[cfg(windows)]
#[path = "windows.rs"]
mod install_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod install_platform;


use crate::assets::BuildAssets;
use anyhow::Context;

pub fn install() -> anyhow::Result<()> {
    log::info!("Starting install");
    log::info!("Writing binary {}", crate::BINARY_PATH.to_str().unwrap());
    let binary = BuildAssets::get("solar-screen-brightness").unwrap();
    std::fs::write(&*crate::BINARY_PATH, &binary.data).context("Writing binary file")?;
    install_platform::install()?;
    log::info!("Completed install");
    Ok(())
}
