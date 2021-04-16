#![deny(warnings)]

use map_tiler::{Config, MapTiler};
use osm_client::{Daylight, OsmClient, Scale};
use raylib::prelude::*;
use url::Url;
//use std::time::Duration;

// links
// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames
// https://wiki.openstreetmap.org/wiki/Zoom_levels
//
// https://github.com/rinigus/osmscout-server/blob/master/README.api.md

// TODO
// - config crate, toml file
//   * table sections: window, maps/tiler, tile_server.url/etc
//   * startup lat/lon, day/night
//   * tile_size or map_scale
//   * max_drawn_route_waypoints
// - cli opts

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    //let url = Url::parse("http://localhost:8080").unwrap();
    let url = Url::parse("http://127.0.0.1:8553/v1/tile").unwrap();
    // TODO - with_timeout
    // add setter methods for changing at runtime
    let client = OsmClient::new(url)
        .with_daylight(Daylight::Day)
        .with_scale(Scale::Four);

    let mut tiler = MapTiler::new(
        client,
        Config {
            width: 1024,
            height: 1024,
            tile_size: 1024,
        },
    )
    .unwrap();

    let zoom = 11;
    let tile_pixmap = tiler
        .request_tiles(47.453551, -116.788118, zoom.into())
        .unwrap();

    let screen_width = 1024;
    let screen_height = 1024;
    //let screen_width = 800 + 200;
    //let screen_height = 600 + 200;

    let (mut rl, rl_t) = RaylibBuilder::default()
        .width(screen_width)
        .height(screen_height)
        .title("VehicleNAV")
        .build();

    rl.set_target_fps(60);
    // rl.get_frame_time()

    let mut tile_image = Image::from(tile_pixmap);
    tile_image.resize(screen_width, screen_height);

    let tile_texture = Texture2D::from(tile_image);

    let bg_color = ffi::Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    let white = ffi::Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    loop {
        if rl.window_should_close() {
            break;
        }

        /*
        // TODO - use dir keys to move center lat/lon
        // use asdf to move origin around with path waypoints
        if IsKeyDown(KeyboardKey::KEY_RIGHT as _) {
            println!("right");
        }
        */

        let mut dh = rl.begin_drawing(&rl_t);

        dh.clear_background(bg_color);

        dh.draw_texture(
            &tile_texture,
            screen_width / 2 - tile_texture.width / 2,
            screen_height / 2 - tile_texture.height / 2,
            white,
        );
    }
}
