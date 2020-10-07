use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice::{Chunks, ChunksMut};

use memmap2::{Mmap, MmapMut, MmapOptions};
use num::{Float, NumCast, Zero, One};
use num::traits::NumAssign;

use crate::bin_formats::error::ConversionError;
use crate::bin_formats::OperableExt;
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
    fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.band_len * self.bands;
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    fn band(&self, band: usize) -> &[T] {
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

impl<C, T> OperableExt<T> for Bsq<C, T> where C: DerefMut<Target=[u8]>, T: Float + NumCast + NumAssign {
    fn rescale(&mut self, bands: &[usize], scale: T, offset: T) -> Result<(), ConversionError> {
        for band_id in bands {
            let band = self.band_mut(*band_id);

            for register in band.chunks_mut(8) {
                if register.len() % 8 == 0 {
                    unsafe {
                        *register.get_unchecked_mut(0) += offset;
                        *register.get_unchecked_mut(1) += offset;
                        *register.get_unchecked_mut(2) += offset;
                        *register.get_unchecked_mut(3) += offset;
                        *register.get_unchecked_mut(4) += offset;
                        *register.get_unchecked_mut(5) += offset;
                        *register.get_unchecked_mut(6) += offset;
                        *register.get_unchecked_mut(7) += offset;

                        *register.get_unchecked_mut(0) *= scale;
                        *register.get_unchecked_mut(1) *= scale;
                        *register.get_unchecked_mut(2) *= scale;
                        *register.get_unchecked_mut(3) *= scale;
                        *register.get_unchecked_mut(4) *= scale;
                        *register.get_unchecked_mut(5) *= scale;
                        *register.get_unchecked_mut(6) *= scale;
                        *register.get_unchecked_mut(7) *= scale;
                    }
                } else {
                    for pixel in register {
                        *pixel += offset;
                        *pixel *= scale;
                    }
                }
            }
        }

        Ok(())
    }

    fn normalize(&mut self, bands: &[usize], floor: T, ceiling: T) -> Result<(), ConversionError> {
        let range = ceiling - floor;
        self.rescale(bands, range, -floor)?;

        for band_id in bands {
            let band = self.band_mut(*band_id);

            for pixel in band {
                if *pixel < T::zero() {
                    pixel.set_zero()
                } else if *pixel > T::one() {
                    pixel.set_one()
                }
            }
        }

        Ok(())
    }
}