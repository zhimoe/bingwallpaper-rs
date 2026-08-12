use reqwest::blocking::Client;
use serde_json::Value;

use crate::webutil;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub copyright: String,
    pub market: String,
    pub enddate: chrono::NaiveDate,
    pub fullstartdate: chrono::NaiveDateTime,
}

pub trait HighResolutionSetting {
    fn get_pic_url(
        &self,
        client: &Client,
        rooturl: &str,
        imgurlbase: &str,
        fallbackurl: &str,
        has_wp: bool,
        resolution: &str,
    ) -> Option<String>;
}

pub struct UHDResolution;
impl HighResolutionSetting for UHDResolution {
    fn get_pic_url(
        &self,
        _client: &Client,
        rooturl: &str,
        imgurlbase: &str,
        _fallbackurl: &str,
        _has_wp: bool,
        _resolution: &str,
    ) -> Option<String> {
        let wplink = format!("{}{}_UHD.jpg", rooturl, imgurlbase);
        log::debug!("in UHD mode, get url {}", wplink);
        Some(wplink)
    }
}

pub struct PreferHighResolution;
impl HighResolutionSetting for PreferHighResolution {
    fn get_pic_url(
        &self,
        client: &Client,
        rooturl: &str,
        imgurlbase: &str,
        fallbackurl: &str,
        _has_wp: bool,
        _resolution: &str,
    ) -> Option<String> {
        let resolutions = ["UHD", "1920x1200", "1920x1080"];
        for suffix in &resolutions {
            let url = format!("{}{}_{}.jpg", rooturl, imgurlbase, suffix);
            log::debug!("in prefer mode, detect existence of pic {}", url);
            if webutil::test_header(client, &url) {
                log::debug!("in prefer mode, decided url: {}", url);
                return Some(url);
            }
        }
        Some(format!("{}{}", rooturl, fallbackurl))
    }
}

pub struct InsistHighResolution;
impl HighResolutionSetting for InsistHighResolution {
    fn get_pic_url(
        &self,
        _client: &Client,
        rooturl: &str,
        imgurlbase: &str,
        _fallbackurl: &str,
        has_wp: bool,
        _resolution: &str,
    ) -> Option<String> {
        if has_wp {
            let wplink = format!("{}{}_1920x1200.jpg", rooturl, imgurlbase);
            log::debug!("in insist mode, get high resolution url {}", wplink);
            Some(wplink)
        } else {
            log::debug!("in insist mode, drop normal resolution pic");
            None
        }
    }
}

pub struct NeverHighResolution;
impl HighResolutionSetting for NeverHighResolution {
    fn get_pic_url(
        &self,
        _client: &Client,
        rooturl: &str,
        _imgurlbase: &str,
        fallbackurl: &str,
        _has_wp: bool,
        _resolution: &str,
    ) -> Option<String> {
        let wplink = format!("{}{}", rooturl, fallbackurl);
        log::debug!("never use high resolution, use {}", wplink);
        Some(wplink)
    }
}

pub struct HighestResolution;
impl HighResolutionSetting for HighestResolution {
    fn get_pic_url(
        &self,
        _client: &Client,
        rooturl: &str,
        imgurlbase: &str,
        _fallbackurl: &str,
        has_wp: bool,
        _resolution: &str,
    ) -> Option<String> {
        if has_wp {
            let wplink = format!("{}{}_1920x1200.jpg", rooturl, imgurlbase);
            log::debug!("support wallpaper, get high resolution url {}", wplink);
            Some(wplink)
        } else {
            let wplink = format!("{}{}_1920x1080.jpg", rooturl, imgurlbase);
            log::debug!(
                "not support wallpaper, use second highest resolution {}",
                wplink
            );
            Some(wplink)
        }
    }
}

