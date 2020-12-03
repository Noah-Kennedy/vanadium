use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Div, Sub};

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::ProgressBar;
use num::integer::Roots;
use num::Zero;

use crate::bin_formats::{FileDims, FileInner};
use crate::headers::Interleave;
use std::cmp::Ordering;

pub type MatType = Interleave;

pub trait FileIndex: {
    fn order(&self) -> MatType;
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize;
}

pub struct Mat<C, T, I> {
    pub inner: FileInner<C, T>,
    pub index: I,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum ColorFlag {
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
    Teal
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
    pub fn convert<C2, I2>(&self, out: &mut Mat<C2, f32, I2>)
        where
            I2: 'static + FileIndex + Sync + Send + Copy + Clone,
            C2: DerefMut<Target=[u8]> + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();

        let bar = ProgressBar::new((lines * samples * bands) as u64);

        let r_idx_gen = self.index;
        let w_idx_gen = out.index;

        let r_ptr = unsafe { self.inner.get_unchecked() };
        let w_ptr = unsafe { out.inner.get_unchecked_mut() };

        for b in 0..bands {
            for l in 0..lines {
                for s in 0..samples {
                    let read_idx = r_idx_gen.get_idx(l, s, b);
                    let write_idx = w_idx_gen.get_idx(l, s, b);

                    unsafe {
                        let r = r_ptr.0.add(read_idx);
                        let w = w_ptr.0.add(write_idx);

                        w.write_volatile(r.read_volatile());
                    }
                }
            }

            bar.inc((lines * samples) as u64)
        }
    }

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
            self.std_dev(band, min, max)
        };

        let max_z = max / std_dev;
        let min_z = min / std_dev;
        let scale = max_z - min_z;

        println!("Applying color map");
        let bar = ProgressBar::new((lines * samples) as u64);
        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);

                let val = unsafe {
                    r_ptr.0.add(idx).read_volatile()
                };

                let normed = (normify(val / std_dev, scale, min_z, max_z) * 2.0) - 1.0;

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

    pub fn cool_warm_norm(&self, out: &mut RgbImage, min: f32, max: f32, band: usize)
        where I1: 'static + FileIndex + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();
        assert!(band < bands);

        let scale = max - min;

        let r_ptr = unsafe {
            self.inner.get_unchecked()
        };

        println!("Applying color map");
        let bar = ProgressBar::new((lines * samples) as u64);
        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);

                let val = unsafe {
                    r_ptr.0.add(idx).read_volatile().sqrt()
                };

                let normed = (normify(val, scale, min.sqrt(), max.sqrt()) * 2.0) - 1.0;

                let pix = if normed > 0.0 {
                    let r = (normed * 255.0).floor() as u8;
                    let alt = (255 - r).sqrt();
                    [r, alt, alt]
                } else if normed < 0.0 {
                    let b = ((1.0 - normed) * 255.0).floor() as u8;
                    let alt = (255 - b).sqrt();
                    [alt, b, alt]
                } else {
                    [255, 255, 255]
                };

                out.put_pixel(s as u32, l as u32, Rgb(pix))
            }
            bar.inc(samples as u64)
        }
    }

    unsafe fn mean(&self, band: usize, min: f32, max: f32) -> f32 {
        let FileDims { bands, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mut sum = 0.0;
        let mut count = 0.0;

        println!("Computing mean");
        let bar = ProgressBar::new((lines * samples) as u64);
        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                let cmp = (x < max) as u8 as f32 * (x > min) as u8 as f32;
                let n = x * cmp;

                sum += n;
                count += cmp;
            }
            bar.inc(samples as u64)
        }

        sum / count
    }

    unsafe fn std_dev(&self, band: usize, min: f32, max: f32) -> f32 {
        let FileDims { bands, samples, lines } = self.inner.size();

        let r_ptr = self.inner.get_unchecked();

        let mean = self.mean(band, min, max);
        let mut sum = 0.0;
        let mut count = 0.0;

        println!("Computing standard deviation");
        let bar = ProgressBar::new((lines * samples) as u64);
        for l in 0..lines {
            for s in 0..samples {
                let idx = self.index.get_idx(l, s, band);
                let x = r_ptr.0.add(idx).read_volatile();

                let cmp = (x < max) as u8 as f32 * (x > min) as u8 as f32;

                let dif = (x - mean) * cmp;

                sum += dif * dif;
                count += cmp;
            }
            bar.inc(samples as u64)
        }

        sum /= count - 1.0;

        sum.sqrt()
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
                    ColorFlag::Red       => [pri, alt, alt],
                    ColorFlag::Green     => [alt, pri, alt],
                    ColorFlag::Blue      => [alt, alt, pri],
                    ColorFlag::Purple    => [pri, alt, pri],
                    ColorFlag::Yellow    => [pri, pri, alt],
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