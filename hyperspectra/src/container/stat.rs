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
            let include = x > min && x <= max;

            if include {
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
        let mut count = T::zero();

        let itc = self.inner.band(bands[0])
            .zip(self.inner.band(bands[1]));

        for (&x0, &x1) in itc {
            if x0 > min && x0 <= max && x1 > min && x1 <= max {
                let diffs = [
                    x0 - means[0],
                    x1 - means[1]
                ];

                sum += diffs[0] * diffs[1];
                count += T::one();
            }
        }

        (sum / count).sqrt()
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
        config_bar(&status_bar, "Calculating band means...");

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
        if let Either::Right(_) = self.inner.fastest() {
            let num_samples = lines * samples;

            let status_bar = mp.add(ProgressBar::new(num_samples as u64));
            config_bar(&status_bar, "Samples...");

            let (mut sums, counts) = self.inner.samples_chunked()
                .map(|chunk| {
                    let mut sums = vec![T::zero(); channels * channels];
                    let mut counts = vec![0usize; channels * channels];

                    for s in chunk {
                        let outer = s.clone();

                        for (outer_i, outer_b) in outer.enumerate() {
                            let inner = s.clone();

                            for (inner_i, inner_b) in inner.take(channels - outer_i).enumerate() {
                                if *outer_b > min && *outer_b <= max
                                    && *inner_b > min && *inner_b <= max
                                {
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

                    status_bar.inc(CHUNK_SIZE as u64);

                    (sums, counts)
                })
                .fold((vec![T::zero(); channels * channels], vec![0usize; channels * channels]),
                      |(mut sa, mut ca), (mut sb, mut cb)| {
                          sa.append(&mut sb);
                          ca.append(&mut cb);

                          (sa, ca)
                      },
                );

            sums.iter_mut().zip(counts).for_each(|(s, c)| {
                *s = (*s / T::from_usize(c).unwrap()).sqrt()
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