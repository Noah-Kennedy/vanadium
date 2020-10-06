use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice::{Chunks, ChunksMut};

use memmap2::{Mmap, MmapMut, MmapOptions};

use crate::headers::{FileByteOrder, Headers};

pub struct Bsq<C, T> {
    pub bands: usize,
    pub band_len: usize,
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<T> Bsq<Mmap, T> {
    pub unsafe fn with_headers(headers: &Headers, file: File) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map(&file)?;

        let bands = headers.bands;
        let samples = headers.samples;

        Ok(Self {
            bands,
            band_len: samples * headers.lines,
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<T> Bsq<MmapMut, T> {
    pub unsafe fn with_headers_mut(
        headers: &Headers, file: File,
    ) -> Result<Self, Box<dyn Error>>
    {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_mut(&file)?;

        let bands = headers.bands;
        let samples = headers.samples;

        Ok(Self {
            bands,
            band_len: samples * headers.lines,
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn with_headers_copy(
        headers: &Headers, file: &File,
    ) -> Result<Self, Box<dyn Error>>
    {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
            .map_copy(file)?;

        let bands = headers.bands;
        let samples = headers.samples;

        Ok(Self {
            bands,
            band_len: samples * headers.lines,
            container: raw,
            phantom: Default::default(),
        })
    }

    pub unsafe fn with_headers_anon(headers: &Headers) -> Result<Self, Box<dyn Error>> {
        assert_eq!(FileByteOrder::Intel, headers.byte_order);

        let len = headers.bands * headers.samples * headers.lines * mem::size_of::<T>();

        let raw = MmapOptions::new()
            .offset(headers.header_offset as u64)
            .len(len)
            .map_anon()?;

        let bands = headers.bands;
        let samples = headers.samples;

        Ok(Self {
            bands,
            band_len: samples * headers.lines,
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<C, T> Bsq<C, T> where C: Deref<Target=[u8]> {
    pub fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.band_len * self.bands;
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    pub fn band(&self, band: usize) -> &[T] {
        assert!(band < self.bands);
        &self.slice()[(band * self.band_len)..((band * self.band_len) + self.band_len)]
    }

    pub fn split_bands(&self) -> Chunks<T> {
        self.slice().chunks(self.band_len)
    }
}

impl<C, T> Bsq<C, T> where C: DerefMut<Target=[u8]> {
    pub fn slice_mut(&mut self) -> &mut [T] {
        let ptr = self.container.as_mut_ptr() as *mut T;
        let len = self.band_len * self.bands;
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    pub fn band_mut(&mut self, band: usize) -> &mut [T] {
        assert!(band < self.bands);
        let len = self.band_len;
        let offset = band * len;

        &mut self.slice_mut()[offset..(offset + len)]
    }

    pub fn split_bands_mut(&mut self) -> ChunksMut<T> {
        let len = self.band_len;
        self.slice_mut().chunks_mut(len)
    }
}