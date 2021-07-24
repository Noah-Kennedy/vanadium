use vanadium_benches::{bench_covariance, small_header};
use vanadium_core::ops::{get_image_f32, BackendSelector};

fn main() {
    let mut image = get_image_f32(BackendSelector::Syscall, small_header()).unwrap();
    bench_covariance(image.as_mut())
}