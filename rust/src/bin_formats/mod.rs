use std::ops::DerefMut;

use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::ConversionError;
use crate::bin_formats::bip::Bip;

pub mod bsq;
pub mod bip;
pub mod error;

const WORK_UNIT_SIZE: usize = 65536;

pub trait OperableFile<T, C> {
    fn to_bsq(&self, out: &mut Bsq<C, T>) -> Result<(), ConversionError>;
    fn to_bip(&self, out: &mut Bip<C, T>) -> Result<(), ConversionError>;
}