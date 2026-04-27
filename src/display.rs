use crate::error::Error;
use embedded_graphics_core::geometry::{Point, Size};
use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565};
use embedded_graphics_core::primitives::Rectangle;

impl<SPI, DC, RST, BL> Display<SPI, DC, RST, BL>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
    RST: embedded_hal::digital::OutputPin,
    BL: embedded_hal::digital::OutputPin,
{
    /// Fills the entire display with a single solid color.
    pub fn clear(&mut self, color: Rgb565) -> Result<(), Error<SPI::Error, DC::Error>> {
        let color_u16 = color.into_storage();
        let (width, height) = self.size();

        crate::address_window::set_address_window(
            &mut self.iface,
            0,
            0,
            width - 1,
            height - 1,
            self.config.x_offset,
            self.config.y_offset,
        )?;

        let pixel_count = (width as u32) * (height as u32);
        self.iface.fill_pixels(color_u16, pixel_count)
    }

    /// Fills a specified rectangular area with a single solid color.
    pub fn fill_solid(
        &mut self,
        area: Rectangle,
        color: Rgb565,
    ) -> Result<(), Error<SPI::Error, DC::Error>> {
        let (width, height) = self.size();
        let display_bounds = Rectangle::new(Point::zero(), Size::new(width as u32, height as u32));

        // Clip area to the display bounds
        let intersection = area.intersection(&display_bounds);
        if intersection.is_zero_sized() {
            return Ok(());
        }

        let x0 = intersection.top_left.x as u16;
        let y0 = intersection.top_left.y as u16;
        let x1 = x0 + intersection.size.width as u16 - 1;
        let y1 = y0 + intersection.size.height as u16 - 1;

        crate::address_window::set_address_window(
            &mut self.iface,
            x0,
            y0,
            x1,
            y1,
            self.config.x_offset,
            self.config.y_offset,
        )?;

        let count = intersection.size.width * intersection.size.height;
        self.iface.fill_pixels(color.into_storage(), count)
    }
}
