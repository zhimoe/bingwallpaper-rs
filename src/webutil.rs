use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

static USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/29.0.1521.3 Safari/537.36";

pub fn build_client(
    proxy_config: Option<(&str, u16, Option<&str>, Option<&str>)>,
) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    let mut builder = Client::builder().gzip(true).default_headers(headers);

    if let Some((server, port, username, password)) = proxy_config {
        if !server.is_empty() {
            let proxy_url = if let Some(user) = username {
                if let Some(pass) = password {
                    format!("http://{}:{}@{}:{}", user, pass, server, port)
                } else {
                    format!("http://{}@{}:{}", user, server, port)
                }
            } else {
                format!("http://{}:{}", server, port)
            };
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(proxy);
            }
        }
    }

    builder.build().unwrap_or_else(|_| Client::new())
}

pub fn loadurl(client: &Client, url: &str, optional: bool) -> Option<Vec<u8>> {
    if url.is_empty() {
        return None;
    }
    match client.get(url).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.bytes().ok().map(|b| b.to_vec())
            } else {
                if !optional {
                    log::error!("HTTP error {} for {}", resp.status(), url);
                }
                None
            }
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
