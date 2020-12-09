use std::mem;
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use memmap2::{Mmap, MmapMut, MmapOptions};

use crate::headers::{FileByteOrder, Headers};

#[derive(Copy, Clone)]
pub struct FileBuf<T>(pub(crate) *const T);

#[derive(Copy, Clone)]
pub struct FileBufMut<T>(pub(crate) *mut T);

unsafe impl<T> Send for FileBuf<T> where T: Send {}

unsafe impl<T> Send for FileBufMut<T> where T: Send {}

pub struct SpectralImageContainer<C, T> {
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

impl<T> SpectralImageContainer<Mmap, T> {
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

    pub unsafe fn _from_dims(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
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

impl<T> SpectralImageContainer<MmapMut, T> {
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

    pub unsafe fn _headers_copy(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_copy(&file)?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn _headers_anon(headers: &Headers) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_anon()?;

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn _from_dims_mut(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
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

    pub unsafe fn _from_dims_copy(dims: &FileDims, file: &File) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map_copy(&file)?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn _from_dims_anon(dims: &FileDims) -> Result<Self, Box<dyn Error>> {
        let raw = MmapOptions::new()
            .offset(0)
            .len(dims.bands.len() * dims.samples * dims.lines * mem::size_of::<T>())
            .map_anon()?;

        Ok(Self {
            dims: dims.clone(),
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<C, T> SpectralImageContainer<C, T> {
    pub fn size(&self) -> FileDims {
        self.dims.clone()
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: Deref<Target=[u8]> {
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> FileBuf<T> {
        FileBuf(self.container.as_ptr() as *const T)
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self) -> FileBufMut<T> {
        FileBufMut(self.container.as_mut_ptr() as *mut T)
    }
}