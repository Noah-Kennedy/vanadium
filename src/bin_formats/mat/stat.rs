use std::ops::{Deref, DerefMut};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use nalgebra::{DMatrix, Dynamic, SymmetricEigen};
use rayon::prelude::*;

use crate::bin_formats::{FileDims, FileIndex, Mat};
use crate::bin_formats::bsq::Bsq;

impl<C1, I1> Mat<C1, f32, I1>
    where I1: 'static + FileIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    pub unsafe fn mean(&self, band: usize) -> f32 {
        let FileDims { bands: _, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mut sum = 0.0;
        let mut count = 0;

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                sum += x;

                count += (x != 0.0) as usize;
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
        let mut count = 0;

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                if x > 0.0 {
                    let dif = x - mean;

                    sum += dif * dif;

                    count += (x != 0.0) as usize;
                }
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

                let xs = [
                    (r_vals[0] - means[0]) / std_devs[0],
                    (r_vals[1] - means[1]) / std_devs[1]
                ];

                sum += xs[0] * xs[1] * (r_vals[0] != 0.0 && r_vals[1] != 0.0) as u8 as f32;
                count += (r_vals[0] != 0.0 && r_vals[1] != 0.0) as usize;
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
        status_bar.set_message("Band Means");

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
        status_bar.set_message("Band Standard Deviations");

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
        status_bar.set_message("Band Covariances");

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

    pub fn pca_write<C2>(
        &self, other: &mut Mat<C2, f32, Bsq>,
        sty: &ProgressStyle, mp: &MultiProgress,
        kept_bands: u64,
        means: &[f32], std_devs: &[f32],
        eigen: &SymmetricEigen<f32, Dynamic>,
    )
        where C2: DerefMut<Target=[u8]> + Send + Sync
    {
        let FileDims { bands, samples, lines } = self.inner.size();

        let r_ptr = unsafe { self.inner.get_unchecked() };
        let w_ptr = unsafe { other.inner.get_unchecked_mut() };

        let status_bar = mp.add(ProgressBar::new(kept_bands));
        status_bar.set_style(sty.clone());
        status_bar.enable_steady_tick(200);
        status_bar.set_message("Band Writes");

        rayon::scope(move |s| {
            (0..kept_bands)
                .into_iter()
                .for_each(|b1| {
                    let r_ptr = r_ptr.clone();
                    let w_ptr = w_ptr.clone();
                    let band_len = bands.len();
                    let o_index = other.index.clone();

                    for l in 0..lines {
                        let eig = eigen.eigenvectors.clone();
                        let means = means.clone();
                        let std_devs = std_devs.clone();

                        s.spawn(move |_| {
                            let col = eig.column(b1 as usize);
                            for s in 0..samples {
                                let read: Vec<f32> = (0..band_len)
                                    .map(|b2| self.index.get_idx(l, s, b2))
                                    .map(|read_idx| unsafe {
                                        r_ptr.0.add(read_idx).read_volatile()
                                    })
                                    .collect();

                                let w_val: f32 = read.into_iter().zip(col.iter())
                                    .enumerate()
                                    .map(|(b2, (d, s))| ((d * s) - means[b2]) / std_devs[b2])
                                    .sum();

                                let w_idx = o_index.get_idx(l, s, b1 as usize);

                                unsafe {
                                    w_ptr.0.add(w_idx).write_volatile(w_val);
                                }
                            }
                        });
                    }

                    status_bar.inc(1);
                    status_bar.println("Updated!");
                });

            status_bar.finish();
        });
    }
}