use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Div, Sub};

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::ProgressBar;
use num::Zero;

use crate::bin_formats::{FileDims, FileInner};
use crate::headers::Interleave;

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
    pub unsafe fn convert<C2, I2>(&self, out: &mut Mat<C2, f32, I2>)
        where
            I2: 'static + FileIndex + Sync + Send + Copy + Clone,
            C2: DerefMut<Target=[u8]> + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();
        let bar = ProgressBar::new((lines * samples * bands) as u64);

        let r_idx_gen = self.index;
        let w_idx_gen = out.index;

        let r_ptr = self.inner.get_unchecked();
        let w_ptr = out.inner.get_unchecked_mut();

        for b in 0..bands {
            for l in 0..lines {
                for s in 0..samples {
                    let read_idx = r_idx_gen.get_idx(l, s, b);
                    let write_idx = w_idx_gen.get_idx(l, s, b);

                    let r = r_ptr.0.add(read_idx);
                    let w = w_ptr.0.add(write_idx);

                    w.write_volatile(r.read_volatile());
                }
            }
            bar.inc((lines * samples) as u64)
        }
    }

    pub fn cool_warm(&self, out: &mut RgbImage, min: f32, max: f32, band: usize)
        where I1: 'static + FileIndex + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();
        let bar = ProgressBar::new((lines * samples) as u64);
        assert!(band < bands);

        let scale = max - min;

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);

                let val = unsafe {
                    normify(r_ptr.0.add(idx).read_volatile(), scale, min, max)
                };

                let r = (val * 255.0).floor() as u8;
                let b = ((1.0 - val) * 255.0).floor() as u8;

                out.put_pixel(s as u32, l as u32, Rgb([r, 0, b]))
            }
            bar.inc(samples as u64)
        }
    }

    pub fn gray(&self, out: &mut GrayImage, min: f32, max: f32, band: usize)
        where I1: 'static + FileIndex + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();
        let bar = ProgressBar::new((lines * samples) as u64);
        assert!(band < bands);

        let scale = max - min;

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);

                let val = unsafe {
                    normify(r_ptr.0.add(idx).read_volatile(), scale, min, max)
                };

                let r = (val * 255.0).floor() as u8;

                out.put_pixel(s as u32, l as u32, Luma([r]))
            }

            bar.inc(samples as u64)
        }
    }

    pub fn mask(&self, out: &mut GrayImage, min: f32)
        where I1: 'static + FileIndex + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();
        let bar = ProgressBar::new((lines * samples) as u64);

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        for l in 0..lines {
            for s in 0..samples {
                let mut sum = 0.0;
                for b in 0..bands {
                    let idx = self.index.get_idx(l, s, b);

                    let val = unsafe {
                        r_ptr.0.add(idx).read_volatile()
                    };

                    sum += val;
                }

                let r = num::clamp(sum - min, 0.0, 1.0).ceil() * 255.0;

                out.put_pixel(s as u32, l as u32, Luma([r as u8]))
            }
            bar.inc(samples as u64)
        }
    }

    pub fn rgb(&self, out: &mut RgbImage, mins: [f32; 3], maxes: [f32; 3], bands: [usize; 3])
        where I1: 'static + FileIndex + Sync + Send,
    {
        assert!(bands[0] < self.inner.dims.bands.len());
        assert!(bands[1] < self.inner.dims.bands.len());
        assert!(bands[2] < self.inner.dims.bands.len());

        let FileDims { samples, lines, .. } = self.inner.size();
        let bar = ProgressBar::new((lines * samples) as u64);

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        let scales = [
            maxes[0] - mins[0],
            maxes[1] - mins[1],
            maxes[2] - mins[2]
        ];

        for l in 0..lines {
            for s in 0..samples {
                let rgb = unsafe {
                    let indices = [
                        self.index.get_idx(l, s, bands[0]),
                        self.index.get_idx(l, s, bands[1]),
                        self.index.get_idx(l, s, bands[2]),
                    ];


                    let vals = [
                        r_ptr.0.add(indices[0]).read_volatile(),
                        r_ptr.0.add(indices[1]).read_volatile(),
                        r_ptr.0.add(indices[2]).read_volatile(),
                    ];

                    let norms = [
                        normify(vals[0], scales[0], mins[0], maxes[0]),
                        normify(vals[1], scales[1], mins[1], maxes[1]),
                        normify(vals[2], scales[2], mins[2], maxes[2])
                    ];

                    [
                        (norms[0] * 255.0).floor() as u8,
                        (norms[1] * 255.0).floor() as u8,
                        (norms[2] * 255.0).floor() as u8,
                    ]
                };

                out.put_pixel(s as u32, l as u32, Rgb(rgb))
            }
            bar.inc(samples as u64)
        }
    }
}

#[inline(always)]
fn normify<T>(val: T, scale: T, min: T, max: T) -> T
    where T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T> + Debug + Zero
{
    let clamped = num::clamp(val, min, max);
    let shifted = clamped - min;
    shifted / scale
}