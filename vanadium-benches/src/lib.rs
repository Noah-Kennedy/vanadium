use vanadium_core::Image;
use std::time::Instant;
use vanadium_core::headers::{Header, ImageDims, ImageFormat};
use std::path::PathBuf;

pub fn small_header() -> Header {
    Header {
        dims: ImageDims {
            channels: 5,
            lines: 21954,
            pixels: 28740
        },
        format: ImageFormat::Bip,
        path: PathBuf::from("/data/undergrad-research/bench-data/small-bip")
    }
}

pub fn large_header() -> Header {
    Header {
        dims: ImageDims {
            channels: 394,
            lines: 17408,
            pixels: 18176
        },
        format: ImageFormat::Bip,
        path: PathBuf::from("/data/undergrad-research/bench-data/large-bip")
    }
}

pub fn bench_image(image: &mut dyn Image<f32>) {
    let timer = Instant::now();

    let mean_timer = Instant::now();
    let means = image.means().unwrap();
    println!("Means: {}", mean_timer.elapsed().as_secs_f64());

    let std_timer = Instant::now();
    let std_devs = image.std_deviations(&means).unwrap();
    println!("Std: {}", std_timer.elapsed().as_secs_f64());

    let cov_timer = Instant::now();
    let covariances = image.covariance_matrix(&means, &std_devs).unwrap();
    println!("Covariances: {}", cov_timer.elapsed().as_secs_f64());

    println!("Total: {}", timer.elapsed().as_secs_f64());

    println!("Cov matrix: {:?}", covariances)
}