use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, MatOrder};

pub struct Bip<C, T> {
    pub(crate) inner: FileInner<C, T>
}

impl<C, T> From<FileInner<C, T>> for Bip<C, T> {
    fn from(inner: FileInner<C, T>) -> Self {
        Self { inner }
    }
}

impl<C, T> Index<(usize, usize)> for Bip<C, T> where C: Deref<Target=[u8]> {
    type Output = T;

    #[inline(always)]
    fn index(&self, (pixel, band): (usize, usize)) -> &Self::Output {
        let idx = (pixel * self.inner.dims.bands.len()) + band;
        self.inner.slice().index(idx)
    }
}

impl<C, T> IndexMut<(usize, usize)> for Bip<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    fn index_mut(&mut self, (pixel, band): (usize, usize)) -> &mut Self::Output {
        let idx = (pixel * self.inner.dims.bands.len()) + band;
        self.inner.slice_mut().index_mut(idx)
    }
}

impl<C, T> FileIndex<T> for Bip<C, T> where C: Deref<Target=[u8]> {
    #[inline(always)]
    fn size(&self) -> (usize, usize) {
        (self.inner.dims.samples * self.inner.dims.lines, self.inner.dims.bands.len())
    }

    #[inline(always)]
    fn order(&self) -> MatOrder {
        MatOrder::RowOrder
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, pixel: usize, band: usize) -> &T {
        let idx = (pixel * self.inner.dims.bands.len()) + band;
        self.inner.slice().get_unchecked(idx)
    }
}

impl<C, T> FileIndexMut<T> for Bip<C, T> where C: DerefMut<Target=[u8]> {
    #[inline(always)]
    unsafe fn get_mut_unchecked(&mut self, pixel: usize, band: usize) -> &mut T {
        let idx = (pixel * self.inner.dims.bands.len()) + band;
        self.inner.slice_mut().get_unchecked_mut(idx)
    }
}