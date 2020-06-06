use serde::{Deserialize, Serialize};
use config::{File, FileFormat};
use validator::{Validate, ValidationErrors};
use directories::BaseDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::{fs, io};

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

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Location {
    #[validate(range(min = -90, max = 90))]
    pub latitude: f32,
    #[validate(range(min = -180, max = 180))]
    pub longitude: f32,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Config {
    #[validate(range(max = 100))]
    pub brightness_day: u32,
    #[validate(range(max = 100))]
    pub brightness_night: u32,
    #[validate(range(max = 360))]
    pub transition_mins: u32,
    #[validate]
    pub location: Location,
}

impl Config {

    pub fn load() -> Result<Self, String> {
        let mut s = config::Config::new();
        s.merge(File::from(CONFIG_FILE.as_path()).format(FileFormat::Toml))
            .map_err(|e| e.to_string())?;
        let res: Config = s.try_into().map_err(|e| e.to_string())?;
        res.validate().map_err(|e: ValidationErrors| e.to_string())?;
        Ok(res)
    }

    pub fn save(&self) -> io::Result<()> {
        let toml = toml::to_string(self).unwrap();
        fs::write(CONFIG_FILE.as_path(), toml)
    }

}
impl Default for Config {
    fn default() -> Self {
        Config {
            brightness_day: 100,
            brightness_night: 60,
            transition_mins: 40,
            location: Location{ latitude: 0.0, longitude: 0.0 },
        }
    }
}
