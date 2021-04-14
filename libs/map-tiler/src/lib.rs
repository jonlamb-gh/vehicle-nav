//! NOTE: a lot of stuff in here was taken directly from https://crates.io/crates/staticmap

//#![deny(warnings)]

use crate::util::*;
use bytes::Bytes;
use err_derive::Error;
use osm_client::OsmClient;
use rayon::prelude::*;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

// const TILE_SIZE: usize = 256

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "The image width ({}) or height ({}) is invalid", _0, _1)]
    ImageSize(u32, u32),

    #[error(display = "Client error: {}", _0)]
    Client(#[error(source)] osm_client::Error),

    #[error(display = "PNG decode error: {}", _0)]
    TileDecodeError(#[error(source)] png::DecodingError),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
}

// TODO - adjust these
impl Default for Config {
    fn default() -> Self {
        Config {
            width: 1024,
            height: 1024,
            tile_size: 1024,
        }
    }
}

impl Config {
    pub(crate) fn x_to_px(&self, x_center: f64, x: f64) -> f64 {
        let px = (x - x_center) * self.tile_size as f64 + self.width as f64 / 2f64;
        px.round()
    }

    pub(crate) fn y_to_px(&self, y_center: f64, y: f64) -> f64 {
        let px = (y - y_center) * self.tile_size as f64 + self.height as f64 / 2f64;
        px.round()
    }
}

// API things
// lat_center: f64,
// lon_center: f64,
// zoom: u8

// TODO
// - consider NonZeroU32 for image size
// - make image w/h be a function of num_tiles(rows, cols), or other way around
//   * don't do any resize/scaling stuff here
// - https://github.com/rinigus/osmscout-server/blob/master/README.api.md can do
//   1024x1024 tiles
#[derive(Debug)]
pub struct MapTiler {
    client: OsmClient,
    image: Pixmap,
    config: Config,
}

impl MapTiler {
    pub fn new(client: OsmClient, config: Config) -> Result<Self, Error> {
        let image = Pixmap::new(config.width, config.height)
            .ok_or(Error::ImageSize(config.width, config.height))?;
        log::debug!(
            "Created new MapTiler w={}, h={}, tile_size={}",
            config.width,
            config.height,
            config.tile_size
        );
        Ok(MapTiler {
            client,
            image,
            config,
        })
    }

    // TODO - rm the debug's
    pub fn request_tiles(
        &mut self,
        lat_center: f64,
        lon_center: f64,
        zoom: u8,
    ) -> Result<&Pixmap, Error> {
        let x_center = lon_to_x(lon_center, zoom);
        let y_center = lat_to_y(lat_center, zoom);

        let x_min = (x_center - (0.5 * self.config.width as f64 / self.config.tile_size as f64))
            .floor() as i32;
        let y_min = (y_center - (0.5 * self.config.height as f64 / self.config.tile_size as f64))
            .floor() as i32;
        let x_max = (x_center + (0.5 * self.config.width as f64 / self.config.tile_size as f64))
            .ceil() as i32;
        let y_max = (y_center + (0.5 * self.config.height as f64 / self.config.tile_size as f64))
            .ceil() as i32;

        log::info!("x_center {}", x_center);
        log::info!("y_center {}", y_center);
        log::info!("x_min = {}", x_min);
        log::info!("x_max = {}", x_max);
        log::info!("y_min = {}", y_min);
        log::info!("y_max = {}", y_max);

        let max_tile: i32 = 2i32.pow(zoom as u32);
        log::info!("max_tile = {}", max_tile);

        // TODO - refactor
        struct Coord {
            tile_x: u32,
            tile_y: u32,
            x: i32,
            y: i32,
        }

        // TODO - collapse this, pre-allocate the dynamic stuff
        let mut tiles: Vec<Coord> = Vec::new();
        for x in x_min..x_max {
            for y in y_min..y_max {
                let tile_x: i32 = (x + max_tile) % max_tile;
                let tile_y: i32 = (y + max_tile) % max_tile;
                log::info!("req tile x={}, y={}, x={}, y={}", tile_x, tile_y, x, y);
                tiles.push(Coord {
                    tile_x: tile_x as u32,
                    tile_y: tile_y as u32,
                    x,
                    y,
                });
            }
        }
        log::info!("num_tiles {}", tiles.len());

        // TODO - alloc
        let tile_image_results: Vec<Result<Bytes, osm_client::Error>> = tiles
            .par_iter()
            .map(|c| self.client.request_tile(c.tile_x, c.tile_y, zoom))
            .collect();

        for (tile, tile_image_result) in tiles.iter().zip(tile_image_results) {
            let tile_image = tile_image_result?;
            let (x, y) = (tile.x, tile.y);
            let (x_px, y_px) = (
                self.config.x_to_px(x_center, x.into()),
                self.config.y_to_px(y_center, y.into()),
            );

            let pixmap = Pixmap::decode_png(&tile_image)?;
            log::info!(
                "tile pixmap (w={}, h={}) @ pixel (x={}, y={})",
                pixmap.width(),
                pixmap.height(),
                x_px,
                y_px
            );

            self.image.draw_pixmap(
                x_px as i32,
                y_px as i32,
                pixmap.as_ref(),
                &PixmapPaint::default(),
                Transform::default(),
                None,
            );
        }

        Ok(&self.image)
    }
}

pub mod util {
    use std::f64::consts::PI;

    pub fn lon_to_x(mut lon: f64, zoom: u8) -> f64 {
        if !(-180_f64..180_f64).contains(&lon) {
            lon = (lon + 180_f64) % 360_f64 - 180_f64;
        }

        ((lon + 180_f64) / 360_f64) * 2_f64.powi(zoom.into())
    }

    pub fn lat_to_y(mut lat: f64, zoom: u8) -> f64 {
        if !(-90_f64..90_f64).contains(&lat) {
            lat = (lat + 90_f64) % 180_f64 - 90_f64;
        }

        (1_f64 - ((lat * PI / 180_f64).tan() + 1_f64 / (lat * PI / 180_f64).cos()).ln() / PI)
            / 2_f64
            * 2_f64.powi(zoom.into())
    }

    pub fn y_to_lat(y: f64, zoom: u8) -> f64 {
        (PI * (1_f64 - 2_f64 * y / 2_f64.powi(zoom.into())))
            .sinh()
            .atan()
            / PI
            * 180_f64
    }

    pub fn x_to_lon(x: f64, zoom: u8) -> f64 {
        x / 2_f64.powi(zoom.into()) * 360_f64 - 180_f64
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {
        assert_eq!(2 + 2, 4);
    }
}
