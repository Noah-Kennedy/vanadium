use std::ops::Deref;

use envi_header::Interleave;

use crate::SpectralImageContainer;
use envi_image::{ImageIndex, FileDims};

pub type MatType = Interleave;

pub struct SpectralImage<C, T, I> {
    pub inner: SpectralImageContainer<C, T>,
    pub index: I,
}

impl<C1, C2, T, I1, I2> PartialEq<SpectralImage<C2, T, I2>> for SpectralImage<C1, T, I1>
    where
        I1: ImageIndex,
        I2: ImageIndex,
        T: Copy + PartialEq,
        C1: Deref<Target=[u8]>,
        C2: Deref<Target=[u8]>,
{
    fn eq(&self, other: &SpectralImage<C2, T, I2>) -> bool {
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