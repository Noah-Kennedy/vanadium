use crate::bin_formats::error::ConversionError;

pub mod bsq;
pub mod bip;
pub mod error;

const WORK_UNIT_SIZE: usize = 32768;
// const WORK_UNIT_SIZE: usize = 65536;

pub trait OutOfPlaceConvert<T> {
    fn convert_into(&self, out: &mut T) -> Result<(), ConversionError>;
}