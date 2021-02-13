use std::marker::PhantomData;

use either::Either;

use crate::container::IterableImageMut;
use crate::container::mapped::Bsq;

#[derive(Clone)]
pub struct BsqSampleIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BsqSampleIterMut<'a, T> {}

#[derive(Clone)]
pub struct BsqAllSamplesIterMut<'a, T> {
    start: *mut T,
    count: usize,
    jump: usize,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BsqAllSamplesIterMut<'a, T> {}

impl<'a, T> Iterator for BsqSampleIterMut<'a, T> {
    type Item = &'a mut T;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

impl<'a, T> Iterator for BsqAllSamplesIterMut<'a, T> where T: Copy {
    type Item = BsqSampleIterMut<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.num_samples {
            self.count += 1;
            unsafe {
                let r = Some(BsqSampleIterMut {
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
pub struct BsqChannelIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BsqChannelIterMut<'a, T> {}

#[derive(Clone)]
pub struct BsqAllChannelsIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    num_samples: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BsqAllChannelsIterMut<'a, T> {}

impl<'a, T> Iterator for BsqChannelIterMut<'a, T> where T: Copy {
    type Item = &'a mut T;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

impl<'a, T> Iterator for BsqAllChannelsIterMut<'a, T> where T: Copy {
    type Item = BsqChannelIterMut<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let x = Some(BsqChannelIterMut {
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

impl<'a, C, T> IterableImageMut<'a, T> for Bsq<C, T>
    where T: 'static + Copy,
          C: AsMut<[u8]>
{
    type BandMut = BsqChannelIterMut<'a, T>;
    type SampleMut = BsqSampleIterMut<'a, T>;
    type BandsMut = BsqAllChannelsIterMut<'a, T>;
    type SamplesMut = BsqAllSamplesIterMut<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn fastest_mut(&mut self) -> Either<Self::BandsMut, Self::SamplesMut> {
        Either::Left(self.bands_mut())
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

#[cfg(not(tarpaulin_include))]
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
        let mut mat: Bsq<_, u32> = Bsq {
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

        for (ba, be) in mat.bands_mut().zip(BANDS.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bsq_samples() {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT.clone()) };
        let mut mat: Bsq<_, u32> = Bsq {
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

        for (ba, be) in mat.samples_mut().zip(SAMPLES.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bsq_single_band_mut() {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT.clone()) };

        let mut mat: Bsq<_, u32> = Bsq {
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

        for (a, e) in mat.band_mut(0).zip(BANDS[0].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.band_mut(1).zip(BANDS[1].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.band_mut(2).zip(BANDS[2].iter()) {
            assert_eq!(a, e);
        }
    }

    #[test]
    fn test_bsq_single_sample_mut() {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT.clone()) };

        let mut mat: Bsq<_, u32> = Bsq {
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

        for (a, e) in mat.sample_mut(0).zip(SAMPLES[0].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.sample_mut(1).zip(SAMPLES[1].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.sample_mut(2).zip(SAMPLES[2].iter()) {
            assert_eq!(a, e);
        }
    }
}