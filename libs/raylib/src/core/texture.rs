use crate::core::{RaylibHandle, RaylibThread};
use crate::{error::Error, ffi};
use std::convert::TryInto;
use std::ffi::CString;
use tiny_skia::Pixmap;

make_thin_wrapper!(Image, ffi::Image, ffi::UnloadImage);
make_thin_wrapper!(Texture2D, ffi::Texture2D, ffi::UnloadTexture);

// TODO - cleanup bindgen PixelFormat, remove the PIXELFORMAT_ prefix on enum
// - add a common Error enum type, things like load_texture_from_image return string error,
// refactor it
// - consider giving Image Send blessing, should be ok

impl Image {
    pub fn width(&self) -> i32 {
        self.0.width
    }

    pub fn height(&self) -> i32 {
        self.0.height
    }

    pub fn mipmaps(&self) -> i32 {
        self.0.mipmaps
    }

    /// # Safety
    /// TODO
    pub unsafe fn raw_data(&self) -> *mut ::std::os::raw::c_void {
        self.0.data
    }

    /*
    #[inline]
    pub fn format(&self) -> crate::consts::PixelFormat {
        let i: u32 = self.format as u32;
        unsafe { std::mem::transmute(i) }
    }
    */

    /// Resizes `image` (bilinear filtering).
    #[inline]
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        unsafe {
            ffi::ImageResize(&mut self.0, new_width, new_height);
        }
    }

    /// Resizes `image` (nearest-neighbor scaling).
    #[inline]
    pub fn resize_nn(&mut self, new_width: i32, new_height: i32) {
        unsafe {
            ffi::ImageResizeNN(&mut self.0, new_width, new_height);
        }
    }
}

impl From<&Pixmap> for Image {
    fn from(pixmap: &Pixmap) -> Self {
        // Pixmap byte order: RGBA
        let src_img = ffi::Image {
            data: pixmap.data().as_ptr() as *mut _, // Ok to cast, it's read-only used just in ImageCopy
            width: pixmap.width().try_into().unwrap(),
            height: pixmap.height().try_into().unwrap(),
            mipmaps: 1,
            format: ffi::PixelFormat::PIXELFORMAT_PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as _,
        };
        unsafe { Image::from_raw(ffi::ImageCopy(src_img)) }
    }
}

impl RaylibHandle {
    /// Loads texture from file into GPU memory (VRAM).
    pub fn load_texture(&mut self, _: &RaylibThread, filename: &str) -> Result<Texture2D, Error> {
        let c_filename = CString::new(filename).unwrap();
        let t = unsafe { ffi::LoadTexture(c_filename.as_ptr()) };
        if t.id == 0 {
            return Err(Error::TextureLoadFromFile(filename.to_string()));
        }
        Ok(Texture2D(t))
    }

    /// Loads texture from image data.
    #[inline]
    pub fn load_texture_from_image(
        &mut self,
        _: &RaylibThread,
        image: &Image,
    ) -> Result<Texture2D, String> {
        let t = unsafe { ffi::LoadTextureFromImage(image.0) };
        if t.id == 0 {
            return Err("failed to load image as a texture.".to_string()); // TODO - error type
        }
        Ok(Texture2D(t))
    }
}
