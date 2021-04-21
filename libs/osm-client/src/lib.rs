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
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
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

#[derive(Debug, Error)]
#[error(display = "Failed to parse scale")]
pub struct ScaleParseError;

/// Size of a tile in pixels is scale * 256
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum Scale {
    /// x1 == 256 tile size
    One,
    /// x2 == 512 tile size
    Two,
    /// x5 == 1024 tile size
    Four,
}

impl Default for Scale {
    fn default() -> Self {
        Scale::One
    }
}

impl Scale {
    pub fn tile_size(&self) -> u32 {
        match self {
            Scale::One => 256,
            Scale::Two => 512,
            Scale::Four => 1024,
        }
    }
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

impl FromStr for Scale {
    type Err = ScaleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Scale::One),
            "2" => Ok(Scale::Two),
            "4" => Ok(Scale::Four),
            _ => Err(ScaleParseError),
        }
    }
}

#[derive(Debug, Error)]
#[error(display = "Failed to parse daylight")]
pub struct DaylighParseError;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
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

impl FromStr for Daylight {
    type Err = DaylighParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "night" => Ok(Daylight::Night),
            "day" => Ok(Daylight::Day),
            _ => Err(DaylighParseError),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
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

impl FromStr for Zoom {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Zoom::new_clamped(s.parse::<u8>()?))
    }
}

impl fmt::Display for Zoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
        self.set_timeout(Some(timeout));
        self
    }

    pub fn set_timeout<T: Into<Option<Duration>>>(&mut self, timeout: T) {
        self.timeout = timeout.into();
    }

    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.set_scale(Some(scale));
        self
    }

    pub fn set_scale<T: Into<Option<Scale>>>(&mut self, scale: T) {
        self.scale = scale.into();
        self.update_base_url_query_pairs();
    }

    pub fn with_daylight(mut self, daylight: Daylight) -> Self {
        self.set_daylight(Some(daylight));
        self
    }

    pub fn set_daylight<T: Into<Option<Daylight>>>(&mut self, daylight: T) {
        self.daylight = daylight.into();
        self.update_base_url_query_pairs();
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
