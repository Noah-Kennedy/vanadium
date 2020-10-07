use std::ops::{Deref, DerefMut};

use indicatif::ProgressBar;
use num::traits::NumAssign;
use rayon::prelude::*;

use crate::bin_formats::{FileConvert, FileInner, WORK_UNIT_SIZE, FileAlgebra};
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::{ConversionError, ConversionErrorKind, SizeMismatchError};
use num::{Float, NumCast};

pub struct Bip<C, T> {
    pub(crate) inner: FileInner<C, T>
}

impl<C, T> From<FileInner<C, T>> for Bip<C, T> {
    fn from(inner: FileInner<C, T>) -> Self {
        Self { inner }
    }
}

impl<C, C2, T> FileConvert<T, C2> for Bip<C, T>
    where C: Deref<Target=[u8]> + Sync,
          C2: DerefMut<Target=[u8]> + Sync,
          T: Copy + Send + Sync + NumAssign
{
    fn to_bsq(&self, out: &mut Bsq<C2, T>) -> Result<(), ConversionError> {
        if self.inner.container.len() == out.inner.container.len() {
            let pixel_chunks = self.inner.slice()
                .par_chunks(WORK_UNIT_SIZE * self.inner.dims.bands.len());

            let out_bands: Vec<&mut [T]> = out.split_bands_mut().collect();

            let bar = ProgressBar::new(self.inner.container.len() as u64);

            out_bands.into_par_iter()
                .enumerate()
                .for_each(|(band_idx, band)| band
                    .par_chunks_mut(WORK_UNIT_SIZE)
                    .zip(pixel_chunks.clone())
                    .for_each(|(channel_chunks, pixel_chunk)| {
                        channel_chunks
                            .iter_mut()
                            .zip(pixel_chunk.chunks(self.inner.dims.bands.len()))
                            .for_each(|(channel, pixel)| {
                                *channel = unsafe { *pixel.get_unchecked(band_idx) };
                            });

                        let inc = WORK_UNIT_SIZE * (self.inner.dims.bands.len() - 1);
                        bar.inc(inc as u64);
                    })
                );

            bar.finish();

            Ok(())
        } else {
            let size_error = SizeMismatchError {
                input_size: self.inner.container.len(),
                output_size: out.inner.container.len(),
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

impl<C, T> FileAlgebra<T> for Bip<C, T>
    where C: DerefMut<Target=[u8]>, T: Float + NumCast + NumAssign
{
    fn rescale(&mut self, bands: &[usize], scale: T, offset: T) {
        todo!()
    }

    fn normalize(&mut self, bands: &[usize], floor: T, ceiling: T) {
        todo!()
    }
}

impl<C, C2> FileConvert<u8, C2> for Bip<C, f32>
    where C: Deref<Target=[u8]> + Sync,
          C2: DerefMut<Target=[u8]> + Sync,
{
    fn to_bsq(&self, out: &mut Bsq<C2, u8>) -> Result<(), ConversionError> {
        todo!()
    }

    fn to_bip(&self, _out: &mut Bip<C2, u8>) -> Result<(), ConversionError> {
        todo!()
    }
}