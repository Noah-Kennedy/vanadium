use std::marker::PhantomData;

use crate::container::IterableImageMut;
use crate::container::mapped::Bip;

#[derive(Clone)]
pub struct BipBandIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipAllBandsIterMut<'a, T> {
    start: *mut T,
    count: usize,
    jump: usize,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipBandIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_mut();
                self.start = self.start.add(self.num_bands);
                v
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for BipAllBandsIterMut<'a, T> where T: Copy {
    type Item = BipBandIterMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.num_bands {
            self.count += 1;
            unsafe {
                let r = Some(BipBandIterMut {
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
pub struct BipSampleIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipAllSamplesIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipSampleIterMut<'a, T> where T: Copy {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_mut();
                self.start = self.start.add(1);
                v
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for BipAllSamplesIterMut<'a, T> where T: Copy {
    type Item = BipSampleIterMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                self.start = self.start.add(self.num_bands);

                Some(BipSampleIterMut {
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

impl<'a, C, T> IterableImageMut<'a, T> for Bip<C, T>
    where T: 'static + Copy,
          C: AsMut<[u8]>
{
    type BandMut = BipBandIterMut<'a, T>;
    type SampleMut = BipSampleIterMut<'a, T>;
    type BandsMut = BipAllBandsIterMut<'a, T>;
    type SamplesMut = BipAllSamplesIterMut<'a, T>;

    fn bands_mut(&mut self) -> Self::BandsMut {
        unsafe {
            Self::BandsMut {
                start: self.container.inner_mut().as_mut_ptr(),
                count: 0,
                jump: self.dims.samples * self.dims.lines * self.dims.channels,
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    fn samples_mut(&mut self) -> Self::SamplesMut {
        unsafe {
            Self::SamplesMut {
                start: self.container.inner_mut().as_mut_ptr(),
                end: self.container.inner_mut()
                    .as_mut_ptr()
                    .add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    fn band_mut(&mut self, index: usize) -> Self::BandMut {
        unsafe {
            let start = self.container.inner_mut()
                .as_mut_ptr()
                .add(index);

            Self::BandMut {
                start,
                end: start.add(self.dims.channels),
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    fn sample_mut(&mut self, index: usize) -> Self::SampleMut {
        unsafe {
            let start = self.container.inner_mut()
                .as_mut_ptr()
                .add(index * self.dims.channels);

            Self::SampleMut {
                start,
                end: start.add(self.dims.channels),
                _phantom: Default::default(),
            }
        }
    }
}