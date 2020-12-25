pub use bil::*;
pub use bip::*;
pub use bsq::*;
use envi_header::Headers;

mod bsq;
mod bip;
mod bil;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileDims {
    pub bands: Vec<u64>,
    pub samples: usize,
    pub lines: usize,
}

impl From<&Headers> for FileDims {
    fn from(headers: &Headers) -> Self {
        let lines = headers.lines;
        let bands = (0..headers.bands as u64).collect();
        let samples = headers.samples;

        Self {
            bands,
            samples,
            lines,
        }
    }
}