use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Div, Sub};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;

use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{ComplexField, DMatrix, Dynamic, RealField, SymmetricEigen};
use num::{Bounded, Zero};
use num::traits::NumAssign;
use rayon::prelude::*;

use crate::bar::config_bar;
use crate::header::Headers;

pub mod mapped;

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
    unsafe fn get_unchecked(&self, index: &ImageIndex) -> &T;
}

pub trait IndexImageMut<T> {
    unsafe fn get_unchecked_mut(&mut self, index: &ImageIndex) -> &mut T;
}

pub struct LockImage<T, I> {
    inner: RwLock<I>,
    _phantom: PhantomData<T>,
}

pub struct ReadImageGuard<'a, T, I> {
    inner: RwLockReadGuard<'a, I>,
    _phantom: PhantomData<T>,
}

pub struct WriteImageGuard<'a, T, I> {
    inner: RwLockWriteGuard<'a, I>,
    _phantom: PhantomData<T>,
}

impl<T, I> LockImage<T, I> where T: 'static, I: 'static {
    pub fn new(inner: I) -> Self {
        Self {
            inner: RwLock::new(inner),
            _phantom: Default::default(),
        }
    }
    pub fn read(&self) -> ReadImageGuard<T, I> {
        ReadImageGuard { inner: self.inner.read().unwrap(), _phantom: Default::default() }
    }

    pub fn write(&self) -> WriteImageGuard<T, I> {
        WriteImageGuard { inner: self.inner.write().unwrap(), _phantom: Default::default() }
    }
}

pub trait PCA<T> where T: PartialEq + Copy + Debug + 'static {
    fn pca(&self, out: &Self, kept: usize, verbose: bool, min: Option<T>, max: Option<T>);
}

pub trait IterableImage<'a, T: 'static>: SizedImage {
    type Band: Iterator<Item=&'a T> + Clone + Send;
    type Sample: Iterator<Item=&'a T> + Clone + Send;
    type Bands: Iterator<Item=Self::Band> + Clone + Send;
    type Samples: Iterator<Item=Self::Sample> + Clone + Send;

    fn bands(&self) -> Self::Bands;
    fn samples(&self) -> Self::Samples;

    fn band(&self, index: usize) -> Self::Band;
    fn sample(&self, index: usize) -> Self::Sample;
}

pub trait IterableImageMut<'a, T: 'static>: SizedImage {
    type BandMut: Iterator<Item=&'a mut T> + Send;
    type SampleMut: Iterator<Item=&'a mut T> + Send;
    type BandsMut: Iterator<Item=Self::BandMut> + Send;
    type SamplesMut: Iterator<Item=Self::SampleMut> + Send;

    fn bands_mut(&mut self) -> Self::BandsMut;
    fn samples_mut(&mut self) -> Self::SamplesMut;

    fn band_mut(&mut self, index: usize) -> Self::BandMut;
    fn sample_mut(&mut self, index: usize) -> Self::SampleMut;
}

impl<'a, I, T> PCA<T> for LockImage<T, I>
    where I: IterableImage<'a, T> + Sync + IterableImageMut<'a, T>,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded
          + Display + ComplexField + ComplexField<RealField=T> + RealField + Sum
{
    fn pca(&self, out: &Self, kept: usize, verbose: bool, min: Option<T>, max: Option<T>) {
        let min = min.unwrap_or_else(T::min_value);
        let max = max.unwrap_or_else(T::max_value);

        let a = &self.inner;
        let b = a.read();
        let c = b.unwrap();

        let input = ReadImageGuard { inner: c, _phantom: Default::default() };

        let mp = Arc::new(MultiProgress::new());

        let stages_bar = mp.add(ProgressBar::new(5));
        config_bar(&stages_bar, "Performing PCA stages...");

        let mm2 = mp.clone();

        let j = thread::Builder::new()
            .name("progbar-manager".to_owned())
            .spawn(move || {
                mm2.join_and_clear().unwrap();
            }).unwrap();

        stages_bar.set_message("Stage: Averages");
        let means: Vec<_> = input.all_band_means(&mp, min, max);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Averages:");
            let message = format!("{:#?}", &means);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Standard Deviations");
        let std_devs: Vec<_> = input.all_band_std_devs(&mp, &means, min, max);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Standard Deviations:");
            let message = format!("{:#?}", &std_devs);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Covariances");
        let covariances = input.covariance_matrix(&mp, &means, min, max);
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Covariances:");
            let message = format!("{}", covariances);
            stages_bar.println(message);
        }

        stages_bar.set_message("Stage: Eigendecomposition");
        let eigen = covariances.symmetric_eigen();
        stages_bar.inc(1);

        if verbose {
            stages_bar.println("Eigen:");
            let message = format!("{:#?}", eigen);
            stages_bar.println(message);
        }

        let mut output = WriteImageGuard { inner: out.inner.write().unwrap(), _phantom: Default::default() };

        stages_bar.set_message("Stage: Writes");
        input.write_standardized_results(&mut output, &mp, kept, &means, &std_devs, &eigen);
        stages_bar.inc(1);

        stages_bar.finish();

        j.join().unwrap();
    }
}

