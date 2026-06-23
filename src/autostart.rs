#[cfg(windows)]
pub fn install_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use windows::core::w;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
    };

    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
    let value = format!("\"{}\" -b", exe_path);
    let wide_value: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();

    unsafe {
        let mut hkey = HKEY::default();
        let sub_key = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        RegOpenKeyExW(HKEY_CURRENT_USER, sub_key, 0, KEY_SET_VALUE, &mut hkey)?;

        let name = w!("BingWallpaper");
        RegSetValueExW(
            hkey,
            name,
            0,
            REG_SZ,
            Some(wide_value.as_ptr() as *const u8),
            (wide_value.len() * 2) as u32,
        )?;
        let _ = RegCloseKey(hkey);
    }
    log::info!("autostart installed: {}", value);
    Ok(())
}

#[cfg(windows)]
pub fn uninstall_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use windows::core::w;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE,
    };

    unsafe {
        let mut hkey = HKEY::default();
        let sub_key = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        RegOpenKeyExW(HKEY_CURRENT_USER, sub_key, 0, KEY_SET_VALUE, &mut hkey)?;

        let name = w!("BingWallpaper");
        RegDeleteValueW(hkey, name)?;
        let _ = RegCloseKey(hkey);
    }
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
