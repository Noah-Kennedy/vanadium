use num::traits::float::Float;

use crate::container::LockImage;
use crate::container::mapped::{Bip, Bsq, SpectralImageContainer};

use super::*;

const FLOAT_COMP: f32 = 0.00001;

const DATA_BIP: [f32; 15] = [
    4.0, 2.0, 0.60,
    4.2, 2.1, 0.59,
    3.9, 2.0, 0.58,
    4.3, 2.1, 0.62,
    4.1, 2.2, 0.63,
];

const DATA_BSQ: [f32; 15] = [
    4.00, 4.20, 3.90, 4.30, 4.10,
    2.00, 2.10, 2.00, 2.10, 2.20,
    0.60, 0.59, 0.58, 0.62, 0.63,
];

const COV: [f32; 9] = [
    0.02500, 0.00750, 0.00175,
    0.00750, 0.00700, 0.00135,
    0.00175, 0.00135, 0.00043,
];

const MEANS: [f32; 3] = [4.1, 2.08, 0.604];
const STD_DEVS: [f32; 3] = [0.15811388, 0.083666004, 0.020736441];

fn data_bip() -> Vec<u8> {
    let mut r = Vec::with_capacity(60);

    for item in &DATA_BIP {
        r.extend_from_slice(&item.to_ne_bytes());
    }

    r
}

fn data_bsq() -> Vec<u8> {
    let mut r = Vec::with_capacity(60);

    for item in &DATA_BSQ {
        r.extend_from_slice(&item.to_ne_bytes());
    }

    r
}

fn approx_eq(expected: &[f32], actual: &[f32]) {
    let mut is_eq = true;

    is_eq &= expected.len() == actual.len();

    if is_eq {
        for (e, a) in expected.iter().zip(actual) {
            is_eq &= ((e.min(*a) / a.max(*e)) - 1.0).abs() <= FLOAT_COMP;
        }
    }

    if !is_eq {
        panic!("Error: expected != actual\nExpected:\t{:?}\nActual:\t{:?}", expected, actual);
    }
}

#[test]
fn test_mean_bip_small() {
    let bip: Bip<Vec<u8>, f32> = Bip {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bip(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let means = guard.all_band_means(&mp, f32::neg_infinity(), f32::infinity());

    approx_eq(&MEANS, &means);
}

#[test]
fn test_mean_bip_big() {
    let bip: Bip<Vec<u8>, f32> = Bip {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bip(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let means = guard.big_bip_means(&mp, f32::neg_infinity(), f32::infinity());

    approx_eq(&MEANS, &means);
}

#[test]
fn test_mean_bsq() {
    let bip: Bsq<Vec<u8>, f32> = Bsq {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bsq(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let means = guard.all_band_means(&mp, f32::neg_infinity(), f32::infinity());

    approx_eq(&MEANS, &means);
}

#[test]
fn test_std_dev_bip_small() {
    let bip: Bip<Vec<u8>, f32> = Bip {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bip(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let std_devs = guard.all_band_std_devs(&mp, &MEANS, f32::neg_infinity(), f32::infinity());

    approx_eq(&STD_DEVS, &std_devs);
}

#[test]
fn test_std_dev_bip_big() {
    let bip: Bip<Vec<u8>, f32> = Bip {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bip(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let std_devs = guard.big_bip_std_devs(&mp, &MEANS, f32::neg_infinity(), f32::infinity());

    approx_eq(&STD_DEVS, &std_devs);
}

#[test]
fn test_std_dev_bsq() {
    let image: Bsq<Vec<u8>, f32> = Bsq {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bsq(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(image);

    let guard = image.read();

    let mp = MultiProgress::new();

    let means = guard.all_band_means(&mp, f32::neg_infinity(), f32::infinity());

    let std_devs = guard.all_band_std_devs(&mp, &means, f32::neg_infinity(), f32::infinity());

    approx_eq(&STD_DEVS, &std_devs);
}

#[test]
fn test_covariance_bip_small() {
    let bip: Bip<Vec<u8>, f32> = Bip {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bip(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let means = guard.all_band_means(&mp, f32::neg_infinity(), f32::infinity());

    let cov_mat = guard.covariance_matrix(&mp, &means, f32::neg_infinity(), f32::infinity());

    let expected = DMatrix::from_row_slice(3, 3, &COV);

    approx_eq(&expected.data.as_vec(), &cov_mat.data.as_vec());
}

#[test]
fn test_covariance_bip_big() {
    let bip: Bip<Vec<u8>, f32> = Bip {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bip(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let cov_mat = guard.big_bip_cov_mat(&mp, &MEANS, f32::neg_infinity(), f32::infinity());

    let expected = DMatrix::from_row_slice(3, 3, &COV);

    approx_eq(&expected.data.as_vec(), &cov_mat.data.as_vec());
}

#[test]
fn test_covariance_bsq() {
    let bip: Bsq<Vec<u8>, f32> = Bsq {
        dims: ImageDims {
            channels: 3,
            lines: 1,
            samples: 5,
        },
        container: SpectralImageContainer {
            container: data_bsq(),
            phantom: Default::default(),
        },
    };

    let image: LockImage<f32, _> = LockImage::new(bip);

    let guard = image.read();

    let mp = MultiProgress::new();

    let means = guard.all_band_means(&mp, f32::neg_infinity(), f32::infinity());

    let cov_mat = guard.covariance_matrix(&mp, &means, f32::neg_infinity(), f32::infinity());

    let expected = DMatrix::from_row_slice(3, 3, &COV);

    approx_eq(&expected.data.as_vec(), &cov_mat.data.as_vec());
}
