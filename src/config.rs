use clap::Parser;
use configparser::ini::Ini;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "bingwallpaper",
    about = "Download the wallpaper offered by Bing.com and set it current wallpaper background.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct CliArgs {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[arg(long)]
    pub config_file: Option<String>,

    #[arg(long)]
    pub generate_config: bool,

    #[arg(short, long)]
    pub background: bool,

    #[arg(long)]
    pub foreground: bool,

    #[arg(short, long, value_name = "COUNTRY")]
    pub country: Option<String>,

    #[arg(long, value_name = "MARKET")]
    pub market: Option<String>,

    #[arg(long)]
    pub list_markets: bool,

    #[arg(short, long, value_name = "INTERVAL")]
    pub interval: Option<u64>,

    #[arg(short, long)]
    pub keep_file_name: bool,

    #[arg(short, long, value_name = "MODE")]
    pub size_mode: Option<String>,

    #[arg(long, value_name = "ITEM", action = clap::ArgAction::Append)]
    pub collect: Vec<String>,

    #[arg(long, value_name = "SIZE")]
    pub image_size: Option<String>,

    #[arg(short, long, value_name = "OFFSET")]
    pub offset: Option<i32>,

    #[arg(long, value_name = "URL")]
    pub proxy_server: Option<String>,

    #[arg(long, value_name = "PORT")]
    pub proxy_port: Option<u16>,

    #[arg(long, value_name = "USERNAME")]
    pub proxy_username: Option<String>,

    #[arg(long, value_name = "PASSWORD")]
    pub proxy_password: Option<String>,

    #[arg(long)]
    pub redownload: bool,

    #[arg(short, long, value_name = "SETTER")]
    pub setter: Option<String>,

    #[arg(long, value_name = "ARGS", action = clap::ArgAction::Append)]
    pub setter_args: Vec<String>,

    #[arg(short, long, value_name = "FOLDER")]
    pub output_folder: Option<String>,

    #[arg(long, value_name = "FILE")]
    pub database_file: Option<String>,

    #[arg(long)]
    pub database_no_image: bool,

    #[arg(long, value_name = "SERVER")]
    pub server: Option<String>,

    #[arg(long, value_name = "URL")]
    pub custom_server: Option<String>,

    #[arg(long)]
    pub install_autostart: bool,

    #[arg(long)]
    pub uninstall_autostart: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub config_file: String,
    pub generate_config: bool,
    pub background: bool,
    pub foreground: bool,
    pub country: String,
    pub market: String,
    pub list_markets: bool,
    pub debug: u8,
    pub interval: u64,
    pub keep_file_name: bool,
    pub size_mode: String,
    pub collect: Vec<String>,
    pub image_size: String,
    pub offset: i32,
    pub proxy_server: String,
    pub proxy_port: u16,
    pub proxy_username: String,
    pub proxy_password: String,
    pub redownload: bool,
    pub setter: String,
    pub setter_args: Vec<String>,
    pub output_folder: String,
    pub database_file: String,
    pub database_no_image: bool,
    pub server: String,
    pub customserver: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        let app_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        Config {
            config_file: app_dir.join("settings.conf").to_string_lossy().to_string(),
            generate_config: false,
            background: false,
            foreground: false,
            country: "auto".to_string(),
            market: "".to_string(),
            list_markets: false,
            debug: 0,
            interval: 2,
            keep_file_name: false,
            size_mode: "prefer".to_string(),
            collect: vec![],
            image_size: "".to_string(),
            offset: 0,
            proxy_server: "".to_string(),
            proxy_port: 80,
            proxy_username: "".to_string(),
            proxy_password: "".to_string(),
            redownload: false,
            setter: "win".to_string(),
            setter_args: vec![],
            output_folder: PathBuf::from(&home).join("MyBingWallpapers").to_string_lossy().to_string(),
            database_file: "".to_string(),
            database_no_image: false,
            server: "global".to_string(),
            customserver: "".to_string(),
        }
    }
}

fn parse_bool(s: &str) -> bool {
    !(s.is_empty() || s.eq_ignore_ascii_case("false") || s == "0")
}

