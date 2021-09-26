use crate::common::{APP_NAME, BINARY_NAME};
use directories::BaseDirs;
use lazy_static::lazy_static;
use std::env::consts::EXE_EXTENSION;
use std::path::PathBuf;
use std::process::{Command, Stdio};

lazy_static! {
    static ref START_MENU: PathBuf = BaseDirs::new()
        .unwrap()
        .config_dir()
        .join(r#"Microsoft\Windows\Start Menu\Programs"#);
    pub static ref START_MENU_SHORTCUT: PathBuf = START_MENU.join(APP_NAME).with_extension("lnk");
    pub static ref STARTUP_SHORTCUT_NAME: String = format!("{} (Minimised)", APP_NAME);
    pub static ref STARTUP_SHORTCUT: PathBuf = START_MENU
        .join("Startup")
        .join(STARTUP_SHOTCUT_NAME.as_str())
        .with_extension("lnk");
}

pub fn ensure_not_running() {
    log::info!("Ensuring {}.{} not running", BINARY_NAME, EXE_EXTENSION);
    Command::new("Taskkill")
        .arg("/IM")
        .arg(format!("{}.{}", BINARY_NAME, EXE_EXTENSION).as_str())
        .arg("/f")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok();
}
