use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi;

impl RaylibHandle {
    /// Setup canvas (framebuffer) to start drawing
    pub fn begin_drawing(&mut self, _: &RaylibThread) -> RaylibDrawHandle {
        unsafe {
            ffi::BeginDrawing();
        };
        RaylibDrawHandle(self)
    }
}

pub struct RaylibDrawHandle<'a>(&'a mut RaylibHandle);

impl<'a> Drop for RaylibDrawHandle<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::EndDrawing();
        }
    }
}

impl<'a> std::ops::Deref for RaylibDrawHandle<'a> {
    type Target = RaylibHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> RaylibDraw for RaylibDrawHandle<'a> {}

pub trait RaylibDraw {
    /// Sets background color (framebuffer clear color).
    #[inline]
    fn clear_background(&mut self, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::ClearBackground(color.into());
        }
    }

    /// Draws a `texture` using specified position and `tint` color.
    #[inline]
    fn draw_texture(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        x: i32,
        y: i32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTexture(*texture.as_ref(), x, y, tint.into());
        }
    }
}
