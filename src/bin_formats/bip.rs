use crate::bin_formats::{FileDims, FileIndex, MatType};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct Bip {
    samples: usize,
    bands: usize,
}

impl From<FileDims> for Bip {
    fn from(dims: FileDims) -> Self {
        Self {
            bands: dims.bands.len(),
            samples: dims.samples,
        }
    }
}

impl FileIndex for Bip {
    #[inline(always)]
    fn order(&self) -> MatType {
        MatType::Bip
    }

    #[inline(always)]
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        (((line * self.samples) + pixel) * self.bands) + band
    }
}