fn parse_collect(s: &str) -> Vec<String> {
    s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

impl Config {
    pub fn from_cli_and_file(cli: &CliArgs) -> Self {
        let mut cfg = Config::default();

        // Determine config file path from CLI first
        if let Some(ref path) = cli.config_file {
            cfg.config_file = path.clone();
        }

        // Load from INI if exists
        if std::path::Path::new(&cfg.config_file).is_file() {
            let mut ini = Ini::new();
            if ini.load(&cfg.config_file).is_ok() {
                if let Some(v) = ini.get("Daemon", "background") {
                    cfg.background = parse_bool(&v);
                }
                if let Some(v) = ini.get("Daemon", "foreground") {
                    cfg.foreground = parse_bool(&v);
                }
                if let Some(v) = ini.get("Daemon", "interval") {
                    if let Ok(i) = v.parse::<u64>() {
                        cfg.interval = if i >= 1 { i } else { 1 };
                    }
                }
                if let Some(v) = ini.get("Download", "country") {
                    cfg.country = v;
                }
                if let Some(v) = ini.get("Download", "market") {
                    cfg.market = v;
                }
                if let Some(v) = ini.get("Download", "keep_file_name") {
                    cfg.keep_file_name = parse_bool(&v);
                }
                if let Some(v) = ini.get("Download", "size_mode") {
                    cfg.size_mode = v;
                }
                if let Some(v) = ini.get("Download", "collect") {
                    cfg.collect = parse_collect(&v);
                }
                if let Some(v) = ini.get("Download", "image_size") {
                    cfg.image_size = v;
                }
                if let Some(v) = ini.get("Download", "offset") {
                    if let Ok(i) = v.parse::<i32>() {
                        cfg.offset = i;
                    }
                }
                if let Some(v) = ini.get("Download", "output_folder") {
                    cfg.output_folder = v;
                }
                if let Some(v) = ini.get("Download", "redownload") {
                    cfg.redownload = parse_bool(&v);
                }
                if let Some(v) = ini.get("Download", "server") {
                    cfg.server = v;
                }
                if let Some(v) = ini.get("Download", "customserver") {
                    cfg.customserver = v;
                }
                if let Some(v) = ini.get("Proxy", "proxy_server") {
                    cfg.proxy_server = v;
                }
                if let Some(v) = ini.get("Proxy", "proxy_port") {
                    if let Ok(p) = v.parse::<u16>() {
                        cfg.proxy_port = p;
                    }
                }
                if let Some(v) = ini.get("Proxy", "proxy_username") {
                    cfg.proxy_username = v;
                }
                if let Some(v) = ini.get("Proxy", "proxy_password") {
                    cfg.proxy_password = v;
                }
                if let Some(v) = ini.get("Setter", "setter") {
                    cfg.setter = v;
                }
                if let Some(v) = ini.get("Setter", "setter_args") {
                    cfg.setter_args = parse_collect(&v);
                }
                if let Some(v) = ini.get("Database", "database_file") {
                    cfg.database_file = v;
                }
                if let Some(v) = ini.get("Database", "database_no_image") {
                    cfg.database_no_image = parse_bool(&v);
                }
                if let Some(v) = ini.get("Debug", "debug") {
                    if let Ok(d) = v.parse::<u8>() {
                        cfg.debug = d;
                    }
                }
            }
        }

        // CLI overrides (apply after INI)
        cfg.generate_config = cli.generate_config;
        cfg.list_markets = cli.list_markets;
        if cli.background { cfg.background = true; }
        if cli.foreground { cfg.foreground = true; }
        if cli.keep_file_name { cfg.keep_file_name = true; }
        if cli.redownload { cfg.redownload = true; }
        if cli.database_no_image { cfg.database_no_image = true; }

        if cli.debug > 0 {
            cfg.debug = cli.debug;
        }
        if let Some(ref v) = cli.config_file {
            cfg.config_file = v.clone();
        }
        if let Some(v) = cli.interval {
            cfg.interval = if v >= 1 { v } else { 1 };
        }
        if let Some(ref v) = cli.country {
            cfg.country = v.clone();
        }
        if let Some(ref v) = cli.market {
            cfg.market = v.clone();
        }
        if let Some(ref v) = cli.size_mode {
            cfg.size_mode = v.clone();
        }
        if !cli.collect.is_empty() {
            cfg.collect = cli.collect.clone();
        }
        if let Some(ref v) = cli.image_size {
            cfg.image_size = v.clone();
        }
        if let Some(v) = cli.offset {
            cfg.offset = v;
        }
        if let Some(ref v) = cli.proxy_server {
            cfg.proxy_server = v.clone();
        }
        if let Some(v) = cli.proxy_port {
            cfg.proxy_port = v;
        }
        if let Some(ref v) = cli.proxy_username {
            cfg.proxy_username = v.clone();
        }
        if let Some(ref v) = cli.proxy_password {
            cfg.proxy_password = v.clone();
        }
        if let Some(ref v) = cli.setter {
            cfg.setter = v.clone();
        }
        if !cli.setter_args.is_empty() {
            cfg.setter_args = cli.setter_args.clone();
        }
        if let Some(ref v) = cli.output_folder {
            cfg.output_folder = v.clone();
        }
        if let Some(ref v) = cli.database_file {
            cfg.database_file = v.clone();
        }
        if let Some(ref v) = cli.server {
            cfg.server = v.clone();
        }
        if let Some(ref v) = cli.custom_server {
            cfg.customserver = v.clone();
        }

        // Backward compatibility: collect mode
        if cfg.size_mode == "collect" {
            log::warn!("size_mode=collect is obsolete, considering use collect=accompany instead");
            cfg.size_mode = "prefer".to_string();
            if !cfg.collect.contains(&"accompany".to_string()) {
                cfg.collect.push("accompany".to_string());
            }
        }

        cfg
    }

    pub fn generate_config_file(&self) {
        let mut ini = Ini::new();
        ini.set("Daemon", "background", Some(self.background.to_string()));
        ini.set("Daemon", "foreground", Some(self.foreground.to_string()));
        ini.set("Daemon", "interval", Some(self.interval.to_string()));

        ini.set("Download", "country", Some(self.country.clone()));
        ini.set("Download", "market", Some(self.market.clone()));
        ini.set("Download", "keep_file_name", Some(self.keep_file_name.to_string()));
        ini.set("Download", "size_mode", Some(self.size_mode.clone()));
        ini.set("Download", "collect", Some(self.collect.join(",")));
        ini.set("Download", "image_size", Some(self.image_size.clone()));
        ini.set("Download", "offset", Some(self.offset.to_string()));
        ini.set("Download", "output_folder", Some(self.output_folder.clone()));
        ini.set("Download", "redownload", Some(self.redownload.to_string()));
        ini.set("Download", "server", Some(self.server.clone()));
        ini.set("Download", "customserver", Some(self.customserver.clone()));

        ini.set("Proxy", "proxy_server", Some(self.proxy_server.clone()));
        ini.set("Proxy", "proxy_port", Some(self.proxy_port.to_string()));
        ini.set("Proxy", "proxy_username", Some(self.proxy_username.clone()));
        ini.set("Proxy", "proxy_password", Some(self.proxy_password.clone()));

        ini.set("Setter", "setter", Some(self.setter.clone()));
        ini.set("Setter", "setter_args", Some(self.setter_args.join(",")));

        ini.set("Database", "database_file", Some(self.database_file.clone()));
        ini.set("Database", "database_no_image", Some(self.database_no_image.to_string()));

        ini.set("Debug", "debug", Some(self.debug.to_string()));

        let _ = ini.write(&self.config_file);
        log::info!("config saved to {}", self.config_file);
    }
}

pub fn list_markets() {
    let markets = [
        ("es-AR", "Argentina"),
        ("en-AU", "Australia"),
        ("de-AT", "Austria"),
        ("nl-BE", "Belgium - Dutch"),
        ("fr-BE", "Belgium - French"),
        ("pt-BR", "Brazil"),
        ("en-CA", "Canada - English"),
        ("fr-CA", "Canada - French"),
        ("es-CL", "Chile"),
        ("zh-CN", "China"),
        ("da-DK", "Denmark"),
        ("ar-EG", "Egypt"),
        ("fi-FI", "Finland"),
        ("fr-FR", "France"),
        ("de-DE", "Germany"),
        ("zh-HK", "Hong Kong SAR"),
        ("en-IN", "India"),
        ("en-ID", "Indonesia"),
        ("en-IE", "Ireland"),
        ("it-IT", "Italy"),
        ("ja-JP", "Japan"),
        ("ko-KR", "Korea"),
        ("en-MY", "Malaysia"),
        ("es-MX", "Mexico"),
        ("nl-NL", "Netherlands"),
        ("en-NZ", "New Zealand"),
        ("nb-NO", "Norway"),
        ("en-PH", "Philippines"),
        ("pl-PL", "Poland"),
        ("pt-PT", "Portugal"),
        ("ru-RU", "Russia"),
        ("ar-SA", "Saudi Arabia"),
        ("en-SG", "Singapore"),
        ("en-ZA", "South Africa"),
        ("es-ES", "Spain"),
        ("sv-SE", "Sweden"),
        ("fr-CH", "Switzerland - French"),
        ("de-CH", "Switzerland - German"),
        ("zh-TW", "Taiwan"),
        ("tr-TR", "Turkey"),
        ("ar-AE", "United Arab Emirates"),
        ("en-GB", "United Kingdom"),
        ("en-US", "United States - English"),
        ("es-US", "United States - Spanish"),
    ];
    println!("Available markets:");
    for (k, v) in markets {
        println!("{}     {}", k, v);
    }
}
