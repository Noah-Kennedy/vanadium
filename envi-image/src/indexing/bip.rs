use crate::ImageIndex;
use crate::indexing::FileDims;
use envi_header::Interleave;

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

impl ImageIndex for Bip {
    #[inline(always)]
    fn order(&self) -> Interleave {
        Interleave::Bip
    }

    #[inline(always)]
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        (((line * self.samples) + pixel) * self.bands) + band
    }
}