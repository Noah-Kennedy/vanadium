extern crate openblas_src;

use tcmalloc::TCMalloc;

use vanadium_benches::{bench_covariance, small_header};
use vanadium_core::ops::{BackendSelector, get_image_f32};

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

fn main() {
    let mut image = get_image_f32(BackendSelector::Glommio, small_header()).unwrap();
    bench_covariance(image.as_mut())
}