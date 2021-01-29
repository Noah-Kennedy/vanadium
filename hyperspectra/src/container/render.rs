use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Div, Sub};

use image::{Rgb, RgbImage};
use indicatif::ProgressBar;
use num::{Bounded, FromPrimitive, ToPrimitive};
use num::traits::NumAssign;

use crate::bar::config_bar;
use crate::container::{ImageIndex, IndexImage, IterableImage, LockImage, normify};
use nalgebra::RealField;

pub trait Render<T> where T: PartialEq + Copy + Debug + 'static {
    fn solid(&self, out: &mut RgbImage, min: T, max: T, band: usize, flag: ColorFlag);
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
}