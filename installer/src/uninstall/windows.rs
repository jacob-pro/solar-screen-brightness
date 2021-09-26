use crate::common::{remove_file_if_exists, STARTUP_SHORTCUT, START_MENU_SHORTCUT};

pub fn uninstall() -> anyhow::Result<()> {
    remove_file_if_exists(STARTUP_SHORTCUT.as_path())?;
    remove_file_if_exists(START_MENU_SHORTCUT.as_path())?;

    Ok(())
}
