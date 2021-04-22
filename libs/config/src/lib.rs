#![deny(warnings)]

use common::{Daylight, Latitude, Longitude, Scale, Zoom};
use err_derive::Error;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};
use url::Url;

// TODO add a ValidationError
#[derive(Debug, Error)]
pub enum LoadError {
    #[error(display = "IO error")]
    Io(#[error(source)] io::Error),

    #[error(display = "The config file path {:?} does not exist", _0)]
    LoadPath(PathBuf),

    #[error(display = "Config TOML format error")]
    TomlFormat(#[error(source)] toml::de::Error),
    //#[error(display = "Config validation error")]
    //Validation(#[error(source)] ValidationError),
}

// TODO - write to file tests
#[derive(Debug, Error)]
pub enum WriteError {
    #[error(display = "TOML serialization error.")]
    Toml(#[error(source)] toml::ser::Error),

    #[error(display = "IO error")]
    Io(#[error(source)] Box<std::io::Error>),
}

// TODO - add Default impl

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub window: Window,
    pub tiler: Tiler,
    #[serde(rename(serialize = "imu-gps", deserialize = "imu-gps"))]
    pub imu_gps: ImuGps,
    #[serde(rename(serialize = "startup-defaults", deserialize = "startup-defaults"))]
    pub startup_defaults: StartupDefaults,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Tiler {
    pub url: Url,
    /// Determines tile_size, default is 256 if scale not provided/supported
    /// "1" => 256
    /// "2" => 512
    /// "4" => 1024
    #[serde(default)]
    pub scale: Option<Scale>,
    #[serde(default)]
    pub support_daynight: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Window {
    pub title: String,
    pub width: u16,
    pub height: u16,
    pub target_fps: u8,
    // fullscreen/etc
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ImuGps {
    /// Location relative to center of rear axle, [x, y, z] meters
    pub mount_location: [f64; 3],
    // TODO - orientation/rotation for alignment
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StartupDefaults {
    pub daynight: Daylight,
    // TODO - need to use the new_clamped() methods or do validation checks
    pub zoom: Zoom,
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl FromStr for Config {
    type Err = LoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config = toml::from_str(s)?;
        Ok(config)
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, LoadError> {
        if !path.as_ref().exists() {
            return Err(LoadError::LoadPath(path.as_ref().to_path_buf()));
        }
        let p = fs::canonicalize(path)?;
        let content = fs::read_to_string(&p)?;
        let config = Config::from_str(&content)?;
        log::debug!("Loading config {}", p.display());
        //config.validate()?; TODO
        Ok(config)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), WriteError> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn sample_config() -> Self {
        Config {
            name: "sample config".to_string(),
            window: Window {
                title: "VehicleNAV".to_string(),
                width: 800,
                height: 600,
                target_fps: 60,
            },
            tiler: Tiler {
                url: Url::parse("http://127.0.0.1:8553/v1/tile").unwrap(),
                scale: Some(Scale::Four),
                support_daynight: true,
            },
            imu_gps: ImuGps {
                mount_location: [0.0; 3],
            },
            startup_defaults: StartupDefaults {
                daynight: Daylight::Day,
                zoom: Zoom::new_clamped(11),
                latitude: Latitude(47.453551),
                longitude: Longitude(-116.788118),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn load_path_err() {
        let path = std::env::current_dir().unwrap().join("no_dir");
        let res = Config::load(&path);
        assert!(matches!(res, Err(LoadError::LoadPath(p)) if p == path));
    }

    #[test]
    fn load_sample() {
        let path = std::env::current_dir()
            .unwrap()
            .join("sample_config")
            .join("config.toml");
        let config = Config::load(&path).unwrap();

        assert_eq!(config.name.as_str(), "sample config");

        assert_eq!(config.window.title.as_str(), "VehicleNAV");
        assert_eq!(config.window.width, 800);
        assert_eq!(config.window.height, 600);
        assert_eq!(config.window.target_fps, 60);

        assert_eq!(
            config.tiler.url,
            Url::parse("http://127.0.0.1:8553/v1/tile").unwrap()
        );
        assert_eq!(config.tiler.scale, Some(Scale::Four));
        assert_eq!(config.tiler.support_daynight, true);

        assert_relative_eq!(config.imu_gps.mount_location[0], 0.0);
        assert_relative_eq!(config.imu_gps.mount_location[1], 0.0);
        assert_relative_eq!(config.imu_gps.mount_location[2], 0.0);

        assert_eq!(config.startup_defaults.daynight, Daylight::Day);
        assert_eq!(config.startup_defaults.zoom, Zoom::new_clamped(11));
        assert_relative_eq!(config.startup_defaults.latitude.0, 47.453551);
        assert_relative_eq!(config.startup_defaults.longitude.0, -116.788118);

        assert_eq!(config, Config::sample_config());
    }
}
