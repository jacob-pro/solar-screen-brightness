use crate::common::{refresh_desktop, remove_file_if_exists, AUTOSTART_ENTRY, DESKTOP_ENTRY};

pub fn uninstall() -> anyhow::Result<()> {
    remove_file_if_exists(AUTOSTART_ENTRY.as_path())?;
    remove_file_if_exists(DESKTOP_ENTRY.as_path())?;

    if let Err(e) = refresh_desktop() {
        log::warn!("Failed to refresh desktop {:#}", e)
    }

    Ok(())
}
