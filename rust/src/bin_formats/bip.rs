use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use memmap2::{Mmap, MmapOptions};
use rayon::prelude::*;

use crate::bin_formats::bsq::Bsq;
use crate::headers::envi::{EnviByteOrder, EnviHeaders};

pub struct Bip<C, T> {
    pub bands: usize,
    pub band_len: usize,
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<T> Bip<Mmap, T> {
    pub unsafe fn with_headers(headers: &EnviHeaders, file: &File) -> Result<Self, Box<dyn Error>> {
        assert_eq!(EnviByteOrder::Intel, headers.byte_order);

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

impl<C, T> Bip<C, T> where C: Deref<Target=[u8]> {
    pub fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.band_len * self.bands;
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

impl<C, T> Bip<C, T> where C: Deref<Target=[u8]>, T: Copy + Send + Sync {
    pub fn convert_bsq<C2: DerefMut<Target=[u8]>>(&self, out: &mut Bsq<C2, T>) {
        let pixels = self.slice().chunks(self.bands);

        let mut out_bands: Vec<&mut [T]> = out.split_bands_mut().collect();
        let _len = pixels.len() as u64;

        out_bands.par_iter_mut()
            .enumerate()
            .for_each(|(band_idx, band)| band
                .iter_mut()
                .zip(pixels.clone())
                .for_each(|(channel, pixel)| *channel = unsafe { *pixel.get_unchecked(band_idx) })
            );

        //for (row, bands) in pixels.enumerate().progress_count(len) {
        //    for (col, value) in bands.iter().enumerate() {
        //        unsafe {
        //            *out_bands.get_unchecked_mut(col)
        //                .get_unchecked_mut(row) = *value;
        //        }
        //    }
        //}
    }
}