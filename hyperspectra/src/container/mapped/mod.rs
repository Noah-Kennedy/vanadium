use std::marker::PhantomData;
use std::mem;

pub use bip::*;
pub use bsq::*;

struct SpectralImageContainer<C, T> {
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<C, T> SpectralImageContainer<C, T> where C: AsRef<[u8]> {
    unsafe fn inner(&self) -> &[T] {
        mem::transmute(self.container.as_ref())
    }
}

impl<C, T> SpectralImageContainer<C, T> where C: AsMut<[u8]> {
    unsafe fn inner_mut(&mut self) -> &mut [T] {
        mem::transmute(self.container.as_mut())
    }
}

mod bip;

mod bsq;