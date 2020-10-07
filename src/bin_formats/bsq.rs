use std::ops::{Deref, DerefMut};
use std::slice::{Chunks, ChunksMut};

use num::{Float, NumCast};
use num::traits::NumAssign;

use crate::bin_formats::{FileAlgebra, FileConvert, FileInner};
use crate::bin_formats::bip::Bip;
use crate::bin_formats::error::ConversionError;

pub struct Bsq<C, T> {
    pub(crate) inner: FileInner<C, T>,
    pub(crate) band_len: usize,
}

impl<C, T> From<FileInner<C, T>> for Bsq<C, T> {
    fn from(inner: FileInner<C, T>) -> Self {
        Self {
            band_len: inner.dims.samples * inner.dims.lines,
            inner,
        }
    }
}

impl<C, T> Bsq<C, T> where C: Deref<Target=[u8]> {
    pub fn band(&self, band: usize) -> &[T] {
        assert!(band < self.inner.dims.bands.len());
        &self.inner.slice()[(band * self.band_len)..((band * self.band_len) + self.band_len)]
    }

    pub fn split_bands(&self) -> Chunks<T> {
        self.inner.slice().chunks(self.band_len)
    }
}

impl<C, T> Bsq<C, T> where C: DerefMut<Target=[u8]> {
    pub fn band_mut(&mut self, band: usize) -> &mut [T] {
        assert!(band < self.inner.dims.bands.len());
        &mut self.inner
            .slice_mut()[(band * self.band_len)..((band * self.band_len) + self.band_len)]
    }

    pub fn split_bands_mut(&mut self) -> ChunksMut<T> {
        self.inner.slice_mut().chunks_mut(self.band_len)
    }
}

impl<C, T> FileAlgebra<T> for Bsq<C, T> where C: DerefMut<Target=[u8]>, T: Float + NumCast + NumAssign {
    fn rescale(&mut self, bands: &[usize], scale: T, offset: T) {
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
    }

    fn normalize(&mut self, bands: &[usize], floor: T, ceiling: T) {
        let range = ceiling - floor;
        self.rescale(bands, range, -floor);

        for band_id in bands {
            let band = self.band_mut(*band_id);

            for pixel in band {
                if *pixel < T::zero() {
                    T::set_zero(pixel)
                } else if *pixel > T::one() {
                    T::set_one(pixel)
                }
            }
        }
    }
}

impl<C, C2, T> FileConvert<T, C2> for Bsq<C, T>
    where C: Deref<Target=[u8]> + Sync,
          C2: DerefMut<Target=[u8]> + Sync,
          T: Copy + Send + Sync + NumAssign
{
    fn to_bsq(&self, _out: &mut Bsq<C2, T>) -> Result<(), ConversionError> {
        unimplemented!("Support for bsq->bsq is not implemented. Why are you doing this anyways?")
    }

    fn to_bip(&self, _out: &mut Bip<C2, T>) -> Result<(), ConversionError> {
        todo!()
    }
}

impl<C, C2> FileConvert<u8, C2> for Bsq<C, f32>
    where C: Deref<Target=[u8]> + Sync,
          C2: DerefMut<Target=[u8]> + Sync,
{
    fn to_bsq(&self, out: &mut Bsq<C2, u8>) -> Result<(), ConversionError> {
        let out_bands = out.split_bands_mut();
        let in_bands = self.split_bands();

        // in_bands.iter

        Ok(())
    }

    fn to_bip(&self, _out: &mut Bip<C2, u8>) -> Result<(), ConversionError> {
        todo!()
    }
}