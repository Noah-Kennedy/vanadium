extern crate openblas_src;

use tcmalloc::TCMalloc;

use vanadium_benches::{large_header, bench_means};
use vanadium_io::bip::GlommioBip;

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

fn main() {
    let mut image = GlommioBip::new(large_header());
    bench_means(&mut image)
}