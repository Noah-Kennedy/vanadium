use std::marker::PhantomData;

use either::Either;

use crate::container::{chunk_size, IterableImage};
use crate::container::mapped::Bip;

#[derive(Clone)]
pub struct BipBandIter<'a, T> {
    start: *const T,
    end: *const T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BipBandIter<'a, T> {}

#[derive(Clone)]
pub struct BipAllBandsIter<'a, T> {
    start: *const T,
    count: usize,
    jump: usize,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BipAllBandsIter<'a, T> {}


impl<'a, T> Iterator for BipBandIter<'a, T> {
    type Item = &'a T;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

unsafe impl<'a, T> Send for BipSampleIter<'a, T> {}

#[derive(Clone)]
pub struct BipAllSamplesIter<'a, T> {
    start: *const T,
    end: *const T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

#[derive(Clone)]
pub struct BipSamplesChunkedIter<'a, T> {
    start: *const T,
    end: *const T,
    num_bands: usize,
    jump: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BipSamplesChunkedIter<'a, T> {}

impl<'a, T> Iterator for BipSamplesChunkedIter<'a, T> where T: Copy {
    type Item = BipAllSamplesIter<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let start = self.start;
                self.start = start.add(self.jump);

                Some(Self::Item {
                    start,
                    end: self.start,
                    num_bands: self.num_bands,
                    _phantom: Default::default(),
                })
            }
        } else {
            None
        }
    }
}

unsafe impl<'a, T> Send for BipAllSamplesIter<'a, T> {}

#[derive(Clone)]
pub struct BipSampleIter<'a, T> {
    start: *const T,
    end: *const T,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for BipSampleIter<'a, T> where T: Copy {
    type Item = &'a T;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let start = self.start;

                self.start = self.start.add(self.num_bands);

                let end = self.start;

                Some(BipSampleIter {
                    start,
                    end,
                    _phantom: Default::default(),
                })
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
    type SamplesChunked = BipSamplesChunkedIter<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn fastest(&self) -> Either<Self::Bands, Self::Samples> {
        Either::Right(self.samples())
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn band(&self, index: usize) -> Self::Band {
        unsafe {
            let start = self.container.inner()
                .as_ptr()
                .add(index);

            let end = self.container.inner()
                .as_ptr()
                .add(self.dims.channels * self.dims.samples * self.dims.lines);

            Self::Band {
                start,
                end,
                num_bands: self.dims.channels,
                _phantom: Default::default(),
            }
        }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    fn samples_chunked(&self) -> Self::SamplesChunked {
        unsafe {
            Self::SamplesChunked {
                start: self.container.inner().as_ptr(),
                end: self.container.inner()
                    .as_ptr()
                    .add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_bands: self.dims.channels,
                jump: chunk_size::<T>(&self.dims),
                _phantom: Default::default(),
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
    use std::mem;

    use rand::distributions::Standard;
    use rand::Rng;

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

    struct BipTest {
        bands: Vec<Vec<u32>>,
        samples: Vec<Vec<u32>>,
        bip: Bip<Vec<u8>, u32>,
    }

    fn make_bip() -> Bip<Vec<u8>, u32> {
        let c: [u8; 9 * 4] = unsafe { mem::transmute(MAT) };
        Bip {
            dims: ImageDims {
                channels: 3,
                lines: 1,
                samples: 3,
            },
            container: SpectralImageContainer {
                container: c.to_vec(),
                phantom: Default::default(),
            },
        }
    }

    fn make_big_bip(dims: &ImageDims) -> BipTest {
        let num_pixels = dims.lines * dims.samples;
        let num_bands = dims.channels;

        let mut bands = vec![Vec::with_capacity(num_pixels); num_bands];

        let mut samples = Vec::with_capacity(num_pixels);

        let mut buffer = Vec::with_capacity(num_bands * num_pixels);

        for _ in 0..num_pixels {
            let pixel: Vec<u32> = rand::thread_rng()
                .sample_iter(Standard)
                .take(num_bands)
                .collect();

            for (i, &b) in pixel.iter().enumerate() {
                bands[i].push(b);
            }

            buffer.extend_from_slice(&pixel);

            samples.push(pixel);
        }

        let mut container = Vec::with_capacity(4 * buffer.len());

        for b in buffer {
            container.extend_from_slice(&b.to_ne_bytes())
        }

        BipTest {
            bands,
            samples,
            bip: Bip {
                dims: dims.to_owned(),
                container: SpectralImageContainer {
                    container,
                    phantom: Default::default(),
                },
            },
        }
    }

    fn check_bands(test: &BipTest) {
        for (ba, be) in test.bip.bands().zip(test.bands.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    fn check_samples(test: &BipTest) {
        for (ba, be) in test.bip.samples().zip(test.samples.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_large_bip() {
        let dims = ImageDims {
            channels: 500,
            lines: 100,
            samples: 100
        };

        let test = make_big_bip(&dims);

        check_samples(&test);
        check_bands(&test);
    }

    #[test]
    fn test_bip_bands() {
        let mat: Bip<_, u32> = make_bip();

        for (ba, be) in mat.bands().zip(BANDS.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bip_samples() {
        let mat: Bip<_, u32> = make_bip();

        for (ba, be) in mat.samples().zip(SAMPLES.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bip_single_band() {
        let mat: Bip<_, u32> = make_bip();

        for (a, e) in mat.band(0).zip(BANDS[0].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.band(1).zip(BANDS[1].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.band(2).zip(BANDS[2].iter()) {
            assert_eq!(a, e);
        }
    }

    #[test]
    fn test_bip_single_sample() {
        let mat: Bip<_, u32> = make_bip();

        for (a, e) in mat.sample(0).zip(SAMPLES[0].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.sample(1).zip(SAMPLES[1].iter()) {
            assert_eq!(a, e);
        }

        for (a, e) in mat.sample(2).zip(SAMPLES[2].iter()) {
            assert_eq!(a, e);
        }
    }
}