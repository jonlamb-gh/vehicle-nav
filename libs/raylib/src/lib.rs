//! NOTE: a lot of stuff in here was taken directly from https://github.com/deltaphc/raylib-rs/tree/master/raylib

#![deny(warnings)]

pub mod core;
pub mod prelude;

pub mod ffi {
    pub use raylib_sys::*;
}
