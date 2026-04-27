//! Hardware register constants for the GC9A01 controller.

pub const SWRESET: u8 = 0x01; // Software Reset
pub const SLPIN: u8 = 0x10; // Sleep In
pub const SLPOUT: u8 = 0x11; // Sleep Out
pub const INVOFF: u8 = 0x20; // Display Inversion Off
pub const INVON: u8 = 0x21; // Display Inversion On
pub const DISPOFF: u8 = 0x28; // Display Off
pub const DISPON: u8 = 0x29; // Display On
pub const CASET: u8 = 0x2A; // Column Address Set
pub const RASET: u8 = 0x2B; // Row Address Set
pub const RAMWR: u8 = 0x2C; // Memory Write
pub const MADCTL: u8 = 0x36; // Memory Data Access Control
pub const COLMOD: u8 = 0x3A; // Interface Pixel Format

/// Pixel Format constants for COLMOD
pub const COLMOD_16BIT: u8 = 0x05; // 16-bit/pixel (RGB565)
pub const COLMOD_18BIT: u8 = 0x06; // 18-bit/pixel
