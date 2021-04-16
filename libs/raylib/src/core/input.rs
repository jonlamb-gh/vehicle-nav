use crate::core::RaylibHandle;
use crate::ffi;

impl RaylibHandle {
    /// Detect if a key has been pressed once.
    #[inline]
    pub fn is_key_pressed(&self, key: ffi::KeyboardKey) -> bool {
        unsafe { ffi::IsKeyPressed((key as u32) as i32) }
    }

    /// Detect if a key is being pressed.
    #[inline]
    pub fn is_key_down(&self, key: ffi::KeyboardKey) -> bool {
        unsafe { ffi::IsKeyDown((key as u32) as i32) }
    }

    /// Detect if a key has been released once.
    #[inline]
    pub fn is_key_released(&self, key: ffi::KeyboardKey) -> bool {
        unsafe { ffi::IsKeyReleased((key as u32) as i32) }
    }

    /// Detect if a key is NOT being pressed.
    #[inline]
    pub fn is_key_up(&self, key: ffi::KeyboardKey) -> bool {
        unsafe { ffi::IsKeyUp((key as u32) as i32) }
    }

    /// Gets latest key pressed.
    #[inline]
    pub fn get_key_pressed(&mut self) -> Option<ffi::KeyboardKey> {
        let key = unsafe { ffi::GetKeyPressed() };
        if key > 0 {
            return key_from_i32(key);
        }
        None
    }

    /// Gets latest key pressed.
    #[inline]
    pub fn get_key_pressed_number(&mut self) -> Option<u32> {
        let key = unsafe { ffi::GetKeyPressed() };
        if key > 0 {
            return Some(key as u32);
        }
        None
    }

    /// Sets a custom key to exit program (default is ESC).
    #[inline]
    pub fn set_exit_key(&mut self, key: Option<ffi::KeyboardKey>) {
        unsafe {
            match key {
                Some(k) => ffi::SetExitKey((k as u32) as i32),
                None => ffi::SetExitKey(0),
            }
        }
    }

    /// Detect if a mouse button has been pressed once.
    #[inline]
    pub fn is_mouse_button_pressed(&self, button: ffi::MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonPressed(button as i32) }
    }

    /// Detect if a mouse button is being pressed.
    #[inline]
    pub fn is_mouse_button_down(&self, button: ffi::MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonDown(button as i32) }
    }

    /// Detect if a mouse button has been released once.
    #[inline]
    pub fn is_mouse_button_released(&self, button: ffi::MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonReleased(button as i32) }
    }

    /// Detect if a mouse button is NOT being pressed.
    #[inline]
    pub fn is_mouse_button_up(&self, button: ffi::MouseButton) -> bool {
        unsafe { ffi::IsMouseButtonUp(button as i32) }
    }

    /// Returns mouse position X.
    #[inline]
    pub fn get_mouse_x(&self) -> i32 {
        unsafe { ffi::GetMouseX() }
    }

    /// Returns mouse position Y.
    #[inline]
    pub fn get_mouse_y(&self) -> i32 {
        unsafe { ffi::GetMouseY() }
    }

    /// Returns mouse wheel movement Y.
    #[inline]
    pub fn get_mouse_wheel_move(&self) -> f32 {
        unsafe { ffi::GetMouseWheelMove() }
    }

    /// Returns touch position X for touch point 0 (relative to screen size).
    #[inline]
    pub fn get_touch_x(&self) -> i32 {
        unsafe { ffi::GetTouchX() }
    }

    /// Returns touch position Y for touch point 0 (relative to screen size).
    #[inline]
    pub fn get_touch_y(&self) -> i32 {
        unsafe { ffi::GetTouchY() }
    }
}

