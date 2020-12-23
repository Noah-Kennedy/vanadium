use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

use crate::FileDims;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ConversionError {
    pub input_type: &'static str,
    pub output_type: &'static str,
    pub kind: ConversionErrorKind,
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error performing {}->{} conversion!", self.input_type, self.output_type)?;
        match self.kind.clone() {
            ConversionErrorKind::SizeMismatch(e) => e.fmt(f)
        }
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ConversionErrorKind::SizeMismatch(e) => Some(e),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ConversionErrorKind {
    SizeMismatch(SizeMismatchError)
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SizeMismatchError {
    pub input_size: FileDims,
    pub output_size: FileDims,
}

impl Display for SizeMismatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "SizeMismatchError: Input size was {:?}, output size was {:?}.",
            &self.input_size,
            &self.output_size
        )
    }
}

impl Error for SizeMismatchError {}