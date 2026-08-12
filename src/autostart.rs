#[cfg(windows)]
pub fn install_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use windows_registry::CURRENT_USER;

    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
    let value = format!("\"{}\" --background", exe_path);
    let key = CURRENT_USER.create("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    key.set_string("BingWallpaper", &value)?;
    log::info!("autostart installed: {}", value);
    Ok(())
}

#[cfg(windows)]
pub fn uninstall_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use windows_registry::CURRENT_USER;
    let key = CURRENT_USER.create("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    key.remove_value("BingWallpaper")?;
    log::info!("autostart uninstalled");
    Ok(())
}

#[cfg(not(windows))]
pub fn install_autostart() -> Result<(), Box<dyn std::error::Error>> {
    log::warn!("autostart is only supported on Windows");
    Ok(())
}

#[cfg(not(windows))]
pub fn uninstall_autostart() -> Result<(), Box<dyn std::error::Error>> {
    log::warn!("autostart is only supported on Windows");
    Ok(())
}
