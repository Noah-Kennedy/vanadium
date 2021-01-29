use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::sync::Arc;
use std::thread;

use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{ComplexField, RealField, SymmetricEigen, Dynamic};
use num::Bounded;
use num::traits::NumAssign;

use crate::bar::config_bar;
use crate::container::{IterableImage, IterableImageMut, LockImage, ReadImageGuard, WriteImageGuard};

pub trait PCA<T> where T: PartialEq + Copy + Debug + 'static {
    fn pca(&self, out: &Self, verbose: bool, min: Option<T>, max: Option<T>);
}

impl<'a, I, T> PCA<T> for LockImage<T, I>
    where I: IterableImage<'a, T> + Sync + IterableImageMut<'a, T>,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded
          + Display + ComplexField + ComplexField<RealField=T> + RealField + Sum
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn pca(&self, out: &Self, verbose: bool, min: Option<T>, max: Option<T>) {
        let min = min.unwrap_or_else(T::min_value);
        let max = max.unwrap_or_else(T::max_value);

        let a = &self.inner;
        let b = a.read();
        let c = b.unwrap();

        let input = ReadImageGuard { inner: c, _phantom: Default::default() };

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
        let means: Vec<_> = input.all_band_means(&mp, min, max);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Averages:");
            let message = format!("{:#?}", &means);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Standard Deviations");
        let std_devs: Vec<_> = input.all_band_std_devs(&mp, &means, min, max);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Standard Deviations:");
            let message = format!("{:#?}", &std_devs);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Covariances");
        let covariances = input.covariance_matrix(&mp, &means, min, max);
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

        let mut output = WriteImageGuard { inner: out.inner.write().unwrap(), _phantom: Default::default() };

        stages_bar.set_message("Stage: Writes");
        input.write_standardized_results(&mut output, &mp, &means, &std_devs, &eigen);
        stages_bar.inc(1);

        stages_bar.finish();

        j.join().unwrap();
    }
}

impl<'a, 'b, I, T> ReadImageGuard<'a, T, I>
    where I: IterableImage<'b, T> + Sync + IterableImageMut<'b, T>,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded
          + Display + ComplexField + ComplexField<RealField=T> + RealField + Sum
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn write_standardized_results(
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