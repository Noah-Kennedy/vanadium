use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{AddAssign, DivAssign, SubAssign};

use ndarray::{Array1, Array2};
use num_traits::{Float, FromPrimitive};

use vanadium_core::image_formats::bip::Bip;

use crate::{GenericResult, Image};

#[cfg(feature = "glommio-backend")]
pub use crate::glommio::bip::GlommioBip;

#[cfg(feature = "syscall-backend")]
pub use crate::syscall::bip::SyscallBip;

pub trait BatchedPixelReduce<T> {
    fn reduce_pixels_batched<F, A>(&mut self, name: &str, accumulator: A, f: F) -> GenericResult<A>
        where F: FnMut(&mut Array2<T>, &mut A);
    fn bip(&self) -> &Bip<T>;
}

impl<C, T> Image<T> for C
    where C: BatchedPixelReduce<T>,
          T: Float + Clone + FromPrimitive + Sum + AddAssign + SubAssign + DivAssign + Debug
          + 'static
{
    fn means(&mut self) -> GenericResult<Array1<T>> {
        let accumulator = Array1::zeros(self.bip().pixel_length());

        let mut res = self.reduce_pixels_batched("mean", accumulator, |pixels, acc| {
            Bip::accumulate_means(pixels, acc)
        })?;

        self.bip().normalize_means_accumulator(&mut res);

        Ok(res)
    }

    fn std_deviations(&mut self, means: &Array1<T>) -> GenericResult<Array1<T>> {
        let accumulator = Array1::zeros(self.bip().pixel_length());

        let mut res = self.reduce_pixels_batched("std", accumulator, |pixels, acc| {
            Bip::accumulate_standard_deviations(pixels, means, acc)
        })?;

        self.bip().normalize_standard_deviations_accumulator(&mut res);

        Ok(res)
    }

    fn covariance_matrix(&mut self, means: Option<&Array1<T>>, std_devs: Option<&Array1<T>>) -> GenericResult<Array2<T>> {
        let accumulator = Array2::zeros((self.bip().dims.channels, self.bip().dims.channels));

        let mut res = self.reduce_pixels_batched("cov", accumulator, |pixels, acc| {
            Bip::accumulate_covariances(pixels, means, std_devs, acc)
        })?;

        self.bip().normalize_covariances_accumulator(&mut res);

        Ok(res)
    }
}
