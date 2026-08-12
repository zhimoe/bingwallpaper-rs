use crate::setter::WallpaperSetter;
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};
use windows_registry::CURRENT_USER;

pub struct Win32WallpaperSetter;

impl Win32WallpaperSetter {
    pub fn new() -> Self {
        Win32WallpaperSetter
    }
}

#[cfg(windows)]
impl WallpaperSetter for Win32WallpaperSetter {
    fn set(&self, path: &str, _args: &[String]) -> bool {
        let image_full_path = path.replace('/', "\\");

        // 设置壁纸样式 (10 = 填充，0 = 居中等)
        if let Ok(key) = CURRENT_USER.open("Control Panel\\Desktop") {
            let _ = key.set_string("WallpaperStyle", "10");
            let _ = key.set_string("TileWallpaper", "0");
        }

        // Windows API 大多使用 UTF-16 编码的宽字符 (Wide String)
        // 将 &str 转换为 Vec<u16>，并在末尾添加 null 终止符
        let wide_path: Vec<u16> = image_full_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // 调用 Windows API
        // SystemParametersInfoW 的参数说明：
        // 1. SPI_SETDESKWALLPAPER: 表示我们要设置壁纸
        // 2. 0: 未使用 (对于设置壁纸通常是 0)
        // 3. wide_path.as_ptr() as PCWSTR: 指向图片路径的指针
        // 4. SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE: 更新配置文件并广播系统环境改变的消息，使设置立即生效
        let result = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_path.as_ptr() as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(SPIF_UPDATEINIFILE.0 | SPIF_SENDCHANGE.0),
            )
        };
        result.is_ok()
    }
}

#[cfg(not(windows))]
impl WallpaperSetter for Win32WallpaperSetter {
    fn set(&self, _path: &str, _args: &[String]) -> bool {
        log::warn!("Win32WallpaperSetter is a stub on non-Windows platforms");
        true
    }
}
