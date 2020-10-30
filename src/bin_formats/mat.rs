use std::fmt::Debug;
use std::ops::{Div, Sub};

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::ProgressBar;
use num::Zero;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MatType {
    Bip,
    Bil,
    Bsq,
}

pub trait FileIndex<T> {
    fn size(&self) -> (usize, usize, usize);
    fn order(&self) -> MatType;
    unsafe fn get_unchecked(&self, line: usize, pixel: usize, band: usize) -> &T;
}

pub trait FileIndexMut<T>: FileIndex<T> {
    unsafe fn get_mut_unchecked(&mut self, line: usize, pixel: usize, band: usize) -> &mut T;
}

pub struct Mat<F> {
    pub(crate) inner: F
}

impl<F> From<F> for Mat<F> {
    fn from(inner: F) -> Self {
        Self { inner }
    }
}

impl<I> Mat<I> {
    pub fn convert<T, O>(&self, out: &mut Mat<O>)
        where
            I: 'static + FileIndex<T> + Sync + Send,
            O: 'static + FileIndexMut<T> + Sync + Send,
            T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T> + Debug

    {
        let (lines, pixels, bands) = self.inner.size();
        let bar = ProgressBar::new((lines * pixels * bands) as u64);

        for l in 0..lines {
            for b in 0..bands {
                for p in 0..pixels {
                    unsafe {
                        *out.inner.get_mut_unchecked(l, p, b) = *self.inner.get_unchecked(l, p, b);
                    }
                }
            }
            bar.inc((bands * pixels) as u64)
        }
    }

    pub fn cool_warm(&self, out: &mut RgbImage, min: f32, max: f32, band: usize)
        where I: 'static + FileIndex<f32> + Sync + Send,
    {
        let (lines, samples, bands) = self.inner.size();
        let bar = ProgressBar::new((lines * samples) as u64);
        assert!(band < bands);

        let scale = max - min;

        for l in 0..lines {
            for s in 0..samples {
                let val = unsafe {
                    normify(*self.inner.get_unchecked(l, s, band), scale, min, max)
                };

                let r = (val * 255.0).floor() as u8;
                let b = ((1.0 - val) * 255.0).floor() as u8;

                out.put_pixel(s as u32, l as u32, Rgb([r, 0, b]))
            }
            bar.inc(samples as u64)
        }
    }

    pub fn gray(&self, out: &mut GrayImage, min: f32, max: f32, band: usize)
        where I: 'static + FileIndex<f32> + Sync + Send,
    {
        let (lines, samples, bands) = self.inner.size();
        let bar = ProgressBar::new((lines * samples) as u64);
        assert!(band < bands);

        let scale = max - min;

        for l in 0..lines {
            for s in 0..samples {
                let val = unsafe {
                    normify(*self.inner.get_unchecked(l, s, band), scale, min, max)
                };

                let r = (val * 255.0).floor() as u8;

                out.put_pixel(s as u32, l as u32, Luma([r]))
            }
            bar.inc(samples as u64)
        }
    }

    pub fn rgb(&self, out: &mut RgbImage, mins: [f32; 3], maxes: [f32; 3], bands: [usize; 3])
        where I: 'static + FileIndex<f32> + Sync + Send,
    {
        let (lines, samples, _) = self.inner.size();
        let bar = ProgressBar::new((lines * samples) as u64);

        let scales = [
            maxes[0] - mins[0],
            maxes[1] - mins[1],
            maxes[2] - mins[2]
        ];

        for l in 0..lines {
            for s in 0..samples {
                let rgb = unsafe {
                    let vals = [
                        *self.inner.get_unchecked(l, s, bands[0]),
                        *self.inner.get_unchecked(l, s, bands[1]),
                        *self.inner.get_unchecked(l, s, bands[2]),
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
    let norm = shifted / scale;
    norm
}