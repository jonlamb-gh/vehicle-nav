use crate::types::{Coordinate, Scale, Zoom};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct CoordinateTransform {
    tile_size: u32,
    zoom: Zoom,
    x_center: f64,
    y_center: f64,
    image_width: u32,
    image_height: u32,
}

impl CoordinateTransform {
    pub fn new(
        center: &Coordinate,
        scale: Scale,
        zoom: Zoom,
        image_width: u32,
        image_height: u32,
    ) -> Self {
        CoordinateTransform {
            tile_size: scale.tile_size(),
            zoom,
            x_center: util::lon_to_x(center.longitude, zoom),
            y_center: util::lat_to_y(center.latitude, zoom),
            image_width,
            image_height,
        }
    }

    pub fn update(&mut self, center: &Coordinate, zoom: Zoom) {
        self.zoom = zoom;
        self.x_center = util::lon_to_x(center.longitude, zoom);
        self.y_center = util::lat_to_y(center.latitude, zoom);
    }

    pub fn coordinate_to_pixel(&self, coord: &Coordinate) -> (f64, f64) {
        let x = self.x_to_px(util::lon_to_x(coord.longitude, self.zoom));
        let y = self.y_to_px(util::lat_to_y(coord.latitude, self.zoom));
        (x, y)
    }

    fn x_to_px(&self, x: f64) -> f64 {
        let px = (x - self.x_center) * self.tile_size as f64 + self.image_width as f64 / 2f64;
        px.round()
    }

    fn y_to_px(&self, y: f64) -> f64 {
        let px = (y - self.y_center) * self.tile_size as f64 + self.image_height as f64 / 2f64;
        px.round()
    }
}

pub mod util {
    use crate::types::{Latitude, Longitude, Zoom};
    use std::f64::consts::PI;

    pub fn lon_to_x(mut lon: Longitude, zoom: Zoom) -> f64 {
        if !(-180_f64..180_f64).contains(&lon.0) {
            lon.0 = (lon.0 + 180_f64) % 360_f64 - 180_f64;
        }

        ((lon.0 + 180_f64) / 360_f64) * 2_f64.powi(zoom.get().into())
    }

    pub fn lat_to_y(mut lat: Latitude, zoom: Zoom) -> f64 {
        if !(-90_f64..90_f64).contains(&lat.0) {
            lat.0 = (lat.0 + 90_f64) % 180_f64 - 90_f64;
        }

        (1_f64 - ((lat.0 * PI / 180_f64).tan() + 1_f64 / (lat.0 * PI / 180_f64).cos()).ln() / PI)
            / 2_f64
            * 2_f64.powi(zoom.get().into())
    }

    pub fn y_to_lat(y: f64, zoom: Zoom) -> Latitude {
        Latitude(
            (PI * (1_f64 - 2_f64 * y / 2_f64.powi(zoom.get().into())))
                .sinh()
                .atan()
                / PI
                * 180_f64,
        )
    }

    pub fn x_to_lon(x: f64, zoom: Zoom) -> Longitude {
        Longitude(x / 2_f64.powi(zoom.get().into()) * 360_f64 - 180_f64)
    }
}
