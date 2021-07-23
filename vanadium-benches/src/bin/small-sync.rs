use vanadium_benches::{bench_image, small_header};
use vanadium_core::sync_syscall::SyncBip;

fn main() {
    let mut image: SyncBip<f32> = SyncBip::new(small_header()).unwrap();

    bench_image(&mut image)
}