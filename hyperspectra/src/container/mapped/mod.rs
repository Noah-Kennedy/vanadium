use std::marker::PhantomData;

pub use bip::*;
pub use bsq::*;

pub (crate) struct SpectralImageContainer<C, T> {
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<C, T> SpectralImageContainer<C, T> where C: AsRef<[u8]> {
    unsafe fn inner(&self) -> &[T] {
        &*(self.container.as_ref() as *const [u8] as *const [T])
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: AsMut<[u8]> {
    unsafe fn inner_mut(&mut self) -> &mut [T] {
        &mut *(self.container.as_mut() as *mut [u8] as *mut [T])
    }
}

mod bip;

mod bsq;