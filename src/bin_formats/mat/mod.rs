use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Div, Sub};
use std::sync::Arc;
use std::thread;

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use nalgebra::DMatrix;
use num::Zero;
use rayon::prelude::*;

pub use conversion::*;
pub use color_maps::*;

use crate::bin_formats::{FileDims, FileInner};
use crate::headers::Interleave;

mod conversion;
mod color_maps;
mod stat;


pub type MatType = Interleave;

pub trait FileIndex: {
    fn order(&self) -> MatType;
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize;
}

pub struct Mat<C, T, I> {
    pub inner: FileInner<C, T>,
    pub index: I,
}


impl<C1, C2, T, I1, I2> PartialEq<Mat<C2, T, I2>> for Mat<C1, T, I1>
    where
        I1: FileIndex,
        I2: FileIndex,
        T: Copy + PartialEq,
        C1: Deref<Target=[u8]>,
        C2: Deref<Target=[u8]>,
{
    fn eq(&self, other: &Mat<C2, T, I2>) -> bool {
        if self.inner.size() == other.inner.size() {
            let FileDims { bands, samples, lines } = self.inner.size();
            let bands = bands.len();

            let mut res = true;

            let (p1, p2) = unsafe {
                (
                    self.inner.get_unchecked(),
                    other.inner.get_unchecked()
                )
            };

            for l in 0..lines {
                for s in 0..samples {
                    for b in 0..bands {
                        let idx_1 = self.index.get_idx(l, s, b);
                        let idx_2 = other.index.get_idx(l, s, b);

                        unsafe {
                            let i1 = *p1.0.add(idx_1);
                            let i2 = *p2.0.add(idx_2);
                            res &= i1 == i2;
                        }
                    }
                }
            }

            res
        } else {
            false
        }
    }
}

impl<C1, I1> Mat<C1, f32, I1>
    where I1: 'static + FileIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    // , other: &mut Mat<C2, f32, Bsq>
    pub unsafe fn pca(&self) {
        let FileDims { bands, samples: _, lines: _ } = self.inner.size();

        let sty = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} [{eta_precise}] {msg}")
            .progress_chars("##-");

        let mp = Arc::new(MultiProgress::new());

        let stages_bar = mp.add(ProgressBar::new(5));
        stages_bar.set_style(sty.clone());
        stages_bar.enable_steady_tick(200);
        stages_bar.set_message("Stages");

        let mm2 = mp.clone();

        let j = thread::Builder::new()
            .name("progbar-manager".to_owned())
            .spawn(move || {
                mm2.join().unwrap();
            }).unwrap();

        let means: Vec<f32> = self.average_bulk(&sty, &mp);
        stages_bar.inc(1);

        let std_devs: Vec<f32> = self.std_dev_bulk(&sty, &mp, &means);
        stages_bar.inc(1);

        let mut covariances = self.covariances_bulk(&sty, &mp, &means, &std_devs);
        covariances.fill_upper_triangle_with_lower_triangle();
        let message = format!("{}", covariances);
        stages_bar.println(message);
        stages_bar.inc(1);

        stages_bar.println("Finding eigenvectors and eigenvalues...");
        let eigen = covariances.symmetric_eigen();
        let message = format!("{:#?}", eigen);
        stages_bar.println(message);

        stages_bar.inc(1);

        stages_bar.finish();

        j.join().unwrap();
    }


}