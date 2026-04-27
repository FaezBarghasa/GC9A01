//! Low-level SPI transport layer handling Data/Command (DC) pin logic and raw byte streaming.

use crate::error::Error;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

/// Crate-private low-level SPI transport layer.
/// This strictly separates raw SPI byte transmission and Data/Command pin toggling
/// from higher-level driver logic.
pub(crate) struct SpiInterface<SPI, DC> {
    pub(crate) spi: SPI,
    pub(crate) dc: DC,
}

impl<SPI, DC> SpiInterface<SPI, DC> {
    /// Creates a new SPI interface.
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI, DC, SpiE, PinE> SpiInterface<SPI, DC>
where
    SPI: SpiDevice<Error = SpiE>,
    DC: OutputPin<Error = PinE>,
{
    /// Selects Command mode and writes a single byte over SPI.
    #[inline]
    pub(crate) fn send_command(&mut self, command: u8) -> Result<(), Error<SpiE, PinE>> {
        self.dc.set_low().map_err(Error::Pin)?;
        self.spi.write(&[command]).map_err(Error::Spi)?;
        Ok(())
    }

    /// Selects Data mode and streams a slice of bytes over SPI.
    #[inline]
    pub(crate) fn send_data(&mut self, data: &[u8]) -> Result<(), Error<SpiE, PinE>> {
        if data.is_empty() {
            return Ok(());
        }
        self.dc.set_high().map_err(Error::Pin)?;
        self.spi.write(data).map_err(Error::Spi)?;
        Ok(())
    }

    /// Convenience pipeline for a command followed by its associated data payloads.
    #[inline]
    pub(crate) fn send_command_data(
        &mut self,
        command: u8,
        data: &[u8],
    ) -> Result<(), Error<SpiE, PinE>> {
        self.send_command(command)?;
        if !data.is_empty() {
            self.send_data(data)?;
        }
        Ok(())
    }

    /// High-performance bulk pixel fill.
    /// Batches writes in 128-byte (64 pixel) chunks to eliminate single-pixel SPI overhead
    /// while remaining completely dynamically allocated memory (heap) free.
    pub(crate) fn fill_pixels(&mut self, color: u16, mut count: u32) -> Result<(), Error<SpiE, PinE>> {
        if count == 0 {
            return Ok(());
        }

        // Prepare a 128-byte (64 pixel) stack buffer populated with Big-Endian color bytes.
        let mut buf = [0u8; 128];
        crate::color::pack_rgb565_fill(color, &mut buf);

        // Drive DC High ONCE for the entire batch operation.
        self.dc.set_high().map_err(Error::Pin)?;

        while count > 0 {
            let pixels_this_chunk = if count > 64 { 64 } else { count };
            let bytes_this_chunk = (pixels_this_chunk * 2) as usize;

            self.spi
                .write(&buf[..bytes_this_chunk])
                .map_err(Error::Spi)?;

            count -= pixels_this_chunk;
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    extern crate std;
    use std::vec::Vec;
    use super::*;
    use embedded_hal::spi::SpiDevice;
    use embedded_hal::digital::OutputPin;
    use core::convert::Infallible;
    use embedded_hal::spi::Operation;

    pub struct MockSpi {
        pub written: Vec<u8>,
    }

    impl MockSpi {
        pub fn new() -> Self {
            Self { written: Vec::new() }
        }
        pub fn written_bytes(&self) -> &[u8] {
            &self.written
        }
    }

    impl embedded_hal::spi::ErrorType for MockSpi {
        type Error = Infallible;
    }

    impl SpiDevice for MockSpi {
        fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
            for op in operations {
                if let Operation::Write(bytes) = op {
                    self.written.extend_from_slice(bytes);
                }
            }
            Ok(())
        }

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.written.extend_from_slice(words);
            Ok(())
        }
    }

    pub struct MockPin {
        pub is_high: bool,
    }

    impl MockPin {
        pub fn new() -> Self {
            Self { is_high: false }
        }
    }

    impl embedded_hal::digital::ErrorType for MockPin {
        type Error = Infallible;
    }

    impl OutputPin for MockPin {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.is_high = false;
            Ok(())
        }
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.is_high = true;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::*;
    use std::vec;

    #[test]
    fn test_send_command_drives_dc_low() {
        let mut spi = MockSpi { written: Vec::new() };
        let mut dc = MockPin { is_high: true };
        let mut iface = SpiInterface { spi: &mut spi, dc: &mut dc };

        iface.send_command(0x3A).unwrap();

        assert_eq!(dc.is_high, false); // DC must be driven low
        assert_eq!(spi.written, vec![0x3A]); // Command sent
    }

    #[test]
    fn test_send_data_drives_dc_high() {
        let mut spi = MockSpi { written: Vec::new() };
        let mut dc = MockPin { is_high: false };
        let mut iface = SpiInterface { spi: &mut spi, dc: &mut dc };

        iface.send_data(&[0x11, 0x22]).unwrap();

        assert_eq!(dc.is_high, true); // DC must be driven high
        assert_eq!(spi.written, vec![0x11, 0x22]); // Data sent
    }

    #[test]
    fn test_fill_pixels_batching() {
        let mut spi = MockSpi { written: Vec::new() };
        let mut dc = MockPin { is_high: false };
        let mut iface = SpiInterface { spi: &mut spi, dc: &mut dc };

        // Test with White (0xFFFF) for 100 pixels
        iface.fill_pixels(0xFFFF, 100).unwrap();

        assert_eq!(dc.is_high, true); // DC must be high
        // 100 pixels * 2 bytes/pixel = 200 bytes total written
        assert_eq!(spi.written.len(), 200);
        // All bytes should be 0xFF
        assert!(spi.written.iter().all(|&b| b == 0xFF));
    }
}
