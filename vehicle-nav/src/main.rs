//#![deny(warnings)]

use crate::gui_resources::GuiResources;
use crate::map_tile_service::MapTileService;
use crate::opts::Opts;
use crate::route_transform_service::RouteTransformService;
use crate::zoom_delta_map::ZoomDeltaMap;
use common::Coordinate;
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
mod route_transform_service;
mod thread;
mod zoom_delta_map;

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
// - rm/cleanup all the log::debug's
//
// - need to tune/revisit the bounded channel sizes/capacity, add config items
// - same with the thread error types, probably just do strings for the hard errors,
//   some stuff is tolerable
// - iron out the error handling patterns with the threads
// - timeout on the shutdown recvrs
// - some of the services don't need a cloned Config, just a ref will do

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

// RouteTransformService thread
// in-mem, ring buffer maybe
// holds the N most recent coordinates
// send converted coords in batches to main, main has buffer of converted cords,
// mains buffer gets cleared when map changes, requests updated stuff
// only need to re-compute when a new map texture/image is recvd/changed
// probably knows about zoom, tolerance filter to reduce nearby points, omit offscreen points, etc
// support multiple routes, RouteId, colored/etc

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

    let zoom_delta_map = ZoomDeltaMap::new(&config);

    let (map_client, map_shutdown_handle) = MapTileService::start(config.clone())?;
    let (route_transform_client, route_transform_shutdown_handle) =
        RouteTransformService::start(config.clone())?;

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

    // these would come from the sensor service
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

    // TODO - manage this somewhere
    let mut route_points: Vec<ffi::Vector2> = Vec::with_capacity(route_coords.len());

    for c in route_coords.into_iter() {
        route_transform_client.push_coordinate(c)?;
    }

    loop {
        let should_close = running.load(Ordering::SeqCst) != 0 || rl.window_should_close();
        if should_close {
            break;
        }

        // coord shift is a function of zoom, constant distince in pixels,
        // put that dist in the config
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_M) {
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_UP) {
            let (d_lat, _) = zoom_delta_map.get(zoom);
            center_coord.latitude.saturating_add(d_lat);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_DOWN) {
            let (d_lat, _) = zoom_delta_map.get(zoom);
            center_coord.latitude.saturating_sub(d_lat);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_RIGHT) {
            let (_, d_lon) = zoom_delta_map.get(zoom);
            center_coord.longitude.saturating_add(d_lon);
            map_client.request(center_coord, zoom)?;
        }
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_LEFT) {
            let (_, d_lon) = zoom_delta_map.get(zoom);
            center_coord.longitude.saturating_sub(d_lon);
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

            route_points.clear();
            route_transform_client.get_route(center_coord, zoom)?;

            let mut map_image = Image::from(&map_pixmap);
            map_image.resize(screen_width, screen_height);
            let _ = resources
                .map_texture
                .replace(rl.load_texture_from_image(&rl_t, &map_image)?);
        }

        if let Some(mut route) = route_transform_client.try_recv()? {
            log::debug!("Got route len={}", route.route_chunk.len());
            route_points = route.route_chunk.drain(..).collect();
        }

        let mut dh = rl.begin_drawing(&rl_t);

        // TODO - consider clearing ealier on, route stuff gets messed up on quick changes
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
            dh.draw_line_ex(pair[0], pair[1], 2.0, GuiResources::ROUTE_COLOR);
        }

        dh.draw_fps(25, 25);
    }

    map_shutdown_handle.blocking_shutdown()?;
    route_transform_shutdown_handle.blocking_shutdown()?;

    log::debug!("Shutdown complete");

    Ok(())
}
