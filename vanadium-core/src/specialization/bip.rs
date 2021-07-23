use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{AddAssign, DivAssign, SubAssign};

use nalgebra::{DMatrix, Scalar};
use num_traits::{Float, FromPrimitive};

use crate::headers::ImageDims;
use crate::util::standardize;

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
    + AddAssign + SubAssign + DivAssign + Scalar
{
    #[inline(always)]
    pub fn map_mean(&self, pixel: &[T], acc: &mut [T]) {
        for (acc, c) in acc.iter_mut().zip(pixel.iter()) {
            *acc += c.clone();
        }
    }

    #[inline(always)]
    pub fn reduce_mean(&self, acc: &mut [T]) {
        let length = T::from_usize(self.num_pixels()).unwrap();

        for x in acc.iter_mut() {
            *x /= length;
        }
    }

    #[inline(always)]
    pub fn map_std_dev(&self, pixel: &[T], means: &[T], acc: &mut [T]) {
        for (acc, (c, m)) in acc.iter_mut().zip(pixel.iter().zip(means)) {
            *acc += (*c - *m).powi(2);
        }
    }

    #[inline(always)]
    pub fn reduce_std_dev(&self, acc: &mut [T]) {
        let length = T::from_usize(self.num_pixels()).unwrap();

        for c in acc.iter_mut() {
            *c = (*c / length).sqrt();
        }
    }

    #[inline(always)]
    pub fn map_covariance(&self, pixel: &mut [T], means: &[T], std_devs: &[T], acc: &mut DMatrix<T>) {
        for ((x, m), s) in pixel.iter_mut().zip(means).zip(std_devs) {
            *x = standardize(*x, *m, *s)
        }

        for i in 0..pixel.len() {
            for j in i..pixel.len() {
                unsafe {
                    let x = *pixel.get_unchecked(i);
                    let y = *pixel.get_unchecked(j);

                    let r = acc.get_unchecked_mut((i, j));

                    *r += (x * y).powi(2);
                }
            }
        }
    }

    #[inline(always)]
    pub fn reduce_covariance(&self, acc: &mut DMatrix<T>) {
        let length = T::from_usize(self.num_pixels()).unwrap();

        let (rows, cols) = acc.shape();

        for i in 0..rows {
            for j in i..cols {
                acc[(i, j)] /= length;
            }
        }
    }
}