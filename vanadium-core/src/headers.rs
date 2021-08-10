use std::path::PathBuf;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub dims: ImageDims,
    pub format: ImageFormat,
    pub path: PathBuf,
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