use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub use color_maps::*;
pub use conversion::*;

use crate::bin_formats::{FileDims, FileInner};
use crate::bin_formats::bsq::Bsq;
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
    pub unsafe fn pca<C2>(&self, other: &mut Mat<C2, f32, Bsq>, kept_bands: u64)
        where C2: DerefMut<Target=[u8]> + Send + Sync
    {
        let sty = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} [{eta_precise}] {msg}")
            .progress_chars("##-");

        let mp = Arc::new(MultiProgress::new());

        let stages_bar = mp.add(ProgressBar::new(5));
        stages_bar.set_style(sty.clone());
        stages_bar.enable_steady_tick(200);

        let mm2 = mp.clone();

        let j = thread::Builder::new()
            .name("progbar-manager".to_owned())
            .spawn(move || {
                mm2.join_and_clear().unwrap();
            }).unwrap();

        stages_bar.set_message("Stage: Averages");
        let means: Vec<f32> = self.average_bulk(&sty, &mp);
        stages_bar.inc(1);

        stages_bar.set_message("Stage: Standard Deviations");
        let std_devs: Vec<f32> = self.std_dev_bulk(&sty, &mp, &means);
        stages_bar.inc(1);
        let message = format!("{:#?}", &std_devs);
        stages_bar.println(message);

        stages_bar.set_message("Stage: Covariances");
        let covariances = self.covariances_bulk(&sty, &mp, &means, &std_devs);
        let message = format!("{}", covariances);
        stages_bar.println(message);
        stages_bar.inc(1);

        stages_bar.set_message("Stage: Eigendecomposition");
        let eigen = covariances.symmetric_eigen();
        let message = format!("{:#?}", eigen);
        stages_bar.println(message);
        stages_bar.inc(1);

        stages_bar.set_message("Stage: Writes");
        self.pca_write(other, &sty, &mp, kept_bands, &means, &std_devs, &eigen);
        stages_bar.inc(1);

        stages_bar.finish();

        j.join().unwrap();
    }
}