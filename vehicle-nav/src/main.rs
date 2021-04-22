//#![deny(warnings)]

use crate::gui_resources::GuiResources;
use crate::map_tile_service::MapTileService;
use crate::opts::Opts;
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
// - add a Coordinate (lat/lon) type, with coordinate transform helpers
//

// main
// does all the GUI stuff with raylib
// gets new Image's from map service, turns them into Texture2D
// knows how to draw features/waypoints/etc on the texture
// deals with user input event management

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

    loop {
        let should_close = running.load(Ordering::SeqCst) != 0 || rl.window_should_close();
        if should_close {
            break;
        }

        // TODO - use arrow keys to move center lat/lon
        // use asdf to move origin around with path waypoints
        // maybe add a helper method on Coordinate, for shift/inc/dec stuff
        // also add saturating_add/sub with clamps
        // coord shift is a function of zoom, constant distince in pixels
        if rl.is_key_pressed(ffi::KeyboardKey::KEY_M) {
            log::debug!("M key pressed, requesting tiles");
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

        // TODO - test out some texture drawing
        // DrawLine*
        // DrawLineStrip
        // probably the Vector2 variants, need to check the coordinate space texture or screen?

        dh.draw_fps(screen_width / 2, screen_height / 2);
    }

    map_shutdown_handle.blocking_shutdown()?;

    log::debug!("Shutdown complete");

    Ok(())
}
