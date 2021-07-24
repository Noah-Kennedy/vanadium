use vanadium_benches::{bench_image, large_header};
use vanadium_core::ops::{get_image_f32, BackendSelector};

fn main() {
    let mut image = get_image_f32(BackendSelector::Syscall, large_header()).unwrap();
    bench_image(image.as_mut())
}