use std::error::Error;
use std::fs::File;
use std::mem;

use memmap2::{Mmap, MmapMut, MmapOptions};
use serde::export::PhantomData;

use crate::container::{ImageDims, IterableImage, SizedImage};
use crate::container::mapped::SpectralImageContainer;
use crate::header::{FileByteOrder, Headers, Interleave};

pub struct Bip<C, T> {
    dims: ImageDims,
    container: SpectralImageContainer<C, T>,
}

impl<C, T> SizedImage for Bip<C, T> {
    fn dims(&self) -> ImageDims {
        self.dims.clone()
    }
}

#[derive(Clone)]
pub struct BipBandIter<'a, T> {
    start: *mut T,
    end: *mut T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipAllBandsIter<'a, T> {
    start: *mut T,
    count: usize,
    jump: usize,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipBandIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_ref();
                self.start = self.start.add(self.num_bands);
                v
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for BipAllBandsIter<'a, T> where T: Copy {
    type Item = BipBandIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.num_bands {
            self.count += 1;
            unsafe {
                let r = Some(BipBandIter {
                    start: self.start,
                    end: self.start.add(self.jump),
                    num_bands: self.num_bands,
                    _phantom: Default::default(),
                });

                self.start = self.start.add(1);

                r
            }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct BipSampleIter<'a, T> {
    start: *mut T,
    end: *mut T,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipAllSamplesIter<'a, T> {
    start: *mut T,
    end: *mut T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipSampleIter<'a, T> where T: Copy {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_ref();
                self.start = self.start.add(1);
                v
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for BipAllSamplesIter<'a, T> where T: Copy {
    type Item = BipSampleIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                self.start = self.start.add(self.num_bands);

                Some(BipSampleIter {
                    start: self.start,
                    end: self.start.add(self.num_bands),
                    _phantom: Default::default(),
                })
            }
        } else {
            None
        }
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

impl<'a, C, T> IterableImage<'a, T> for Bip<C, T>
    where T: 'static + Copy
{
    type Band = BipBandIter<'a, T>;
    type Sample = BipSampleIter<'a, T>;
    type Bands = BipAllBandsIter<'a, T>;
    type Samples = BipAllSamplesIter<'a, T>;

    fn bands(&self) -> Self::Bands {
        unimplemented!()
    }

    fn samples(&self) -> Self::Samples {
        unimplemented!()
    }

    fn band(&self, index: usize) -> Self::Band {
        unimplemented!()
    }

    fn pixel(&self, index: usize) -> Self::Sample {
        unimplemented!()
    }
}
