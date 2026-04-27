#![no_std]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

//! A highly optimized, no_std driver for the GC9A01 240x240 SPI TFT display.
//! Uses embedded-hal 1.x and embedded-graphics-core 0.4.

pub mod address_window;
pub mod backlight;
pub mod color;
pub mod commands;
pub mod config;
pub mod display;
pub mod error;
pub mod graphics;
pub mod init;
pub mod interface;
pub mod reset;

// Re-export core types for easy access
pub use config::{ColorOrder, DisplayConfig, Orientation};
pub use display::Display;