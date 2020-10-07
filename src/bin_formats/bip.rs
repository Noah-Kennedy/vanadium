use std::{mem, slice};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use indicatif::ProgressBar;
use memmap2::{Mmap, MmapOptions};
use num::traits::NumAssign;
use rayon::prelude::*;

use crate::bin_formats::{OperableFile, WORK_UNIT_SIZE};
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
    fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.band_len * self.bands;
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

impl<C, T> Bip<C, T> where C: Deref<Target=[u8]>, T: Copy + Send + Sync {}

impl<C, C2, T> OperableFile<T, C2> for Bip<C, T>
    where C: Deref<Target=[u8]> + Sync,
          C2: DerefMut<Target=[u8]> + Sync,
          T: Copy + Send + Sync + NumAssign
{
    fn to_bsq(&self, out: &mut Bsq<C2, T>) -> Result<(), ConversionError> {
        if self.container.len() == out.container.len() {
            let pixel_chunks = self.slice().par_chunks(WORK_UNIT_SIZE * self.bands);

            let out_bands: Vec<&mut [T]> = out.split_bands_mut().collect();

            let bar = ProgressBar::new(self.container.len() as u64);

            out_bands.into_par_iter()
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

    fn to_bip(&self, _out: &mut Bip<C2, T>) -> Result<(), ConversionError> {
        unimplemented!("Support for bip->bip is not implemented. Why are you doing this anyways?")
    }
}