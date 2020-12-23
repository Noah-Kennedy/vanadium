pub use buffer::*;
pub use error::*;
pub use indexing::*;
pub use mat::*;
pub use transforms::*;

mod indexing;
mod error;
mod buffer;
mod mat;
mod transforms;
mod util;

pub trait ImageIndex: {
    fn order(&self) -> MatType;
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize;
}
