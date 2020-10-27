use std::ops::{Deref, DerefMut};

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, MatOrder};

pub struct Bip<C, T> {
    pub(crate) inner: FileInner<C, T>
}

impl<C, T> From<FileInner<C, T>> for Bip<C, T> {
    fn from(inner: FileInner<C, T>) -> Self {
        Self { inner }
    }
}

impl<C, T> Bip<C, T> {
    #[inline(always)]
    fn find_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        (((line * self.inner.dims.samples) + pixel) * self.inner.dims.bands.len()) + band
    }
}

impl<C, T> FileIndex<T> for Bip<C, T> where C: Deref<Target=[u8]> {
    #[inline(always)]
    fn size(&self) -> (usize, usize, usize) {
        (self.inner.dims.lines, self.inner.dims.samples, self.inner.dims.bands.len())
    }

    #[inline(always)]
    fn order(&self) -> MatOrder {
        MatOrder::RowOrder
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, line: usize, pixel: usize, band: usize) -> &T {
        self.inner.slice().get_unchecked(self.find_idx(line, pixel, band))
    }
}

impl<C, T> FileIndexMut<T> for Bip<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    unsafe fn get_mut_unchecked(&mut self, line: usize, pixel: usize, band: usize) -> &mut T {
        let idx = self.find_idx(line, pixel, band);
        self.inner.slice_mut().get_unchecked_mut(idx)
    }
}