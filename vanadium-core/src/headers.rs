use std::path::{Path};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header<P> where P: AsRef<Path> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub dims: ImageDims,
    pub format: ImageFormat,
    pub path: P,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImageDims {
    pub channels: usize,
    pub lines: usize,
    pub pixels: usize,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ImageFormat {
    Bip,
    Bsq,
}