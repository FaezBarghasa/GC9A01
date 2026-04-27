//! Adapter implementing `embedded-graphics` DrawTarget and OriginDimensions traits.

use crate::display::Display;
use crate::error::Error;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{IntoStorage, Rgb565},
    primitives::Rectangle,
    prelude::PointsIter,
};
use embedded_hal::{digital::OutputPin, spi::SpiDevice};

impl<SPI, DC, RST, BL, SpiE, PinE> OriginDimensions for Display<SPI, DC, RST, BL>
where
    SPI: SpiDevice<Error = SpiE>,
    DC: OutputPin<Error = PinE>,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    #[inline]
    fn size(&self) -> Size {
        // Defer to the built-in size() which calculates dimensions based on orientation
        let (width, height) = self.size();
        Size::new(width as u32, height as u32)
    }
}

impl<SPI, DC, RST, BL, SpiE, PinE> DrawTarget for Display<SPI, DC, RST, BL>
where
    SPI: SpiDevice<Error = SpiE>,
    DC: OutputPin<Error = PinE>,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    type Color = Rgb565;
    type Error = Error<SpiE, PinE>;

    /// Mandatory fallback for drawing scattered/sparse pixels.
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let (width, height) = self.size();
        let display_bounds = Rectangle::new(Point::zero(), Size::new(width as u32, height as u32));

        for Pixel(coord, color) in pixels.into_iter() {
            if display_bounds.contains(coord) {
                let x = coord.x as u16;
                let y = coord.y as u16;

                // Temporarily borrow hardware for the interface
                let mut iface = crate::interface::SpiInterface {
                    spi: &mut self.iface.spi,
                    dc: &mut self.iface.dc,
                };

                // Set a 1x1 address window
                crate::address_window::set_address_window(
                    &mut iface,
                    x,
                    y,
                    x,
                    y,
                    self.config.x_offset,
                    self.config.y_offset,
                )?;

                let bytes = crate::color::rgb565_to_bytes(color.into_storage());
                iface.send_data(&bytes)?;
            }
        }
        Ok(())
    }

    /// Override `fill_solid` to use our native, heavily optimized bounding intersection routine.
    #[inline]
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        Display::fill_solid(self, *area, color)
    }

    /// High-performance bulk pixel transfer override (CRITICAL OPTIMIZATION).
    /// Prevents falling back to the 1-by-1 `draw_iter` for contiguous pixel blocks like Images.
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let (width, height) = self.size();
        let display_bounds = Rectangle::new(Point::zero(), Size::new(width as u32, height as u32));
        let intersection = area.intersection(&display_bounds);

        if intersection.is_zero_sized() {
            return Ok(());
        }

        let mut colors = colors.into_iter();

        if intersection == *area {
            // Fast Path: Entire rectangle is fully within screen bounds
            let x0 = area.top_left.x as u16;
            let y0 = area.top_left.y as u16;
            let x1 = x0 + area.size.width as u16 - 1;
            let y1 = y0 + area.size.height as u16 - 1;

            let mut iface = crate::interface::SpiInterface {
                spi: &mut self.iface.spi,
                dc: &mut self.iface.dc,
            };

            crate::address_window::set_address_window(
                &mut iface,
                x0,
                y0,
                x1,
                y1,
                self.config.x_offset,
                self.config.y_offset,
            )?;

            // Stream colors across SPI using a zero-allocation 128-byte (64-pixel) chunk buffer
            let mut buf = [0u8; 128];
            let mut idx = 0;

            // DC remains High explicitly for the data streaming phase
            iface.dc.set_high().map_err(Error::Pin)?;

            for color in colors {
                let bytes = crate::color::rgb565_to_bytes(color.into_storage());
                buf[idx] = bytes[0];
                buf[idx + 1] = bytes[1];
                idx += 2;

                if idx >= buf.len() {
                    iface.spi.write(&buf).map_err(Error::Spi)?;
                    idx = 0;
                }
            }

            // Flush remaining buffer residues
            if idx > 0 {
                iface.spi.write(&buf[..idx]).map_err(Error::Spi)?;
            }
        } else {
            // Slow path (Fallback): Coordinates clip off-screen.
            // Map the colors back to physical Points and filter them against bounds natively.
            let pixels = area.points().zip(colors).map(|(p, c)| Pixel(p, c));
            self.draw_iter(pixels.filter(|&Pixel(p, _)| display_bounds.contains(p)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DisplayConfig;
    use core::cell::Cell;
    use core::convert::Infallible;
    use embedded_hal::spi::Operation;
    use embedded_graphics_core::pixelcolor::RgbColor;

    // Specialized tracker to observe SPI states
    struct Tracker {
        dc_is_high: Cell<bool>,
        caset_calls: Cell<usize>,
    }

    struct MockSpi<'a>(&'a Tracker);
    struct MockPin<'a>(&'a Tracker);

    impl<'a> embedded_hal::spi::ErrorType for MockSpi<'a> {
        type Error = Infallible;
    }

    impl<'a> SpiDevice for MockSpi<'a> {
        fn transaction(&mut self, _ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            // Look for CASET (0x2A) command bytes.
            // When sent as a command, DC is low and payload is `[0x2A]`.
            if !self.0.dc_is_high.get() && words.len() == 1 && words[0] == crate::commands::CASET {
                self.0.caset_calls.set(self.0.caset_calls.get() + 1);
            }
            Ok(())
        }
    }

    impl<'a> embedded_hal::digital::ErrorType for MockPin<'a> {
        type Error = Infallible;
    }

    impl<'a> OutputPin for MockPin<'a> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.0.dc_is_high.set(false);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.0.dc_is_high.set(true);
            Ok(())
        }
    }

    #[test]
    fn test_fill_contiguous_calls_address_window_once() {
        let tracker = Tracker {
            dc_is_high: Cell::new(true),
            caset_calls: Cell::new(0),
        };

        let spi = MockSpi(&tracker);
        let dc = MockPin(&tracker);
        let rst = MockPin(&tracker);
        let bl = MockPin(&tracker);

        let mut display = Display::new(spi, dc, rst, bl, DisplayConfig::default());

        let area = Rectangle::new(Point::new(10, 10), Size::new(10, 10));
        // Provide 100 pixels (simulating an image)
        let colors = core::iter::repeat(Rgb565::RED).take(100);

        // Fill contiguous area
        display.fill_contiguous(&area, colors).unwrap();

        // Ensure that our optimized path set the bounds (CASET) exactly once for the entire batch.
        assert_eq!(
            tracker.caset_calls.get(),
            1,
            "CASET should be sent exactly once for contiguous fill areas"
        );
    }
}
