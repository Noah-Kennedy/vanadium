use vanadium_benches::{bench_image, large_header};
use vanadium_core::asynchronous_ops::glommio::bip::GlommioBip;

fn main() {
    let mut image: GlommioBip<f32> = GlommioBip::new(large_header());

    bench_image(&mut image)
}