mod autostart;
mod bing;
mod config;
mod logger;
mod record;
mod setter;
mod webutil;
mod winsetter;

use bing::BingWallpaperPage;
use clap::Parser;
use config::{CliArgs, Config};
use record::{DownloadRecord, DownloadRecordManager, SqlDatabaseRecordManager};
use setter::WallpaperSetterFactory;
use std::path::Path;
use std::time::Duration;

fn history_file() -> String {
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    Path::new(&app_data)
        .join("Genzj")
        .join("PyBingWallpaper")
        .join("bing-wallpaper-history.json")
        .to_string_lossy()
        .to_string()
}

fn main() {
    logger::init();
    let cli = CliArgs::parse();

    if cli.list_markets {
        config::list_markets();
        return;
    }

    let cfg = Config::from_cli_and_file(&cli);

    if cfg.debug == 1 {
        logger::set_debug_level(log::LevelFilter::Debug);
    } else if cfg.debug >= 2 {
        logger::set_debug_level(log::LevelFilter::Trace);
    }

    if cfg.generate_config {
        cfg.generate_config_file();
        return;
    }

    if cli.install_autostart {
        if let Err(e) = autostart::install_autostart() {
            log::error!("failed to install autostart: {}", e);
        } else {
            println!("Autostart installed successfully.");
        }
        return;
    }

    if cli.uninstall_autostart {
        if let Err(e) = autostart::uninstall_autostart() {
            log::error!("failed to uninstall autostart: {}", e);
        } else {
            println!("Autostart uninstalled successfully.");
        }
        return;
    }

    let mut factory = WallpaperSetterFactory::new();
    factory.register("win", || Box::new(winsetter::Win32WallpaperSetter::new()));
    factory.register("no", || Box::new(NoopSetter));

    if !cfg.foreground && cfg.background {
        log::info!("daemon is running");
        loop {
            let cfg = Config::from_cli_and_file(&cli);
            let timeout = run_cycle(&cfg, &factory);
            log::debug!("schedule next running in {} seconds", timeout);
            std::thread::sleep(Duration::from_secs(timeout));
        }
    } else {
        run_cycle(&cfg, &factory);
    }
}

struct NoopSetter;
impl setter::WallpaperSetter for NoopSetter {
    fn set(&self, _path: &str, _args: &[String]) -> bool {
        true
    }
}

fn run_cycle(cfg: &Config, factory: &WallpaperSetterFactory) -> u64 {
    let client = webutil::build_client(if cfg.proxy_server.is_empty() {
        None
    } else {
        Some((
            &cfg.proxy_server,
            cfg.proxy_port,
            Some(&cfg.proxy_username)
                .filter(|s| !s.is_empty())
                .map(|x| x.as_str()),
            Some(&cfg.proxy_password)
                .filter(|s| !s.is_empty())
                .map(|x| x.as_str()),
        ))
    });

    prepare_output_dir(&cfg.output_folder);
    if let Some(parent) = Path::new(&history_file()).parent() {
        prepare_output_dir(&parent.to_string_lossy());
    }

    let mut record_mgr = DownloadRecordManager::new();
    record_mgr.load(&history_file());

    match download_wallpaper(cfg, &client, &record_mgr) {
        Ok(records) => {
            if let Some(records) = records {
                save_history(&records, cfg, &mut record_mgr);
                if records.is_empty() || cfg.setter == "no" {
                    log::info!("nothing to set");
                } else if let Some(factory_fn) = factory.get(&cfg.setter) {
                    let s = factory_fn();
                    let wallpaper_record = &records[0];
                    log::info!("setting wallpaper {}", wallpaper_record.local_file);
                    s.set(&wallpaper_record.local_file, &cfg.setter_args);
                    log::info!("all done. enjoy your new wallpaper");
                }
            }
            cfg.interval * 3600
        }
        Err(CannotLoadImagePage) => {
            if !cfg.foreground && cfg.background {
                log::info!("network error happened, daemon will retry in 60 seconds");
                60
            } else {
                log::info!("network error happened. please retry after Internet connection restore.");
                cfg.interval * 3600
            }
        }
    }
}

#[derive(Debug)]
struct CannotLoadImagePage;

