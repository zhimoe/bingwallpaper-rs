# BingWallpaper

一个使用 Rust 编写的 Bing 每日壁纸下载器。程序主要面向 Windows：可以下载每日壁纸、设置桌面背景，并通过当前用户注册表配置开机自启动；在 macOS 和 Linux 上也可使用 `--setter no` 仅下载图片。

## 功能

- 支持 `country`、`market`、日期偏移以及 global/china/custom 服务地址
- 支持 `prefer`、`highest`、`insist`、`never`、`uhd` 和 `manual` 分辨率策略
- 可额外收集伴随图片、普通视频和高清视频
- 支持单次运行与后台定时检查，后台遇到网络错误时会在 60 秒后重试
- 支持 INI 配置文件，优先级为“默认值 < 配置文件 < 命令行参数”
- 使用 JSON 保存最近一次下载记录，并可选写入 SQLite 数据库
- 支持 HTTP/HTTPS 代理和 Basic 代理认证
- Windows 下通过 Win32 API 设置壁纸，并通过 `HKCU` 管理自启动，无需管理员权限

## 构建

需要 Rust 1.85 或更高版本。安装 Rust 工具链后运行：

```powershell
cargo build --release
```

Windows 可执行文件位于：

```text
target\release\bingwallpaper.exe
```

## 快速使用

Windows 下默认进入后台循环并保持进程运行；需要只执行一次时请显式使用 `--foreground`：

```powershell
# 下载并设置今天的壁纸，然后退出
.\target\release\bingwallpaper.exe --foreground

# 只下载，不设置桌面背景
.\target\release\bingwallpaper.exe --foreground --setter no

# 使用中国市场并保留 Bing 原始文件名
.\target\release\bingwallpaper.exe --foreground --market zh-CN --keep-file-name

# 下载两天前的壁纸
.\target\release\bingwallpaper.exe --foreground --offset 2

# 后台运行，每 2 小时检查一次
.\target\release\bingwallpaper.exe --background --interval 2
```

macOS 或 Linux 上应使用：

```bash
cargo run -- --foreground --setter no
```

## 配置

默认配置文件是可执行文件同目录下的 `settings.conf`。可生成一份当前默认配置：

```powershell
.\target\release\bingwallpaper.exe --generate-config
```

也可以复制并修改仓库中的 [`settings.example.conf`](settings.example.conf)，再通过 `--config-file` 指定：

```powershell
.\target\release\bingwallpaper.exe --foreground --config-file .\settings.conf
```

常用配置项：

- `size_mode`: 分辨率策略；使用 `manual` 时需同时设置 `image_size`，例如 `2560x1440`
- `collect`: 逗号分隔的附加资源类型，可选 `accompany`、`video`、`hdvideo`
- `setter`: Windows 下使用 `win`，仅下载使用 `no`
- `server`: `global`、`china` 或 `custom`；使用 `custom` 时需设置 `customserver`
- `database_file`: 非空时把下载记录写入 SQLite；`database_no_image=true` 可避免在数据库内保存图片二进制

默认壁纸目录为 `%USERPROFILE%\MyBingWallpapers`，下载历史位于 `%APPDATA%\zhimoe\BingWallpaper\bing-wallpaper-history.json`。

代理密码会以明文形式保存在配置文件中，请限制该文件的访问权限。命令行中的 `--collect` 和 `--setter-args` 可重复传入；配置文件中则使用逗号分隔。

## Windows 自启动

安装自启动项：

```powershell
.\target\release\bingwallpaper.exe --install-autostart
```

删除自启动项：

```powershell
.\target\release\bingwallpaper.exe --uninstall-autostart
```

自启动项写入 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`。移动可执行文件后，需要重新执行安装命令以更新路径。

## 查看完整参数

```powershell
.\target\release\bingwallpaper.exe --help
```

## 开发与验证

```text
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

GitHub Actions 会在 Windows 环境构建 release 版本，并验证命令行、自启动和下载流程。
