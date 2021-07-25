extern crate openblas_src;

use tcmalloc::TCMalloc;

use vanadium_benches::{bench_covariance, small_header};
use vanadium_io::bip::SyscallBip;

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

fn main() {
    let mut image = SyscallBip::new(small_header()).unwrap();
    bench_covariance(&mut image)
}