impl<'a, 'b, I, T> ReadImageGuard<'a, T, I>
    where I: IterableImage<'b, T> + Sync + IterableImageMut<'b, T>,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded
          + Display + ComplexField + ComplexField<RealField=T> + RealField + Sum
{
    fn band_mean(&self, band: usize, min: T, max: T) -> T {
        let mut sum = T::zero();
        let mut count = T::zero();

        for &x in self.inner.band(band) {
            let include = x > min && x <= max;

            if include {
                sum += x;
                count += T::one();
            }
        }

        sum / count
    }

    fn band_std_dev(&self, band: usize, mean: Option<T>, min: T, max: T) -> T {
        let mean = if let Some(mean) = mean {
            mean
        } else {
            self.band_mean(band, min, max)
        };

        let mut sum = T::zero();
        let mut count = T::zero();

        for &x in self.inner.band(band) {
            let diff = x - mean;

            if x > min && x <= max {
                sum += diff * diff;
                count += T::one();
            }
        }

        (sum / count).sqrt()
    }

    fn covariance_pair(&self, bands: [usize; 2], means: [T; 2], min: T, max: T) -> T {
        let mut sum = T::zero();
        let mut count = T::zero();

        let itc = self.inner.band(bands[0])
            .zip(self.inner.band(bands[1]));

        for (&x0, &x1) in itc {
            if x0 > min && x0 <= max && x1 > min && x1 <= max {
                let diffs = [
                    x0 - means[0],
                    x1 - means[1]
                ];

                sum += diffs[0] * diffs[1];
                count += T::one();
            }
        }

        (sum / count).sqrt()
    }

    fn all_band_means(&self, mp: &MultiProgress, min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        let status_bar = mp.add(ProgressBar::new(channels as u64));
        config_bar(&status_bar, "Calculating band means...");

        let means = (0..channels)
            .into_par_iter()
            .map(|b| {
                let out = self.band_mean(b, min, max);
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        means
    }

    fn all_band_std_devs(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> Vec<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        let status_bar = mp.add(ProgressBar::new(channels as u64));
        config_bar(&status_bar, "Calculating band means...");

        let devs = (0..channels)
            .into_par_iter()
            .zip(means.par_iter())
            .map(|(b, m)| {
                let out = self.band_std_dev(b, Some(*m), min, max);
                status_bar.inc(1);
                out
            })
            .collect();

        status_bar.finish();

        devs
    }

    fn covariance_matrix(&self, mp: &MultiProgress, means: &[T], min: T, max: T) -> DMatrix<T> {
        let ImageDims { channels, lines: _, samples: _ } = self.inner.dims();

        let tot_val = (channels + 1) * (channels + 1);

        let status_bar = mp.add(ProgressBar::new(tot_val as u64));
        config_bar(&status_bar, "Calculating band means...");

        let covariances: Vec<T> = (0..channels)
            .into_par_iter()
            .map(|b1| {
                let mut v: Vec<T> = (0..=b1)
                    .map(|b2| {
                        let out =
                            self.covariance_pair(
                                [b1, b2],
                                [means[b1], means[b2]],
                                min, max,
                            );
                        status_bar.inc(1);
                        out
                    })
                    .collect();

                v.reserve(channels - v.len());

                while v.len() < channels {
                    v.push(T::zero())
                }

                v
            })
            .flatten()
            .collect();

        status_bar.finish();

        let mut out = DMatrix::from_row_slice(channels, channels, &covariances);
        out.fill_upper_triangle_with_lower_triangle();

        out
    }

    fn write_standardized_results(
        &self,
        output: &mut WriteImageGuard<T, I>,
        mp: &MultiProgress,
        kept_bands: usize,
        means: &[T], std_devs: &[T],
        eigen: &SymmetricEigen<T, Dynamic>,
    )
    {
        let status_bar = mp.add(ProgressBar::new(kept_bands as u64));
        config_bar(&status_bar, "Writing standardized output bands...");
        let sc = status_bar.clone();

        let itc = self.inner.samples().zip(output.inner.samples_mut());


        for (read, write) in itc {
            let eig = eigen.eigenvectors.clone();

            for (i, b) in write.enumerate() {
                let col = eig.column(i);

                let col_write: T = read
                    .clone()
                    .zip(means)
                    .zip(std_devs)
                    .map(|((r, m), s)| {
                        let z_val: T = (*r - *m) / *s;
                        let z_off: T = (-*m) / *s;

                        if (z_val - z_off).abs() < T::zero() {
                            T::min_value()
                        } else {
                            z_val
                        }
                    })
                    .zip(col.into_iter())
                    .map(|(d, s)| d * (*s))
                    .sum();

                *b = col_write;
            }
        }

        sc.finish();
    }
}

pub fn convert<'a, I, O, T>(input: &LockImage<T, I>, output: &mut LockImage<T, O>)
    where I: IterableImage<'a, T> + SizedImage + 'static + Send + Sync,
          O: IterableImageMut<'a, T> + SizedImage + 'static + Send + Sync,
          T: Copy + 'static + Send + Sync
{
    rayon::scope(move |s| {
        let input = input.read().inner;
        let mut output = output.write().inner;
        assert_eq!(input.dims(), output.dims(), "Dims mismatch error, contact the developer!");

        let bar = indicatif::ProgressBar::new(input.dims().channels as u64);

        config_bar(&bar, "Converting bands");

        for (in_band, out_band) in input.bands().zip(output.bands_mut()) {
            let bar = bar.clone();
            s.spawn(move |_| {
                for (in_cell, out_cell) in in_band.zip(out_band) {
                    *out_cell = *in_cell
                }

                bar.inc(1);
            })
        };

        bar.finish();
    });
}

#[inline(always)]
pub fn normify<T>(val: T, scale: T, min: T, max: T) -> T
    where T: Copy + PartialOrd + Div<Output=T> + Sub<Output=T> + Debug + Zero
{
    let clamped = num::clamp(val, min, max);
    let shifted = clamped - min;
    shifted / scale
}