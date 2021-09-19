#[cfg(windows)]
#[path = "windows.rs"]
pub mod install_platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod install_platform;

use crate::assets::BuildAssets;
use crate::APP_NAME;
use anyhow::Context;
use directories::BaseDirs;
use lazy_static::lazy_static;
use std::env::consts::EXE_EXTENSION;
use std::path::PathBuf;
use std::process::{Command, Stdio};

lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let p = BaseDirs::new().unwrap().config_dir().join(APP_NAME);
        log::trace!("Ensuring {:?} folder exists", p);
        std::fs::create_dir_all(&p).unwrap();
        p
    };
    pub static ref BINARY_PATH: PathBuf = CONFIG_DIR
        .join("solar-screen-brightness")
        .with_extension(EXE_EXTENSION);
}

pub fn install() -> anyhow::Result<()> {
    log::info!("Starting install");
    log::info!("Writing binary {}", BINARY_PATH.to_str().unwrap());
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
