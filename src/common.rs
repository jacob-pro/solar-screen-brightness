//! Common constants and helper functions used in both CLI and GUI applications

use anyhow::Context;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, SharedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs;
use std::fs::File;
use std::path::PathBuf;

pub const APP_NAME: &str = "Solar Screen Brightness";

pub const APP_DIRECTORY_NAME: &str = "solar-screen-brightness";

/// Path to the local application data folder
/// This is where the SSB logs will be stored
pub fn local_data_directory() -> PathBuf {
    let path = dirs::data_local_dir()
        .expect("Unable to get data_local_dir()")
        .join(APP_DIRECTORY_NAME);
    fs::create_dir_all(&path).expect("Unable to create data directory");
    path
}

/// Path to the local application config folder
/// This is where the SSB config will be stored
pub fn config_directory() -> PathBuf {
    let path = dirs::config_local_dir()
        .expect("Unable to get data_local_dir()")
        .join(APP_DIRECTORY_NAME);
    fs::create_dir_all(&path).expect("Unable to create local config directory");
    path
}

pub fn install_logger(debug: bool, to_disk: bool) -> anyhow::Result<()> {
    let filter = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let config = simplelog::ConfigBuilder::default()
        .add_filter_ignore_str("wgpu")
        .add_filter_ignore_str("naga")
        .set_target_level(LevelFilter::Debug)
        .build();
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        filter,
        config.clone(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )];
    if to_disk {
        let file = File::create(get_log_path()).context("Unable to create log file")?;
        let file_logger = WriteLogger::new(filter, config, file);
        loggers.push(file_logger);
    }
    CombinedLogger::init(loggers)?;
    if debug {
        log::warn!("Debug logging enabled");
    }
    Ok(())
}

pub fn get_log_path() -> PathBuf {
    local_data_directory().join("log.txt")
}
