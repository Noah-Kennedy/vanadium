use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice::Chunks;

use indicatif::{ParProgressBarIter, ProgressBar};
use memmap2::{Mmap, MmapOptions};
use rayon::prelude::*;

use crate::bin_formats::{OutOfPlaceConvert, WORK_UNIT_SIZE};
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::{ConversionError, ConversionErrorKind, SizeMismatchError};
use crate::headers::{FileByteOrder, Headers};

pub struct Bip<C, T> {
    pub bands: usize,
    pub band_len: usize,
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<T> Bip<Mmap, T> {
    pub unsafe fn with_headers(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
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
    }
}

impl<C, C2, T> OutOfPlaceConvert<Bsq<C2, T>> for Bip<C, T>
    where C: Deref<Target=[u8]> + Sync, C2: DerefMut<Target=[u8]> + Sync, T: Copy + Send + Sync
{
    fn convert_into(&self, out: &mut Bsq<C2, T>) -> Result<(), ConversionError> {
        if self.container.len() == out.container.len() {
            // let pixels = self.slice().chunks(self.bands);
            let pixel_chunks = self.slice().par_chunks(self.bands * WORK_UNIT_SIZE);

            let out_bands: Vec<&mut [T]> = out.split_bands_mut().collect();

            let bar_len = self.container.len();

            let bar = ProgressBar::new(bar_len as u64);

            out_bands.into_iter()
                .enumerate()
                .for_each(|(band_idx, band)| band
                    .par_chunks_mut(WORK_UNIT_SIZE)
                    .zip(pixel_chunks.clone())
                    .for_each(|(channel_chunks, pixel_chunk)| {
                        channel_chunks
                            .iter_mut()
                            .zip(pixel_chunk.chunks(self.bands))
                            .for_each(|(channel, pixel)| {
                                *channel = unsafe { *pixel.get_unchecked(band_idx) };
                            });

                        let inc = WORK_UNIT_SIZE * (self.bands - 1);
                        bar.inc(inc as u64);
                    })
                );

            bar.finish();

            Ok(())
        } else {
            let size_error = SizeMismatchError {
                input_size: self.container.len(),
                output_size: out.container.len(),
            };

            let kind = ConversionErrorKind::SizeMismatch(size_error);

            let conversion_error = ConversionError {
                input_type: "bip",
                output_type: "bsq",
                kind,
            };

            Err(conversion_error)
        }
    }
}

// fn encode_band_bsq<T>(pixels: &[T], band_idx: usize, band: &mut &mut [T])
//     where T: Copy + Send + Sync
// {
//
// }