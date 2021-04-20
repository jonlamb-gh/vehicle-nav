//#![deny(warnings)]

use crate::map_tile_service::MapTileService;
use crate::opts::Opts;
use config::Config;
use map_tiler::{Config as MapTilerConfig, MapTiler};
use osm_client::{Daylight, OsmClient, Scale};
use raylib::prelude::*;
use std::process;
use structopt::StructOpt;

//use std::time::Duration;

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
//   * table sections: window, maps/tiler, tile_server.url/etc
//   * startup lat/lon, day/night
//   * tile_size or map_scale
//   * imu table, mount location/rotation/alignment stuff
//   * max_drawn_route_waypoints
// - cli opts
// - make a proper error type in lib/raylib, rm the string errors

// main
// does all the GUI stuff with raylib
// gets new Image's from map service, turns them into Texture2D
// knows how to draw features/waypoints/etc on the texture
// deals with user input event management

// MapTileService, thread
// owns MapTiler
// deals with requests
// yields a new Image (Image->Texture2D done in main thread)
//   - might not do Image in the thread, it's got c_void ptr, need to consider Send stuff
// req/resp is async, main will check each iter if a new response recvd/ready
// req has center lat/lon and zoom

// SensorService, thread
// IMU/GPS serial driver
// takes config as setup
// yields structured IMU and GPS data

// StorageService, thread
// deals with the sqlite read/write
// should eventually use the cipher/encryption features
// waypoints/routes/sensor-data/etc

const BG_COLOR: ffi::Color = ffi::Color {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

const MAP_TEXTURE_COLOR: ffi::Color = ffi::Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};

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

    let config = Config::load(&opts.config)?;

    let (map_client, map_shutdown_handle) = MapTileService::start(config.clone())?;

    let mut map_texture = None;

    /*
    map_client.request(
        config.startup_defaults.lat,
        config.startup_defaults.lon,
        config.startup_defaults.zoom,
    )?;
    */

    // TODO - with_timeout, add timeout (seconds) to Config
    // add setter methods for changing at runtime
    // TODO - use the config items
    /*
    let client = OsmClient::new(config.tiler.url)
        .with_daylight(Daylight::Day)
        .with_scale(Scale::Four);

    let mut tiler = MapTiler::new(
        client,
        MapTilerConfig {
            width: config.window.width.into(),
            height: config.window.height.into(),
            tile_size: 1024, // TODO - config
        },
    )?;

    let tile_pixmap = tiler.request_tiles(
        config.startup_defaults.lat,
        config.startup_defaults.lon,
        config.startup_defaults.zoom,
    )?;
    */

    //let screen_width = 1024;
    //let screen_height = 1024;
    let screen_width = config.window.width.into();
    let screen_height = config.window.height.into();

    let (mut rl, rl_t) = RaylibBuilder::default()
        .width(screen_width)
        .height(screen_height)
        .title(config.window.title.as_str())
        .build();

    rl.set_target_fps(config.window.target_fps.into());
    // rl.get_frame_time()

    /*
    let mut tile_image = Image::from(tile_pixmap);
    tile_image.resize(screen_width, screen_height);

    let tile_texture = rl.load_texture_from_image(&rl_t, &tile_image)?;
    */

    // TODO - also setup control-c/sig handler
    loop {
        if rl.window_should_close() {
            break;
        }

        if rl.is_key_released(ffi::KeyboardKey::KEY_M) {
            println!("M key released, request tiles");
            map_client.request(
                config.startup_defaults.lat,
                config.startup_defaults.lon,
                config.startup_defaults.zoom,
            )?;
        }

        /*
        // TODO - use dir keys to move center lat/lon
        // use asdf to move origin around with path waypoints
        if IsKeyDown(KeyboardKey::KEY_RIGHT as _) {
            println!("right");
        }
        */

        if let Some(map_pixmap) = map_client.try_recv()? {
            log::debug!("Got pixmap");
            let mut map_image = Image::from(&map_pixmap);
            map_image.resize(screen_width, screen_height);
            map_texture = Some(rl.load_texture_from_image(&rl_t, &map_image)?);
        }

        let mut dh = rl.begin_drawing(&rl_t);

        dh.clear_background(BG_COLOR);

        if let Some(t) = &map_texture {
            dh.draw_texture(
                t,
                screen_width / 2 - t.width / 2,
                screen_height / 2 - t.height / 2,
                MAP_TEXTURE_COLOR,
            );
        }
    }

    map_shutdown_handle.blocking_shutdown()?;

    // TODO - doing this make the texture drop correctly and doesn't segfault
    // FIXME
    // otherwise, it seems to get dropped in the wrong order / too late?
    let _t = map_texture.take();

    log::debug!("Shutdown complete");

    // TODO - segfaulting again
    // INFO: Window closed successfully
    // Segmentation fault (core dumped)

    Ok(())
}
