use std::error::Error;
use crate::header::Interleave;
use std::fs::File;

pub trait SpectralImage {

}



pub trait ImageConvert<T, E> where E: Error {
    fn convert(&self, file: File, format: Interleave) -> Result<T, E>;
}

pub trait ImageIndex: {
    fn order(&self) -> Interleave;
    fn get_idx(&self, line: usize, pixel: usize, band: usize) -> usize;
}