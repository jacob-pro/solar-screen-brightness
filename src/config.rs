//! SSB Config file definition

use crate::common::config_directory;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use validator::Validate;

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Location {
    #[validate(range(min = -90, max = 90))]
    pub latitude: f64,
    #[validate(range(min = -180, max = 180))]
    pub longitude: f64,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct SsbConfig {
    #[validate(range(max = 100))]
    pub brightness_day: u32,
    #[validate(range(max = 100))]
    pub brightness_night: u32,
    #[validate(range(max = 360))]
    pub transition_mins: u32,
    #[validate]
    pub location: Option<Location>,
    #[serde(default)]
    #[validate]
    pub overrides: Vec<MonitorOverride>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct MonitorOverride {
    pub pattern: String,
    pub key: String,
    #[validate]
    pub brightness: Option<BrightnessValues>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct BrightnessValues {
    #[validate(range(max = 100))]
    pub brightness_day: u32,
    #[validate(range(max = 100))]
    pub brightness_night: u32,
}

impl SsbConfig {
    pub fn load(path_override: Option<PathBuf>) -> anyhow::Result<Option<Self>> {
        let path = match path_override {
            None => get_default_config_path(),
            Some(p) => p,
        };
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&path)
            .context(format!("Unable to read file '{}'", path.display()))?;
        let config = serde_json::from_str::<SsbConfig>(&contents).context(format!(
            "Unable to deserialize config file '{}'",
            path.display()
        ))?;
        config
            .validate()
            .context(format!("Invalid config file '{}'", path.display()))?;
        Ok(Some(config))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = get_default_config_path();
        let serialised = serde_json::to_string_pretty(&self).unwrap();
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(serialised.as_bytes())?;
        temp_file.flush()?;
        temp_file.persist(&path)?;
        log::debug!("Successfully saved config to {}", path.display());
        Ok(())
    }
}

pub fn get_default_config_path() -> PathBuf {
    config_directory().join(CONFIG_FILE_NAME)
}

impl Default for SsbConfig {
    fn default() -> Self {
        SsbConfig {
            brightness_day: 100,
            brightness_night: 60,
            transition_mins: 40,
            location: None,
            overrides: vec![],
        }
    }
}
