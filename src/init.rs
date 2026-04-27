//! Vendor-specific undocumented initialization sequence execution using optimized bytecode.

use crate::error::Error;
use crate::interface::SpiInterface;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

/// Flash-optimized initialization bytecode.
/// Format: [command, param_count, p1, p2, ... pN]
///
/// This sequence is derived from vendor reference code for the GC9A01.
/// It configures the internal oscillator, power levels, and gamma curves.
const VENDOR_INIT_BYTECODE: &[u8] = &[
    0xEF, 0x00,
    0xEB, 0x01, 0x14,
    0xFE, 0x00,
    0xEF, 0x00,
    0xEB, 0x01, 0x14,
    0x84, 0x01, 0x40,
    0x85, 0x01, 0xFF,
    0x86, 0x01, 0xFF,
    0x87, 0x01, 0xFF,
    0x88, 0x01, 0x0A,
    0x89, 0x01, 0x21,
    0x8A, 0x01, 0x00,
    0x8B, 0x01, 0x80,
    0x8C, 0x01, 0x01,
    0x8D, 0x01, 0x01,
    0x8E, 0x01, 0xFF,
    0x8F, 0x01, 0xFF,
    0xB6, 0x02, 0x00, 0x00,
    0x36, 0x01, 0x08,
    0x3A, 0x01, 0x05,
    0x90, 0x04, 0x08, 0x08, 0x08, 0x08,
    0xBD, 0x01, 0x06,
    0xBC, 0x01, 0x00,
    0xFF, 0x03, 0x60, 0x01, 0x04,
    0xC3, 0x01, 0x13,
    0xC4, 0x01, 0x13,
    0xC9, 0x01, 0x22,
    0xBE, 0x01, 0x11,
    0xE1, 0x02, 0x10, 0x0E,
    0xDF, 0x03, 0x21, 0x0C, 0x02,
    0xF0, 0x06, 0x45, 0x09, 0x08, 0x08, 0x26, 0x2A,
    0xF1, 0x06, 0x43, 0x70, 0x72, 0x36, 0x37, 0x6F,
    0xF2, 0x06, 0x45, 0x09, 0x08, 0x08, 0x26, 0x2A,
    0xF3, 0x06, 0x43, 0x70, 0x72, 0x36, 0x37, 0x6F,
    0xED, 0x02, 0x1B, 0x0B,
    0xAE, 0x01, 0x77,
    0xCD, 0x01, 0x63,
    0x70, 0x09, 0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03,
    0xE8, 0x01, 0x34,
    0x62, 0x0C, 0x18, 0x0D, 0x71, 0xED, 0x70, 0x70, 0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70,
    0x63, 0x0C, 0x18, 0x11, 0x71, 0xF1, 0x70, 0x70, 0x18, 0x13, 0x71, 0xF3, 0x70, 0x70,
    0x64, 0x07, 0x28, 0x29, 0xF1, 0x01, 0xF1, 0x00, 0x07,
    0x66, 0x0A, 0x3C, 0x00, 0xCD, 0x67, 0x45, 0x45, 0x10, 0x00, 0x00, 0x00,
    0x67, 0x0A, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x01, 0x54, 0x10, 0x32, 0x98,
    0x74, 0x07, 0x10, 0x85, 0x80, 0x00, 0x00, 0x4E, 0x00,
    0x98, 0x02, 0x3E, 0x07,
    0x35, 0x01, 0x00,
    0x21, 0x00,
    0x11, 0x00,
];

/// Interprets the compact bytecode and sends commands to the controller.
pub(crate) fn run_vendor_init<SPI, DC, SpiE, PinE>(
    iface: &mut SpiInterface<SPI, DC>,
) -> Result<(), Error<SpiE, PinE>>
where
    SPI: SpiDevice<Error = SpiE>,
    DC: OutputPin<Error = PinE>,
{
    let mut cursor = 0;
    while cursor < VENDOR_INIT_BYTECODE.len() {
        let cmd = VENDOR_INIT_BYTECODE[cursor];
        let num_params = VENDOR_INIT_BYTECODE[cursor + 1] as usize;
        let start_params = cursor + 2;
        let end_params = start_params + num_params;

        if end_params > VENDOR_INIT_BYTECODE.len() {
            return Err(Error::InvalidConfig);
        }

        let params = &VENDOR_INIT_BYTECODE[start_params..end_params];
        iface.send_command_data(cmd, params)?;

        cursor = end_params;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;
    use embedded_hal::spi::Operation;

    struct MockSpi {
        pub logs: Vec<(u8, Vec<u8>)>, // (Command, Params)
        pub current_cmd: Option<u8>,
        pub dc_is_high: bool,
    }

    impl embedded_hal::spi::ErrorType for MockSpi { type Error = Infallible; }

    impl SpiDevice for MockSpi {
        fn transaction(&mut self, _ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> { Ok(()) }
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            if !self.dc_is_high {
                self.current_cmd = Some(words[0]);
                self.logs.push((words[0], Vec::new()));
            } else if let Some(_) = self.current_cmd {
                if let Some(entry) = self.logs.last_mut() {
                    entry.1.extend_from_slice(words);
                }
            }
            Ok(())
        }
    }

    struct MockPin<'a>(&'a mut MockSpi);
    impl<'a> embedded_hal::digital::ErrorType for MockPin<'a> { type Error = Infallible; }
    impl<'a> OutputPin for MockPin<'a> {
        fn set_low(&mut self) -> Result<(), Self::Error> { self.0.dc_is_high = false; Ok(()) }
        fn set_high(&mut self) -> Result<(), Self::Error> { self.0.dc_is_high = true; Ok(()) }
    }

    #[test]
    fn test_bytecode_parser() {
        let mut spi = MockSpi { logs: Vec::new(), current_cmd: None, dc_is_high: false };
        {
            let mut dc = MockPin(&mut spi);
            let mut iface = SpiInterface { spi: &mut dc.0, dc: &mut dc };
            run_vendor_init(&mut iface).unwrap();
        }

        // Verify specific entries in the bytecode
        // First command: 0xEF with 0 params
        assert_eq!(spi.logs[0].0, 0xEF);
        assert_eq!(spi.logs[0].1.len(), 0);

        // Second command: 0xEB with 1 param (0x14)
        assert_eq!(spi.logs[1].0, 0xEB);
        assert_eq!(spi.logs[1].1, vec![0x14]);

        // Final commands should be SLPOUT (0x11)
        assert_eq!(spi.logs.last().unwrap().0, 0x11);
    }
}