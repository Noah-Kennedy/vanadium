use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ConversionError {
    pub input_type: &'static str,
    pub output_type: &'static str,
    pub kind: ConversionErrorKind,
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error performing {}->{} conversion!", self.input_type, self.output_type)?;
        match self.kind {
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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SizeMismatchError {
    pub input_size: usize,
    pub output_size: usize,
}

impl Display for SizeMismatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "SizeMismatchError: Input size was {}, output size was {}.",
                 self.input_size,
                 self.output_size)
    }
}

impl Error for SizeMismatchError {}