use crate::common::{refresh_desktop, AUTOSTART_ENTRY, BINARY_PATH, DESKTOP_ENTRY};
use anyhow::Context;
use std::os::unix::fs::PermissionsExt;

pub fn install() -> anyhow::Result<()> {
    log::info!("Setting binary execute permissions");
    let perms = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(BINARY_PATH.as_path(), perms)?;

    create_desktop_entry()?;
    create_autostart_entry()?;

    if let Err(e) = refresh_desktop() {
        log::warn!("Failed to refresh desktop {:#}", e)
    }

    Ok(())
}

fn create_desktop_entry() -> anyhow::Result<()> {
    log::info!("Creating desktop entry {}", DESKTOP_ENTRY.display());
    let data = format!(
        "[Desktop Entry]
Type=Application
Name=\"Solar Screen Brightness\"
Exec=\"{}\"
Terminal=false
    ",
        BINARY_PATH.display()
    );
    std::fs::create_dir_all(DESKTOP_ENTRY.parent().unwrap())
        .context("Ensuring applications folder exists")?;
    std::fs::write(&*DESKTOP_ENTRY, &data).context("Writing desktop entry")?;
    Ok(())
}

fn create_autostart_entry() -> anyhow::Result<()> {
    log::info!("Creating autostart entry {}", AUTOSTART_ENTRY.display());
    let data = format!(
        "[Desktop Entry]
Type=Application
Name=\"Solar Screen Brightness\"
Exec=\"{} launch --hide-console\"
Terminal=false
Hidden=true
X-GNOME-Autostart-enabled=true
    ",
        BINARY_PATH.display()
    );
    std::fs::create_dir_all(AUTOSTART_ENTRY.parent().unwrap())
        .context("Ensuring autostart folder exists")?;
    std::fs::write(&*AUTOSTART_ENTRY, &data).context("Writing autostart entry")?;
    Ok(())
}
