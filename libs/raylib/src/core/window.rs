use crate::core::RaylibHandle;
use crate::ffi;
use std::ffi::{CString, IntoStringError};

/// MonitorInfo grabs the sizes (virtual and physical) of your monitor
#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub width: i32,
    pub height: i32,
    pub physical_width: i32,
    pub physical_height: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WindowState(i32);

impl WindowState {
    pub fn vsync_hint(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_VSYNC_HINT as i32) != 0
    }
    /// Set to try enabling V-Sync on GPU
    pub fn set_vsync_hint(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_VSYNC_HINT as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_VSYNC_HINT as i32);
        }
        self
    }

    pub fn fullscreen_mode(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as i32) != 0
    }
    /// Set to run program in fullscreen
    pub fn set_fullscreen_mode(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as i32);
        }
        self
    }

    pub fn window_resizable(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as i32) != 0
    }
    /// Set to allow resizable window
    pub fn set_window_resizable(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as i32);
        }
        self
    }

    pub fn window_undecorated(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as i32) != 0
    }
    /// Set to disable window decoration (frame and buttons)
    pub fn set_window_undecorated(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as i32);
        }
        self
    }

    pub fn window_hidden(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as i32) != 0
    }
    /// Set to hide window
    pub fn set_window_hidden(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as i32);
        }
        self
    }

    pub fn window_minimized(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as i32) != 0
    }
    /// Set to minimize window (iconify)
    pub fn set_window_minimized(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as i32);
        }
        self
    }

    pub fn window_maximized(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as i32) != 0
    }
    /// Set to maximize window (expanded to monitor)
    pub fn set_window_maximized(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as i32);
        }
        self
    }

    pub fn window_unfocused(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as i32) != 0
    }
    /// Set to window non focused
    pub fn set_window_unfocused(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as i32);
        }
        self
    }

    pub fn window_topmost(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as i32) != 0
    }
    /// Set to window always on top
    pub fn set_window_topmost(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as i32);
        }
        self
    }

    pub fn window_always_run(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as i32) != 0
    }
    /// Set to allow windows running while minimized
    pub fn set_window_always_run(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as i32);
        }
        self
    }

    pub fn window_transparent(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as i32) != 0
    }
    /// Set to allow transparent framebuffer
    pub fn set_window_transparent(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as i32);
        }
        self
    }

    pub fn window_highdpi(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as i32) != 0
    }
    /// Set to support HighDPI
    pub fn set_window_highdpi(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as i32);
        }
        self
    }

    pub fn msaa(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_MSAA_4X_HINT as i32) != 0
    }
    /// Set to try enabling MSAA 4X
    pub fn set_msaa(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_MSAA_4X_HINT as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_MSAA_4X_HINT as i32);
        }
        self
    }

    pub fn interlaced_hint(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_INTERLACED_HINT as i32) != 0
    }
    /// Set to try enabling interlaced video format (for V3D)
    pub fn set_interlaced_hint(mut self, enabled: bool) -> Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_INTERLACED_HINT as i32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_INTERLACED_HINT as i32);
        }
        self
    }
}

/// Get number of connected monitors
#[inline]
pub fn get_monitor_count() -> i32 {
    unsafe { ffi::GetMonitorCount() }
}

/// Get number of connected monitors
/// Only checks that monitor index is in range in debug mode
#[inline]
pub fn get_monitor_width(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");

    unsafe { ffi::GetMonitorWidth(monitor) }
}

/// Get number of connected monitors
/// Only checks that monitor index is in range in debug mode
#[inline]
pub fn get_monitor_height(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");

    unsafe { ffi::GetMonitorHeight(monitor) }
}

/// Get number of connected monitors
/// Only checks that monitor index is in range in debug mode
#[inline]
pub fn get_monitor_physical_width(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");

    unsafe { ffi::GetMonitorPhysicalWidth(monitor) }
}

/// Get number of connected monitors
/// Only checks that monitor index is in range in debug mode
#[inline]
pub fn get_monitor_physical_height(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");

    unsafe { ffi::GetMonitorPhysicalHeight(monitor) }
}

/// Get number of connected monitors
/// Only checks that monitor index is in range in debug mode
#[inline]
pub fn get_monitor_name(monitor: i32) -> Result<String, IntoStringError> {
    let len = get_monitor_count();
    debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");

    Ok(unsafe {
        let c = CString::from_raw(ffi::GetMonitorName(monitor) as *mut i8);
        c.into_string()?
    })
}

pub fn get_monitor_info(monitor: i32) -> Result<MonitorInfo, IntoStringError> {
    let len = get_monitor_count();
    debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");

    Ok(MonitorInfo {
        width: get_monitor_width(monitor),
        height: get_monitor_height(monitor),
        physical_height: get_monitor_physical_height(monitor),
        physical_width: get_monitor_physical_width(monitor),
        name: get_monitor_name(monitor)?,
    })
}

impl RaylibHandle {
    /// Set target FPS (maximum)
    pub fn set_target_fps(&mut self, fps: u32) {
        unsafe {
            ffi::SetTargetFPS(fps as i32);
        }
    }

    /// Returns current FPS
    pub fn get_fps(&self) -> u32 {
        unsafe { ffi::GetFPS() as u32 }
    }

    /// Returns time in seconds for last frame drawn
    pub fn get_frame_time(&self) -> f32 {
        unsafe { ffi::GetFrameTime() }
    }

