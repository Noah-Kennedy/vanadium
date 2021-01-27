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

unsafe impl <'a, T> Send for BsqSampleIter<'a, T> {}

#[derive(Clone)]
pub struct BsqAllSamplesIter<'a, T> {
    start: *const T,
    count: usize,
    jump: usize,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl <'a, T> Send for BsqAllSamplesIter<'a, T> {}

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

unsafe impl <'a, T> Send for BsqChannelIter<'a, T> {}

#[derive(Clone)]
pub struct BsqAllChannelsIter<'a, T> {
    start: *const T,
    end: *const T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl <'a, T> Send for BsqAllChannelsIter<'a, T> {}

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
                let x = Some(BsqChannelIter {
                    start: self.start,
                    end: self.start.add(self.num_samples),
                    _phantom: Default::default(),
                });

                self.start = self.start.add(self.num_samples);

                x
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
                end: start.add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_samples: self.dims.lines * self.dims.samples,
                _phantom: Default::default(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::container::ImageDims;
    use crate::container::mapped::SpectralImageContainer;

    use super::*;

    const MAT: [u32; 9] = [
        11, 12, 13,
        21, 22, 23,
        31, 32, 33,
    ];

    const BANDS: [[u32; 3]; 3] = [
        [11, 12, 13],
        [21, 22, 23],
        [31, 32, 33],
    ];

    const SAMPLES: [[u32; 3]; 3] = [
        [11, 21, 31],
        [12, 22, 32],
        [13, 23, 33],
    ];

    #[test]
    fn test_bsq_bands() {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT.clone()) };
        let mat: Bsq<_, u32> = Bsq {
            dims: ImageDims {
                channels: 3,
                lines: 1,
                samples: 3,
            },
            container: SpectralImageContainer {
                container: c.to_vec(),
                phantom: Default::default(),
            },
        };

        for (ba, be) in mat.bands().zip(BANDS.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bsq_samples() {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT.clone()) };
        let mat: Bsq<_, u32> = Bsq {
            dims: ImageDims {
                channels: 3,
                lines: 1,
                samples: 3,
            },
            container: SpectralImageContainer {
                container: c.to_vec(),
                phantom: Default::default(),
            },
        };

        for (ba, be) in mat.samples().zip(SAMPLES.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }
}