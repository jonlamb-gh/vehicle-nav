use err_derive::Error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{fmt, num};

// TODO - write up some newtype helper macros to do trait impls
// impl approx::RelEqual+ traits for lat/lon/coord

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Latitude(pub f64);

impl Latitude {
    pub const MIN: Latitude = Latitude(-90.0);
    pub const MAX: Latitude = Latitude(90.0);

    pub fn new_clamped(val: f64) -> Self {
        Self(val.clamp(Self::MIN.0, Self::MAX.0))
    }

    pub const fn get(&self) -> f64 {
        self.0
    }

    pub fn saturating_add(&mut self, val: f64) {
        self.0 = Self::new_clamped(self.0 + val).0;
    }

    pub fn saturating_sub(&mut self, val: f64) {
        self.0 = Self::new_clamped(self.0 - val).0;
    }
}

impl From<f64> for Latitude {
    fn from(l: f64) -> Self {
        Latitude(l)
    }
}

impl From<Latitude> for f64 {
    fn from(l: Latitude) -> Self {
        l.0
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Longitude(pub f64);

impl Longitude {
    pub const MIN: Longitude = Longitude(-180.0);
    pub const MAX: Longitude = Longitude(180.0);

    pub fn new_clamped(val: f64) -> Self {
        Self(val.clamp(Self::MIN.0, Self::MAX.0))
    }

    pub const fn get(&self) -> f64 {
        self.0
    }

    pub fn saturating_add(&mut self, val: f64) {
        self.0 = Self::new_clamped(self.0 + val).0;
    }

    pub fn saturating_sub(&mut self, val: f64) {
        self.0 = Self::new_clamped(self.0 - val).0;
    }
}

impl From<f64> for Longitude {
    fn from(l: f64) -> Self {
        Longitude(l)
    }
}

impl From<Longitude> for f64 {
    fn from(l: Longitude) -> Self {
        l.0
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Coordinate {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl Coordinate {
    pub fn new<Lat: Into<Latitude>, Lon: Into<Longitude>>(latitude: Lat, longitude: Lon) -> Self {
        Coordinate {
            latitude: latitude.into(),
            longitude: longitude.into(),
        }
    }
}

impl From<(Latitude, Longitude)> for Coordinate {
    fn from(c: (Latitude, Longitude)) -> Self {
        Coordinate {
            latitude: c.0,
            longitude: c.1,
        }
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.latitude, self.longitude)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct TileNumber(pub u32);

impl From<u32> for TileNumber {
    fn from(t: u32) -> Self {
        TileNumber(t)
    }
}

impl From<TileNumber> for u32 {
    fn from(t: TileNumber) -> Self {
        t.0
    }
}
impl fmt::Display for TileNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
    pub fn url_query_pair(&self) -> (&'static str, &'static str) {
        match self {
            Scale::One => ("scale", "1"),
            Scale::Two => ("scale", "2"),
            Scale::Four => ("scale", "4"),
        }
    }
}

#[derive(Debug, Error)]
#[error(display = "Failed to parse scale")]
pub struct ScaleParseError;

impl FromStr for Scale {
    type Err = ScaleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "one" => Ok(Scale::One),
            "2" | "two" => Ok(Scale::Two),
            "4" | "four" => Ok(Scale::Four),
            _ => Err(ScaleParseError),
        }
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scale::One => f.write_str("1"),
            Scale::Two => f.write_str("2"),
            Scale::Four => f.write_str("4"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum Daylight {
    Night,
    Day,
}

impl Daylight {
    pub fn url_query_pair(&self) -> (&'static str, &'static str) {
        match self {
            Daylight::Night => ("daylight", "0"),
            Daylight::Day => ("daylight", "1"),
        }
    }
}

#[derive(Debug, Error)]
#[error(display = "Failed to parse daylight")]
pub struct DaylighParseError;

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

impl fmt::Display for Daylight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Daylight::Night => f.write_str("night"),
            Daylight::Day => f.write_str("day"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Zoom(u8);

impl Zoom {
    pub const MIN: Zoom = Zoom(1);
    pub const MAX: Zoom = Zoom(18);

    pub fn new_clamped(val: u8) -> Self {
        Self(val.clamp(Self::MIN.0, Self::MAX.0))
    }

    pub const fn get(&self) -> u8 {
        self.0
    }

    pub fn increment(&mut self) {
        self.saturating_add(1);
    }

    pub fn decrement(&mut self) {
        self.saturating_sub(1);
    }

    pub fn saturating_add(&mut self, val: u8) {
        let new_zoom = self.0.saturating_add(val);
        self.0 = Self::new_clamped(new_zoom).0;
    }

    pub fn saturating_sub(&mut self, val: u8) {
        let new_zoom = self.0.saturating_sub(val);
        self.0 = Self::new_clamped(new_zoom).0;
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
    type Err = num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Zoom::new_clamped(s.parse::<u8>()?))
    }
}

impl fmt::Display for Zoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
