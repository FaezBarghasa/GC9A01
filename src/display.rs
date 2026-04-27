//! Display implementation
//!
//! This module contains the `Display` struct which provides a high-level API to interact with the GC9A01 display.
//! It handles initialization, primitive drawing operations like filling areas or the whole screen, and
//! delegates pixel rendering to `SpiInterface`.

use crate::error::Error;
use crate::config::{DisplayConfig, Orientation};
use crate::interface::SpiInterface;
use embedded_graphics_core::geometry::{Point, Size};
use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565};
use embedded_graphics_core::primitives::Rectangle;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

/// High-level driver for the GC9A01 display.
///
/// Provides methods to initialize the display, send commands, and draw primitives.
/// It wraps the `SpiInterface` and manages the reset and backlight pins.
pub struct Display<SPI, DC, RST, BL> {
    pub(crate) iface: SpiInterface<SPI, DC>,
    pub(crate) rst: RST,
    pub(crate) bl: BL,
    pub(crate) config: DisplayConfig,
}

impl<SPI, DC, RST, BL, SpiE, PinE> Display<SPI, DC, RST, BL>
where
    SPI: SpiDevice<Error = SpiE>,
    DC: OutputPin<Error = PinE>,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    /// Creates a new display instance.
    ///
    /// # Arguments
    /// * `spi` - SPI device implementing `embedded_hal::spi::SpiDevice`
    /// * `dc` - Data/Command pin implementing `embedded_hal::digital::OutputPin`
    /// * `rst` - Reset pin implementing `embedded_hal::digital::OutputPin`
    /// * `bl` - Backlight pin implementing `embedded_hal::digital::OutputPin`
    /// * `config` - `DisplayConfig` containing orientation and other settings
    pub fn new(spi: SPI, dc: DC, rst: RST, bl: BL, config: DisplayConfig) -> Self {
        Self {
            iface: SpiInterface { spi, dc },
            rst,
            bl,
            config,
        }
    }

    /// Returns the current logical size of the display based on orientation.
    ///
    /// The size is calculated by checking the current `Orientation` in the `DisplayConfig`.
    /// For portrait modes, width and height are returned as configured. For landscape modes,
    /// width and height are swapped.
    pub fn size(&self) -> (u16, u16) {
        match self.config.orientation {
            Orientation::Portrait | Orientation::PortraitFlipped => (self.config.width, self.config.height),
            Orientation::LandscapeRight | Orientation::LandscapeLeft => (self.config.height, self.config.width),
        }
    }

    /// Initializes the display.
    ///
    /// Performs a hardware reset, runs the vendor initialization sequence,
    /// configures orientation/color order, and enables the backlight.
    ///
    /// # Arguments
    /// * `delay` - Delay provider implementing `embedded_hal::delay::DelayNs`
    ///
    /// # Errors
    /// Returns an `Error` if any SPI communication or pin manipulation fails.
    pub fn init<D>(&mut self, delay: &mut D) -> Result<(), Error<SPI::Error, DC::Error>>
    where
        D: DelayNs,
    {
        // 1. Hardware Reset
        crate::reset::hardware_reset(
            &mut self.rst,
            delay,
            self.config.reset_duration_ms,
            self.config.reset_after_ms,
        ).map_err(Error::Pin)?;

        // 2. Vendor Bytecode Init
        crate::init::run_vendor_init(&mut self.iface)?;

        // 3. Configure MADCTL (Orientation + Color Order)
        let madctl = self.config.orientation.madctl_bits() | self.config.color_order.bgr_bit();
        self.iface.send_command_data(crate::commands::MADCTL, &[madctl])?;

        // 4. Inversion Control
        if self.config.invert_colors {
            self.iface.send_command(crate::commands::INVON)?;
        } else {
            self.iface.send_command(crate::commands::INVOFF)?;
        }

        // 5. Backlight
        if self.config.backlight_on_after_init {
            crate::backlight::set_backlight(&mut self.bl, true).map_err(Error::Pin)?;
        }

        // 6. Wait for SLPOUT setup time
        delay.delay_ms(self.config.slpout_wait_ms);

        // 7. Display ON
        self.iface.send_command(crate::commands::DISPON)?;

        Ok(())
    }
}

impl<SPI, DC, RST, BL, SpiE, PinE> Display<SPI, DC, RST, BL>
where
    SPI: SpiDevice<Error = SpiE>,
    DC: OutputPin<Error = PinE>,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    /// Fills the entire display with a single solid color.
    ///
    /// # Arguments
    /// * `color` - The `Rgb565` color to fill the screen with.
    ///
    /// # Errors
    /// Returns an `Error` if SPI communication fails while setting the address window or writing pixels.
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
    ///
    /// The rectangle will be clipped to the display boundaries to prevent writing outside
    /// the visible area.
    ///
    /// # Arguments
    /// * `area` - The `Rectangle` defining the bounds to fill.
    /// * `color` - The `Rgb565` color to fill the area with.
    ///
    /// # Errors
    /// Returns an `Error` if SPI communication fails while setting the address window or writing pixels.
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