pub struct ManualHighResolution {
    pub resolution: String,
}
impl HighResolutionSetting for ManualHighResolution {
    fn get_pic_url(
        &self,
        _client: &Client,
        rooturl: &str,
        imgurlbase: &str,
        _fallbackurl: &str,
        _has_wp: bool,
        _resolution: &str,
    ) -> Option<String> {
        let valid = self
            .resolution
            .split_once('x')
            .and_then(|(width, height)| {
                Some((width.parse::<u32>().ok()?, height.parse::<u32>().ok()?))
            })
            .is_some_and(|(width, height)| width > 0 && height > 0);
        if !valid {
            log::error!("invalid resolution \"{}\" for manual mode", self.resolution);
            return None;
        }
        let wplink = format!("{}{}_{}.jpg", rooturl, imgurlbase, self.resolution);
        log::debug!("manually specify resolution, use {}", wplink);
        Some(wplink)
    }
}

pub fn get_high_resolution_setting(
    name: &str,
    resolution: &str,
) -> Result<Box<dyn HighResolutionSetting>, String> {
    match name {
        "uhd" => Ok(Box::new(UHDResolution)),
        "prefer" => Ok(Box::new(PreferHighResolution)),
        "insist" => Ok(Box::new(InsistHighResolution)),
        "never" => Ok(Box::new(NeverHighResolution)),
        "highest" => Ok(Box::new(HighestResolution)),
        "manual" => Ok(Box::new(ManualHighResolution {
            resolution: resolution.to_string(),
        })),
        _ => Err(format!("{} is not a legal resolution setting", name)),
    }
}

pub trait AssetCollector {
    fn collect(&self, rooturl: &str, curimage: &Value) -> Option<Vec<String>>;
}

pub struct AccompanyImageCollector;
impl AssetCollector for AccompanyImageCollector {
    fn collect(&self, rooturl: &str, curimage: &Value) -> Option<Vec<String>> {
        let img_url_base = curimage["urlbase"].as_str()?;
        let has_wp = curimage
            .get("wp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if has_wp && !img_url_base.contains("_ZH_") {
            log::debug!("{} may have a Chinese brother", img_url_base);
            let zh_link = format!("{}{}_ZH_1920x1200.jpg", rooturl, img_url_base);
            Some(vec![zh_link])
        } else {
            log::debug!("no chinese logo for {}", img_url_base);
            None
        }
    }
}

fn collect_video_urls(curimage: &Value, hd: bool) -> Option<Vec<String>> {
    let vid = curimage.get("vid")?;
    let sources = vid["sources"].as_array()?;
    let mut vlink = Vec::new();
    for item in sources {
        let Some(arr) = item.as_array() else {
            continue;
        };
        if arr.len() < 3 {
            continue;
        }
        let (Some(video_format), Some(video_url)) = (arr[0].as_str(), arr[2].as_str()) else {
            continue;
        };
        if video_format.ends_with("hd") != hd {
            continue;
        }
        let url = if video_url.starts_with("//") {
            format!("https:{}", video_url)
        } else {
            video_url.to_string()
        };
        vlink.push(url);
    }
    if vlink.is_empty() { None } else { Some(vlink) }
}

pub struct VideoCollector;
impl AssetCollector for VideoCollector {
    fn collect(&self, _rooturl: &str, curimage: &Value) -> Option<Vec<String>> {
        collect_video_urls(curimage, false)
    }
}

pub struct HdVideoCollector;
impl AssetCollector for HdVideoCollector {
    fn collect(&self, _rooturl: &str, curimage: &Value) -> Option<Vec<String>> {
        collect_video_urls(curimage, true)
    }
}

pub fn get_asset_collector(name: &str) -> Option<Box<dyn AssetCollector>> {
    match name {
        "accompany" => Some(Box::new(AccompanyImageCollector)),
        "video" => Some(Box::new(VideoCollector)),
        "hdvideo" => Some(Box::new(HdVideoCollector)),
        _ => {
            log::warn!("collector {} is not supported", name);
            None
        }
    }
}

