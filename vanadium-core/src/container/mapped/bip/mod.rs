use std::error::Error;
use std::fs::File;
use std::mem;

use memmap2::{Mmap, MmapMut, MmapOptions};

pub use bip_iter::*;
pub use bip_iter_mut::*;

use crate::container::{ImageDims, ImageIndex, IndexImage, IndexImageMut, SizedImage};
use crate::container::mapped::SpectralImageContainer;
use crate::header::{FileByteOrder, Headers, Interleave};

mod bip_iter;
mod bip_iter_mut;

pub struct Bip<C, T> {
    pub(crate) dims: ImageDims,
    pub(crate) container: SpectralImageContainer<C, T>,
}

impl<C, T> SizedImage for Bip<C, T> {
    fn dims(&self) -> ImageDims {
        self.dims.clone()
    }
}

impl<C, T> IndexImage<T> for Bip<C, T>
    where T: 'static + Copy,
          C: AsRef<[u8]>
{
    unsafe fn get_unchecked(&self, index: &ImageIndex) -> &T {
        let d = &self.dims;
        let channel_offset = index.channel;
        let sample_offset = index.sample * d.channels;
        let lines_offset = index.line * d.samples * d.channels;

        let off = channel_offset + sample_offset + lines_offset;

        self.container.inner().get_unchecked(off)
    }
}

impl<C, T> IndexImageMut<T> for Bip<C, T>
    where T: 'static + Copy,
          C: AsMut<[u8]>
{
    unsafe fn get_unchecked_mut(&mut self, index: &ImageIndex) -> &mut T {
        let d = &self.dims;
        let channel_offset = index.channel;
        let sample_offset = index.sample * d.channels;
        let lines_offset = index.line * d.samples * d.channels;

        let off = channel_offset + sample_offset + lines_offset;

        self.container.inner_mut().get_unchecked_mut(off)
    }
}

impl<C, T> Bip<C, T> {
    fn check_header_preconditions(headers: &Headers, file: &File) -> Result<(), Box<dyn Error>> {
        assert_eq!(
            FileByteOrder::Intel, headers.byte_order,
            "Only Intel byte order is supported"
        );

        assert_eq!(Interleave::Bip, headers.interleave);

        assert_eq!(
            headers.bands * headers.lines * headers.samples * mem::size_of::<T>(),
            file.metadata()?.len() as usize,
            "File size does not match that expected from header"
        );

        Ok(())
    }
}

impl<T> Bip<Mmap, T> {
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

impl<T> Bip<MmapMut, T> {
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