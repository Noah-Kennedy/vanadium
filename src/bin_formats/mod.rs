use std::{mem, slice};
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Div, Sub};

use image::{GrayImage, Luma, Rgb, RgbImage};
use indicatif::ProgressBar;
use memmap2::{Mmap, MmapMut, MmapOptions};
use num::Zero;

use crate::headers::{FileByteOrder, Headers};

pub mod bsq;
pub mod bip;
pub mod bil;
pub mod error;

const DEFAULT_WORK_UNIT_SIZE: usize = 2097152;
pub static mut WORK_UNIT_SIZE: usize = DEFAULT_WORK_UNIT_SIZE;

pub struct FileInner<C, T> {
    pub dims: FileDims,
    pub container: C,
    pub phantom: PhantomData<T>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileDims {
    pub bands: Vec<u64>,
    pub samples: usize,
    pub lines: usize,
}

impl<T> FileInner<Mmap, T> {
    pub unsafe fn headers(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .populate(true)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map(&file)?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn from_dims(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .populate(true)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map(&file)?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<T> FileInner<MmapMut, T> {
    pub unsafe fn headers_mut(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .populate(true)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_mut(&file)?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn headers_copy(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .populate(true)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_copy(&file)?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn headers_anon(headers: &Headers) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .populate(true)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_anon()?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn from_dims_mut(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .populate(true)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map_mut(&file)?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn from_dims_copy(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .populate(true)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map_copy(&file)?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn from_dims_anon(dims: &FileDims) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .populate(true)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map_anon()?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<C, T> FileInner<C, T> where C: Deref<Target=[u8]> {
    #[inline(always)]
    pub fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.dims.lines * self.dims.bands.len() * self.dims.samples;
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

impl<C, T> FileInner<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    pub fn slice_mut(&mut self) -> &mut [T] {
        let ptr = self.container.as_mut_ptr() as *mut T;
        let len = self.dims.lines * self.dims.bands.len() * self.dims.samples;
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

impl From<&Headers> for FileDims {
    fn from(headers: &Headers) -> Self {
        let lines = headers.lines;
        let bands = (0..headers.bands as u64).collect();
        let samples = headers.samples;

        Self {
            bands,
            samples,
            lines,
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MatOrder {
    RowOrder,
    ColumnOrder,
}

pub trait FileIndex<T> {
    fn size(&self) -> (usize, usize, usize);
    fn order(&self) -> MatOrder;
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

    pub fn norm_between<T>(&mut self, min: &[T], max: &[T], bands: &[usize])
        where
            I: 'static + FileIndex<T> + FileIndexMut<T> + Sync + Send,
            T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T> + Debug + Zero
    {
        let (lines, pixels, _) = self.inner.size();
        let bar = ProgressBar::new((lines * pixels * bands.len()) as u64);

        for ((&b, &min), &max) in bands.iter().zip(min.iter()).zip(max.iter()) {
            let scale = max - min;

            for l in 0..lines {
                for p in 0..pixels {
                    unsafe {
                        let val = *self.inner.get_unchecked(l, p, b);
                        let clamped = num::clamp(val, min, max);
                        let shifted = clamped - min;
                        let norm = shifted / scale;

                        *self.inner.get_mut_unchecked(l, p, b) = norm;
                    }
                }
                bar.inc(pixels as u64)
            }
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