#[derive(Debug)]
pub struct BingWallpaperPage {
    pub base: String,
    pub url: String,
    pub resolution: String,
    pub high_resolution_name: String,
    pub collect: Vec<String>,
    loaded: bool,
    images: Vec<Value>,
    pub wplinks: Vec<(Vec<String>, Metadata)>,
    act_market: String,
}

impl BingWallpaperPage {
    pub const BASE_URL: &'static str = "https://www.bing.com";

    pub fn new(
        idx: i32,
        n: i32,
        base: &str,
        locale: (Option<String>, Option<String>),
        resolution_setting: (&str, &str),
        collect: Vec<String>,
    ) -> Self {
        let (country_code, market_code) = locale;
        let (high_resolution_name, resolution) = resolution_setting;
        let base = base.trim_end_matches('/');
        let api = format!(
            "/HPImageArchive.aspx?format=js&mbl=1&idx={}&n={}&video=1",
            idx, n
        );
        let mut url = format!("{}{}", base, api);
        if let Some(ref mkt) = market_code {
            match Self::validate_market(mkt) {
                Ok(()) => url.push_str(&format!("&mkt={}", mkt)),
                Err(e) => {
                    log::warn!("{}", e);
                    if let Some(ref cc) = country_code {
                        url.push_str(&format!("&cc={}", cc));
                    }
                }
            }
        } else if let Some(ref cc) = country_code {
            url.push_str(&format!("&cc={}", cc));
        }
        BingWallpaperPage {
            base: base.to_string(),
            url,
            resolution: resolution.to_string(),
            high_resolution_name: high_resolution_name.to_string(),
            collect,
            loaded: false,
            images: vec![],
            wplinks: vec![],
            act_market: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.loaded = false;
        self.images.clear();
        self.wplinks.clear();
        self.act_market.clear();
    }

    pub fn loaded(&self) -> bool {
        self.loaded
    }

    pub fn image_links(&self) -> Option<&Vec<(Vec<String>, Metadata)>> {
        if self.loaded {
            Some(&self.wplinks)
        } else {
            None
        }
    }

    pub fn load(&mut self, client: &Client) {
        self.reset();
        log::info!("loading from {}", self.url);
        if let Some(raw_file) = webutil::loadpage(client, &self.url, false) {
            log::info!("{} bytes loaded", raw_file.len());
            self.loaded = self.parse(client, &raw_file);
        } else {
            log::error!("can't download photo page");
        }
    }

    fn parse(&mut self, client: &Client, raw_file: &str) -> bool {
        let content: Value = match serde_json::from_str(raw_file) {
            Ok(v) => v,
            Err(e) => {
                log::error!("JSON parse error: {}", e);
                return false;
            }
        };
        if content.is_null() {
            return false;
        }
        log::debug!("{}", content);
        self.images = match content["images"].as_array() {
            Some(arr) => arr.clone(),
            None => return false,
        };
        if let Some(market) = content.get("market")
            && let Some(mkt) = market["mkt"].as_str()
        {
            self.act_market = mkt.to_string();
        }
        self.update_img_link(client);
        log::debug!("links to be downloaded: {:?}", self.wplinks);
        true
    }

    fn get_metadata(&self, i: &Value) -> Metadata {
        let enddate = i["enddate"].as_str().unwrap_or("19700101");
        let fullstartdate = i["fullstartdate"].as_str().unwrap_or("197001010000");
        Metadata {
            copyright: i["copyright"].as_str().unwrap_or("").to_string(),
            market: self.act_market.clone(),
            enddate: chrono::NaiveDate::parse_from_str(enddate, "%Y%m%d")
                .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            fullstartdate: chrono::NaiveDateTime::parse_from_str(fullstartdate, "%Y%m%d%H%M")
                .unwrap_or_else(|_| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        }
    }

    fn update_img_link(&mut self, client: &Client) {
        self.wplinks.clear();
        let hr = match get_high_resolution_setting(&self.high_resolution_name, &self.resolution) {
            Ok(hr) => hr,
            Err(e) => {
                log::error!("{}", e);
                return;
            }
        };
        for i in &self.images {
            let metadata = self.get_metadata(i);
            let has_wp = i.get("wp").and_then(|v| v.as_bool()).unwrap_or(false);
            let imgurlbase = i["urlbase"].as_str().unwrap_or("");
            let fallbackurl = i["url"].as_str().unwrap_or("");
            log::debug!(
                "handling {}, rooturl={}, img_url_base={}, has_wp={}, resolution={}, act_market={}",
                fallbackurl,
                self.base,
                imgurlbase,
                has_wp,
                self.resolution,
                self.act_market
            );
            let wplink = hr.get_pic_url(
                client,
                &self.base,
                imgurlbase,
                fallbackurl,
                has_wp,
                &self.resolution,
            );
            let mut collections: Vec<String> = Vec::new();
            for collector_name in &self.collect {
                if let Some(collector) = get_asset_collector(collector_name)
                    && let Some(assets) = collector.collect(&self.base, i)
                {
                    collections.extend(assets);
                }
            }
            if let Some(main) = wplink {
                let mut urls = vec![main];
                urls.extend(collections);
                self.wplinks.push((urls, metadata));
            }
        }
    }

    pub fn validate_market(market_code: &str) -> Result<(), String> {
        if market_code.len() == 5
            && market_code.as_bytes()[0].is_ascii_alphanumeric()
            && market_code.as_bytes()[1].is_ascii_alphanumeric()
            && market_code.as_bytes()[2] == b'-'
            && market_code.as_bytes()[3].is_ascii_alphanumeric()
            && market_code.as_bytes()[4].is_ascii_alphanumeric()
        {
            Ok(())
        } else {
            Err(format!("{} is not a valid market code.", market_code))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn video_collector_skips_malformed_sources() {
        let image = json!({
            "vid": {
                "sources": [
                    null,
                    ["mp4", "missing URL"],
                    [42, null, false],
                    ["mp4", 0, "//example.com/video.mp4"],
                    ["mp4hd", 0, "//example.com/video-hd.mp4"]
                ]
            }
        });

        assert_eq!(
            collect_video_urls(&image, false),
            Some(vec!["https://example.com/video.mp4".to_string()])
        );
        assert_eq!(
            collect_video_urls(&image, true),
            Some(vec!["https://example.com/video-hd.mp4".to_string()])
        );
    }

    #[test]
    fn validates_market_shape() {
        assert!(BingWallpaperPage::validate_market("zh-CN").is_ok());
        assert!(BingWallpaperPage::validate_market("zh_CN").is_err());
        assert!(BingWallpaperPage::validate_market("中文").is_err());
    }

    #[test]
    fn validates_manual_resolution() {
        let client = Client::new();
        let valid = ManualHighResolution {
            resolution: "2560x1440".to_string(),
        };
        let invalid = ManualHighResolution {
            resolution: "widexhigh".to_string(),
        };

        assert!(
            valid
                .get_pic_url(&client, "https://example.com", "/image", "", false, "")
                .is_some()
        );
        assert!(
            invalid
                .get_pic_url(&client, "https://example.com", "/image", "", false, "")
                .is_none()
        );
    }

    #[test]
    fn normalizes_base_url_and_falls_back_from_invalid_market() {
        let page = BingWallpaperPage::new(
            0,
            1,
            "https://example.com/",
            (Some("CN".to_string()), Some("invalid".to_string())),
            ("never", ""),
            vec![],
        );

        assert_eq!(page.base, "https://example.com");
        assert!(
            page.url
                .starts_with("https://example.com/HPImageArchive.aspx")
        );
        assert!(page.url.ends_with("&cc=CN"));
    }
}
