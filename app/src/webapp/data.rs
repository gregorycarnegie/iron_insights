use gloo_net::http::Request;
use serde::Deserialize;

pub(super) async fn fetch_json_first<T: for<'de> Deserialize<'de>>(urls: &[&str]) -> Result<T, String> {
    let mut errors = Vec::new();
    for url in urls {
        match Request::get(url).send().await {
            Ok(resp) if resp.ok() => match resp.json::<T>().await {
                Ok(value) => return Ok(value),
                Err(err) => errors.push(format!("{url}: parse error: {err}")),
            },
            Ok(resp) => errors.push(format!("{url}: http {}", resp.status())),
            Err(err) => errors.push(format!("{url}: request error: {err}")),
        }
    }
    Err(errors.join(" | "))
}

pub(super) async fn fetch_binary_first(urls: &[&str]) -> Result<Vec<u8>, String> {
    let mut errors = Vec::new();
    for url in urls {
        match Request::get(url).send().await {
            Ok(resp) if resp.ok() => match resp.binary().await {
                Ok(bytes) => return Ok(bytes),
                Err(err) => errors.push(format!("{url}: binary read error: {err}")),
            },
            Ok(resp) => errors.push(format!("{url}: http {}", resp.status())),
            Err(err) => errors.push(format!("{url}: request error: {err}")),
        }
    }
    Err(errors.join(" | "))
}
