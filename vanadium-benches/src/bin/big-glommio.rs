extern crate openblas_src;

use tcmalloc::TCMalloc;

use vanadium_benches::{bench_covariance, large_header};
use vanadium_io::bip::GlommioBip;

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

fn main() {
    let mut image = GlommioBip::new(large_header());
    bench_covariance(&mut image)
}