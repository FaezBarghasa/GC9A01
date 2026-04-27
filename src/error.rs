//! Unified error types mapping SPI transactions and GPIO pin failures.

use core::fmt::Debug;

/// Represents all possible errors that can occur during display operation.
///
/// We use two generic parameters to preserve the exact error types
/// returned by the underlying HAL's SPI and GPIO implementations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error<SpiE, PinE> {
    /// An error occurred during an SPI transaction.
    Spi(SpiE),
    /// An error occurred while toggling a GPIO pin (DC, RST, or BL).
    Pin(PinE),
    /// The provided display configuration is invalid.
    InvalidConfig,
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
    }
}