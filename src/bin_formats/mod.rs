pub use file_buffer::*;
pub use mat::*;

pub mod bsq;
pub mod bip;
pub mod bil;
pub mod error;
mod file_buffer;
mod mat;

const DEFAULT_WORK_UNIT_SIZE: usize = 2097152;
pub static mut WORK_UNIT_SIZE: usize = DEFAULT_WORK_UNIT_SIZE;