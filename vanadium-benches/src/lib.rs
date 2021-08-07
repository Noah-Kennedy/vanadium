use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::Instant;

use vanadium_core::headers::{Header, ImageDims, ImageFormat};
use vanadium_core::io::ImageStats;

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

pub fn bench_means(image: &mut dyn ImageStats<f32>) {
    let mean_timer = Instant::now();
    let means = image.means().unwrap();

    println!("Means:\t{}\n\t{}", mean_timer.elapsed().as_secs_f64(), &means);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./means.json")
        .unwrap();

    serde_json::to_writer(file, &means).unwrap();
}

pub fn bench_stds(image: &mut dyn ImageStats<f32>) {
    let means = serde_json::from_reader(File::open("means.json").unwrap()).unwrap();

    let timer = Instant::now();
    let std_devs = image.std_deviations(&means).unwrap();

    println!("STD:\t{}\n\t{}", timer.elapsed().as_secs_f64(), &std_devs);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./std_devs.json")
        .unwrap();

    serde_json::to_writer(file, &std_devs).unwrap();
}

pub fn bench_covariance(image: &mut dyn ImageStats<f32>) {
    let means = serde_json::from_reader(File::open("means.json").unwrap()).unwrap();
    let std_devs = serde_json::from_reader(File::open("std_devs.json").unwrap()).unwrap();

    let timer = Instant::now();
    let covariances = image.covariance_matrix(Some(&means), Some(&std_devs)).unwrap();

    println!("Covariances:\t{}\n\t{}", timer.elapsed().as_secs_f64(), &covariances);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./covariances.json")
        .unwrap();

    serde_json::to_writer(file, &covariances).unwrap();
}

pub fn bench_pca_eigen(image: &mut dyn ImageStats<f32>) {
    let covariances = serde_json::from_reader(File::open("covariances.json").unwrap()).unwrap();

    let timer = Instant::now();

    let eigen = image.pca_eigen(10, &covariances).unwrap();

    println!("Eigen:\t{}\n\t{}", timer.elapsed().as_secs_f64(), &eigen);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./eigen.json")
        .unwrap();

    serde_json::to_writer(file, &eigen).unwrap();
}

pub fn bench_pca_write(image: &mut dyn ImageStats<f32>) {
    let means = serde_json::from_reader(File::open("means.json").unwrap()).unwrap();
    let std_devs = serde_json::from_reader(File::open("std_devs.json").unwrap()).unwrap();
    let eigen = serde_json::from_reader(File::open("eigen.json").unwrap()).unwrap();

    let timer = Instant::now();

    image.write_transformed(&eigen, &"./pcafile".to_string(), Some(&means), Some(&std_devs)).unwrap();

    println!("Write:\t{}", timer.elapsed().as_secs_f64());
}