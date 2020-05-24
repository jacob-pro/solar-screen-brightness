use serde::{Deserialize};
use config::{Config, ConfigError, File, FileFormat};

#[derive(Debug, Deserialize)]
pub struct Settings {


}

impl Settings {

    pub fn load() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("asdf").format(FileFormat::Toml))?;
        s.try_into()
    }

    pub fn save(&self) {

    }

}
