use crate::install::install_platform::{STARTUP_SHORTCUT, START_MENU_SHORTCUT};
use crate::uninstall::remove_file_if_exists;

pub fn uninstall() -> anyhow::Result<()> {
    log::info!(
        "Removing startup shortcut {}",
        STARTUP_SHORTCUT.to_str().unwrap()
    );
    remove_file_if_exists(STARTUP_SHORTCUT.as_path())?;

    log::info!(
        "Removing start menu shortcut {}",
        START_MENU_SHORTCUT.to_str().unwrap()
    );
    remove_file_if_exists(START_MENU_SHORTCUT.as_path())?;

    Ok(())
}
