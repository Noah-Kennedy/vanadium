use std::path::PathBuf;
use std::time::Instant;

use vanadium_core::headers::{Header, ImageDims, ImageFormat};
use vanadium_core::ops::Image;

pub fn small_header() -> Header {
    Header {
        dims: ImageDims {
            channels: 5,
            lines: 21954,
            pixels: 28740,
        },
        format: ImageFormat::Bip,
        path: PathBuf::from("/data/undergrad-research/bench-data/small-bip"),
    }
}

pub fn large_header() -> Header {
    Header {
        dims: ImageDims {
            channels: 394,
            lines: 17408,
            pixels: 18176,
        },
        format: ImageFormat::Bip,
        path: PathBuf::from("/data/undergrad-research/bench-data/large-bip"),
    }
}

pub fn bench_covariance(image: &mut dyn Image<f32>) {
    let timer = Instant::now();

    let mean_timer = Instant::now();
    let means = image.means().unwrap();
    println!("Means:\t{}\n\t{}", mean_timer.elapsed().as_secs_f64(), &means);

    let cov_timer = Instant::now();
    let covariances = image.covariance_matrix(Some(&means), None).unwrap();
    println!("Covs:\t{}\n\t{}", cov_timer.elapsed().as_secs_f64(), &covariances);

    println!("Total: {}", timer.elapsed().as_secs_f64());
}

pub fn bench_covariance_standardized(image: &mut dyn Image<f32>) {
    let timer = Instant::now();

    let mean_timer = Instant::now();
    let means = image.means().unwrap();
    println!("Means:\t{}\n\t{}", mean_timer.elapsed().as_secs_f64(), &means);

    let std_timer = Instant::now();
    let std_devs = image.std_deviations(&means).unwrap();
    println!("STDs:\t{}\n\t{}", std_timer.elapsed().as_secs_f64(), &std_devs);

    let cov_timer = Instant::now();
    let covariances = image.covariance_matrix(Some(&means), Some(&std_devs)).unwrap();
    println!("Covs:\t{}\n\t{}", cov_timer.elapsed().as_secs_f64(), &covariances);

    println!("Total: {}", timer.elapsed().as_secs_f64());
}