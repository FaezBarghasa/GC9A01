//! # GC9A01 Bluepill Example
//!
//! This example demonstrates how to use the `GC9A01` crate to drive a 240x240
//! SPI TFT display using an STM32F103 (Bluepill) microcontroller.
//!
//! It initializes the display and utilizes `embedded-graphics` to draw text,
//! basic shapes, and dynamically update a counter on the screen.

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
};
use GC9A01::{
    config::DisplayConfig,
    display::Display,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use heapless::String;

#[entry]
fn main() -> ! {
    // Acquire hardware peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Configure system clocks for 72MHz
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split();

    // ------------------------------------------------------------------------
    // Pin Configuration
    // ------------------------------------------------------------------------

    // SPI pins
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    // Display control pins
    // CS is unused directly in the library if we hold it low or manage it externally,
    // but typically it needs to be configured as an output.
    let mut _cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let dc = gpioa.pa3.into_push_pull_output(&mut gpioa.crl); // Data/Command pin
    let rst = gpioa.pa2.into_push_pull_output(&mut gpioa.crl); // Reset pin
    let bl = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);  // Backlight pin

    // ------------------------------------------------------------------------
    // SPI & Display Initialization
    // ------------------------------------------------------------------------

    // GC9A01 requires SPI mode 0 (IdleLow, CaptureOnFirstTransition)
    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut gpioa.mapr,
        spi_mode,
        10.mhz(),
        clocks,
    );

    let mut delay = Delay::new(cp.SYST, clocks);

    // Create the display instance with default configuration
    let config = DisplayConfig::default();
    let mut display = Display::new(spi, dc, rst, bl, config);

    // Initialize the display hardware (resets display and sends initialization commands)
    display.init(&mut delay).unwrap();

    // Clear the screen with a black background
    display.clear(Rgb565::BLACK).unwrap();

    // ------------------------------------------------------------------------
    // Static Graphics Rendering
    // ------------------------------------------------------------------------

    // Define text styles for rendering using standard embedded-graphics mono fonts
    let title_style = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
    let info_style = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);

    // Draw the main title text centered at the top
    Text::with_alignment(
        "GC9A01 TFT",
        Point::new(120, 60),
        title_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    // Draw a horizontal line separating the title from the content
    Line::new(Point::new(40, 75), Point::new(200, 75))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(&mut display)
        .unwrap();

    // Draw a stroked red circle
    let circle_style = PrimitiveStyle::with_stroke(Rgb565::RED, 3);
    Circle::new(Point::new(60, 100), 40)
        .into_styled(circle_style)
        .draw(&mut display)
        .unwrap();

    // Draw a filled green rectangle
    let rect_style = PrimitiveStyle::with_fill(Rgb565::GREEN);
    Rectangle::new(Point::new(130, 100), Size::new(40, 40))
        .into_styled(rect_style)
        .draw(&mut display)
        .unwrap();

    // ------------------------------------------------------------------------
    // Main Loop (Dynamic Content)
    // ------------------------------------------------------------------------

    let mut counter = 0;
    loop {
        // 1. Clear the previous text area by drawing a black rectangle over it.
        // This prevents the text from overlapping and becoming unreadable.
        Rectangle::new(Point::new(70, 170), Size::new(100, 20))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut display)
            .unwrap();

        // 2. Format the counter number into a heapless string buffer
        let mut text_buf: String<32> = String::new();
        write!(&mut text_buf, "Ticks: {}", counter).unwrap();

        // 3. Draw the newly updated text
        Text::with_alignment(
            &text_buf,
            Point::new(120, 185),
            info_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        counter += 1;

        // 4. Delay to make the update visually pleasing (2 FPS)
        delay.delay_ms(500_u16);
    }
}
