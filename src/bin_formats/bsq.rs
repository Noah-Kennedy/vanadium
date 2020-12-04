use crate::bin_formats::{FileDims, FileIndex, MatType};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct Bsq {
    band_len: usize,
    samples: usize,
}

impl From<FileDims> for Bsq {
    fn from(dims: FileDims) -> Self {
        Self {
            band_len: dims.samples * dims.lines,
            samples: dims.samples,
        }
    }
}

impl FileIndex for Bsq {
    #[inline(always)]
    fn order(&self) -> MatType {
        MatType::Bsq
    }

    #[inline(always)]
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        (band * self.band_len) + (self.samples * line) + pixel
    }
}