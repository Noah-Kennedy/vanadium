use vanadium_benches::{bench_image, small_header};
use vanadium_core::asynchronous_ops::glommio::bip::GlommioBip;

fn main() {
    let mut image: GlommioBip<f32> = GlommioBip::new(small_header());

    bench_image(&mut image)
}