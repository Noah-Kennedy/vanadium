use std::fmt::Debug;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{AddAssign, DivAssign, SubAssign};

use ndarray::{Array1, Array2, Axis};
use num_traits::{Float, FromPrimitive};

use crate::backends::BATCH_SIZE;
use crate::headers::ImageDims;

#[derive(Clone)]
pub struct Bip<T> {
    pub dims: ImageDims,
    pub phantom: PhantomData<T>,
}

impl<T> Bip<T> {
    #[inline(always)]
    pub fn _index_pixel(&self, pixel: usize) -> usize {
        self.pixel_length() * pixel
    }

    #[inline(always)]
    pub fn pixel_length(&self) -> usize {
        self.dims.channels
    }

    #[inline(always)]
    pub fn num_pixels(&self) -> usize {
        self.dims.lines * self.dims.pixels
    }
}

impl<T> Bip<T>
    where T: Float + Clone + FromPrimitive + Sum
    + AddAssign + SubAssign + DivAssign + 'static + Debug
{
    pub fn map_mean(pixel: &mut Array2<T>, acc: &mut Array1<T>) {
        *acc += &pixel.mean_axis(Axis(0)).unwrap()
    }

    pub fn reduce_mean(&self, acc: &mut Array1<T>) {
        let length = T::from_usize(self.num_pixels() / BATCH_SIZE).unwrap();
        acc.mapv_inplace(|x| x / length);
    }

    pub fn map_std_dev(pixel: &mut Array2<T>, means: &Array1<T>, acc: &mut Array1<T>) {
        *pixel -= means;

        let batch = T::from_usize(BATCH_SIZE).unwrap();

        pixel.mapv_inplace(|x| x.powi(2) / batch);

        pixel.accumulate_axis_inplace(Axis(0), |x, sum| *sum += *x);

        *acc += &pixel.row(BATCH_SIZE - 1);
    }

    pub fn reduce_std_dev(&self, acc: &mut Array1<T>) {
        let length = T::from_usize(self.num_pixels() / BATCH_SIZE).unwrap();
        acc.mapv_inplace(|x| (x / length).sqrt());
    }

    pub fn map_covariance(
        pixel: &mut Array2<T>,
        means: &Array1<T>,
        std_devs: &Array1<T>,
        acc: &mut Array2<T>,
    ) {
        let batch = T::from_usize(BATCH_SIZE).unwrap();

        *pixel -= means;
        *pixel /= std_devs;

        let mut cov = pixel.t().dot(pixel);

        cov.mapv_inplace(|x| (x / batch));

        *acc += &cov;
    }

    pub fn reduce_covariance(&self, acc: &mut Array2<T>) {
        let length = T::from_usize(self.num_pixels() / BATCH_SIZE).unwrap();
        acc.mapv_inplace(|x| x / length);
    }
}