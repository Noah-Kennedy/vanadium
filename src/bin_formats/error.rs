use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

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
    pub input_size: (usize, usize, usize),
    pub output_size: (usize, usize, usize),
}

impl Display for SizeMismatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "SizeMismatchError: Input size was ({}, {}, {}), output size was ({}, {}, {}).",
            self.input_size.0,
            self.input_size.1,
            self.input_size.2,
            self.output_size.0,
            self.output_size.1,
            self.output_size.2
        )
    }
}

impl Error for SizeMismatchError {}