use crate::common::{
    APP_NAME, BINARY_PATH, STARTUP_SHORTCUT, STARTUP_SHORTCUT_NAME, START_MENU_SHORTCUT,
};
use solar_screen_brightness_windows::windows::Interface;
use solar_screen_brightness_windows::Windows::Win32::{
    System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    },
    UI::Shell::{IShellLinkW, ShellLink},
};
use std::path::Path;

pub fn install() -> anyhow::Result<()> {
    com_initialise()?;

    create_shortcut(
        BINARY_PATH.as_path(),
        None,
        APP_NAME,
        START_MENU_SHORTCUT.as_path(),
    )?;

    create_shortcut(
        BINARY_PATH.as_path(),
        Some("launch --hide-console"),
        &STARTUP_SHORTCUT_NAME,
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
    log::info!("Creating shortcut {}", save_to.display());
    unsafe {
        let psl: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        psl.SetPath(to.to_str().unwrap())?;
        psl.SetDescription(description)?;
        match args {
            None => {}
            Some(args) => {
                psl.SetArguments(args)?;
            }
        }
        let ppf: IPersistFile = psl.cast()?;
        ppf.Save(save_to.to_str().unwrap(), true)?;
    }
    Ok(())
}
