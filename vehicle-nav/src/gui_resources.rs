use raylib::prelude::*;
use tiny_skia::Pixmap;

// TODO - consider not including this in the binary, LoadImage(file_path)
// expose LoadImageFromMemory() in raylib
// convert to texture, instead of going through a Pixmap
const BACKGROUND_TEXTURE_PNG_BYTES: &'static [u8; 59961] =
    include_bytes!("../assets/toyota_emblem.png");

#[derive(Debug)]
pub struct GuiResources {
    pub background_texture: Texture2D,
    pub map_texture: Option<Texture2D>,
    // vehicle_marker_texture, could just be a primitive shape for now
}

impl GuiResources {
    pub const BG_COLOR: ffi::Color = ffi::Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    pub const MAP_TEXTURE_COLOR: ffi::Color = ffi::Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub fn load(
        rh: &mut RaylibHandle,
        rl_t: &RaylibThread,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pixmap = Pixmap::decode_png(BACKGROUND_TEXTURE_PNG_BYTES)?;
        let image = Image::from(&pixmap);
        let background_texture = rh.load_texture_from_image(rl_t, &image)?;
        Ok(GuiResources {
            background_texture,
            map_texture: None,
        })
    }
}
