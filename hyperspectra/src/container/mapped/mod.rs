use std::marker::PhantomData;
use std::mem;

pub use bip::*;

struct SpectralImageContainer<C, T> {
    pub container: C,
    pub phantom: PhantomData<T>,
}

impl<C, T> SpectralImageContainer<C, T> where C: AsRef<[u8]> {
    unsafe fn inner(&self) -> &[T] {
        mem::transmute(self.container.as_ref())
    }
}

mod bip;

mod bsq;