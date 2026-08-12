use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

static USER_AGENT_STR: &str = concat!("bingwallpaper/", env!("CARGO_PKG_VERSION"));

pub fn build_client(proxy_config: Option<(&str, u16, Option<&str>, Option<&str>)>) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    let mut builder = Client::builder()
        .gzip(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .default_headers(headers);

    if let Some((server, port, username, password)) = proxy_config
        && !server.is_empty()
    {
        let proxy_url = format!("http://{}:{}", server, port);
        match reqwest::Proxy::all(&proxy_url) {
            Ok(proxy) => {
                let proxy = match (username, password) {
                    (Some(user), Some(pass)) => proxy.basic_auth(user, pass),
                    _ => proxy,
                };
                builder = builder.proxy(proxy);
            }
            Err(e) => log::error!("invalid proxy URL {}: {}", proxy_url, e),
        }
    }

    builder.build().unwrap_or_else(|e| {
        log::error!("failed to build HTTP client: {}", e);
        Client::new()
    })
}

pub fn loadurl(client: &Client, url: &str, optional: bool) -> Option<Vec<u8>> {
    if url.is_empty() {
        return None;
    }
    match client.get(url).send() {
        Ok(resp) if resp.status().is_success() => match resp.bytes() {
            Ok(bytes) => Some(bytes.to_vec()),
            Err(e) => {
                if !optional {
                    log::error!("failed to read response from {}: {}", url, e);
                }
                None
            }
        },
        Ok(resp) => {
            if !optional {
                log::error!("HTTP error {} for {}", resp.status(), url);
            }
            None
        }
        Err(e) => {
            if !optional {
                log::error!("error {} occurs during load {}", e, url);
            }
            None
        }
    }
}

pub fn loadpage(client: &Client, url: &str, optional: bool) -> Option<String> {
    loadurl(client, url, optional).and_then(|data| String::from_utf8(data).ok())
}

pub fn test_header(client: &Client, url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    match client.head(url).send() {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
