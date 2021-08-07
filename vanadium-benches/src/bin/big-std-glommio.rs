extern crate openblas_src;

use tcmalloc::TCMalloc;

use vanadium_benches::{bench_stds, large_header};
use vanadium_core::io::bip::GlommioBip;

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

fn main() {
    let mut image = GlommioBip::new(large_header());
    bench_stds(&mut image)
}