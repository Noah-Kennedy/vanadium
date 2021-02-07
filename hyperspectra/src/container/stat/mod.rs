use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::mem;
use std::ops::{Div, Sub};
use std::sync::atomic::{AtomicUsize, Ordering};

use either::Either;
use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{ComplexField, DMatrix, RealField};
use num::{Bounded, Zero};
use num::traits::NumAssign;
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::bar::config_bar;
use crate::container::{MAX_CHUNK_SIZE, ImageDims, IterableImage, IterableImageMut, ReadImageGuard, chunk_size};

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

        (sum / (count - T::one())).sqrt()
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
    pub fn big_bip_means(&self, mp: &MultiProgress, min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();

        let num_samples = lines * samples;
        let chunk_size = (
            channels
                * (MAX_CHUNK_SIZE / (mem::size_of::<T>() * channels)).max(1))
            .min(channels * samples * lines) / channels;

        let sums = Mutex::new(vec![T::zero(); channels]);
        let mut counts = Vec::with_capacity(channels);

        for _ in 0..channels {
            counts.push(AtomicUsize::new(0));
        }

        let status_bar = mp.add(ProgressBar::new(num_samples as u64));
        config_bar(&status_bar, "Calculating means...");

        status_bar.println(format!("{}:{}", chunk_size, num_samples / chunk_size));

        rayon::scope(|scope| {
            for c in self.inner.samples_chunked() {
                scope.spawn(|_| {
                    let mut local_sums = vec![T::zero(); channels];

                    for s in c {
                        let inner = s.clone();
                        for (i, b) in inner.enumerate() {
                            if *b > min && *b <= max {
                                local_sums[i] += *b;
                                counts[i].fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    let mut guard = sums.lock();

                    for (input, output) in local_sums.iter().zip(guard.iter_mut()) {
                        *output += *input;
                    }

                    drop(local_sums);
                    drop(guard);

                    status_bar.inc(chunk_size as u64);
                });
            }
        });

        status_bar.finish();

        let mut sums = sums.into_inner();

        sums.iter_mut().zip(counts.iter()).for_each(|(s, c_atom)| {
            let c = c_atom.load(Ordering::Relaxed);
            if c > 1 {
                *s /= T::from_usize(c).unwrap();
            }
        });

        sums
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn small_bip_means(&self, mp: &MultiProgress, min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();

        let num_samples = lines * samples;
        let chunk_size = (
            channels
                * (MAX_CHUNK_SIZE / (mem::size_of::<T>() * channels)).max(1))
            .min(channels * samples * lines) / channels;

        let mut sums = vec![T::zero(); channels];
        let mut counts = vec![0; channels];

        let status_bar = mp.add(ProgressBar::new(num_samples as u64));
        config_bar(&status_bar, "Calculating means...");

        for c in self.inner.samples_chunked() {
            for s in c {
                let inner = s.clone();
                for (i, b) in inner.enumerate() {
                    if *b > min && *b <= max {
                        sums[i] += *b;
                        counts[i] += 1;
                    }
                }
            }

            status_bar.inc(chunk_size as u64)
        }

        status_bar.finish();

        sums.iter_mut().zip(counts.iter()).for_each(|(s, c)| {
            if *c > 1 {
                *s /= T::from_usize(*c).unwrap();
            }
        });

        sums
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn small_bip_std_devs(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();

        let num_samples = lines * samples;

        let chunk_size = chunk_size::<T>(&self.inner.dims());

        let mut sums = vec![T::zero(); channels];
        let mut counts = vec![0; channels];

        let status_bar = mp.add(ProgressBar::new(num_samples as u64));
        config_bar(&status_bar, "Calculating means...");

        for c in self.inner.samples_chunked() {
            for s in c {
                let inner = s.clone();
                for (i, b) in inner.enumerate() {
                    if *b > min && *b <= max {
                        let diff = *b - means[i];

                        sums[i] += diff * diff;
                        counts[i] += 1;
                    }
                }
            }

            status_bar.inc(chunk_size as u64)
        }

        status_bar.finish();

        sums.iter_mut().zip(counts.iter()).for_each(|(s, c)| {
            if *c > 2 {
                *s = (*s / T::from_usize(*c - 1).unwrap()).sqrt();
            }
        });

        sums
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn big_bip_std_devs(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();

        let num_samples = lines * samples;
        let chunk_size = chunk_size::<T>(&self.inner.dims());


        let sums = Mutex::new(vec![T::zero(); channels]);
        let mut counts = Vec::with_capacity(channels);

        for _ in 0..channels {
            counts.push(AtomicUsize::new(0));
        }

        let status_bar = mp.add(ProgressBar::new(num_samples as u64));
        config_bar(&status_bar, "Calculating means...");

        rayon::scope(|scope| {
            for c in self.inner.samples_chunked() {
                scope.spawn(|_| {
                    let mut local_sums = vec![T::zero(); channels];

                    for s in c {
                        let inner = s.clone();
                        for (i, b) in inner.enumerate() {
                            if *b > min && *b <= max {
                                let diff = *b - means[i];

                                local_sums[i] += diff * diff;
                                counts[i].fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    let mut guard = sums.lock();

                    for (input, output) in local_sums.iter().zip(guard.iter_mut()) {
                        *output += *input;
                    }

                    drop(local_sums);

                    status_bar.inc(chunk_size as u64)
                });
            }
        });

        status_bar.finish();

        let mut sums = sums.into_inner();

        sums.iter_mut().zip(counts.iter()).for_each(|(s, c)| {
            let c = c.load(Ordering::Relaxed);
            if c > 1 {
                *s = (*s / T::from_usize(c - 1).unwrap()).sqrt();
            }
        });

        sums
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn small_bip_cov_mat(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> DMatrix<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();

        let num_samples = lines * samples;
        let chunk_size = chunk_size::<T>(&self.inner.dims());


        let mut sums = vec![T::zero(); channels * channels];
        let mut counts = vec![0; channels * channels];

        let status_bar = mp.add(ProgressBar::new(num_samples as u64));
        config_bar(&status_bar, "Calculating covariances...");

        for c in self.inner.samples_chunked() {
            for s in c {
                let outer = s.clone();

                for (outer_i, outer_b) in outer.enumerate() {
                    let inner = s.clone();

                    for (inner_i, inner_b) in inner
                        .enumerate()
                        .take(outer_i + 1)
                    {
                        if *outer_b > min && *outer_b <= max && *inner_b > min && *inner_b <= max {
                            let diffs = [
                                *outer_b - means[outer_i],
                                *inner_b - means[inner_i]
                            ];

                            let idx = (outer_i * channels) + inner_i;

                            sums[idx] += diffs[0] * diffs[1];
                            counts[idx] += 1;
                        }
                    }
                }
            }

            status_bar.inc(chunk_size as u64)
        }

        sums.iter_mut().zip(counts.iter()).for_each(|(s, c)| {
            if *c > 1 {
                *s /= T::from_usize(*c - 1).unwrap();
            }
        });

        status_bar.finish();

        let mut out = DMatrix::from_row_slice(channels, channels, &sums);
        out.fill_upper_triangle_with_lower_triangle();

        out
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn big_bip_cov_mat(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> DMatrix<T> {
        let ImageDims { channels, lines, samples } = self.inner.dims();

        let num_samples = lines * samples;
        let chunk_size = chunk_size::<T>(&self.inner.dims());


        let sums = Mutex::new(vec![T::zero(); channels * channels]);
        let mut counts = Vec::with_capacity(channels * channels);

        for _ in 0..(channels * channels) {
            counts.push(AtomicUsize::new(0));
        }

        let status_bar = mp.add(ProgressBar::new(num_samples as u64));
        config_bar(&status_bar, "Calculating covariances...");

        rayon::scope(|scope| {
            for c in self.inner.samples_chunked() {
                scope.spawn(|_| {
                    let mut local_sum = vec![T::zero(); channels * channels];
                    for s in c {
                        let outer = s.clone();

                        for (outer_i, outer_b) in outer.enumerate() {
                            let inner = s.clone();

                            for (inner_i, inner_b) in inner
                                .enumerate()
                                .take(outer_i + 1)
                            {
                                if *outer_b > min && *outer_b <= max && *inner_b > min && *inner_b <= max {
                                    let diffs = [
                                        *outer_b - means[outer_i],
                                        *inner_b - means[inner_i]
                                    ];

                                    let idx = (outer_i * channels) + inner_i;

                                    local_sum[idx] += diffs[0] * diffs[1];
                                    counts[idx].fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }

                    let mut guard = sums.lock();

                    for (input, output) in local_sum.iter().zip(guard.iter_mut()) {
                        *output += *input;
                    }

                    drop(local_sum);
                    drop(guard)
                });

                status_bar.inc(chunk_size as u64)
            }
        });

        let mut sums = sums.into_inner();

        sums.iter_mut().zip(counts.iter()).for_each(|(s, c)| {
            let c = c.load(Ordering::Relaxed);
            if c > 1 {
                *s /= T::from_usize(c - 1).unwrap();
            }
        });

        status_bar.finish();

        let mut out = DMatrix::from_row_slice(channels, channels, &sums);
        out.fill_upper_triangle_with_lower_triangle();

        out
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn all_band_means(&self, mp: &MultiProgress, min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        if let Either::Right(_) = self.inner.fastest() {
            // todo investigate memory leaks
            // self.small_bip_means(mp, min, max)

            if channels < 64 {
                self.small_bip_means(mp, min, max)
            } else {
                self.big_bip_means(mp, min, max)
            }
        } else {
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
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn all_band_std_devs(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        if self.inner.fastest().is_right() {
            // todo investigate memory leaks
            // self.small_bip_std_devs(mp, means, min, max)

            if channels < 64 {
                self.small_bip_std_devs(mp, means, min, max)
            } else {
                self.big_bip_std_devs(mp, means, min, max)
            }
        } else {
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
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn covariance_matrix(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> DMatrix<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();
        if let Either::Right(_) = self.inner.fastest() {
            // todo investigate memory leaks
            // self.small_bip_cov_mat(mp, means, min, max)

            if channels < 64 {
                self.small_bip_cov_mat(mp, means, min, max)
            } else {
                self.big_bip_cov_mat(mp, means, min, max)
            }
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
mod tests;