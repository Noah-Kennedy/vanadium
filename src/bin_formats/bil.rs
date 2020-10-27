use std::ops::{Deref, DerefMut};
use std::slice::{Chunks, ChunksMut};

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, MatOrder};

pub struct Bil<C, T> {
    pub(crate) inner: FileInner<C, T>,
    pub(crate) band_len: usize,
}

impl<C, T> From<FileInner<C, T>> for Bil<C, T> {
    fn from(inner: FileInner<C, T>) -> Self {
        Self {
            band_len: inner.dims.samples * inner.dims.lines,
            inner,
        }
    }
}

impl<C, T> Bil<C, T> where C: Deref<Target=[u8]> {
    pub fn band(&self, band: usize) -> &[T] {
        assert!(band < self.inner.dims.bands.len());
        &self.inner.slice()[(band * self.band_len)..((band * self.band_len) + self.band_len)]
    }

    pub fn split_bands(&self) -> Chunks<T> {
        self.inner.slice().chunks(self.band_len)
    }
}

impl<C, T> Bil<C, T> where C: DerefMut<Target=[u8]> {
    pub fn band_mut(&mut self, band: usize) -> &mut [T] {
        assert!(band < self.inner.dims.bands.len());
        &mut self.inner
            .slice_mut()[(band * self.band_len)..((band * self.band_len) + self.band_len)]
    }

    pub fn split_bands_mut(&mut self) -> ChunksMut<T> {
        self.inner.slice_mut().chunks_mut(self.band_len)
    }
}

impl<C, T> Bil<C, T> {
    #[inline(always)]
    fn idx_3d(&self, line: usize, pixel: usize, band: usize) -> usize {
        (line * self.inner.dims.samples * self.inner.dims.bands.len())
            + (band * self.inner.dims.samples)
            + pixel
    }
}

impl<C, T> FileIndex<T> for Bil<C, T> where C: Deref<Target=[u8]> {
    #[inline(always)]
    fn size(&self) -> (usize, usize, usize) {
        (self.inner.dims.lines, self.inner.dims.samples, self.inner.dims.bands.len())
    }

    #[inline(always)]
    fn order(&self) -> MatOrder {
        MatOrder::ColumnOrder
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, line: usize, pixel: usize, band: usize) -> &T {
        self.inner.slice().get_unchecked(self.idx_3d(line, pixel, band))
    }
}

impl<C, T> FileIndexMut<T> for Bil<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    unsafe fn get_mut_unchecked(&mut self, line: usize, pixel: usize, band: usize) -> &mut T {
        let idx = self.idx_3d(line, pixel, band);
        self.inner.slice_mut().get_unchecked_mut(idx)
    }
}