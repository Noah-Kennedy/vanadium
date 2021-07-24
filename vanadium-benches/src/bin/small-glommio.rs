extern crate openblas_src;

use vanadium_benches::{bench_covariance, small_header};
use vanadium_core::ops::{get_image_f32, BackendSelector};

fn main() {
    let mut image = get_image_f32(BackendSelector::Glommio, small_header()).unwrap();
    bench_covariance(image.as_mut())
}