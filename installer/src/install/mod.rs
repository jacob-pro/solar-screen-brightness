#[cfg(windows)]
#[path = "windows.rs"]
mod install_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod install_platform;

use crate::assets::BuildAssets;
use crate::common::{ensure_not_running, remove_file_if_exists, BINARY_PATH};
use anyhow::Context;
use std::process::{Command, Stdio};

pub fn install() -> anyhow::Result<()> {
    log::info!("Starting install");
    ensure_not_running();
    remove_file_if_exists(&*BINARY_PATH)?;

    log::info!("Writing binary {}", BINARY_PATH.to_str().unwrap());
    std::fs::create_dir_all(BINARY_PATH.parent().unwrap())
        .context("Ensuring config folder exists")?;
    let binary = BuildAssets::get("solar-screen-brightness").unwrap();
    std::fs::write(&*BINARY_PATH, &binary.data).context("Writing binary file")?;

    install_platform::install()?;
    log::info!("Completed install");
    Ok(())
}

pub fn launch() -> anyhow::Result<()> {
    log::info!("Attempting to launch app");
    Command::new(BINARY_PATH.as_path())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;
    Ok(())
}
