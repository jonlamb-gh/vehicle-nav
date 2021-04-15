//! TODO docs
//!
//! zoom: 1..=18
//! http://localhost:8080/{z}/{x}/{y}.png
//! http://localhost:8080/{z}/{x}/{y}{r}.png
//!
//! http://127.0.0.1:8553/v1/tile?daylight=1&scale=4&z={z}&x={x}&y={y}
//! http://localhost:8553/v1/tile?style={style}&daylight={dlight}&shift={shift}&scale={scale}&z={z}&x={x}&y={y}
//! https://github.com/rinigus/osmscout-server/blob/master/README.api.md#raster-tiles

#![deny(warnings)]

use bytes::Bytes;
use err_derive::Error;
use reqwest::blocking::Client;
use std::time::Duration;
use url::Url;

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

/// Size of a tile in pixels is scale * 256
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Scale {
    /// x1 == 256 tile size
    One,
    /// x2 == 512 tile size
    Two,
    /// x5 == 1024 tile size
    Four,
}

impl Scale {
    fn query_pair(&self) -> (&'static str, &'static str) {
        match self {
            Scale::One => ("scale", "1"),
            Scale::Two => ("scale", "2"),
            Scale::Four => ("scale", "4"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Daylight {
    Night,
    Day,
}

impl Daylight {
    fn query_pair(&self) -> (&'static str, &'static str) {
        match self {
            Daylight::Night => ("daylight", "0"),
            Daylight::Day => ("daylight", "1"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Zoom(u8);

impl Zoom {
    pub const MIN: Zoom = Zoom(1);
    pub const MAX: Zoom = Zoom(18);

    pub fn new_clamped(z: u8) -> Self {
        Zoom(z.clamp(Self::MIN.0, Self::MAX.0))
    }

    pub const fn get(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Zoom {
    fn from(z: u8) -> Self {
        Zoom::new_clamped(z)
    }
}

impl From<Zoom> for u8 {
    fn from(z: Zoom) -> Self {
        z.0
    }
}

#[derive(Debug)]
pub struct OsmClient {
    client: Client,
    server_url: Url,
    timeout: Option<Duration>,
    scale: Option<Scale>,
    daylight: Option<Daylight>,
}

impl OsmClient {
    pub fn new(server_url: Url) -> Self {
        log::debug!("Created new OsmClient {}", server_url);
        OsmClient {
            client: Client::new(),
            server_url,
            timeout: None,
            scale: None,
            daylight: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = Some(scale);
        self.update_base_url_query_pairs();
        self
    }

    pub fn with_daylight(mut self, daylight: Daylight) -> Self {
        self.daylight = Some(daylight);
        self.update_base_url_query_pairs();
        self
    }

    // TODO - newtype units
    pub fn request_tile(&self, x: u32, y: u32, zoom: Zoom) -> Result<Bytes, Error> {
        let z = zoom.0.to_string();
        let x = x.to_string();
        let y = y.to_string();
        let mut base = self.server_url.clone();
        let url = base
            .query_pairs_mut()
            .append_pair("z", &z)
            .append_pair("x", &x)
            .append_pair("y", &y)
            .finish();
        let req = if let Some(timeout) = self.timeout {
            self.client.get(url.as_str()).timeout(timeout)
        } else {
            self.client.get(url.as_str())
        };
        log::debug!("Sending tile request {:?}", req);
        let resp = req.send().map_err(Error::Request)?;
        log::debug!("Received tile response {:?}", resp);
        let bytes = resp.bytes().map_err(Error::ParseResponseBytes)?;
        Ok(bytes)
    }

    /*
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
    */

    fn update_base_url_query_pairs(&mut self) {
        self.server_url.query_pairs_mut().clear();
        if let Some(daylight) = &self.daylight {
            let qp = daylight.query_pair();
            self.server_url.query_pairs_mut().append_pair(qp.0, qp.1);
        }
        if let Some(scale) = &self.scale {
            let qp = scale.query_pair();
            self.server_url.query_pairs_mut().append_pair(qp.0, qp.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_manipulation() {
        let url = Url::parse("http://127.0.0.1:8553/v1/tile").unwrap();

        let client = OsmClient::new(url.clone());
        assert_eq!(client.server_url.as_str(), "http://127.0.0.1:8553/v1/tile");
        let client = client.with_timeout(Duration::from_secs(1));
        assert_eq!(client.server_url.as_str(), "http://127.0.0.1:8553/v1/tile");

        let client = client.with_daylight(Daylight::Day);
        assert_eq!(
            client.server_url.as_str(),
            "http://127.0.0.1:8553/v1/tile?daylight=1"
        );
        let client = client.with_daylight(Daylight::Night);
        assert_eq!(
            client.server_url.as_str(),
            "http://127.0.0.1:8553/v1/tile?daylight=0"
        );

        let client = OsmClient::new(url.clone());
        let client = client.with_scale(Scale::One);
        assert_eq!(
            client.server_url.as_str(),
            "http://127.0.0.1:8553/v1/tile?scale=1"
        );
        let client = client.with_scale(Scale::Two);
        assert_eq!(
            client.server_url.as_str(),
            "http://127.0.0.1:8553/v1/tile?scale=2"
        );
        let client = client.with_scale(Scale::Four);
        assert_eq!(
            client.server_url.as_str(),
            "http://127.0.0.1:8553/v1/tile?scale=4"
        );

        let client = OsmClient::new(url)
            .with_daylight(Daylight::Day)
            .with_scale(Scale::Four);
        assert_eq!(
            client.server_url.as_str(),
            "http://127.0.0.1:8553/v1/tile?daylight=1&scale=4"
        );
    }
}
