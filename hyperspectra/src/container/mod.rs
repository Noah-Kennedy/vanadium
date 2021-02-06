use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use convert::*;
pub use pca::*;
pub use stat::*;
pub use render::*;

use crate::header::Headers;
use either::Either;

pub mod mapped;

mod pca;
mod stat;
mod convert;
mod render;

const CHUNK_SIZE: usize = 4096*4096;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug)]
pub struct ImageDims {
    /// bands in image
    pub channels: usize,
    /// Lines in image
    pub lines: usize,
    /// Pixels in image
    pub samples: usize,
}

impl From<&Headers> for ImageDims {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn from(headers: &Headers) -> Self {
        ImageDims {
            channels: headers.bands,
            lines: headers.lines,
            samples: headers.samples,
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug)]
pub struct ImageIndex {
    pub channel: usize,
    pub line: usize,
    pub sample: usize,
}

pub trait SizedImage {
    fn dims(&self) -> ImageDims;
}

pub trait IndexImage<T> {
    /// # Safety
    /// This function is safe if the index is within the bounds of the image
    unsafe fn get_unchecked(&self, index: &ImageIndex) -> &T;
}

pub trait IndexImageMut<T> {
    /// # Safety
    /// This function is safe if the index is within the bounds of the image
    unsafe fn get_unchecked_mut(&mut self, index: &ImageIndex) -> &mut T;
}

pub struct LockImage<T, I> {
    inner: RwLock<I>,
    _phantom: PhantomData<T>,
}

pub struct ReadImageGuard<'a, T, I> {
    pub inner: RwLockReadGuard<'a, I>,
    _phantom: PhantomData<T>,
}

pub struct WriteImageGuard<'a, T, I> {
    pub inner: RwLockWriteGuard<'a, I>,
    _phantom: PhantomData<T>,
}

impl<T, I> LockImage<T, I> where T: 'static, I: 'static {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn new(inner: I) -> Self {
        Self {
            inner: RwLock::new(inner),
            _phantom: Default::default(),
        }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn read(&self) -> ReadImageGuard<T, I> {
        ReadImageGuard { inner: self.inner.read().unwrap(), _phantom: Default::default() }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    pub fn write(&self) -> WriteImageGuard<T, I> {
        WriteImageGuard { inner: self.inner.write().unwrap(), _phantom: Default::default() }
    }
}

pub trait IterableImage<'a, T: 'static>: SizedImage {
    type Band: Iterator<Item=&'a T> + Clone + Send;
    type Sample: Iterator<Item=&'a T> + Clone + Send;
    type Bands: Iterator<Item=Self::Band> + Clone + Send;
    type Samples: Iterator<Item=Self::Sample> + Clone + Send;
    type SamplesChunked: Iterator<Item=Self::Samples> + Clone + Send;

    fn fastest(&self) -> Either<Self::Bands, Self::Samples>;

    fn bands(&self) -> Self::Bands;
    fn samples(&self) -> Self::Samples;

    fn band(&self, index: usize) -> Self::Band;
    fn sample(&self, index: usize) -> Self::Sample;

    fn samples_chunked(&self) -> Self::SamplesChunked;
}

pub trait IterableImageMut<'a, T: 'static>: SizedImage {
    type BandMut: Iterator<Item=&'a mut T> + Send;
    type SampleMut: Iterator<Item=&'a mut T> + Send;
    type BandsMut: Iterator<Item=Self::BandMut> + Send;
    type SamplesMut: Iterator<Item=Self::SampleMut> + Send;

    fn fastest_mut(&mut self) -> Either<Self::BandsMut, Self::SamplesMut>;

    fn bands_mut(&mut self) -> Self::BandsMut;
    fn samples_mut(&mut self) -> Self::SamplesMut;

    fn band_mut(&mut self, index: usize) -> Self::BandMut;
    fn sample_mut(&mut self, index: usize) -> Self::SampleMut;
}