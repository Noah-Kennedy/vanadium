use vanadium_benches::{bench_image, small_header};
use vanadium_core::image_backends::get_image_f32;

fn main() {
    let mut image = get_image_f32(Some("sync"), small_header()).unwrap();
    bench_image(image.as_mut())
}