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
    display::GC9A01,
    interface::SpiInterface,
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

    // SPI pins
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    // Display control pins
    let mut _cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let dc = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    let rst = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);

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

    // Initialize display interface
    let interface = SpiInterface::new(spi, dc);

    // Initialize display
    let config = DisplayConfig::default();
    let mut display = GC9A01::new(interface, rst, config);

    // Initialize and clear screen
    display.init(&mut delay).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // Setup Text Styles
    let title_style = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
    let info_style = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);

    // Draw Title Text
    Text::with_alignment(
        "GC9A01 TFT",
        Point::new(120, 60),
        title_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    // Draw Line Separator
    Line::new(Point::new(40, 75), Point::new(200, 75))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(&mut display)
        .unwrap();

    // Draw some shapes
    let circle_style = PrimitiveStyle::with_stroke(Rgb565::RED, 3);
    Circle::new(Point::new(60, 100), 40)
        .into_styled(circle_style)
        .draw(&mut display)
        .unwrap();

    let rect_style = PrimitiveStyle::with_fill(Rgb565::GREEN);
    Rectangle::new(Point::new(130, 100), Size::new(40, 40))
        .into_styled(rect_style)
        .draw(&mut display)
        .unwrap();

    // Dynamic text with numbers
    let mut counter = 0;
    loop {
        // Clear previous text area by drawing a black rectangle
        Rectangle::new(Point::new(70, 170), Size::new(100, 20))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut display)
            .unwrap();

        // Format number into a string buffer
        let mut text_buf: String<32> = String::new();
        write!(&mut text_buf, "Ticks: {}", counter).unwrap();

        // Draw updated text
        Text::with_alignment(
            &text_buf,
            Point::new(120, 185),
            info_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        counter += 1;
        delay.delay_ms(500_u16);
    }
}
