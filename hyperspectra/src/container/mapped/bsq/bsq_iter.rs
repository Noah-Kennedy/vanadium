use std::marker::PhantomData;

use crate::container::IterableImage;
use crate::container::mapped::Bsq;

#[derive(Clone)]
pub struct BsqSampleIter<'a, T> {
    start: *const T,
    end: *const T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BsqAllSamplesIter<'a, T> {
    start: *const T,
    count: usize,
    jump: usize,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BsqSampleIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_ref();
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
    start: *const T,
    end: *const T,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BsqAllChannelsIter<'a, T> {
    start: *const T,
    end: *const T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BsqChannelIter<'a, T> where T: Copy {
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

impl<'a, C, T> IterableImage<'a, T> for Bsq<C, T>
    where T: 'static + Copy,
          C: AsRef<[u8]>
{
    type Band = BsqChannelIter<'a, T>;
    type Sample = BsqSampleIter<'a, T>;
    type Bands = BsqAllChannelsIter<'a, T>;
    type Samples = BsqAllSamplesIter<'a, T>;

    fn bands(&self) -> Self::Bands {
        unsafe {
            Self::Bands {
                start: self.container.inner().as_ptr(),
                end: self.container.inner()
                    .as_ptr()
                    .add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_samples: self.dims.samples * self.dims.lines,
                _phantom: Default::default(),
            }
        }
    }

    fn samples(&self) -> Self::Samples {
        unsafe {
            Self::Samples {
                start: self.container.inner().as_ptr(),
                count: 0,
                jump: self.dims.samples * self.dims.lines * self.dims.channels,
                num_samples: self.dims.samples * self.dims.lines,
                _phantom: Default::default(),
            }
        }
    }

    fn band(&self, index: usize) -> Self::Band {
        unsafe {
            let start = self.container.inner()
                .as_ptr()
                .add(index * self.dims.lines * self.dims.samples);

            Self::Band {
                start,
                end: start.add(self.dims.lines * self.dims.samples),
                _phantom: Default::default(),
            }
        }
    }

    fn sample(&self, index: usize) -> Self::Sample {
        unsafe {
            let start = self.container.inner()
                .as_ptr()
                .add(index);

            Self::Sample {
                start,
                end: start.add(self.dims.lines * self.dims.samples),
                num_samples: self.dims.lines * self.dims.samples,
                _phantom: Default::default(),
            }
        }
    }
}