use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Div, Sub};

use indicatif::ProgressBar;
use memmap2::{Mmap, MmapMut, MmapOptions};

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
    inner: Box<F>
}

impl <F> From<F> for Mat<F> {
    fn from(inner: F) -> Self {
        let inner = Box::new(inner);
        Self { inner }
    }
}

impl<I> Mat<I> {
    pub fn convert<T, O>(&self, out: &mut Mat<O>)
        where
            I: 'static + FileIndex<T> + Sync + Send,
            O: 'static + FileIndexMut<T> + Sync + Send,
            T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T>

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

    pub fn clamp_between<T, O>(&self, out: &mut Mat<O>, min: T, max: T, bands: &[usize])
        where
            I: 'static + FileIndex<T> + Sync + Send,
            O: 'static + FileIndexMut<T> + Sync + Send,
            T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T>
    {
        let (lines, pixels, _) = self.inner.size();
        let bar = ProgressBar::new((lines * pixels * bands.len()) as u64);

        let scale = max - min;

        for &b in bands {
            for l in 0..lines {
                for p in 0..pixels {
                    unsafe {
                        let clamped = num::clamp(*self.inner.get_unchecked(l, p, b), min, max);
                        let shifted = clamped - min;
                        let norm = shifted / scale;

                        *out.inner.get_mut_unchecked(l, p, b) = norm;
                    }
                }
            }

            bar.inc((lines * pixels) as u64)
        }
    }
}