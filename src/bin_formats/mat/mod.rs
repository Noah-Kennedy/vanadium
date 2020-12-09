use std::ops::Deref;

pub use color_maps::*;
pub use conversion::*;
pub use pca::*;

use crate::bin_formats::{FileDims, FileInner, FileIndex};
use crate::headers::Interleave;

mod conversion;
mod color_maps;
mod stat;
mod pca;

pub type MatType = Interleave;

pub struct Mat<C, T, I> {
    pub inner: FileInner<C, T>,
    pub index: I,
}


impl<C1, C2, T, I1, I2> PartialEq<Mat<C2, T, I2>> for Mat<C1, T, I1>
    where
        I1: FileIndex,
        I2: FileIndex,
        T: Copy + PartialEq,
        C1: Deref<Target=[u8]>,
        C2: Deref<Target=[u8]>,
{
    fn eq(&self, other: &Mat<C2, T, I2>) -> bool {
        if self.inner.size() == other.inner.size() {
            let FileDims { bands, samples, lines } = self.inner.size();
            let bands = bands.len();

            let mut res = true;

            let (p1, p2) = unsafe {
                (
                    self.inner.get_unchecked(),
                    other.inner.get_unchecked()
                )
            };

            for l in 0..lines {
                for s in 0..samples {
                    for b in 0..bands {
                        let idx_1 = self.index.get_idx(l, s, b);
                        let idx_2 = other.index.get_idx(l, s, b);

                        unsafe {
                            let i1 = *p1.0.add(idx_1);
                            let i2 = *p2.0.add(idx_2);
                            res &= i1 == i2;
                        }
                    }
                }
            }

            res
        } else {
            false
        }
    }
}