use std::marker::PhantomData;

use crate::container::IterableImageMut;
use crate::container::mapped::Bsq;

#[derive(Clone)]
pub struct BsqSampleIter<'a, T> {
    start: *mut T,
    end: *mut T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BsqAllSamplesIter<'a, T> {
    start: *mut T,
    count: usize,
    jump: usize,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BsqSampleIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_mut();
                self.start = self.start.add(self.num_samples);
                v
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for BsqAllSamplesIter<'a, T> where T: Copy {
    type Item = BsqSampleIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.num_samples {
            self.count += 1;
            unsafe {
                let r = Some(BsqSampleIter {
                    start: self.start,
                    end: self.start.add(self.jump),
                    num_samples: self.num_samples,
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
pub struct BsqChannelIter<'a, T> {
    start: *mut T,
    end: *mut T,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BsqAllChannelsIter<'a, T> {
    start: *mut T,
    end: *mut T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BsqChannelIter<'a, T> where T: Copy {
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

impl<'a, T> Iterator for BsqAllChannelsIter<'a, T> where T: Copy {
    type Item = BsqChannelIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                self.start = self.start.add(self.num_samples);

                Some(BsqChannelIter {
                    start: self.start,
                    end: self.start.add(self.num_samples),
                    _phantom: Default::default(),
                })
            }
        } else {
            None
        }
    }
}

impl<'a, C, T> IterableImageMut<'a, T> for Bsq<C, T>
    where T: 'static + Copy,
          C: AsMut<[u8]>
{
    type BandMut = BsqChannelIter<'a, T>;
    type SampleMut = BsqSampleIter<'a, T>;
    type BandsMut = BsqAllChannelsIter<'a, T>;
    type SamplesMut = BsqAllSamplesIter<'a, T>;

    fn bands_mut(&mut self) -> Self::BandsMut {
        unsafe {
            Self::BandsMut {
                start: self.container.inner_mut().as_mut_ptr(),
                end: self.container.inner_mut()
                    .as_mut_ptr()
                    .add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_samples: self.dims.samples * self.dims.lines,
                _phantom: Default::default(),
            }
        }
    }

    fn samples_mut(&mut self) -> Self::SamplesMut {
        unsafe {
            Self::SamplesMut {
                start: self.container.inner_mut().as_mut_ptr(),
                count: 0,
                jump: self.dims.samples * self.dims.lines * self.dims.channels,
                num_samples: self.dims.samples * self.dims.lines,
                _phantom: Default::default(),
            }
        }
    }

    fn band_mut(&mut self, index: usize) -> Self::BandMut {
        unsafe {
            let start = self.container.inner_mut()
                .as_mut_ptr()
                .add(index * self.dims.lines * self.dims.samples);

            Self::BandMut {
                start,
                end: start.add(self.dims.lines * self.dims.samples),
                _phantom: Default::default(),
            }
        }
    }

    fn sample_mut(&mut self, index: usize) -> Self::SampleMut {
        unsafe {
            let start = self.container.inner_mut()
                .as_mut_ptr()
                .add(index);

            Self::SampleMut {
                start,
                end: start.add(self.dims.lines * self.dims.samples),
                num_samples: self.dims.lines * self.dims.samples,
                _phantom: Default::default(),
            }
        }
    }
}