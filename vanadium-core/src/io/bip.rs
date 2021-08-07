use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{AddAssign, DivAssign, SubAssign};
use std::path::Path;

use ndarray::{Array1, Array2, ArrayViewMut2};
use ndarray_linalg::{Lapack, Scalar};
use num_traits::{Float, FromPrimitive};

use crate::error::VanadiumResult;
use crate::image_formats::bip::Bip;
use crate::io::ImageStats;

#[cfg(feature = "glommio-backend")]
pub use super::glommio::bip::GlommioBip;
#[cfg(feature = "syscall-backend")]
pub use super::syscall::bip::SyscallBip;

pub trait SequentialPixels<T> {
    fn fold_batched<F, A>(&mut self, name: &str, accumulator: A, f: F) -> VanadiumResult<A>
        where F: FnMut(&mut Array2<T>, &mut A);
    fn bip(&self) -> &Bip<T>;
    fn map_and_write_batched<F>(
        &mut self,
        name: &str,
        out: &dyn AsRef<Path>,
        n_output_channels: usize,
        f: F,
    ) -> VanadiumResult<()>
        where F: FnMut(&mut ArrayViewMut2<T>, &mut Array2<T>);
}

impl<C, T> ImageStats<T> for C
    where C: SequentialPixels<T>,
          T: Float + Clone + FromPrimitive + Sum + AddAssign + SubAssign + DivAssign + Debug + Lapack
          + 'static + Scalar
{
    fn means(&mut self) -> VanadiumResult<Array1<T>> {
        let accumulator = Array1::zeros(self.bip().pixel_length());

        let mut res = self.fold_batched("mean", accumulator, |pixels, acc| {
            Bip::accumulate_means(pixels, acc)
        })?;

        self.bip().normalize_means_accumulator(&mut res);

        Ok(res)
    }

    fn std_deviations(&mut self, means: &Array1<T>) -> VanadiumResult<Array1<T>> {
        let accumulator = Array1::zeros(self.bip().pixel_length());

        let mut res = self.fold_batched("std", accumulator, |pixels, acc| {
            Bip::accumulate_standard_deviations(pixels, means, acc)
        })?;

        self.bip().normalize_standard_deviations_accumulator(&mut res);

        Ok(res)
    }

    fn covariance_matrix(&mut self, means: Option<&Array1<T>>, std_devs: Option<&Array1<T>>) -> VanadiumResult<Array2<T>> {
        let accumulator = Array2::zeros((self.bip().dims.channels, self.bip().dims.channels));

        let mut res = self.fold_batched("cov", accumulator, |pixels, acc| {
            Bip::accumulate_covariances(pixels, means, std_devs, acc)
        })?;

        self.bip().normalize_covariances_accumulator(&mut res);

        Ok(res)
    }

    fn write_transformed(
        &mut self,
        transform: &Array2<T>,
        out: &dyn AsRef<Path>,
        means: Option<&Array1<T>>,
        std_devs: Option<&Array1<T>>,
    ) -> VanadiumResult<()>
    {
        self.map_and_write_batched("write", out, transform.ncols(), |pixels, write_array| {
            Bip::map_transform(pixels, transform, write_array, means, std_devs)
        })
    }
}