pub fn key_from_i32(key: i32) -> Option<ffi::KeyboardKey> {
    use ffi::KeyboardKey::*;
    match key {
        39 => Some(KEY_APOSTROPHE),
        44 => Some(KEY_COMMA),
        45 => Some(KEY_MINUS),
        46 => Some(KEY_PERIOD),
        47 => Some(KEY_SLASH),
        48 => Some(KEY_ZERO),
        49 => Some(KEY_ONE),
        50 => Some(KEY_TWO),
        51 => Some(KEY_THREE),
        52 => Some(KEY_FOUR),
        53 => Some(KEY_FIVE),
        54 => Some(KEY_SIX),
        55 => Some(KEY_SEVEN),
        56 => Some(KEY_EIGHT),
        57 => Some(KEY_NINE),
        59 => Some(KEY_SEMICOLON),
        61 => Some(KEY_EQUAL),
        65 => Some(KEY_A),
        66 => Some(KEY_B),
        67 => Some(KEY_C),
        68 => Some(KEY_D),
        69 => Some(KEY_E),
        70 => Some(KEY_F),
        71 => Some(KEY_G),
        72 => Some(KEY_H),
        73 => Some(KEY_I),
        74 => Some(KEY_J),
        75 => Some(KEY_K),
        76 => Some(KEY_L),
        77 => Some(KEY_M),
        78 => Some(KEY_N),
        79 => Some(KEY_O),
        80 => Some(KEY_P),
        81 => Some(KEY_Q),
        82 => Some(KEY_R),
        83 => Some(KEY_S),
        84 => Some(KEY_T),
        85 => Some(KEY_U),
        86 => Some(KEY_V),
        87 => Some(KEY_W),
        88 => Some(KEY_X),
        89 => Some(KEY_Y),
        90 => Some(KEY_Z),
        32 => Some(KEY_SPACE),
        256 => Some(KEY_ESCAPE),
        257 => Some(KEY_ENTER),
        258 => Some(KEY_TAB),
        259 => Some(KEY_BACKSPACE),
        260 => Some(KEY_INSERT),
        261 => Some(KEY_DELETE),
        262 => Some(KEY_RIGHT),
        263 => Some(KEY_LEFT),
        264 => Some(KEY_DOWN),
        265 => Some(KEY_UP),
        266 => Some(KEY_PAGE_UP),
        267 => Some(KEY_PAGE_DOWN),
        268 => Some(KEY_HOME),
        269 => Some(KEY_END),
        280 => Some(KEY_CAPS_LOCK),
        281 => Some(KEY_SCROLL_LOCK),
        282 => Some(KEY_NUM_LOCK),
        283 => Some(KEY_PRINT_SCREEN),
        284 => Some(KEY_PAUSE),
        290 => Some(KEY_F1),
        291 => Some(KEY_F2),
        292 => Some(KEY_F3),
        293 => Some(KEY_F4),
        294 => Some(KEY_F5),
        295 => Some(KEY_F6),
        296 => Some(KEY_F7),
        297 => Some(KEY_F8),
        298 => Some(KEY_F9),
        299 => Some(KEY_F10),
        300 => Some(KEY_F11),
        301 => Some(KEY_F12),
        340 => Some(KEY_LEFT_SHIFT),
        341 => Some(KEY_LEFT_CONTROL),
        342 => Some(KEY_LEFT_ALT),
        343 => Some(KEY_LEFT_SUPER),
        344 => Some(KEY_RIGHT_SHIFT),
        345 => Some(KEY_RIGHT_CONTROL),
        346 => Some(KEY_RIGHT_ALT),
        347 => Some(KEY_RIGHT_SUPER),
        348 => Some(KEY_KB_MENU),
        91 => Some(KEY_LEFT_BRACKET),
        92 => Some(KEY_BACKSLASH),
        93 => Some(KEY_RIGHT_BRACKET),
        96 => Some(KEY_GRAVE),
        320 => Some(KEY_KP_0),
        321 => Some(KEY_KP_1),
        322 => Some(KEY_KP_2),
        323 => Some(KEY_KP_3),
        324 => Some(KEY_KP_4),
        325 => Some(KEY_KP_5),
        326 => Some(KEY_KP_6),
        327 => Some(KEY_KP_7),
        328 => Some(KEY_KP_8),
        329 => Some(KEY_KP_9),
        330 => Some(KEY_KP_DECIMAL),
        331 => Some(KEY_KP_DIVIDE),
        332 => Some(KEY_KP_MULTIPLY),
        333 => Some(KEY_KP_SUBTRACT),
        334 => Some(KEY_KP_ADD),
        335 => Some(KEY_KP_ENTER),
        336 => Some(KEY_KP_EQUAL),
        _ => None,
    }
}
