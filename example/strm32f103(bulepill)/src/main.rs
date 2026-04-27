#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
};
use GC9A01::{
    config::Config,
    display::GC9A01,
    interface::SPIDisplayInterface,
};

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
    let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let mut dc = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    let mut rst = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);

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
    let interface = SPIDisplayInterface::new(spi, dc, cs);

    // Initialize display
    let config = Config::default();
    let mut display = GC9A01::new(interface, rst, config);

    // Initialize and clear screen
    display.init(&mut delay).unwrap();
    display.clear_screen(0x0000).unwrap();

    loop {
        // Your code here
    }
}
