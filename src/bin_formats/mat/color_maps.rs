use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Deref, Div, Sub};

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::ProgressBar;
use num::Zero;

use crate::bin_formats::{FileDims, FileIndex, Mat};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum ColorFlag {
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
    Teal,
}

impl<C1, I1> Mat<C1, f32, I1>
    where I1: 'static + FileIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    pub fn cool_warm_stat(&self, out: &mut RgbImage, min: f32, max: f32, band: usize)
        where I1: 'static + FileIndex + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();
        assert!(band < bands);

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        let std_dev = unsafe {
            self.std_dev(band, None)
        };

        let max_z = max / std_dev as f32;
        let min_z = min / std_dev as f32;
        let scale = max_z - min_z;

        println!("Applying color map");
        let bar = ProgressBar::new((lines * samples) as u64);
        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);

                let val = unsafe {
                    r_ptr.0.add(idx).read_volatile()
                };

                let normed = (normify(val / std_dev as f32, scale, min_z, max_z) * 2.0) - 1.0;

                let direction = normed.partial_cmp(&0.0).unwrap();

                let mag = normed.abs();

                let pri = (mag.sqrt() * 255.0).floor() as u8;
                let alt = ((1.0 - mag) * 255.0).floor() as u8;

                let pix = match direction {
                    Ordering::Less => {
                        [alt, alt, pri]
                    }
                    Ordering::Equal => {
                        [255, 255, 255]
                    }
                    Ordering::Greater => {
                        [pri, alt, alt]
                    }
                };

                out.put_pixel(s as u32, l as u32, Rgb(pix))
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

    pub fn solid(&self, out: &mut RgbImage, min: f32, max: f32, band: usize, flag: ColorFlag)
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

                let pri = (val.sqrt() * 255.0).floor() as u8;
                let alt = (val * 255.0).floor() as u8;

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

    pub fn rgb(
        &self, out: &mut RgbImage,
        minimums: &[f32], maximums: &[f32], channels: &[usize], summation: [&[usize]; 3],
    )
        where I1: 'static + FileIndex + Sync + Send,
    {
        assert_eq!(channels.len(), minimums.len(), "mins");
        assert_eq!(channels.len(), maximums.len(), "Maxes");
        // assert_eq!(channels.len(), summation.len(), "Summation length");

        let FileDims { samples, lines, .. } = self.inner.size();
        let bar = ProgressBar::new((lines * samples) as u64);

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        let scales: Vec<f32> = maximums.iter()
            .zip(minimums.iter())
            .map(|(max, min)| *max - *min)
            .collect();

        for l in 0..lines {
            for s in 0..samples {
                let norms: Vec<f32> = channels.iter()
                    .zip(scales.iter())
                    .zip(maximums.iter())
                    .zip(minimums.iter())
                    .map(|(((band, scale), max), min)| unsafe {
                        let idx = self.index.get_idx(l, s, *band);
                        let val = r_ptr.0.add(idx).read_volatile();
                        normify(val, *scale, *min, *max)
                    })
                    .collect();

                let mut sums: [f32; 3] = [
                    summation[0].iter()
                        .map(|idx| norms[*idx])
                        .sum(),
                    summation[1].iter()
                        .map(|idx| norms[*idx])
                        .sum(),
                    summation[2].iter()
                        .map(|idx| norms[*idx])
                        .sum(),
                ];

                sums[0] /= summation[0].len() as f32;
                sums[1] /= summation[1].len() as f32;
                sums[2] /= summation[2].len() as f32;

                let rgb = [
                    (sums[0] * 255.0) as u8,
                    (sums[1] * 255.0) as u8,
                    (sums[2] * 255.0) as u8,
                ];

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