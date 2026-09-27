use criterion::{Criterion, black_box, criterion_group, criterion_main};
use embedded_graphics_core::pixelcolor::Rgb565;
use gc9a01::{Display, DisplayConfig};
// Assume mock objects exist in test module
use gc9a01::interface::tests::{MockPin, MockSpi};

fn bench_clear(c: &mut Criterion) {
    let mut display = Display::new(
        MockSpi::new_null_sink(), // Null sink discards bytes instantly
        MockPin::new(),
        MockPin::new(),
        MockPin::new(),
        DisplayConfig::default(),
    );

    c.bench_function("clear_full_screen_null_sink", |b| {
        b.iter(|| {
            display.clear(black_box(Rgb565::BLACK)).unwrap();
        });
    });
}

criterion_group!(benches, bench_clear);
criterion_main!(benches);
