use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::ops::{Div, Sub};

use either::Either;
use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{ComplexField, DMatrix, RealField};
use num::{Bounded, Zero};
use num::traits::NumAssign;
use rayon::prelude::*;

use crate::bar::config_bar;
use crate::container::{CHUNK_SIZE, ImageDims, IterableImage, IterableImageMut, ReadImageGuard};

impl<'a, 'b, I, T> ReadImageGuard<'a, T, I>
    where I: IterableImage<'b, T> + Sync + IterableImageMut<'b, T>,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded
          + Display + ComplexField + ComplexField<RealField=T> + RealField + Sum
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn band_mean(&self, band: usize, min: T, max: T) -> T {
        let mut sum = T::zero();
        let mut count = T::zero();

        for &x in self.inner.band(band) {
            if x > min && x <= max {
                sum += x;
                count += T::one();
            }
        }

        sum / count
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn band_std_dev(&self, band: usize, mean: Option<T>, min: T, max: T) -> T {
        let mean = if let Some(mean) = mean {
            mean
        } else {
            self.band_mean(band, min, max)
        };

        let mut sum = T::zero();
        let mut count = T::zero();

        for &x in self.inner.band(band) {
            let diff = x - mean;

            if x > min && x <= max {
                sum += diff * diff;
                count += T::one();
            }
        }

        (sum / count).sqrt()
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn covariance_pair(&self, bands: [usize; 2], means: [T; 2], min: T, max: T) -> T {
        let mut sum = T::zero();
        let mut count = 0usize;

        let itc = self.inner.band(bands[0])
            .zip(self.inner.band(bands[1]));

        for (&x0, &x1) in itc {
            if x0 > min && x0 <= max && x1 > min && x1 <= max {
                let diffs = [
                    x0 - means[0],
                    x1 - means[1]
                ];

                sum += diffs[0] * diffs[1];
                count += 1;
            }
        }

        sum / T::from_usize(count - 1).unwrap()
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn all_band_means(&self, mp: &MultiProgress, min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        let status_bar = mp.add(ProgressBar::new(channels as u64));
        config_bar(&status_bar, "Calculating band means...");

        let means = (0..channels)
            .into_par_iter()
            .map(|b| {
                let out = self.band_mean(b, min, max);
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        means
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn all_band_std_devs(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        let status_bar = mp.add(ProgressBar::new(channels as u64));
        config_bar(&status_bar, "Calculating band std devs...");

        let devs = (0..channels)
            .into_par_iter()
            .zip(means.par_iter())
            .map(|(b, m)| {
                let out = self.band_std_dev(b, Some(*m), min, max);
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        devs
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn covariance_matrix(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> DMatrix<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();
        if let Either::Right(chunk) = self.inner.fastest() {
            let num_samples = lines * samples;

            let mut sums = vec![T::zero(); channels * channels];
            let mut counts = vec![0; channels * channels];

            let status_bar = mp.add(ProgressBar::new(num_samples as u64));
            config_bar(&status_bar, "Calculating covariances...");

            for s in chunk {
                let outer = s.clone();

                for (outer_i, outer_b) in outer.enumerate() {
                    let inner = s.clone();

                    for (inner_i, inner_b) in inner
                        .take(outer_i + 1)
                        .enumerate()
                    {
                        if *outer_b > min && *outer_b <= max && *inner_b > min && *inner_b <= max {
                            let diffs = [
                                *outer_b - means[0],
                                *inner_b - means[1]
                            ];

                            let idx = (outer_i * channels) + inner_i;

                            sums[idx] += diffs[0] * diffs[1];
                            counts[idx] += 1;
                        }
                    }
                }
            }

            sums.iter_mut().zip(counts).for_each(|(s, c)| {
                if c > 1 {
                    *s /= T::from_usize(c - 1).unwrap();
                }
            });

            status_bar.finish();

            let mut out = DMatrix::from_row_slice(channels, channels, &sums);
            out.fill_upper_triangle_with_lower_triangle();

            out
        } else {
            let tot_val = (channels + 1) * (channels + 1);

            let status_bar = mp.add(ProgressBar::new(tot_val as u64));
            config_bar(&status_bar, "Calculating band means...");

            let covariances: Vec<T> = (0..channels)
                .into_par_iter()
                .map(|b1| {
                    let mut v: Vec<T> = (0..=b1)
                        .map(|b2| {
                            let out =
                                self.covariance_pair(
                                    [b1, b2],
                                    [means[b1], means[b2]],
                                    min, max,
                                );
                            status_bar.inc(1);
                            out
                        })
                        .collect();

                    v.reserve(channels - v.len());

                    while v.len() < channels {
                        v.push(T::zero())
                    }

                    v
                })
                .flatten()
                .collect();

            status_bar.finish();

            let mut out = DMatrix::from_row_slice(channels, channels, &covariances);
            out.fill_upper_triangle_with_lower_triangle();

            out
        }
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline(never))]
pub fn normify<T>(val: T, scale: T, min: T, max: T) -> T
    where T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T> + Debug + Zero
{
    let clamped = num::clamp(val, min, max);
    let shifted = clamped - min;
    shifted / scale
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
    use num::traits::float::Float;

    use crate::container::LockImage;
    use crate::container::mapped::{Bip, Bsq, SpectralImageContainer};

    use super::*;

    const DATA_BIP: [f32; 15] = [
        4.0, 2.0, 0.60,
        4.2, 2.1, 0.59,
        3.9, 2.0, 0.58,
        4.3, 2.1, 0.62,
        4.1, 2.2, 0.63,
    ];

    const DATA_BSQ: [f32; 15] = [
        4.0, 4.2, 3.9, 4.3, 4.1,
        2.0, 2.1, 2.0, 2.1, 2.2,
        0.60, 0.59, 0.58, 0.62, 0.63,
    ];

    const COV: [f32; 9] = [
        0.025, 0.0074999877, 0.00175,
        0.0074999877, 0.007000001, 0.0013499998,
        0.00175, 0.0013499998, 0.0004300003,
    ];

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

    #[test]
    fn test_mean_bip() {
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

        assert_eq!(vec![4.1000004, 2.08, 0.604], means);
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

        assert_eq!(vec![4.1000004, 2.08, 0.604], means);
    }

    #[test]
    fn test_covariance_bip() {
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

        assert_eq!(expected, cov_mat);
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

        assert_eq!(expected, cov_mat);
    }
}