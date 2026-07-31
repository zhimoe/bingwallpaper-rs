use crate::setter::WallpaperSetter;
use std::path::Path;

pub struct Win32WallpaperSetter;

impl Win32WallpaperSetter {
    pub fn new() -> Self {
        Win32WallpaperSetter
    }
}

#[cfg(windows)]
impl WallpaperSetter for Win32WallpaperSetter {
    fn set(&self, path: &str, _args: &[String]) -> bool {
        let in_path = path.replace('/', "\\");
        let dir = Path::new(&in_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        let bmp_path = format!("{}\\wallpaper.bmp", dir);

        if !in_path.to_lowercase().ends_with(".bmp") {
            if let Err(e) = convert_photo_to_bmp(&in_path, &bmp_path) {
                log::error!("failed to convert photo to bmp: {}", e);
                return false;
            }
        }

        unsafe {
            use windows::core::w;
            use windows::Win32::Foundation::ERROR_SUCCESS;
            use windows::Win32::System::Registry::{
                RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
                HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ,
            };
            use windows::Win32::UI::WindowsAndMessaging::{
                SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
            };

            let sub_key = w!("Control Panel\\Desktop");
            let mut hkey = HKEY::default();
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                sub_key,
                0,
                KEY_READ | KEY_SET_VALUE,
                &mut hkey,
            );
            if result != ERROR_SUCCESS {
                log::error!("RegOpenKeyExW failed");
                return false;
            }

            let value_name = w!("Wallpaper");
            let mut buf = vec![0u8; 1024];
            let mut buf_len = buf.len() as u32;
            let mut typ = 0u32;
            let last_value = match RegQueryValueExW(
                hkey,
                value_name,
                Some(&mut 0u32),
                Some(&mut typ),
                Some(&mut buf),
                Some(&mut buf_len),
            ) {
                Ok(()) if typ == REG_SZ.0 => {
                    let wide_slice = std::slice::from_raw_parts(buf.as_ptr() as *const u16, buf_len as usize / 2);
                    let s = String::from_utf16_lossy(wide_slice).trim_end_matches('\0').to_string();
                    Some(s)
                }
                _ => None,
            };

            if let Some(ref v) = last_value {
                let backup_name = w!("WallpaperBackup");
                let wide_backup: Vec<u16> = v.encode_utf16().chain(Some(0)).collect();
                let _ = RegSetValueExW(
                    hkey,
                    backup_name,
                    0,
                    REG_SZ,
                    Some(wide_backup.as_ptr() as *const u8),
                    (wide_backup.len() * 2) as u32,
                );
            }

            let tile_name = w!("TileWallpaper");
            let tile_val: Vec<u16> = "0".encode_utf16().chain(Some(0)).collect();
            let _ = RegSetValueExW(
                hkey,
                tile_name,
                0,
                REG_SZ,
                Some(tile_val.as_ptr() as *const u8),
                (tile_val.len() * 2) as u32,
            );

            let style_name = w!("WallpaperStyle");
            let style_val: Vec<u16> = "10".encode_utf16().chain(Some(0)).collect();
            let _ = RegSetValueExW(
                hkey,
                style_name,
                0,
                REG_SZ,
                Some(style_val.as_ptr() as *const u8),
                (style_val.len() * 2) as u32,
            );

            let wide_bmp: Vec<u16> = bmp_path.encode_utf16().chain(Some(0)).collect();
            let result = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_bmp.as_ptr() as *mut _),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );

            let _ = RegCloseKey(hkey);
            result.as_bool()
        }
    }
}

#[cfg(not(windows))]
impl WallpaperSetter for Win32WallpaperSetter {
    fn set(&self, _path: &str, _args: &[String]) -> bool {
        log::warn!("Win32WallpaperSetter is a stub on non-Windows platforms");
        true
    }
}

#[cfg(windows)]
fn convert_photo_to_bmp(inpath: &str, outpath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(inpath)?;
    img.save_with_format(outpath, image::ImageFormat::Bmp)?;
    Ok(())
}
