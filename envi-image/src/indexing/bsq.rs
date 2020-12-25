use crate::ImageIndex;
use envi_header::Interleave;
use crate::indexing::FileDims;

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

impl ImageIndex for Bsq {
    #[inline(always)]
    fn order(&self) -> Interleave {
        Interleave::Bsq
    }

    #[inline(always)]
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        (band * self.band_len) + (self.samples * line) + pixel
    }
}