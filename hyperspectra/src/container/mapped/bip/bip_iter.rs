use std::marker::PhantomData;

use crate::container::IterableImage;
use crate::container::mapped::Bip;

#[derive(Clone)]
pub struct BipBandIter<'a, T> {
    start: *const T,
    end: *const T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipAllBandsIter<'a, T> {
    start: *const T,
    count: usize,
    jump: usize,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipBandIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let v = self.start.as_ref();
                self.start = self.start.add(self.num_bands);
                v
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for BipAllBandsIter<'a, T> where T: Copy {
    type Item = BipBandIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.num_bands {
            self.count += 1;
            unsafe {
                let r = Some(BipBandIter {
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

#[derive(Clone)]
pub struct BipSampleIter<'a, T> {
    start: *const T,
    end: *const T,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipAllSamplesIter<'a, T> {
    start: *const T,
    end: *const T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipSampleIter<'a, T> where T: Copy {
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

impl<'a, T> Iterator for BipAllSamplesIter<'a, T> where T: Copy {
    type Item = BipSampleIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let x = Some(BipSampleIter {
                    start: self.start,
                    end: self.start.add(self.num_bands),
                    _phantom: Default::default(),
                });

                self.start = self.start.add(self.num_bands);

                x
            }
        } else {
            None
        }
    }
}

impl<'a, C, T> IterableImage<'a, T> for Bip<C, T>
    where T: 'static + Copy,
          C: AsRef<[u8]>
{
    type Band = BipBandIter<'a, T>;
    type Sample = BipSampleIter<'a, T>;
    type Bands = BipAllBandsIter<'a, T>;
    type Samples = BipAllSamplesIter<'a, T>;

    fn bands(&self) -> Self::Bands {
        unsafe {
            Self::Bands {
                start: self.container.inner().as_ptr(),
                count: 0,
                jump: self.dims.samples * self.dims.lines * self.dims.channels,
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    fn samples(&self) -> Self::Samples {
        unsafe {
            Self::Samples {
                start: self.container.inner().as_ptr(),
                end: self.container.inner()
                    .as_ptr()
                    .add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    fn band(&self, index: usize) -> Self::Band {
        unsafe {
            let start = self.container.inner()
                .as_ptr()
                .add(index);

            Self::Band {
                start,
                end: start.add(self.dims.channels),
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    fn sample(&self, index: usize) -> Self::Sample {
        unsafe {
            let start = self.container.inner()
                .as_ptr()
                .add(index * self.dims.channels);

            Self::Sample {
                start,
                end: start.add(self.dims.channels),
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

    const SAMPLES: [[u32; 3]; 3] = [
        [11, 12, 13],
        [21, 22, 23],
        [31, 32, 33],
    ];

    const BANDS: [[u32; 3]; 3] = [
        [11, 21, 31],
        [12, 22, 32],
        [13, 23, 33],
    ];

    #[test]
    fn test_bsq_bands() {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT.clone()) };
        let mat: Bip<_, u32> = Bip {
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
        let mat: Bip<_, u32> = Bip {
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