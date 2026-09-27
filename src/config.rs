//! Hardware register constants, layout configurations, and display builder patterns.

use embedded_graphics_core::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

/// Hardware orientation settings.
/// Encodes the Memory Access Control (MADCTL) bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    LandscapeRight,
    PortraitFlipped,
    LandscapeLeft,
}

impl Orientation {
    /// Returns the MADCTL bits corresponding to the orientation.
    pub const fn madctl_bits(&self) -> u8 {
        match self {
            Orientation::Portrait => 0x00,
            Orientation::LandscapeRight => 0x60,
            Orientation::PortraitFlipped => 0xC0,
            Orientation::LandscapeLeft => 0xA0,
        }
    }
}

/// Hardware color arrangement settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOrder {
    Rgb,
    Bgr,
}

impl ColorOrder {
    /// Returns the MADCTL BGR bit (bit 3).
    pub const fn bgr_bit(&self) -> u8 {
        match self {
            ColorOrder::Rgb => 0x00,
            ColorOrder::Bgr => 0x08,
        }
    }
}

/// Static configuration for the display panel hardware and driver initialization.
#[derive(Debug, Clone, Copy)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub x_offset: u16,
    pub y_offset: u16,
    pub orientation: Orientation,
    pub color_order: ColorOrder,
    pub invert_colors: bool,
    pub backlight_on_after_init: bool,
    pub reset_duration_ms: u32,
    pub reset_after_ms: u32,
    pub slpout_wait_ms: u32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 240,
            height: 240,
            x_offset: 0,
            y_offset: 0,
            orientation: Orientation::Portrait,
            color_order: ColorOrder::Rgb,
            invert_colors: false,
            backlight_on_after_init: true,
            reset_duration_ms: 10,
            reset_after_ms: 120,
            slpout_wait_ms: 120,
        }
    }
}

impl DisplayConfig {
    /// Starts a new builder for customizing the display configuration.
    pub fn builder() -> DisplayConfigBuilder {
        DisplayConfigBuilder::new()
    }

    /// Calculates the largest perfectly inscribed 1:1 square that fits inside the
    /// display area, centering it. This is crucial for round 240x240 displays to
    /// ensure UI elements aren't clipped by the physical bezel.
    pub fn safe_inscribed_square(&self) -> Rectangle {
        // Find the limiting dimension
        let min_dim = core::cmp::min(self.width, self.height) as u32;

        // Multiply by approximation of 1/sqrt(2) = ~0.707 to get the inscribed square side.
        // Doing it purely in integer math: (diameter * 707) / 1000
        let side = (min_dim * 707) / 1000;

        let x_offset = (self.width as u32 - side) / 2;
        let y_offset = (self.height as u32 - side) / 2;

        Rectangle::new(
            Point::new(x_offset as i32, y_offset as i32),
            Size::new(side, side),
        )
    }
}

/// Builder pattern for `DisplayConfig`.
pub struct DisplayConfigBuilder {
    config: DisplayConfig,
}

impl DisplayConfigBuilder {
    pub fn new() -> Self {
        Self { config: DisplayConfig::default() }
    }

    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn with_offsets(mut self, x: u16, y: u16) -> Self {
        self.config.x_offset = x;
        self.config.y_offset = y;
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.config.orientation = orientation;
        self
    }

    pub fn with_color_order(mut self, color_order: ColorOrder) -> Self {
        self.config.color_order = color_order;
        self
    }

    pub fn with_invert_colors(mut self, invert: bool) -> Self {
        self.config.invert_colors = invert;
        self
    }

    pub fn build(self) -> DisplayConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_madctl_bit_calculation() {
        assert_eq!(Orientation::Portrait.madctl_bits(), 0x00);
        assert_eq!(Orientation::LandscapeRight.madctl_bits(), 0x60);
        assert_eq!(Orientation::PortraitFlipped.madctl_bits(), 0xC0);
        assert_eq!(Orientation::LandscapeLeft.madctl_bits(), 0xA0);

        assert_eq!(ColorOrder::Rgb.bgr_bit(), 0x00);
        assert_eq!(ColorOrder::Bgr.bgr_bit(), 0x08);
    }

    #[test]
    fn test_builder_overrides() {
        let config = DisplayConfig::builder()
            .with_size(128, 160)
            .with_invert_colors(true)
            .build();

        assert_eq!(config.width, 128);
        assert_eq!(config.height, 160);
        assert_eq!(config.invert_colors, true);
    }

    #[test]
    fn test_safe_inscribed_square() {
        let config = DisplayConfig::default(); // 240x240
        let safe_area = config.safe_inscribed_square();

        // 240 / sqrt(2) ≈ 169.7 -> 169
        assert_eq!(safe_area.size.width, 169);
        assert_eq!(safe_area.size.height, 169);

        // (240 - 169) / 2 = 35
        assert_eq!(safe_area.top_left.x, 35);
        assert_eq!(safe_area.top_left.y, 35);
    }
}