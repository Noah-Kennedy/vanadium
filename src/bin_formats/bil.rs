use crate::bin_formats::{FileIndex, MatType, FileDims};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct Bil {
    bands: usize,
    samples: usize
}

impl From<FileDims> for Bil {
    fn from(dims: FileDims) -> Self {
        Self {
            bands: dims.bands.len(),
            samples: dims.samples
        }
    }
}

impl FileIndex for Bil where {
    #[inline(always)]
    fn order(&self) -> MatType {
        MatType::Bil
    }

    #[inline(always)]
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        (line * self.samples * self.bands)
            + (band * self.samples)
            + pixel
    }
}