use map_tiler::{Config, MapTiler};
use osm_client::{Daylight, OsmClient, Scale};
use raylib_sys::*;
use std::ffi::CString;
use url::Url;
//use std::time::Duration;

// raylib examples/textures/textures_image_drawing.c
//
// links
// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames
// https://wiki.openstreetmap.org/wiki/Zoom_levels
//
// https://github.com/rinigus/osmscout-server/blob/master/README.api.md

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    //let url = Url::parse("http://localhost:8080").unwrap();
    let url = Url::parse("http://127.0.0.1:8553/v1/tile").unwrap();
    // TODO - with_timeout
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
    let pixmap = tiler.request_tiles(47.453551, -116.788118, zoom).unwrap();
    let rgba_bytes = pixmap.data();

    //let screen_width = 1024;
    //let screen_height = 1024;
    let screen_width = 800 + 200;
    let screen_height = 600 + 200;

    let title = CString::new("raylib [core] example - basic window").unwrap();
    unsafe {
        InitWindow(screen_width, screen_height, title.as_ptr());
        SetTargetFPS(60);
    }

    let image = unsafe {
        let src_img = Image {
            data: rgba_bytes.as_ptr() as *mut _, // Should be used read-only just for ImageCopy
            width: 1024,
            height: 1024,
            mipmaps: 1,
            format: PixelFormat::PIXELFORMAT_PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as _,
        };
        let mut img = ImageCopy(src_img);
        ImageResize(&mut img, screen_width, screen_height);
        img
    };
    let tile_texture = unsafe { LoadTextureFromImage(image) };

    let bg_color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    let white = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    loop {
        unsafe {
            if WindowShouldClose() {
                break;
            }

            BeginDrawing();

            ClearBackground(bg_color);

            DrawTexture(
                tile_texture,
                screen_width / 2 - tile_texture.width / 2,
                screen_height / 2 - tile_texture.height / 2,
                white,
            );

            EndDrawing();
        }
    }

    unsafe {
        UnloadTexture(tile_texture);

        CloseWindow();
    }
}
