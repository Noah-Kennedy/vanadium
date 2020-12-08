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
        let FileDims { bands, samples, lines } = self.inner.size();

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
                mm2.join_and_clear().unwrap();
            }).unwrap();

        let means: Vec<f32> = self.average_bulk(&sty, &mp);
        stages_bar.inc(1);

        let std_devs: Vec<f32> = self.std_dev_bulk(&sty, &mp, &means);
        stages_bar.inc(1);

        let covariances = self.covariances_bulk(&sty, &mp, &means, &std_devs);
        // let message = format!("{}", covariances);
        // stages_bar.println(message);
        stages_bar.inc(1);

        stages_bar.println("Finding eigenvectors and eigenvalues...");
        let eigen = covariances.clone().symmetric_eigen();
        // let message = format!("{:#?}", eigen);
        // stages_bar.println(message);
        stages_bar.inc(1);

        let r_ptr = self.inner.get_unchecked();
        let w_ptr = other.inner.get_unchecked_mut();

        let status_bar = mp.add(ProgressBar::new(kept_bands as u64));
        status_bar.set_style(sty.clone());
        status_bar.enable_steady_tick(200);
        status_bar.set_message("Writes");

        let r_ptr = r_ptr.clone();
        let w_ptr = w_ptr.clone();
        let status_bar_c = status_bar.clone();

        rayon::scope(move |s| {
            (0..kept_bands)
                .into_iter()
                .for_each(|b1| {
                    let eig = eigen.eigenvectors.clone();
                    let r_ptr = r_ptr.clone();
                    let w_ptr = w_ptr.clone();
                    let band_len = bands.len();
                    let means = means.clone();
                    let std_devs = std_devs.clone();
                    let status_bar = status_bar.clone();
                    let o_index = other.index.clone();
                    s.spawn(move |_| {
                        let col = eig.column(b1 as usize);

                        for l in 0..lines {
                            for s in 0..samples {
                                let read: Vec<f32> = (0..band_len)
                                    .map(|b2| self.index.get_idx(l, s, b2))
                                    .map(|read_idx| r_ptr.0.add(read_idx).read_volatile())
                                    .collect();

                                let w_val: f32 = read.iter().zip(col.iter())
                                    .enumerate()
                                    .map(|(b2, (d, s))| (((*d) * (*s)) - means[b2]) / std_devs[b2])
                                    .sum();

                                let w_idx = o_index.get_idx(l, s, b1 as usize);
                                w_ptr.0.add(w_idx).write_volatile(w_val);
                            }
                        }

                        status_bar.inc(1);
                    })
                });
        });

        status_bar_c.finish();

        stages_bar.inc(1);

        stages_bar.finish_and_clear();

        j.join().unwrap();
    }
}