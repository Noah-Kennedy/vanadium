use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use memmap2::{Mmap, MmapMut, MmapOptions};

use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::ConversionError;
use crate::headers::{FileByteOrder, Headers};

pub mod bsq;
pub mod bip;
pub mod error;

const WORK_UNIT_SIZE: usize = 2097152;

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
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_mut(&file)?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn from_dims_mut(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map_mut(&file)?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<C, T> FileInner<C, T> where C: Deref<Target=[u8]> {
    pub fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.dims.lines * self.dims.bands.len() * self.dims.samples;
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

impl<C, T> FileInner<C, T> where C: DerefMut<Target=[u8]> {
    pub fn slice_mut(&mut self) -> &mut [T] {
        let ptr = self.container.as_mut_ptr() as *mut T;
        let len = self.dims.lines* self.dims.bands.len() * self.dims.samples;
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

impl From<&Headers> for FileDims {
    fn from(headers: &Headers) -> Self {
        let lines = headers.lines;
        let bands = (0..headers.bands as u64).into_iter().collect();
        let samples = headers.samples;

        Self {
            bands,
            samples,
            lines,
        }
    }
}

pub trait FileConvert<T, C> {
    fn to_bsq(&self, out: &mut Bsq<C, T>) -> Result<(), ConversionError>;
    fn to_bip(&self, out: &mut Bip<C, T>) -> Result<(), ConversionError>;
}

pub trait FileAlgebra<T> {
    fn rescale(&mut self, bands: &[usize], scale: T, offset: T);
    fn normalize(&mut self, bands: &[usize], floor: T, ceiling: T);
}