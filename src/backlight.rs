//! Simple digital backlight control logic.

use embedded_hal::digital::OutputPin;

/// Sets the state of the backlight control pin.
///
/// True drives the pin high (on), false drives it low (off).
/// Dimming via PWM is not covered in this phase; this is purely digital control.
pub(crate) fn set_backlight<BL, PinE>(bl: &mut BL, on: bool) -> Result<(), PinE>
where
    BL: OutputPin<Error = PinE>,
{
    if on {
        bl.set_high()
    } else {
        bl.set_low()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;

    struct MockPin {
        pub is_high: bool,
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

    #[test]
    fn test_set_backlight_on() {
        let mut pin = MockPin { is_high: false };
        set_backlight(&mut pin, true).unwrap();
        assert_eq!(pin.is_high, true);
    }

    #[test]
    fn test_set_backlight_off() {
        let mut pin = MockPin { is_high: true };
        set_backlight(&mut pin, false).unwrap();
        assert_eq!(pin.is_high, false);
    }
}