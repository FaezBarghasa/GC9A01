//! Unified error types mapping SPI transactions and GPIO pin failures.

use core::fmt::{self, Debug, Display};

/// Represents all possible errors that can occur during display operation.
///
/// We use two generic parameters to preserve the exact error types
/// returned by the underlying HAL's SPI and GPIO implementations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error<SpiE, PinE> {
    /// An error occurred during an SPI transaction.
    /// This typically indicates a communication failure with the display hardware.
    Spi(SpiE),

    /// An error occurred while toggling a GPIO pin.
    /// This could be the Data/Command (DC) pin, Reset (RST) pin, or Backlight (BL) pin.
    Pin(PinE),

    /// The provided configuration parameters are invalid.
    /// For example, specifying a display window that is out of bounds.
    InvalidConfig,

    /// A specified window or drawing area is out of the bounds of the display resolution.
    OutOfBounds,
}

// Implement `core::fmt::Display` to provide user-friendly error messages
impl<SpiE: Display, PinE: Display> Display for Error<SpiE, PinE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Spi(err) => write!(f, "SPI Communication Error: {}", err),
            Error::Pin(err) => write!(f, "GPIO Pin Toggle Error: {}", err),
            Error::InvalidConfig => write!(f, "Invalid Display Configuration"),
            Error::OutOfBounds => write!(f, "Drawing coordinates or window out of bounds"),
        }
    }
}

// Note: Blanket `From` implementations for unconstrained generic parameters
// are disallowed by Rust's coherence rules (as SpiE and PinE could theoretically
// be the same type). Therefore, we map errors explicitly using `Error::Spi`
// and `Error::Pin` in the codebase.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_wrapping() {
        let spi_err: Error<u8, u16> = Error::Spi(42);
        let pin_err: Error<u8, u16> = Error::Pin(100);

        assert_eq!(spi_err, Error::Spi(42));
        assert_eq!(pin_err, Error::Pin(100));
        assert_eq!(Error::<u8, u16>::OutOfBounds, Error::OutOfBounds);
    }
}
