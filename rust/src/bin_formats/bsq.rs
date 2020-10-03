use std::collections::HashSet;
use std::marker::PhantomData;
use std::{mem, slice};
use std::ops::{Deref, DerefMut};

use memmap::{Mmap, MmapMut};
use rayon::prelude::*;

use crate::bin_formats::THREAD_WORK_UNIT_SIZE;
use crate::headers::envi::{EnviByteOrder, EnviDataType};
use crate::prelude::EnviHeaders;

pub struct BsqData<C, T> {
    pub(crate) bands: usize,
    pub(crate) samples: usize,
    pub(crate) container: C,
    pub(crate) phantom: PhantomData<T>,
}

impl<C, T> BsqData<C, T> where C: Deref<Target=[u8]> {
    pub fn from_raw(headers: &EnviHeaders, raw: C) -> Self {
        assert_eq!(EnviByteOrder::Intel, headers.byte_order);
        assert_eq!(0, headers.header_offset);
        assert_eq!(raw.len() / mem::size_of::<T>(),
                   headers.bands * headers.samples * headers.lines);

        let bands = headers.bands;
        let samples = headers.samples;

        Self {
            bands,
            samples,
            container: raw,
            phantom: Default::default(),
        }
    }

    pub(crate) unsafe fn slice(&self) -> &[T] {
        let ptr = self.container.as_ptr() as *mut T;
        let len = self.samples * self.bands;
        slice::from_raw_parts(ptr, len)
    }

    pub(crate) unsafe fn band(&self, band: usize) -> &[T] {
        assert!(band < self.bands);

        &self.slice()[(band * self.samples)..((band * self.samples) + self.samples)]
    }
}

impl<C, T> BsqData<C, T> where C: DerefMut<Target=[u8]> {
    pub(crate) unsafe fn slice_mut(&mut self) -> &mut [T] {
        let ptr = self.container.as_mut_ptr() as *mut T;
        let len = self.samples * self.bands;
        slice::from_raw_parts_mut(ptr, len)
    }

    pub(crate) unsafe fn band_mut(&mut self, band: usize) -> &mut [T] {
        assert!(band < self.bands);
        let len = self.samples;
        let offset = band * len;

        &mut self.slice_mut()[offset..(offset + len)]
    }
}