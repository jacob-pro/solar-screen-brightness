use crate::install::BINARY_PATH;
use crate::APP_NAME;
use lazy_static::lazy_static;
use solar_screen_brightness_windows::windows::Guid;
use solar_screen_brightness_windows::windows::Interface;
use solar_screen_brightness_windows::Windows::Win32::{
    System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    },
    UI::Shell::IShellLinkW,
};
use std::path::{Path, PathBuf};

lazy_static! {
    static ref START_MENU: PathBuf =
        PathBuf::from(std::env::var("APPDATA").expect("%APPDATA% not set"))
            .join(r#"Microsoft\Windows\Start Menu\Programs"#);
    pub static ref START_MENU_SHORTCUT: PathBuf = START_MENU.join(APP_NAME).with_extension("lnk");
    static ref STARTUP_SHOTCUT_NAME: String = format!("{} (Minimised)", APP_NAME);
    pub static ref STARTUP_SHORTCUT: PathBuf = START_MENU
        .join("Startup")
        .join(STARTUP_SHOTCUT_NAME.as_str())
        .with_extension("lnk");
}

// https://github.com/retep998/winapi-rs/issues/986#issuecomment-818749293
const CLSID_SHELLLINK: Guid = Guid::from_values(
    0x00021401,
    0x0000,
    0x0000,
    [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
);

pub fn install() -> anyhow::Result<()> {
    log::info!(
        "Creating start menu shortcut {}",
        START_MENU_SHORTCUT.to_str().unwrap()
    );
    com_initialise()?;
    create_shortcut(
        BINARY_PATH.as_path(),
        None,
        APP_NAME,
        START_MENU_SHORTCUT.as_path(),
    )?;
    log::info!(
        "Enabling launch at start up {}",
        STARTUP_SHORTCUT.to_str().unwrap()
    );
    create_shortcut(
        BINARY_PATH.as_path(),
        Some("launch --hide-console"),
        &STARTUP_SHOTCUT_NAME,
        STARTUP_SHORTCUT.as_path(),
    )?;
    Ok(())
}

fn com_initialise() -> anyhow::Result<()> {
    unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED).map_err(|e| e.into()) }
}

fn create_shortcut(
    to: &Path,
    args: Option<&str>,
    description: &str,
    save_to: &Path,
) -> anyhow::Result<()> {
    unsafe {
        let psl: IShellLinkW = CoCreateInstance(&CLSID_SHELLLINK, None, CLSCTX_INPROC_SERVER)?;
        psl.SetPath(to.to_str().unwrap())?;
        psl.SetDescription(description)?;
        match args {
            None => {}
            Some(args) => {
                psl.SetArguments(args)?;
            }
        }
        let ppf: IPersistFile = psl.cast()?;
        log::info!("{}", save_to.to_str().unwrap());
        ppf.Save(save_to.to_str().unwrap(), true)?;
    }
    Ok(())
}
