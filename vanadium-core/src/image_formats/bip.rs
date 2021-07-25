use std::fmt::Debug;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{AddAssign, DivAssign, SubAssign};

use ndarray::{Array1, Array2, Axis};
use num_traits::{Float, FromPrimitive};

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

/// # Bip-Specific Methods & Functions
///
/// Bip files lend themselves well to a workflow in which operations are split into two
/// sub-operations.
/// Initially, an accumulate operation moves along the file and folds batches of pixels into an
/// accumulator, before a final normalization phase which is conducted solely on the accumulator.
///
/// The accumulation operation should not require `&self` or `&mut self` as a parameter, as the
/// fold method provided by the IO backend requires a mutable reference itself.
///
///
impl<T> Bip<T>
    where T: Float + Clone + FromPrimitive + Sum
    + AddAssign + SubAssign + DivAssign + 'static + Debug
{
    pub fn accumulate_means(pixel: &mut Array2<T>, acc: &mut Array1<T>) {
        pixel.accumulate_axis_inplace(Axis(0), |x, sum| *sum += *x);
        *acc += &pixel.row(pixel.nrows() - 1);
    }

    pub fn normalize_means_accumulator(&self, acc: &mut Array1<T>) {
        let length = T::from_usize(self.num_pixels()).unwrap();
        acc.mapv_inplace(|x| x / length);
    }

    pub fn accumulate_standard_deviations(
        pixel: &mut Array2<T>,
        means: &Array1<T>,
        acc: &mut Array1<T>
    ) {
        *pixel -= means;

        pixel.mapv_inplace(|x| x.powi(2));
        pixel.accumulate_axis_inplace(Axis(0), |x, sum| *sum += *x);

        *acc += &pixel.row(pixel.nrows() - 1);
    }

    pub fn normalize_standard_deviations_accumulator(&self, acc: &mut Array1<T>) {
        let length = T::from_usize(self.num_pixels()).unwrap();
        acc.mapv_inplace(|x| (x / length).sqrt());
    }

    pub fn accumulate_covariances(
        pixel: &mut Array2<T>,
        means: Option<&Array1<T>>,
        std_devs: Option<&Array1<T>>,
        acc: &mut Array2<T>,
    ) {
        if let Some(means) = means {
            *pixel -= means;
        }

        if let Some(std_devs) = std_devs {
            *pixel /= std_devs;
        }

        // hot
        *acc += &pixel.t().dot(pixel);
    }

    pub fn normalize_covariances_accumulator(&self, acc: &mut Array2<T>) {
        let length = T::from_usize(self.num_pixels()).unwrap();
        acc.mapv_inplace(|x| x / length);
    }
}