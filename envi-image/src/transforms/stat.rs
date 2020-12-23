use std::ops::Deref;

use indicatif::{MultiProgress, ProgressBar};
use nalgebra::DMatrix;
use rayon::prelude::*;

use crate::{FileDims, ImageIndex, SpectralImage};
use crate::util::config_bar;

impl<C1, I1> SpectralImage<C1, f32, I1>
    where I1: 'static + ImageIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    pub(crate) unsafe fn band_mean(&self, band: usize, min: f32, max: f32) -> f64 {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mut sum = 0.0;
        let mut count = 0;

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                let include = x > min && x <= max;

                sum += x as f64 * include as usize as f64;

                count += include as usize;
            }
        }

        sum / count as f64
    }

    pub(crate) unsafe fn band_std_dev(&self, band: usize, mean: Option<f64>, min: f32, max: f32) -> f64 {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mean = if let Some(mean) = mean {
            mean
        } else {
            self.band_mean(band, min, max)
        };


        let mut sum = 0.0;
        let mut count = 0;

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                let dif = x as f64 - mean;

                let include = x > min && x <= max;

                sum += dif * dif * include as u8 as f64;

                count += include as usize;
            }
        }

        sum /= count as f64;

        sum.sqrt()
    }

    pub(crate) unsafe fn covariance_pair(
        &self, bands: [usize; 2], means: [f64; 2], min: f32, max: f32,
    )
        -> f64
    {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mut sum = 0.0;
        let mut count = 0;

        for l in 0..lines {
            for s in 0..samples {
                let indices = [
                    self.index.get_idx(l, s, bands[0]),
                    self.index.get_idx(l, s, bands[1])
                ];

                let r_vals = [
                    r_ptr.0.add(indices[0]).read_volatile(),
                    r_ptr.0.add(indices[1]).read_volatile(),
                ];

                let include = r_vals[0] > min && r_vals[1] > min
                    && r_vals[0] <= max && r_vals[1] <= max;

                let xs = [
                    (r_vals[0] as f64 - means[0]),
                    (r_vals[1] as f64 - means[1])
                ];

                sum += xs[0] * xs[1] * include as u8 as f64;
                count += include as usize;
            }
        }

        sum /= count as f64;

        sum.sqrt()
    }

    pub(crate) fn all_band_averages(&self, mp: &MultiProgress, min: f32, max: f32) -> Vec<f64> {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let status_bar = mp.add(ProgressBar::new(bands.len() as u64));
        config_bar(&status_bar, "Calculating band means...");

        let means = (0..bands.len())
            .into_par_iter()
            .map(|b| {
                let out = unsafe { self.band_mean(b, min, max) };
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        means
    }

    pub(crate) fn all_band_standard_deviations(
        &self, mp: &MultiProgress, means: &[f64], min: f32, max: f32,
    )
        -> Vec<f64>
    {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let status_bar = mp.add(ProgressBar::new(bands.len() as u64));
        config_bar(&status_bar, "Calculating band standard deviations...");

        let devs = (0..bands.len())
            .into_par_iter()
            .zip(means.par_iter())
            .map(|(b, m)| {
                let out = unsafe { self.band_std_dev(b, Some(*m), min, max) };
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        devs
    }

    pub(crate) fn calculate_covariance_matrix(
        &self, mp: &MultiProgress, means: &[f64], min: f32, max: f32,
    ) -> DMatrix<f64>
    {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let mut tot_val = 0;

        // todo derive the equation for this
        for i in 0..bands.len() {
            tot_val += i + 1;
        }

        let status_bar = mp.add(ProgressBar::new(tot_val as u64));
        config_bar(&status_bar, "Calculating band covariances...");

        let covariances: Vec<f64> = (0..bands.len())
            .into_par_iter()
            .map(|b1| {
                let mut v: Vec<f64> = (0..=b1)
                    .map(|b2| {
                        let out = unsafe {
                            self.covariance_pair(
                                [b1, b2],
                                [means[b1], means[b2]],
                                min, max,
                            )
                        };
                        status_bar.inc(1);
                        out
                    })
                    .collect();

                v.reserve(bands.len() - v.len());

                while v.len() < bands.len() {
                    v.push(0.0)
                }

                v
            })
            .flatten()
            .collect();

        status_bar.finish();

        let mut out = DMatrix::from_row_slice(bands.len(), bands.len(), &covariances);
        out.fill_upper_triangle_with_lower_triangle();

        out
    }
}