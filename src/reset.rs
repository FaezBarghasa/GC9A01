//! Hardware reset sequences and timing functions.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

/// Executes a hardware reset sequence for the GC9A01.
///
/// By default (following the datasheet power-on sequence):
/// 1. Drives the RST pin low.
/// 2. Waits for a minimum duration (10ms).
/// 3. Drives the RST pin high.
/// 4. Waits for the controller's internal setup time (120ms) before it can accept commands.
pub(crate) fn hardware_reset<RST, D, PinE>(
    rst: &mut RST,
    delay: &mut D,
    reset_duration_ms: u32,
    reset_after_ms: u32,
) -> Result<(), PinE>
where
    RST: OutputPin<Error = PinE>,
    D: DelayNs,
{
    rst.set_low()?;
    delay.delay_ms(reset_duration_ms);
    rst.set_high()?;
    delay.delay_ms(reset_after_ms);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;

    struct MockPin {
        pub state: bool,
    }

    impl embedded_hal::digital::ErrorType for MockPin {
        type Error = Infallible;
    }

    impl OutputPin for MockPin {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state = false;
            Ok(())
        }
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state = true;
            Ok(())
        }
    }

    struct MockDelay {
        pub delays_ms: std::vec::Vec<u32>,
    }

    impl DelayNs for MockDelay {
        fn delay_ns(&mut self, _ns: u32) {
            unimplemented!()
        }
        fn delay_ms(&mut self, ms: u32) {
            self.delays_ms.push(ms);
        }
    }

    #[test]
    fn test_hardware_reset_sequence() {
        let mut rst_pin = MockPin { state: true };
        let mut delay = MockDelay {
            delays_ms: std::vec::Vec::new(),
        };

        hardware_reset(&mut rst_pin, &mut delay, 10, 120).unwrap();

        assert_eq!(rst_pin.state, true); // Pin ends high
        assert_eq!(delay.delays_ms, vec![10, 120]); // Delays were called in order
    }
}