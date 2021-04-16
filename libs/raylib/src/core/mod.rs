#[macro_use]
mod macros;

pub mod drawing;
pub mod input;
pub mod texture;
pub mod window;

use crate::ffi;
use std::ffi::CString;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct RaylibThread(PhantomData<*const ()>);

#[derive(Debug)]
pub struct RaylibHandle(());

impl Drop for RaylibHandle {
    fn drop(&mut self) {
        if IS_INITIALIZED.load(Ordering::Relaxed) {
            unsafe {
                ffi::CloseWindow();
            }
            IS_INITIALIZED.store(false, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, Default)]
pub struct RaylibBuilder {
    fullscreen_mode: bool,
    window_resizable: bool,
    window_undecorated: bool,
    window_transparent: bool,
    msaa_4x_hint: bool,
    vsync_hint: bool,
    width: i32,
    height: i32,
    title: String,
}

pub fn init() -> RaylibBuilder {
    RaylibBuilder {
        width: 640,
        height: 480,
        title: "raylib-rs".to_string(),
        ..Default::default()
    }
}

impl RaylibBuilder {
    /// Sets the window to be fullscreen.
    pub fn fullscreen(&mut self) -> &mut Self {
        self.fullscreen_mode = true;
        self
    }

    /// Sets the window to be resizable.
    pub fn resizable(&mut self) -> &mut Self {
        self.window_resizable = true;
        self
    }

    /// Sets the window to be undecorated (without a border).
    pub fn undecorated(&mut self) -> &mut Self {
        self.window_undecorated = true;
        self
    }

    /// Sets the window to be transparent.
    pub fn transparent(&mut self) -> &mut Self {
        self.window_transparent = true;
        self
    }

    /// Hints that 4x MSAA (anti-aliasing) should be enabled. The system's graphics drivers may override this setting.
    pub fn msaa_4x(&mut self) -> &mut Self {
        self.msaa_4x_hint = true;
        self
    }

    /// Hints that vertical sync (VSync) should be enabled. The system's graphics drivers may override this setting.
    pub fn vsync(&mut self) -> &mut Self {
        self.vsync_hint = true;
        self
    }

    /// Sets the window's width.
    pub fn width(&mut self, w: i32) -> &mut Self {
        self.width = w;
        self
    }

    /// Sets the window's height.
    pub fn height(&mut self, h: i32) -> &mut Self {
        self.height = h;
        self
    }

    /// Sets the window's width and height.
    pub fn size(&mut self, w: i32, h: i32) -> &mut Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Sets the window title.
    pub fn title(&mut self, text: &str) -> &mut Self {
        self.title = text.to_string();
        self
    }

    /// Builds and initializes a Raylib window.
    ///
    /// # Panics
    ///
    /// Attempting to initialize Raylib more than once will result in a panic.
    pub fn build(&self) -> (RaylibHandle, RaylibThread) {
        use ffi::ConfigFlags::*;
        let mut flags = 0u32;
        if self.fullscreen_mode {
            flags |= FLAG_FULLSCREEN_MODE as u32;
        }
        if self.window_resizable {
            flags |= FLAG_WINDOW_RESIZABLE as u32;
        }
        if self.window_undecorated {
            flags |= FLAG_WINDOW_UNDECORATED as u32;
        }
        if self.window_transparent {
            flags |= FLAG_WINDOW_TRANSPARENT as u32;
        }
        if self.msaa_4x_hint {
            flags |= FLAG_MSAA_4X_HINT as u32;
        }
        if self.vsync_hint {
            flags |= FLAG_VSYNC_HINT as u32;
        }

        unsafe {
            ffi::SetConfigFlags(flags as u32);
        }
        let rl = init_window(self.width, self.height, &self.title);
        (rl, RaylibThread(PhantomData))
    }
}

/// Initializes window and OpenGL context.
///
/// # Panics
///
/// Attempting to initialize Raylib more than once will result in a panic.
fn init_window(width: i32, height: i32, title: &str) -> RaylibHandle {
    if IS_INITIALIZED.load(Ordering::Relaxed) {
        panic!("Attempted to initialize raylib-rs more than once!");
    } else {
        unsafe {
            let c_title = CString::new(title).unwrap();
            ffi::InitWindow(width, height, c_title.as_ptr());
        }
        if !unsafe { ffi::IsWindowReady() } {
            panic!("Attempting to create window failed!");
        }
        IS_INITIALIZED.store(true, Ordering::Relaxed);
        RaylibHandle(())
    }
}
