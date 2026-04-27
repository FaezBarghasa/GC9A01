//! Adapter implementing `embedded-graphics` DrawTarget and OriginDimensions traits.

use crate::display::Display;
use crate::error::Error;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{IntoStorage, Rgb565},
    primitives::Rectangle,
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
        self.fill_solid(*area, color)
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

        let mut colors_iter = colors.into_iter();

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

            // Stream colors across SPI using a chunk buffer
            let mut buf = [0u8; 128];
            let mut idx = 0;

            // DC remains High explicitly for the data streaming phase
            iface.dc.set_high().map_err(Error::Pin)?;

            for color in colors_iter {
                let bytes = crate::color::rgb565_to_bytes(color.into_storage());
                buf[idx] = bytes[0];
                buf[idx + 1] = bytes[1];
                idx += 2;

                if idx >= buf.len() {
                    // We need a way to send raw data.
                    // To satisfy the borrow checker and SPI ownership we'll map error
                    // inline.
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
            // Since we don't have iter points from Rectangle easily in embedded_graphics < 0.8
            // We just use `draw_iter` for the whole thing and let it handle bounds checking.

            // To be safe and compatible with embedded-graphics 0.4
            // We'll calculate coordinates manually.
            let x0 = area.top_left.x;
            let y0 = area.top_left.y;
            let w = area.size.width as i32;
            let h = area.size.height as i32;

            for y in 0..h {
                for x in 0..w {
                    if let Some(color) = colors_iter.next() {
                        let px = x0 + x;
                        let py = y0 + y;
                        let pt = Point::new(px, py);

                        if display_bounds.contains(pt) {
                             let mut iface = crate::interface::SpiInterface {
                                spi: &mut self.iface.spi,
                                dc: &mut self.iface.dc,
                            };

                            crate::address_window::set_address_window(
                                &mut iface,
                                px as u16,
                                py as u16,
                                px as u16,
                                py as u16,
                                self.config.x_offset,
                                self.config.y_offset,
                            )?;

                            let bytes = crate::color::rgb565_to_bytes(color.into_storage());
                            iface.send_data(&bytes)?;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
