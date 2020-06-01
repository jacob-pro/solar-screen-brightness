use serde::{Deserialize, Serialize};
use config::{File, FileFormat};
use validator::{Validate, ValidationErrors};
use directories::BaseDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::fs;

lazy_static! {
    static ref CONFIG_DIR: PathBuf = {
        let mut p = BaseDirs::new().unwrap().config_dir().to_owned();
        p.push("Solar Screen Brightness");
        fs::create_dir_all(&p).unwrap();
        p
    };
    static ref CONFIG_FILE: PathBuf = {
        let mut base: PathBuf = CONFIG_DIR.clone();
        base.push("config");
        base.set_extension("toml");
        base
    };
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Config {
    #[validate(range(min = -90, max = 90))]
    latitude: f32,
    #[validate(range(min = -180, max = 180))]
    longitude: f32,
    #[validate(range(min = 0, max = 100))]
    brightness_day: f32,
    #[validate(range(min = 0, max = 100))]
    brightness_night: f32,
    #[validate(range(min = 0, max = 360))]
    transition_mins: u32,
    #[validate(range(min = 0.01, max = 10))]
    refresh_difference: f32,
}



impl Config {

    pub fn load() -> Result<Self, String> {
        let mut s = config::Config::new();
        s.merge(File::from(CONFIG_FILE.as_path()).format(FileFormat::Toml)).map_err(|e| e.to_string())?;
        let res: Config = s.try_into().map_err(|e| e.to_string())?;
        res.validate().map_err(|e: ValidationErrors| e.to_string())?;
        Ok(res)
    }

    pub fn save(&self) {
        let toml = toml::to_string(self).unwrap();
        fs::write(CONFIG_FILE.as_path(), toml).unwrap();
    }

}
impl Default for Config {
    fn default() -> Self {
        Config {
            latitude: 0.0,
            longitude: 0.0,
            brightness_day: 80.0,
            brightness_night: 50.0,
            transition_mins: 40,
            refresh_difference: 0.25
        }
    }
}