    /// Returns elapsed time in seconds since InitWindow()
    pub fn get_time(&self) -> f64 {
        unsafe { ffi::GetTime() }
    }
}

impl RaylibHandle {
    /// Checks if `KEY_ESCAPE` or Close icon was pressed.
    #[inline]
    pub fn window_should_close(&self) -> bool {
        unsafe { ffi::WindowShouldClose() }
    }

    /// Checks if window has been initialized successfully.
    #[inline]
    pub fn is_window_ready(&self) -> bool {
        unsafe { ffi::IsWindowReady() }
    }

    /// Checks if window has been minimized (or lost focus).
    #[inline]
    pub fn is_window_minimized(&self) -> bool {
        unsafe { ffi::IsWindowMinimized() }
    }

    /// Checks if window has been resized.
    #[inline]
    pub fn is_window_resized(&self) -> bool {
        unsafe { ffi::IsWindowResized() }
    }

    /// Checks if window has been hidden.
    #[inline]
    pub fn is_window_hidden(&self) -> bool {
        unsafe { ffi::IsWindowResized() }
    }

    /// Returns whether or not window is in fullscreen mode
    #[inline]
    pub fn is_window_fullscreen(&self) -> bool {
        unsafe { ffi::IsWindowFullscreen() }
    }

    // Check if window is currently focused (only PLATFORM_DESKTOP)
    #[inline]
    pub fn is_window_focused(&self) -> bool {
        unsafe { ffi::IsWindowFocused() }
    }

    /// Check if cursor is on the current screen.
    #[inline]
    pub fn is_cursor_on_screen(&self) -> bool {
        unsafe { ffi::IsCursorOnScreen() }
    }

    /// Toggles fullscreen mode (only on desktop platforms).
    #[inline]
    pub fn toggle_fullscreen(&mut self) {
        unsafe {
            ffi::ToggleFullscreen();
        }
    }

    /// Set window configuration state using flags
    pub fn set_window_state(&mut self, state: WindowState) {
        unsafe { ffi::SetWindowState(state.0 as u32) }
    }

    /// Clear window configuration state flags
    pub fn clear_window_state(&mut self, state: WindowState) {
        unsafe { ffi::ClearWindowState(state.0 as u32) }
    }

    /// Get the window config state
    pub fn get_window_state(&self) -> WindowState {
        let state = WindowState::default();
        unsafe {
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_VSYNC_HINT as u32) {
                state.set_vsync_hint(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as u32) {
                state.set_fullscreen_mode(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32) {
                state.set_window_resizable(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as u32) {
                state.set_window_undecorated(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as u32) {
                state.set_window_hidden(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as u32) {
                state.set_window_minimized(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as u32) {
                state.set_window_maximized(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as u32) {
                state.set_window_unfocused(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as u32) {
                state.set_window_topmost(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as u32) {
                state.set_window_always_run(true);
            }

            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as u32) {
                state.set_window_transparent(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32) {
                state.set_window_highdpi(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_MSAA_4X_HINT as u32) {
                state.set_msaa(true);
            }
            if ffi::IsWindowState(ffi::ConfigFlags::FLAG_INTERLACED_HINT as u32) {
                state.set_interlaced_hint(true);
            }
        }
        state
    }

    /// Sets monitor for the current window (fullscreen mode).
    #[inline]
    pub fn set_window_monitor(&mut self, monitor: i32) {
        let len = get_monitor_count();
        debug_assert!(monitor < len && monitor >= 0, "monitor index out of range");
        unsafe {
            ffi::SetWindowMonitor(monitor);
        }
    }

    /// Sets minimum window dimensions (for `FLAG_WINDOW_RESIZABLE`).
    #[inline]
    pub fn set_window_min_size(&mut self, width: i32, height: i32) {
        unsafe {
            ffi::SetWindowMinSize(width, height);
        }
    }

    /// Sets window dimensions.
    #[inline]
    pub fn set_window_size(&mut self, width: i32, height: i32) {
        unsafe {
            ffi::SetWindowSize(width, height);
        }
    }

    /// Gets current screen width.
    #[inline]
    pub fn get_screen_width(&self) -> i32 {
        unsafe { ffi::GetScreenWidth() }
    }

    /// Gets current screen height.
    #[inline]
    pub fn get_screen_height(&self) -> i32 {
        unsafe { ffi::GetScreenHeight() }
    }
}

impl RaylibHandle {
    /// Shows mouse cursor.
    #[inline]
    pub fn show_cursor(&mut self) {
        unsafe {
            ffi::ShowCursor();
        }
    }

    /// Hides mouse cursor.
    #[inline]
    pub fn hide_cursor(&mut self) {
        unsafe {
            ffi::HideCursor();
        }
    }

    /// Checks if mouse cursor is not visible.
    #[inline]
    pub fn is_cursor_hidden(&self) -> bool {
        unsafe { ffi::IsCursorHidden() }
    }

    /// Enables mouse cursor (unlock cursor).
    #[inline]
    pub fn enable_cursor(&mut self) {
        unsafe {
            ffi::EnableCursor();
        }
    }

    /// Disables mouse cursor (lock cursor).
    #[inline]
    pub fn disable_cursor(&mut self) {
        unsafe {
            ffi::DisableCursor();
        }
    }
}
