//! NOTE: a lot of stuff in here was taken directly from https://crates.io/crates/staticmap

#![deny(warnings)]

use bytes::Bytes;
use common::{util::*, Coordinate, Zoom};
use err_derive::Error;
use osm_client::OsmClient;
use rayon::prelude::*;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

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

// TODO - refactor, use stuff in common lib
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

// TODO - refactor
#[derive(Debug)]
struct Coord {
    tile_x: u32,
    tile_y: u32,
    x: i32,
    y: i32,
}

#[derive(Debug)]
pub struct MapTiler {
    client: OsmClient,
    image: Pixmap,
    config: Config,
    tiles: Vec<Coord>,
    tile_image_results: Vec<Result<Bytes, osm_client::Error>>,
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
            tiles: Vec::with_capacity(8),
            tile_image_results: Vec::with_capacity(8),
        })
    }

    pub fn request_tiles(&mut self, center: Coordinate, zoom: Zoom) -> Result<&Pixmap, Error> {
        let x_center = lon_to_x(center.longitude, zoom);
        let y_center = lat_to_y(center.latitude, zoom);

        let x_min = (x_center - (0.5 * self.config.width as f64 / self.config.tile_size as f64))
            .floor() as i32;
        let y_min = (y_center - (0.5 * self.config.height as f64 / self.config.tile_size as f64))
            .floor() as i32;
        let x_max = (x_center + (0.5 * self.config.width as f64 / self.config.tile_size as f64))
            .ceil() as i32;
        let y_max = (y_center + (0.5 * self.config.height as f64 / self.config.tile_size as f64))
            .ceil() as i32;

        let max_tile: i32 = 2i32.pow(zoom.get() as u32);

        self.tiles.clear();
        for x in x_min..x_max {
            for y in y_min..y_max {
                let tile_x: i32 = (x + max_tile) % max_tile;
                let tile_y: i32 = (y + max_tile) % max_tile;
                self.tiles.push(Coord {
                    tile_x: tile_x as u32,
                    tile_y: tile_y as u32,
                    x,
                    y,
                });
            }
        }

        self.tile_image_results.clear();
        self.tile_image_results = self
            .tiles
            .par_iter()
            .map(|c| self.client.request_tile(c.tile_x, c.tile_y, zoom))
            .collect();

        for (tile, tile_image_result) in self.tiles.iter().zip(self.tile_image_results.drain(..)) {
            let tile_image = tile_image_result?;
            let (x, y) = (tile.x, tile.y);
            let (x_px, y_px) = (
                self.config.x_to_px(x_center, x.into()),
                self.config.y_to_px(y_center, y.into()),
            );

            let pixmap = Pixmap::decode_png(&tile_image)?;
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

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {
        assert_eq!(2 + 2, 4);
    }
}
