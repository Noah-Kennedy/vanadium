use crate::bin_formats::{FileDims, FileIndex, MatType};

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

impl Bsq {
    #[inline(always)]
    fn _idx_2d(&self, pixel: usize, band: usize) -> usize {
        (band * self.band_len) + pixel
    }

    #[inline(always)]
    fn idx_3d(&self, line: usize, pixel: usize, band: usize) -> usize {
        (band * self.band_len) + (self.samples * line) + pixel
    }
}

impl FileIndex for Bsq {
    #[inline(always)]
    fn order(&self) -> MatType {
        MatType::Bsq
    }

    #[inline(always)]
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize {
        self.idx_3d(line, pixel, band)
    }
}