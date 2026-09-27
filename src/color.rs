//! Utilities for handling RGB565 formats and Big-Endian byte packing.

/// Converts a 16-bit RGB565 color value into Big-Endian representation.
/// The GC9A01 controller natively expects the high byte before the low byte over SPI.
#[inline]
pub fn rgb565_to_bytes(color: u16) -> [u8; 2] {
    color.to_be_bytes()
}

/// Packs an array slice with repeating RGB565 pixel values in Big-Endian order.
/// Panics in debug builds if the target buffer length is not even.
pub fn pack_rgb565_fill(color: u16, buf: &mut [u8]) {
    debug_assert!(buf.len() % 2 == 0, "Buffer length must be a multiple of 2");

    let [high, low] = rgb565_to_bytes(color);
    for chunk in buf.chunks_exact_mut(2) {
        chunk[0] = high;
        chunk[1] = low;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb565_to_bytes() {
        // Red (0xF800) -> [0xF8, 0x00]
        assert_eq!(rgb565_to_bytes(0xF800), [0xF8, 0x00]);
        // Green (0x07E0) -> [0x07, 0xE0]
        assert_eq!(rgb565_to_bytes(0x07E0), [0x07, 0xE0]);
        // Blue (0x001F) -> [0x00, 0x1F]
        assert_eq!(rgb565_to_bytes(0x001F), [0x00, 0x1F]);
        // White (0xFFFF) -> [0xFF, 0xFF]
        assert_eq!(rgb565_to_bytes(0xFFFF), [0xFF, 0xFF]);
    }

    #[test]
    fn test_pack_rgb565_fill() {
        let mut buf = [0u8; 6];
        pack_rgb565_fill(0xF800, &mut buf);
        assert_eq!(buf, [0xF8, 0x00, 0xF8, 0x00, 0xF8, 0x00]);
    }
}