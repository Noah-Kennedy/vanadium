use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

use memmap2::{Mmap, MmapMut, MmapOptions};

use envi_header::{FileByteOrder, Headers};
use envi_image::FileDims;

#[derive(Copy, Clone)]
pub struct FileBuf<T>(pub(crate) *const T);

#[derive(Copy, Clone)]
pub struct FileBufMut<T>(pub(crate) *mut T);

unsafe impl<T> Send for FileBuf<T> where T: Send {}

unsafe impl<T> Send for FileBufMut<T> where T: Send {}

pub struct SpectralImageContainer<C, T> {
    pub dims: FileDims,
    pub container: C,
    pub phantom: PhantomData<T>,
}



impl<C, T> SpectralImageContainer<C, T> {
    fn check_header_preconditions(headers: &Headers, file: &File) -> Result<(), Box<dyn Error>> {
        assert_eq!(
            FileByteOrder::Intel, headers.byte_order,
            "Only Intel byte order is supported"
        );
        assert_eq!(
            headers.bands * headers.lines * headers.samples * mem::size_of::<T>(),
            file.metadata()?.len() as usize,
            "File size does not match that expected from header"
        );

        Ok(())
    }
}

impl<T> SpectralImageContainer<Mmap, T> {
    pub fn headers(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        Self::check_header_preconditions(headers, file)?;

        let raw = unsafe {
            MmapOptions::new()
                .offset(headers.header_offset as u64)
                .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
                .map(&file)?
        };

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<T> SpectralImageContainer<MmapMut, T> {
    pub fn headers_mut(headers: &Headers, file: &File) -> Result<Self, Box<dyn Error>> {
        Self::check_header_preconditions(headers, file)?;

        let raw = unsafe {
            MmapOptions::new()
                .offset(headers.header_offset as u64)
                .len(headers.bands * headers.samples * headers.lines * mem::size_of::<T>())
                .map_mut(&file)?
        };

        Ok(Self {
            dims: FileDims::from(headers),
            container: raw,
            phantom: Default::default(),
        })
    }
}

impl<C, T> SpectralImageContainer<C, T> {
    pub fn size(&self) -> FileDims {
        self.dims.clone()
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: Deref<Target=[u8]> {
    /// Get a file buffer pointer
    ///
    /// # Safety
    /// This has all of the normal safety issues associated with raw pointers.
    /// Make sure that you are not indexing out of bounds.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> FileBuf<T> {
        FileBuf(self.container.as_ptr() as *const T)
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: DerefMut<Target=[u8]> {
    /// Get a mutable file buffer pointer
    ///
    /// # Safety
    /// This has all of the normal safety issues associated with raw pointers.
    /// Make sure that you are not indexing out of bounds.
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self) -> FileBufMut<T> {
        FileBufMut(self.container.as_mut_ptr() as *mut T)
    }
}