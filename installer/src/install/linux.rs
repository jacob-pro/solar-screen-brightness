use crate::assets::AppAssets;
use crate::common::{refresh_desktop, AUTOSTART_ENTRY, BINARY_PATH, DESKTOP_ENTRY, ICON_PATH};
use anyhow::Context;
use std::os::unix::fs::PermissionsExt;

pub fn install() -> anyhow::Result<()> {
    log::info!("Setting binary execute permissions");
    let perms = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(BINARY_PATH.as_path(), perms)?;

    log::info!("Writing icon {}", ICON_PATH.display());
    let binary = AppAssets::get("icon-256.png").unwrap();
    std::fs::write(&*ICON_PATH, &binary.data).context("Writing icon file")?;

    create_desktop_entry()?;
    create_autostart_entry()?;

    if let Err(e) = refresh_desktop() {
        log::warn!("Failed to refresh desktop {:#}", e)
    }

    Ok(())
}

// https://askubuntu.com/a/722182
// In the Exec= line, you are not allowed to use spaces, unless in case of an argument.
// In the Icon= line, you are allowed to use spaces:

fn create_desktop_entry() -> anyhow::Result<()> {
    log::info!("Creating desktop entry {}", DESKTOP_ENTRY.display());
    let data = format!(
        "[Desktop Entry]
Type=Application
Name=Solar Screen Brightness
Exec=\"{}\"
Icon={}
Terminal=false
    ",
        BINARY_PATH.display(),
        ICON_PATH.display()
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
Name=Solar Screen Brightness
Exec=\"{}\" launch --hide-console
Icon={}
Terminal=false
    ",
        BINARY_PATH.display(),
        ICON_PATH.display()
    );
    std::fs::create_dir_all(AUTOSTART_ENTRY.parent().unwrap())
        .context("Ensuring autostart folder exists")?;
    std::fs::write(&*AUTOSTART_ENTRY, &data).context("Writing autostart entry")?;
    Ok(())
}
