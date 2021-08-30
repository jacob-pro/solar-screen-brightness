use config::{File, FileFormat};
use directories::BaseDirs;
use geocoding::GeocodingError;
use lazy_static::lazy_static;
use serde::__private::Formatter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, io};
use validator::{Validate, ValidationErrors};

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
    pub latitude: f64,
    #[validate(range(min = -180, max = 180))]
    pub longitude: f64,
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
    pub location: Option<Location>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        match (|| {
            let mut s = config::Config::new();
            s.merge(File::from(CONFIG_FILE.as_path()).format(FileFormat::Toml))
                .map_err(|e| e.to_string())?;
            let res: Config = s.try_into().map_err(|e| e.to_string())?;
            res.validate()
                .map_err(|e: ValidationErrors| e.to_string())?;
            Ok(res)
        })() {
            Ok(r) => {
                log::info!("Successfully loaded config file");
                Ok(r)
            }
            Err(e) => {
                log::error!("Failed to load config file: {}", e);
                Err(e)
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        match (|| {
            let toml = toml::to_string(self).unwrap();
            fs::write(CONFIG_FILE.as_path(), toml)
        })() {
            Ok(r) => {
                log::info!("Successfully saved config file");
                Ok(r)
            }
            Err(e) => {
                log::error!("Failed to save config file: {}", e);
                Err(e)
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            brightness_day: 100,
            brightness_night: 60,
            transition_mins: 40,
            location: None,
        }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("(Lat: {:.5}, Lon: {:.5})", self.latitude, self.longitude).as_str())
    }
}

impl Location {
    pub fn geocode_address<G>(coder: G, address: &str) -> Result<Self, String>
    where
        G: geocoding::Forward<f64>,
    {
        match (|| {
            let x = coder.forward(address).map_err(|x| match x {
                GeocodingError::Request(r) => format!("{}", r),
                _ => format!("{}", x),
            })?;
            match x.first() {
                None => Err("No matches found".to_owned()),
                Some(p) => {
                    let l = Location {
                        latitude: p.y(),
                        longitude: p.x(),
                    };
                    l.validate().map_err(|e: ValidationErrors| e.to_string())?;
                    Ok(l)
                }
            }
        })() {
            Ok(r) => {
                log::info!(
                    "Successfully found location: {} for search string: {}",
                    r,
                    address
                );
                Ok(r)
            }
            Err(e) => {
                log::error!("Failed to find location for: {} with error: {}", address, e);
                Err(e)
            }
        }
    }
}
