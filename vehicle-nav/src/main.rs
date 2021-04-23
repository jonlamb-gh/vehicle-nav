//#![deny(warnings)]

use crate::gui_resources::GuiResources;
use crate::map_tile_service::MapTileService;
use crate::opts::Opts;
use common::{Coordinate, CoordinateTransform};
use config::Config;
use raylib::prelude::*;
use std::process;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use structopt::StructOpt;

//use osm_client::{Daylight, OsmClient, Scale};
//use map_tiler::{Config as MapTilerConfig, MapTiler};
//use std::time::Duration;

mod gui_resources;
mod map_tile_service;
mod opts;
mod thread;

// links for the README
// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames
// https://wiki.openstreetmap.org/wiki/Zoom_levels
//
// https://github.com/rinigus/osmscout-server/blob/master/README.api.md

// TODO
// - config crate, toml file
//   * use the newtypes from the other crates for basic sanity checking
//   * max_rendered_route_waypoints/lines
//   * add client timeout
//   * line color(s)
// - cli opts accept env vars
//

// main
// does all the GUI stuff with raylib
// gets new Image's from map service, turns them into Texture2D
// knows how to draw features/waypoints/etc on the texture
// deals with user input event management
// need some concept of pages/views for the different
// contexts (map/nav, routes, sensor display, etc)

// MapTileService thread
// owns MapTiler
// deals with requests
// yields a new Image (Image->Texture2D done in main thread)
//   - might not do Image in the thread, it's got c_void ptr, need to consider Send stuff
// req/resp is async, main will check each iter if a new response recvd/ready
// req has center lat/lon and zoom

// SensorService thread
// IMU/GPS serial driver
// takes config as setup
// yields structured IMU and GPS data

// StorageService thread
// deals with the sqlite read/write
// should eventually use the cipher/encryption features
// waypoints/routes/sensor-data/etc
// probably hold off on this one until the sensor and routing bits are in order
// figure out when and how often to store/manage the data

// RouteTrackerService thread
// in-mem, ring buffer maybe
// holds the N most recent coordinates
// and another buffer for the converted screen coordinates, maybe, so it's not converted on each
// draw loop
// only need to re-compute when a new map texture/image is recvd/changed
// probably knows about zoom, tolerance filter to reduce nearby points, omit offscreen points, etc

fn main() {
    match do_main() {
        Ok(()) => (),
        Err(e) => {
            log::error!("{}", e);
            process::exit(exitcode::SOFTWARE);
        }
    }
}

fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let opts = Opts::from_args();

    if let Some(path) = &opts.write_default_config {
        let config = Config::sample_config();
        config.write_to_file(path)?;
        log::info!("Wrote default configuration file to {}", path.display());
        return Ok(());
    }

    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            log::debug!("Shutting down");
        } else {
            log::warn!("Forcing exit");
            process::exit(exitcode::SOFTWARE);
        }
    })?;

    let config = Config::load(&opts.config)?;

    let (map_client, map_shutdown_handle) = MapTileService::start(config.clone())?;

    let screen_width = config.window.width.into();
    let screen_height = config.window.height.into();

    let (mut rl, rl_t) = RaylibBuilder::default()
        .width(screen_width)
        .height(screen_height)
        .title(config.window.title.as_str())
        .build();

    rl.set_target_fps(config.window.target_fps.into());
    // rl.get_frame_time()

    let mut zoom = config.startup_defaults.zoom;
    let mut center_coord = Coordinate::from((
        config.startup_defaults.latitude,
        config.startup_defaults.longitude,
    ));
    let mut resources = GuiResources::load(&mut rl, &rl_t)?;

    // one of the services should convert the Coordinates to screen/texture points
    let route_coords: Vec<Coordinate> = vec![
        Coordinate::new(47.453551, -116.788118),
        Coordinate::new(47.453358, -116.787340),
        Coordinate::new(47.454036, -116.787275),
        Coordinate::new(47.454054, -116.787093),
        Coordinate::new(47.453927, -116.786878),
        Coordinate::new(47.453561, -116.786750),
        Coordinate::new(47.454326, -116.786530),
        Coordinate::new(47.454243, -116.785156),
        Coordinate::new(47.455712, -116.784239),
        Coordinate::new(47.456655, -116.783225),
    ];

    // TODO - input stuff needs to update this
    let mut transform = CoordinateTransform::new(
        &center_coord,
        config.tiler.scale.unwrap_or_default(),
        zoom,
        config.window.width.into(),
        config.window.height.into(),
    );

    pub const ROUTE_COLOR: ffi::Color = ffi::Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    let mut route_points: Vec<ffi::Vector2> = Vec::with_capacity(32);

    loop {
        let should_close = running.load(Ordering::SeqCst) != 0 || rl.window_should_close();
        if should_close {
            break;
        }

        // TODO - use arrow keys to move center lat/lon
        // use asdf to move origin around with path waypoints
        // maybe add a helper method on Coordinate, for shift/inc/dec stuff
        // also add saturating_add/sub with clamps
        // coord shift is a function of zoom, constant distince in pixels, put that dist in the
        // config
        //
        // need to update transform stuff
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_M) {
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_UP) {
            center_coord.latitude.saturating_add(0.001);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_DOWN) {
            center_coord.latitude.saturating_sub(0.001);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_RIGHT) {
            center_coord.longitude.saturating_add(0.001);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_LEFT) {
            center_coord.longitude.saturating_sub(0.001);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_I) {
            zoom.increment();
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_O) {
            zoom.decrement();
            map_client.request(center_coord, zoom)?;
        }

        if let Some(map_pixmap) = map_client.try_recv()? {
            log::debug!("Got pixmap");
            let mut map_image = Image::from(&map_pixmap);
            map_image.resize(screen_width, screen_height);
            let _ = resources
                .map_texture
                .replace(rl.load_texture_from_image(&rl_t, &map_image)?);

            // TODO - move the "rebuild route" stuff somewhere
            // not sure about pre-allocated vec stuff and collect
            transform.update(&center_coord, zoom);
            route_points = route_coords
                .iter()
                .map(|c| {
                    let (x, y) = transform.coordinate_to_pixel(c);
                    ffi::Vector2 {
                        x: x as _,
                        y: y as _,
                    }
                })
                .collect();
        }

        let mut dh = rl.begin_drawing(&rl_t);

        dh.clear_background(GuiResources::BG_COLOR);

        let foreground_texture = match &resources.map_texture {
            Some(map_texture) => map_texture,
            None => &resources.background_texture,
        };

        dh.draw_texture(
            foreground_texture,
            screen_width / 2 - foreground_texture.width / 2,
            screen_height / 2 - foreground_texture.height / 2,
            GuiResources::MAP_TEXTURE_COLOR,
        );

        // works but no line thickness
        //dh.draw_line_strip(&route_points, ROUTE_COLOR);

        // line size should be a function of zoom
        for pair in route_points.windows(2) {
            dh.draw_line_ex(pair[0], pair[1], 2.0, ROUTE_COLOR);
        }

        dh.draw_fps(25, 25);
    }

    map_shutdown_handle.blocking_shutdown()?;

    log::debug!("Shutdown complete");

    Ok(())
}
