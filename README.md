# GC9A01 Display Driver

A `no_std`, `embedded-hal` 1.x compatible driver for the **GC9A01** 240x240 SPI TFT display.

This library is designed to work seamlessly with `embedded-graphics` to provide an easy and performant way to draw shapes, text, and images to circular displays using the GC9A01 controller.

## Features

- **`no_std` Support**: Designed for bare-metal embedded systems.
- **`embedded-hal` 1.x**: Built on the latest standard for Rust embedded hardware abstractions.
- **`embedded-graphics` Integration**: Fully implements `embedded-graphics`'s `DrawTarget` trait, enabling rich graphical elements right out of the box.

## Architecture

This crate is structured to cleanly separate concerns:

- **`interface::SpiInterface`**: Handles low-level SPI communication and Data/Command (`DC`) pin toggling.
- **`config::DisplayConfig`**: Configuration struct for modifying the behavior of the display (e.g., orientation).
- **`display::Display`**: The main struct representing the display. It implements the initialization sequence and the `DrawTarget` trait for `embedded-graphics`.

## Quick Start

Add the following to your `Cargo.toml`:

```toml
[dependencies]
GC9A01 = "0.1.0"
embedded-hal = "1.0"
embedded-graphics = "0.8"
```

### Basic Example

Here is a minimal example of initializing the display and drawing some simple graphics using `embedded-graphics`.

```rust
#![no_std]
#![no_main]

use GC9A01::{
    config::DisplayConfig,
    display::Display,
    interface::SpiInterface,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
// Use your specific HAL, e.g., stm32f1xx_hal
use stm32f1xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split();

    // 1. Setup SPI pins
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    // 2. Setup Display control pins
    let mut _cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let dc = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    let rst = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);

    // 3. Setup Backlight Pin
    let bl = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);

    // 4. Initialize SPI
    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };
    let spi = Spi::spi1(dp.SPI1, (sck, miso, mosi), &mut gpioa.mapr, spi_mode, 10.mhz(), clocks);

    // 5. Create a delay provider
    let mut delay = Delay::new(cp.SYST, clocks);

    // 6. Initialize the display interface and the display itself
    let config = DisplayConfig::default();
    let mut display = Display::new(spi, dc, rst, bl, config);

    // 7. Initialize the hardware and clear the screen
    display.init(&mut delay).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // 8. Draw graphics!
    let style = PrimitiveStyle::with_stroke(Rgb565::RED, 3);
    Circle::new(Point::new(60, 60), 120)
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
    Text::with_alignment("Hello, World!", Point::new(120, 120), text_style, Alignment::Center)
        .draw(&mut display)
        .unwrap();

    loop {}
}
```

## Running Examples

An example project exists under the `example/` directory. If you have an STM32F103 (Blue Pill) and an ST-Link, you can run it via:

```sh
cd example/strm32f103\(bulepill\)
cargo run
```
