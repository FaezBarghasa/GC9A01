use crate::commands;
use crate::error::Error;
use crate::interface::SpiInterface;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

/// Sets the rectangular address window for subsequent pixel data writes.
pub(crate) fn set_address_window<SPI, DC>(
    iface: &mut SpiInterface<SPI, DC>,
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    x_offset: u16,
    y_offset: u16,
) -> Result<(), Error<SPI::Error, DC::Error>>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    let actual_x0 = x0 + x_offset;
    let actual_x1 = x1 + x_offset;
    let actual_y0 = y0 + y_offset;
    let actual_y1 = y1 + y_offset;

    // CASET: Column Address Set
    iface.send_command_data(
        commands::CASET,
        &[
            (actual_x0 >> 8) as u8,
            (actual_x0 & 0xFF) as u8,
            (actual_x1 >> 8) as u8,
            (actual_x1 & 0xFF) as u8,
        ],
    )?;

    // RASET: Row Address Set
    iface.send_command_data(
        commands::RASET,
        &[
            (actual_y0 >> 8) as u8,
            (actual_y0 & 0xFF) as u8,
            (actual_y1 >> 8) as u8,
            (actual_y1 & 0xFF) as u8,
        ],
    )?;

    // RAMWR has no data payload in this call.
    // It opens the display RAM write gate. The pixel data that follows is
    // sent in a separate SPI write (with DC high).
    iface.send_command(commands::RAMWR)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::tests::{MockPin, MockSpi}; // Assuming test mocks exist here
    use crate::interface::SpiInterface;

    #[test]
    fn set_address_window_sends_correct_caset_bytes() {
        let spi = MockSpi::new();
        let dc = MockPin::new();
        let mut iface = SpiInterface::new(spi, dc);

        set_address_window(&mut iface, 10, 20, 50, 80, 0, 0).unwrap();

        let written = iface.spi().written_bytes();
        // CASET is 0x2A, RASET is 0x2B, RAMWR is 0x2C
        // Find CASET sequence and check data
        assert!(written.windows(5).any(|w| w == [0x2A, 0x00, 0x0A, 0x00, 0x32]));
    }

    #[test]
    fn set_address_window_applies_x_offset() {
        let spi = MockSpi::new();
        let dc = MockPin::new();
        let mut iface = SpiInterface::new(spi, dc);

        set_address_window(&mut iface, 5, 0, 10, 0, 2, 0).unwrap();

        let written = iface.spi().written_bytes();
        assert!(written.windows(5).any(|w| w == [0x2A, 0x00, 0x07, 0x00, 0x0C]));
    }

    #[test]
    fn set_address_window_sends_ramwr_last() {
        let spi = MockSpi::new();
        let dc = MockPin::new();
        let mut iface = SpiInterface::new(spi, dc);

        set_address_window(&mut iface, 0, 0, 10, 10, 0, 0).unwrap();
        let written_commands = iface.dc_low_writes(); // Hypothetical helper from Mock
        assert_eq!(*written_commands.last().unwrap(), 0x2C);
    }

    #[test]
    fn set_address_window_zero_origin_no_offset() {
        let spi = MockSpi::new();
        let dc = MockPin::new();
        let mut iface = SpiInterface::new(spi, dc);

        set_address_window(&mut iface, 0, 0, 239, 239, 0, 0).unwrap();
        let written = iface.spi().written_bytes();
        assert!(written.windows(5).any(|w| w == [0x2A, 0x00, 0x00, 0x00, 0xEF]));
        assert!(written.windows(5).any(|w| w == [0x2B, 0x00, 0x00, 0x00, 0xEF]));
    }
}
