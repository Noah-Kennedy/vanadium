use std::error::Error;
use std::fs::File;
use std::mem;

use memmap2::{Mmap, MmapMut, MmapOptions};

pub use bsq_iter::*;
pub use bsq_iter_mut::*;

use crate::container::{ImageDims, SizedImage};
use crate::container::mapped::SpectralImageContainer;
use crate::header::{FileByteOrder, Headers, Interleave};

mod bsq_iter;
mod bsq_iter_mut;

pub struct Bsq<C, T> {
    dims: ImageDims,
    container: SpectralImageContainer<C, T>,
}

impl<C, T> SizedImage for Bsq<C, T> {
    fn dims(&self) -> ImageDims {
        self.dims.clone()
    }
}

impl<C, T> Bsq<C, T> {
    fn check_header_preconditions(headers: &Headers, file: &File) -> Result<(), Box<dyn Error>> {
        assert_eq!(
            FileByteOrder::Intel, headers.byte_order,
            "Only Intel byte order is supported"
        );

        assert_eq!(Interleave::Bsq, headers.interleave);

        assert_eq!(
            headers.bands * headers.lines * headers.samples * mem::size_of::<T>(),
            file.metadata()?.len() as usize,
            "File size does not match that expected from header"
        );

        Ok(())
    }
}

impl<T> Bsq<Mmap, T> {
    pub fn headers(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        Self::check_header_preconditions(headers, file)?;

        let raw = unsafe {
            MmapOptions::new()
                .offset(headers.header_offset as u64)
                .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
                .map(&file)?
        };

        Ok(Self {
            dims: ImageDims::from(headers),
            container: SpectralImageContainer {
                container: raw,
                phantom: Default::default(),
            },
        })
    }
}

impl<T> Bsq<MmapMut, T> {
    pub fn headers_mut(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        Self::check_header_preconditions(headers, file)?;

        let raw = unsafe {
            MmapOptions::new()
                .offset(headers.header_offset as u64)
                .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
                .map_mut(&file)?
        };

        Ok(Self {
            dims: ImageDims::from(headers),
            container: SpectralImageContainer {
                container: raw,
                phantom: Default::default(),
            },
        })
    }
}