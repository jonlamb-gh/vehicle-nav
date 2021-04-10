#![deny(warnings)]

use raylib_sys::*;
use std::ffi::CString;

fn main() {
    let screen_width = 800;
    let screen_height = 600;

    let title = CString::new("raylib [core] example - basic window").unwrap();
    unsafe {
        InitWindow(screen_width, screen_height, title.as_ptr());

        SetTargetFPS(60);
    }

    let text = CString::new("Congrats! You created your first window!").unwrap();

    let bg_color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    let text_color = Color {
        r: 200,
        g: 200,
        b: 200,
        a: 255,
    };

    loop {
        unsafe {
            if WindowShouldClose() {
                break;
            }

            BeginDrawing();

            ClearBackground(bg_color);

            DrawText(text.as_ptr(), 190, 200, 20, text_color);

            EndDrawing();
        }
    }

    unsafe {
        CloseWindow();
    }
}
