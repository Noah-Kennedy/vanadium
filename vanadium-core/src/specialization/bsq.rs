use std::ops::{DivAssign, SubAssign};

use num_traits::{Float, FromPrimitive};

use crate::util::standardize;
use crate::headers::ImageDims;
use std::marker::PhantomData;

pub struct Bsq<T> {
    pub dims: ImageDims,
    pub phantom: PhantomData<T>,
}

impl<T> Bsq<T> {
    pub fn index_channel(&self, channel: usize) -> usize {
        self.channel_length() * channel
    }

    pub fn channel_length(&self) -> usize {
        self.dims.pixels * self.dims.lines
    }
}

impl<T> Bsq<T>
    where T: Float + Copy + FromPrimitive + std::iter::Sum + SubAssign + DivAssign
{
    pub fn find_channel_mean(&self, channel: &[T]) -> T {
        let sum: T = channel.iter().map(T::to_owned).sum();
        sum / T::from_usize(channel.len()).unwrap()
    }

    pub fn find_channel_std_dev(&self, channel: &[T], mean: T) -> T {
        let sum: T = channel.iter().map(|x| (*x - mean).powi(2)).sum();
        let variance: T = sum / T::from_usize(channel.len()).unwrap();

        variance.sqrt()
    }

    pub fn standardize_channel(&self, channel: &mut [T], mean: T, std_dev: T) {
        for x in channel.iter_mut() {
            *x -= mean;
            *x /= std_dev;
        }
    }


    ///
    ///
    /// # Arguments
    ///
    /// * `channel1`: STANDARDIZED first channel
    /// * `channel2`: UNSTANDARDIZED second channel
    /// * `mean2`:
    /// * `std2`:
    ///
    /// returns: T
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn covariance(&self, channel1: &[T], channel2: &[T], mean2: T, std2: T) -> T {
        let s: T = channel1.iter().zip(channel2.iter()).map(
            |(x, y)| {
                let std_y = standardize(*y, mean2, std2);

                (*x) * std_y
            })
            .sum();

        s / T::from_usize(channel1.len()).unwrap()
    }

    pub fn variance(&self, channel1: &[T]) -> T {
        let s: T = channel1.iter().map(|x| x.powi(2)).sum();

        s / T::from_usize(channel1.len()).unwrap()
    }
}