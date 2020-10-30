use std::ops::{Deref, DerefMut};

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, MatType};

pub struct Bil<C, T> {
    pub(crate) inner: FileInner<C, T>,
}

impl<C, T> From<FileInner<C, T>> for Bil<C, T> {
    fn from(inner: FileInner<C, T>) -> Self {
        Self {
            inner,
        }
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
    fn order(&self) -> MatType {
        MatType::Bil
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