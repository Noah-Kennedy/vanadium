use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::ConversionError;

pub mod bsq;
pub mod bip;
pub mod error;

const WORK_UNIT_SIZE: usize = 2097152;

pub trait OperableFile<T, C> {
    fn to_bsq(&self, out: &mut Bsq<C, T>) -> Result<(), ConversionError>;
    fn to_bip(&self, out: &mut Bip<C, T>) -> Result<(), ConversionError>;
}

pub trait OperableExt<T> {
    fn rescale(&mut self, bands: &[usize], scale: T, offset: T) -> Result<(), ConversionError>;
    fn normalize(&mut self, bands: &[usize], floor: T, ceiling: T) -> Result<(), ConversionError>;
}