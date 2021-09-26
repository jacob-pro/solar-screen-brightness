use crate::common::{BINARY_NAME, CONFIG_DIR};
use directories::BaseDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::process::{Command, Stdio};

lazy_static! {
    static ref HOME_DIR: PathBuf = BaseDirs::new()
        .expect("Couldn't find home directory")
        .home_dir()
        .to_path_buf();
    pub static ref DESKTOP_ENTRY: PathBuf = HOME_DIR
        .join(".local/share/applications")
        .join(BINARY_NAME)
        .with_extension("desktop");
    pub static ref AUTOSTART_ENTRY: PathBuf = HOME_DIR
        .join(".config/autostart")
        .join(BINARY_NAME)
        .with_extension("desktop");
    pub static ref ICON_PATH: PathBuf = CONFIG_DIR.join("icon.png");
}

pub fn refresh_desktop() -> anyhow::Result<()> {
    log::info!("Refreshing desktop");
    let output = Command::new("xdg-desktop-menu")
        .arg("forceupdate")
        .output()?;
    if !output.status.success() {
        let std_err = std::str::from_utf8(&output.stderr)?;
        anyhow::bail!("{}", std_err);
    }
    Ok(())
}

pub fn ensure_not_running() {
    log::info!("Ensuring {} not running", BINARY_NAME);
    Command::new("killall")
        .arg(BINARY_NAME)
        .stderr(Stdio::null())
        .status()
        .ok();
}
