pub use file_buffer::*;
pub use mat::*;

pub mod bsq;
pub mod bip;
pub mod bil;
pub mod error;
mod file_buffer;
mod mat;

pub trait FileIndex: {
    fn order(&self) -> MatType;
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize;
}
