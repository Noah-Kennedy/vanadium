use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::{Chunks, ChunksMut};

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, MatOrder};

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

impl<C, T> Index<(usize, usize)> for Bsq<C, T> where C: Deref<Target=[u8]> {
    type Output = T;

    #[inline(always)]
    fn index(&self, (pixel, band): (usize, usize)) -> &Self::Output {
        let idx = (band * self.band_len) + pixel;
        self.inner.slice().index(idx)
    }
}

impl<C, T> IndexMut<(usize, usize)> for Bsq<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    fn index_mut(&mut self, (pixel, band): (usize, usize)) -> &mut Self::Output {
        let idx = (band * self.band_len) + pixel;
        self.inner.slice_mut().index_mut(idx)
    }
}

impl<C, T> FileIndex<T> for Bsq<C, T> where C: Deref<Target=[u8]> {
    #[inline(always)]
    fn size(&self) -> (usize, usize) {
        (self.band_len, self.inner.dims.bands.len())
    }

    #[inline(always)]
    fn order(&self) -> MatOrder {
        MatOrder::ColumnOrder
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, pixel: usize, band: usize) -> &T {
        let idx = (band * self.band_len) + pixel;
        self.inner.slice().get_unchecked(idx)
    }
}

impl<C, T> FileIndexMut<T> for Bsq<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    unsafe fn get_mut_unchecked(&mut self, pixel: usize, band: usize) -> &mut T {
        let idx = (band * self.band_len) + pixel;
        self.inner.slice_mut().get_unchecked_mut(idx)
    }
}