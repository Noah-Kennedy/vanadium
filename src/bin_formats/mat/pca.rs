use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread;

use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{Dynamic, SymmetricEigen};
use num::Float;

use crate::bin_formats::{FileDims, ImageIndex, SpectralImage};
use crate::bin_formats::bsq::Bsq;
use crate::util::config_bar;

impl<C1, I1> SpectralImage<C1, f32, I1>
    where I1: 'static + ImageIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    pub fn pca<C2>(&self, other: &mut SpectralImage<C2, f32, Bsq>, kept_bands: u64, verbose: bool)
        where C2: DerefMut<Target=[u8]> + Send + Sync
    {
        let mp = Arc::new(MultiProgress::new());

        let stages_bar = mp.add(ProgressBar::new(5));
        config_bar(&stages_bar, "Performing PCA stages...");

        let mm2 = mp.clone();

        let j = thread::Builder::new()
            .name("progbar-manager".to_owned())
            .spawn(move || {
                mm2.join_and_clear().unwrap();
            }).unwrap();

        stages_bar.set_message("Stage: Averages");
        let means: Vec<_> = self.all_band_averages(&mp);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Averages:");
            let message = format!("{:#?}", &means);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Standard Deviations");
        let std_devs: Vec<_> = self.all_band_standard_deviations(&mp, &means);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Standard Deviations:");
            let message = format!("{:#?}", &std_devs);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Covariances");
        let covariances = self.calculate_covariance_matrix(&mp, &means);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Covariances:");
            let message = format!("{}", covariances);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Eigendecomposition");
        let eigen = covariances.symmetric_eigen();
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Eigen:");
            let message = format!("{:#?}", eigen);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Writes");
        self.write_standardized_results(other, &mp, kept_bands, &means, &std_devs, &eigen);
        stages_bar.inc(1);

        stages_bar.finish();

        j.join().unwrap();
    }

    pub fn write_standardized_results<C2>(
        &self, output: &mut SpectralImage<C2, f32, Bsq>,
        mp: &MultiProgress,
        kept_bands: u64,
        means: &[f64], std_devs: &[f64],
        eigen: &SymmetricEigen<f64, Dynamic>,
    )
        where C2: DerefMut<Target=[u8]> + Send + Sync
    {
        let FileDims { bands, samples, lines } = self.inner.size();

        let r_ptr = unsafe { self.inner.get_unchecked() };
        let w_ptr = unsafe { output.inner.get_unchecked_mut() };

        let status_bar = mp.add(ProgressBar::new(kept_bands));
        config_bar(&status_bar, "Writing standardized output bands...");
        let sc = status_bar.clone();

        (0..kept_bands)
            .into_iter()
            .for_each(|b1| {
                let r_ptr = r_ptr;
                let w_ptr = w_ptr;
                let band_len = bands.len();
                let o_index = output.index;
                let status_bar = status_bar.clone();

                rayon::scope(move |s| {
                    for l in 0..lines {
                        let eig = eigen.eigenvectors.clone();

                        s.spawn(move |_| {
                            let col = eig.column(b1 as usize);
                            for s in 0..samples {
                                let read: Vec<_> = (0..band_len)
                                    .map(|b2| (b2, self.index.get_idx(l, s, b2)))
                                    .map(|(b2, read_idx)| {
                                        let val = unsafe {
                                            r_ptr.0.add(read_idx).read_volatile()
                                        } as f64;

                                        let z_val = (val - means[b2]) / std_devs[b2];
                                        let z_off = (0.0 - means[b2]) / std_devs[b2];

                                        if (z_val - z_off).abs() < f64::EPSILON {
                                            f64::neg_infinity()
                                        } else {
                                            z_val
                                        }
                                    })
                                    .collect();

                                let w_val: f64 = read.into_iter().zip(col.into_iter())
                                    .map(|(d, s)| d * s)
                                    .sum();

                                let w_idx = o_index.get_idx(l, s, b1 as usize);

                                unsafe {
                                    w_ptr.0.add(w_idx).write_volatile(w_val as f32);
                                }
                            }
                        });
                    }

                    status_bar.inc(1);
                });
            });

        sc.finish();
    }
}