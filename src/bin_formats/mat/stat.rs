use std::ops::Deref;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use nalgebra::DMatrix;
use rayon::prelude::*;

use crate::bin_formats::{FileDims, FileIndex, Mat};

impl<C1, I1> Mat<C1, f32, I1>
    where I1: 'static + FileIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    pub unsafe fn mean(&self, band: usize) -> f32 {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mut sum = 0.0;
        let count = lines * samples;

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                sum += x;
            }
        }

        sum / count as f32
    }

    pub unsafe fn std_dev(&self, band: usize, mean: Option<f32>) -> f32 {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mean = if let Some(mean) = mean {
            mean
        } else {
            self.mean(band)
        };


        let mut sum = 0.0;
        let count = lines * samples;

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                let dif = x - mean;

                sum += dif * dif;
            }
        }

        sum /= count as f32;

        sum.sqrt()
    }

    pub unsafe fn covariance(
        &self, bands: [usize; 2], means: [f32; 2], std_devs: [f32; 2],
    ) -> f32
    {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mut sum = 0.0;
        let count = lines * samples;

        for l in 0..lines {
            for s in 0..samples {
                let indices = [
                    self.index.get_idx(l, s, bands[0]),
                    self.index.get_idx(l, s, bands[1])
                ];

                let xs = [
                    (r_ptr.0.add(indices[0]).read_volatile() - means[0]) / std_devs[0],
                    (r_ptr.0.add(indices[1]).read_volatile() - means[1]) / std_devs[1]
                ];

                sum += xs[0] * xs[1];
            }
        }

        sum /= count as f32;

        sum.sqrt()
    }

    pub unsafe fn average_bulk(&self, sty: &ProgressStyle, mp: &MultiProgress) -> Vec<f32> {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let status_bar = mp.add(ProgressBar::new(bands.len() as u64));
        status_bar.set_style(sty.clone());
        status_bar.enable_steady_tick(200);
        status_bar.set_message("Averages");

        let means = (0..bands.len())
            .into_par_iter()
            .map(|b| {
                let out = self.mean(b);
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        means
    }

    pub unsafe fn std_dev_bulk(&self, sty: &ProgressStyle, mp: &MultiProgress, means: &[f32]) -> Vec<f32> {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let status_bar = mp.add(ProgressBar::new(bands.len() as u64));
        status_bar.set_style(sty.clone());
        status_bar.enable_steady_tick(200);
        status_bar.set_message("Std. Devs");

        let devs = (0..bands.len())
            .into_par_iter()
            .zip(means.par_iter())
            .map(|(b, m)| {
                let out = self.std_dev(b, Some(*m));
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        devs
    }

    pub unsafe fn covariances_bulk(
        &self, sty: &ProgressStyle, mp: &MultiProgress, means: &[f32], std_devs: &[f32],
    ) -> DMatrix<f32>
    {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let mut tot_val = 0;

        for i in 0..bands.len() {
            tot_val += i + 1;
        }

        let status_bar = mp.add(ProgressBar::new(tot_val as u64));
        status_bar.set_style(sty.clone());
        status_bar.enable_steady_tick(200);
        status_bar.set_message("Covariances");

        let covariances: Vec<f32> = (0..bands.len())
            .into_par_iter()
            .map(|b1| {
                let mut v: Vec<f32> = (0..=b1)
                    .map(|b2| {
                        let out = self.covariance(
                            [b1, b2],
                            [means[b1], means[b2]],
                            [std_devs[b1], std_devs[b2]],
                        );
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

        status_bar.println(format!("{}", covariances.len()));
        status_bar.finish();

        let mut out = DMatrix::from_row_slice(bands.len(), bands.len(), &covariances);
        out.fill_upper_triangle_with_lower_triangle();

        out
    }
}