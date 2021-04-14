use osm_client::OsmClient;
use raylib_sys::*;
use std::ffi::CString;
use std::time::Duration;
use url::Url;

// raylib examples/textures/textures_image_drawing.c
//
// links
// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames
// https://wiki.openstreetmap.org/wiki/Zoom_levels
//
// https://github.com/rinigus/osmscout-server/blob/master/README.api.md

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // TODO - need to actually stitch tiles together to see something reasonable
    // osm-renderer and staticmap crates do this
    //
    // not using staticmap directly, should be able to do most of the overlay stuff
    // in GPU/GL land
    //
    // probably should do the tiling with textures instead of CPU side Pixmap/etc
    //
    // draw the base map to the texture
    // draw attributes/etc on texture, using coordinate transforms from the map-tiler lib
    // then draw the texture to screen, deal with scale/resize there

    let url = Url::parse("http://localhost:8080").unwrap();
    let client = OsmClient::new(url, Duration::from_secs(10));

    let png_bytes = client.request_tile(1, 4, 1).unwrap();
    let png_data: &[u8] = &png_bytes;

    let screen_width = 800;
    let screen_height = 600;

    let title = CString::new("raylib [core] example - basic window").unwrap();
    unsafe {
        InitWindow(screen_width, screen_height, title.as_ptr());
        SetTargetFPS(60);
    }

    let file_type = CString::new(".png").unwrap();
    let tile = unsafe {
        let mut img =
            LoadImageFromMemory(file_type.as_ptr(), png_data.as_ptr(), png_data.len() as i32);
        ImageResize(&mut img, screen_width, screen_height);
        img
    };

    let tile_texture = unsafe { LoadTextureFromImage(tile) };
    unsafe { UnloadImage(tile) };

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
                screen_height / 2 - tile_texture.height / 2 - 40,
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
