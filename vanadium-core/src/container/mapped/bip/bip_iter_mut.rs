use std::marker::PhantomData;

use either::Either;

use crate::container::IterableImageMut;
use crate::container::mapped::Bip;

#[derive(Clone)]
pub struct BipBandIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BipBandIterMut<'a, T> {}

#[derive(Clone)]
pub struct BipAllBandsIterMut<'a, T> {
    start: *mut T,
    count: usize,
    jump: usize,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BipAllBandsIterMut<'a, T> {}

impl<'a, T> Iterator for BipBandIterMut<'a, T> {
    type Item = &'a mut T;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

unsafe impl<'a, T> Send for BipSampleIterMut<'a, T> {}

#[derive(Clone)]
pub struct BipAllSamplesIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    num_bands: usize,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for BipAllSamplesIterMut<'a, T> {}

impl<'a, T> Iterator for BipSampleIterMut<'a, T> where T: Copy {
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

impl<'a, T> Iterator for BipAllSamplesIterMut<'a, T> where T: Copy {
    type Item = BipSampleIterMut<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            unsafe {
                let x = Some(BipSampleIterMut {
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

impl<'a, C, T> IterableImageMut<'a, T> for Bip<C, T>
    where T: 'static + Copy,
          C: AsMut<[u8]>
{
    type BandMut = BipBandIterMut<'a, T>;
    type SampleMut = BipSampleIterMut<'a, T>;
    type BandsMut = BipAllBandsIterMut<'a, T>;
    type SamplesMut = BipAllSamplesIterMut<'a, T>;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn fastest_mut(&mut self) -> Either<Self::BandsMut, Self::SamplesMut> {
        Either::Right(self.samples_mut())
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
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

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn band_mut(&mut self, index: usize) -> Self::BandMut {
        unsafe {
            let start = self.container.inner_mut()
                .as_mut_ptr()
                .add(index);

            Self::BandMut {
                start,
                end: start.add(self.dims.channels * self.dims.samples * self.dims.lines),
                num_bands: self.dims.channels,
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
                .add(index * self.dims.channels);

            Self::SampleMut {
                start,
                end: start.add(self.dims.channels),
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

    fn check_bands(test: &mut BipTest) {
        for (ba, be) in test.bip.bands_mut().zip(test.bands.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    fn check_samples(test: &mut BipTest) {
        for (ba, be) in test.bip.samples_mut().zip(test.samples.iter()) {
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
            samples: 100,
        };

        let mut test = make_big_bip(&dims);

        check_samples(&mut test);
        check_bands(&mut test);
    }

    #[test]
    fn test_bip_bands() {
        let mut mat: Bip<_, u32> = make_bip();

        for (ba, be) in mat.bands_mut().zip(BANDS.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bip_samples() {
        let mut mat: Bip<_, u32> = make_bip();

        for (ba, be) in mat.samples_mut().zip(SAMPLES.iter()) {
            for (ca, ce) in ba.zip(be.iter()) {
                assert_eq!(ca, ce);
            }
        }
    }

    #[test]
    fn test_bip_single_band() {
        let mut mat: Bip<_, u32> = make_bip();

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
    fn test_bip_single_sample() {
        let mut mat: Bip<_, u32> = make_bip();

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