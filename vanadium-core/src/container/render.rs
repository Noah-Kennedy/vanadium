use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Div, Sub};

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::ProgressBar;
use nalgebra::RealField;
use num::{Bounded, FromPrimitive, ToPrimitive};
use num::traits::NumAssign;

use crate::bar::config_bar;
use crate::container::{ImageIndex, IndexImage, IterableImage, LockImage, normify};

pub trait Render<T> where T: PartialEq + Copy + Debug + 'static {
    fn solid(&self, out: &mut RgbImage, min: T, max: T, band: usize, flag: ColorFlag);
    fn gray(&self, out: &mut GrayImage, min: T, max: T, band: usize);
    fn mask(&self, out: &mut GrayImage, min: T);
    fn rgb(
        &self,
        out: &mut RgbImage,
        minimums: &[T],
        maximums: &[T],
        channels: &[usize],
    );
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum ColorFlag {
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
    Teal,
}

impl<'a, I, T> Render<T> for LockImage<T, I>
    where I: IterableImage<'a, T> + Sync + IndexImage<T> + 'static,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded + Sum + Div
          + Sub + FromPrimitive + ToPrimitive + RealField
{
    fn solid(&self, out: &mut RgbImage, min: T, max: T, band: usize, flag: ColorFlag) {
        let guard = self.read();

        let dims = guard.inner.dims();

        assert!(band < dims.channels);

        let bar = ProgressBar::new((dims.lines * dims.samples) as u64);
        config_bar(&bar, "Mapping Pixels");

        let scale = max - min;

        for l in 0..dims.lines {
            for s in 0..dims.samples {
                let index = ImageIndex {
                    channel: band,
                    line: l,
                    sample: s,
                };
                let val = unsafe {
                    normify(*guard.inner.get_unchecked(&index), scale, min, max)
                };

                let pri = (val.sqrt() * T::from_u8(255).unwrap()).to_u8().unwrap();
                let alt = (val * T::from_u8(255).unwrap()).to_u8().unwrap();

                let pixel = match flag {
                    ColorFlag::Red => [pri, alt, alt],
                    ColorFlag::Green => [alt, pri, alt],
                    ColorFlag::Blue => [alt, alt, pri],
                    ColorFlag::Purple => [pri, alt, pri],
                    ColorFlag::Yellow => [pri, pri, alt],
                    ColorFlag::Teal => [alt, pri, pri],
                };

                out.put_pixel(s as u32, l as u32, Rgb(pixel));
            }

            bar.inc(dims.samples as u64)
        }
    }

    fn gray(&self, out: &mut GrayImage, min: T, max: T, band: usize) {
        let guard = self.read();

        let dims = guard.inner.dims();

        assert!(band < dims.channels);

        let bar = ProgressBar::new((dims.lines * dims.samples) as u64);
        config_bar(&bar, "Mapping Pixels");

        let scale = max - min;

        for l in 0..dims.lines {
            for s in 0..dims.samples {
                let index = ImageIndex {
                    channel: band,
                    line: l,
                    sample: s,
                };
                let val = unsafe {
                    normify(*guard.inner.get_unchecked(&index), scale, min, max)
                };

                let write = (val.sqrt() * T::from_u8(255).unwrap()).to_u8().unwrap();

                out.put_pixel(s as u32, l as u32, Luma([write]));
            }

            bar.inc(dims.samples as u64)
        }
    }

    fn mask(&self, out: &mut GrayImage, min: T) {
        let guard = self.read();

        let dims = guard.inner.dims();

        let bar = ProgressBar::new((dims.lines * dims.samples) as u64);
        config_bar(&bar, "Mapping Pixels");

        for l in 0..dims.lines {
            for s in 0..dims.samples {
                let mut sum = T::zero();

                for b in 0..dims.channels {
                    let index = ImageIndex {
                        channel: b,
                        line: l,
                        sample: s,
                    };

                    let val = unsafe {
                        *guard.inner.get_unchecked(&index)
                    };

                    sum += val;
                }

                let write: T = num::clamp(sum - min, T::zero(), T::one()).ceil() * T::from_u8(255).unwrap();

                out.put_pixel(s as u32, l as u32, Luma([write.to_u8().unwrap()]));
            }

            bar.inc(dims.samples as u64)
        }
    }

    fn rgb(&self, out: &mut RgbImage, minimums: &[T], maximums: &[T], channels: &[usize]) {
        assert_eq!(3, channels.len());
        assert_eq!(3, maximums.len());
        assert_eq!(3, minimums.len());

        let guard = self.read();

        let dims = guard.inner.dims();

        let bar = ProgressBar::new((dims.lines * dims.samples) as u64);
        config_bar(&bar, "Mapping Pixels");

        let scales: Vec<T> = maximums.iter()
            .zip(minimums.iter())
            .map(|(max, min)| *max - *min)
            .collect();

        for l in 0..dims.lines {
            for s in 0..dims.samples {
                let norms: Vec<T> = channels.iter()
                    .zip(scales.iter())
                    .zip(maximums.iter())
                    .zip(minimums.iter())
                    .map(|(((band, scale), max), min)| unsafe {
                        let index = ImageIndex {
                            channel: *band,
                            line: l,
                            sample: s,
                        };

                        let val = guard.inner.get_unchecked(&index);
                        normify(*val, *scale, *min, *max)
                    })
                    .collect();

                let rgb = [
                    (norms[0] * T::from_u8(255).unwrap()).to_u8().unwrap(),
                    (norms[1] * T::from_u8(255).unwrap()).to_u8().unwrap(),
                    (norms[2] * T::from_u8(255).unwrap()).to_u8().unwrap(),
                ];

                out.put_pixel(s as u32, l as u32, Rgb(rgb))
            }

            bar.inc(dims.samples as u64)
        }
    }
}