fn download_wallpaper(
    cfg: &Config,
    client: &reqwest::blocking::Client,
    record_mgr: &DownloadRecordManager,
) -> Result<Option<Vec<DownloadRecord>>, CannotLoadImagePage> {
    let idx = cfg.offset;
    let country_code = if cfg.country == "auto" {
        None
    } else {
        Some(cfg.country.clone())
    };
    let market_code = if cfg.market.is_empty() {
        None
    } else {
        Some(cfg.market.clone())
    };

    let base_url = match cfg.server.as_str() {
        "global" => "http://www.bing.com".to_string(),
        "china" => "http://s.cn.bing.net".to_string(),
        _ => cfg.customserver.clone(),
    };

    let mut page = BingWallpaperPage::new(
        idx,
        1,
        &base_url,
        country_code,
        market_code,
        &cfg.size_mode,
        &cfg.image_size,
        cfg.collect.clone(),
    );

    page.load(client);
    log::debug!("{:?}", page);

    if !page.loaded() {
        log::error!("can not load url {}. aborting...", page.url);
        return Err(CannotLoadImagePage);
    }

    if let Some(links) = page.image_links() {
        for (wplinks, metadata) in links {
            log::debug!("{:?} photo list: {:?}", metadata, wplinks);
            let mainlink = &wplinks[0];
            let copyright = &metadata.copyright;
            let outfile = get_output_filename(cfg, mainlink);

            if let Some(rec) = record_mgr.get_by_url(mainlink) {
                if rec.local_file == outfile {
                    if !cfg.redownload {
                        log::info!("file has been downloaded before, exit");
                        return Ok(None);
                    } else {
                        log::info!("file has been downloaded before, redownload it");
                    }
                }
            }

            log::info!("download photo of \"{}\"", copyright);
            let raw = match save_a_picture(client, mainlink, copyright, &outfile) {
                Some(r) => r,
                None => continue,
            };

            let r = DownloadRecord::new(
                mainlink.clone(),
                outfile.clone(),
                copyright.clone(),
                Some(metadata.fullstartdate),
                Some(
                    metadata
                        .enddate
                        .and_hms_opt(0, 0, 0)
                        .unwrap_or_default(),
                ),
                if cfg.database_no_image {
                    None
                } else {
                    Some(raw)
                },
                false,
                metadata.market.clone(),
            );
            let mut records = vec![r];
            collect_assets(client, &wplinks[1..], metadata, cfg, &mut records);
            return Ok(Some(records));
        }
    }

    log::info!("bad luck, no wallpaper today:(");
    Ok(None)
}

fn collect_assets(
    client: &reqwest::blocking::Client,
    wplinks: &[String],
    metadata: &bing::Metadata,
    cfg: &Config,
    records: &mut Vec<DownloadRecord>,
) {
    for link in wplinks {
        let filename = get_output_filename(cfg, link);
        let raw = match save_a_picture(client, link, &metadata.copyright, &filename) {
            Some(r) => r,
            None => continue,
        };
        if filename.to_lowercase().ends_with(".jpg")
            || filename.to_lowercase().ends_with(".jpeg")
        {
            let r = DownloadRecord::new(
                link.clone(),
                filename.clone(),
                metadata.copyright.clone(),
                None,
                None,
                if cfg.database_no_image {
                    None
                } else {
                    Some(raw)
                },
                true,
                metadata.market.clone(),
            );
            records.push(r);
        }
        log::info!(
            "assets \"{}\" of \"{}\" has been downloaded to {}",
            link,
            metadata.copyright,
            filename
        );
    }
}

fn save_a_picture(
    client: &reqwest::blocking::Client,
    pic_url: &str,
    _copyright: &str,
    outfile: &str,
) -> Option<Vec<u8>> {
    let picture_content = webutil::loadurl(client, pic_url, false);
    if let Some(ref data) = picture_content {
        if let Err(e) = std::fs::write(outfile, data) {
            log::error!("failed to write {}: {}", outfile, e);
            return None;
        }
        log::info!("file saved {}", outfile);
    }
    picture_content
}

fn get_output_filename(cfg: &Config, link: &str) -> String {
    let (path_part, query_part) = if let Some(qpos) = link.find('?') {
        (&link[..qpos], &link[qpos + 1..])
    } else {
        (link, "")
    };
    let mut filename = path_part
        .rsplit('/')
        .next()
        .unwrap_or("wallpaper.jpg")
        .to_string();
    if filename == "th" || filename.is_empty() {
        for pair in query_part.split('&') {
            if let Some(eq) = pair.find('=') {
                let (k, v) = (&pair[..eq], &pair[eq + 1..]);
                if k == "id" {
                    filename = v.to_string();
                    break;
                }
            }
        }
        if filename == "th" || filename.is_empty() {
            filename = format!(
                "bingwallpaper_{}.jpg",
                chrono::Local::now().format("%Y-%m-%d_%H%M%S")
            );
        }
    }
    if !cfg.keep_file_name {
        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        filename = format!("wallpaper.{}", ext);
    }
    Path::new(&cfg.output_folder)
        .join(filename)
        .to_string_lossy()
        .to_string()
}

fn prepare_output_dir(d: &str) {
    if let Err(e) = std::fs::create_dir_all(d) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            log::error!("can not create output folder {}: {}", d, e);
        }
    }
}

fn save_history(
    records: &[DownloadRecord],
    cfg: &Config,
    record_mgr: &mut DownloadRecordManager,
) {
    if records.is_empty() {
        return;
    }
    let last_record = &records[0];
    record_mgr.clear();
    record_mgr.add(last_record.clone());
    if let Err(e) = record_mgr.save(&history_file()) {
        log::warn!("error occurs when store downloading history: {}", e);
    }

    if cfg.database_file.is_empty() {
        return;
    }

    let mut sql_mgr = SqlDatabaseRecordManager::new();
    for r in records {
        sql_mgr.add(r.clone());
    }
    if let Err(e) = sql_mgr.save(&cfg.database_file) {
        log::error!(
            "error occurs when store records into database {}: {}",
            cfg.database_file,
            e
        );
    }
}
