use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::ops::{Div, Sub};

use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{ComplexField, DMatrix, Dynamic, RealField, SymmetricEigen};
use num::{Bounded, Zero};
use num::traits::NumAssign;
use rayon::prelude::*;

use crate::bar::config_bar;
use crate::container::{ImageDims, IterableImage, IterableImageMut, ReadImageGuard, WriteImageGuard};

impl<'a, 'b, I, T> ReadImageGuard<'a, T, I>
    where I: IterableImage<'b, T> + Sync + IterableImageMut<'b, T>,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded
          + Display + ComplexField + ComplexField<RealField=T> + RealField + Sum
{
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

    pub fn covariance_matrix(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> DMatrix<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

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

    pub fn write_standardized_results(
        &self,
        output: &mut WriteImageGuard<T, I>,
        mp: &MultiProgress,
        means: &[T], std_devs: &[T],
        eigen: &SymmetricEigen<T, Dynamic>,
    )
    {
        let status_bar = mp.add(ProgressBar::new(
            self.inner.dims().samples as u64
                * self.inner.dims().lines as u64));
        config_bar(&status_bar, "Writing standardized output bands...");
        let sc = status_bar;

        let itc = self.inner.samples().zip(output.inner.samples_mut());

        // rayon::scope(|s| {
        for (read, write) in itc {
            let eig = eigen.eigenvectors.clone();

            // s.spawn(move |_| {
            for (i, b) in write.enumerate() {
                let col = eig.column(i);

                let col_write: T = read
                    .clone()
                    .zip(means)
                    .zip(std_devs)
                    .map(|((r, m), s)| {
                        let z_val: T = (*r - *m) / *s;
                        let z_off: T = (-*m) / *s;

                        if (z_val - z_off).abs() < T::zero() {
                            T::min_value()
                        } else {
                            z_val
                        }
                    })
                    .zip(col.into_iter())
                    .map(|(d, s)| d * (*s))
                    .sum();

                *b = col_write;
            }
            // });

            // sc.inc(1);
        }
        // });

        sc.finish();
    }
}

#[inline(always)]
pub fn normify<T>(val: T, scale: T, min: T, max: T) -> T
    where T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T> + Debug + Zero
{
    let clamped = num::clamp(val, min, max);
    let shifted = clamped - min;
    shifted / scale
}