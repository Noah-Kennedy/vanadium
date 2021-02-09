use std::marker::PhantomData;
use std::{slice, mem};

pub use bip::*;
pub use bsq::*;

pub(crate) struct SpectralImageContainer<C, T> {
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<C, T> SpectralImageContainer<C, T> where C: AsRef<[u8]>, T: Copy {
    unsafe fn inner(&self) -> &[T] {
        let inner = self.container.as_ref();
        let len = inner.len();
        let data = inner.as_ptr() as *const u8 as *const T;

        slice::from_raw_parts(data, len / mem::size_of::<T>())
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: AsMut<[u8]>, T: Copy {
    unsafe fn inner_mut(&mut self) -> &mut [T] {
        let inner = self.container.as_mut();
        let len = inner.len();
        let data = inner.as_ptr() as *mut u8 as *mut T;

        slice::from_raw_parts_mut(data, len / mem::size_of::<T>())
    }
}

mod bip;

mod bsq;