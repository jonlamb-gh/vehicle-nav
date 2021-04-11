//! TODO docs
//!
//! zoom: 1..=18
//! http://localhost:8080/{z}/{x}/{y}.png
//! http://localhost:8080/{z}/{x}/{y}{r}.png

#![deny(warnings)]

use bytes::Bytes;
use err_derive::Error;
use reqwest::blocking::Client;
use std::time::Duration;
use url::Url;

// TODO - newtype, enum for high res
pub const MIN_ZOOM: u8 = 1;
pub const MAX_ZOOM: u8 = 18;

// TODO - timeout variant
#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Failed to parse request url: {}", _0)]
    UrlParse(url::ParseError),
    #[error(display = "Failed to send request: {}", _0)]
    Request(reqwest::Error),
    #[error(display = "Failed to parse response as bytes: {}", _0)]
    ParseResponseBytes(reqwest::Error),
}

#[derive(Debug)]
pub struct OsmClient {
    client: Client,
    server_url: Url,
    timeout: Duration,
}

impl OsmClient {
    pub fn new(server_url: Url, timeout: Duration) -> Self {
        log::debug!("Created new OsmClient {}", server_url);
        OsmClient {
            client: Client::new(),
            server_url,
            timeout,
        }
    }

    // newtype units?
    pub fn request_tile(&self, x: u32, y: u32, zoom: u8) -> Result<Bytes, Error> {
        let z = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let suffix = format!("{}/{}/{}.png", z, x, y);
        let url = self.server_url.join(&suffix).map_err(Error::UrlParse)?;
        let req = self.client.get(url).timeout(self.timeout);
        log::debug!("Sending tile request {:?}", req);
        let resp = req.send().map_err(Error::Request)?;
        log::debug!("Received tile response {:?}", resp);
        let bytes = resp.bytes().map_err(Error::ParseResponseBytes)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {
        assert_eq!(2 + 2, 4);
    }
}
