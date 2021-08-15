use std::path::Path;

use ndarray::{Array1, Array2};
use ndarray_linalg::{Eig, Lapack, Scalar};
use num_traits::real::Real;

use crate::error::{VanadiumError, VanadiumResult};
use image::{RgbImage};

#[cfg(feature = "progress")]
const UPDATE_FREQ: u64 = 8;

// todo possibly make variable
const BATCH_SIZE: usize = 1024;

macro_rules! make_bar {
    ($i:ident, $x:expr, $m:expr) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "progress")] {
                let $i = indicatif::ProgressBar::new($x);
                $i.set_style(indicatif::ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {eta} {msg}")
                    .progress_chars("##-")
                );
                $i.set_message($m);
                $i.set_draw_rate($crate::io::UPDATE_FREQ);
            }
        }
    }
}

macro_rules! inc_bar {
    ($b:expr, $x:expr) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "progress")] {
                $b.inc($x);
            }
        }
    }
}

// #[cfg(feature = "tokio-uring-backend")]
// pub mod tokio_uring;

pub mod bip;

#[cfg(feature = "glommio-backend")]
pub mod glommio;

#[cfg(feature = "syscall-backend")]
pub mod syscall;

#[cfg(feature = "mapped-backend")]
pub mod mapped;

#[cfg(feature = "tokio-backend")]
pub mod tokio;

pub trait BasicImage<T> where
    T: Real + Lapack
{
    fn means(&mut self) -> VanadiumResult<Array1<T>>;
    fn std_deviations(&mut self, means: &Array1<T>) -> VanadiumResult<Array1<T>>;
    fn covariance_matrix(&mut self, means: Option<&Array1<T>>, std_devs: Option<&Array1<T>>) -> VanadiumResult<Array2<T>>;
    fn write_transformed(
        &mut self,
        transform: &Array2<T>,
        out: &dyn AsRef<Path>,
        means: Option<&Array1<T>>,
        std_devs: Option<&Array1<T>>,
    ) -> VanadiumResult<()>;
    fn pca_eigen(
        &mut self,
        n_dims: usize,
        cov_mat: &Array2<T>,
    ) -> VanadiumResult<Array2<T>> {
        let (_e_val, e_vec) = cov_mat.eig().map_err(|_| VanadiumError::Unknown)?;

        Ok(e_vec.slice(s![..n_dims, ..]).mapv(|x| T::from_real(x.re())))
    }
    fn crop(
        &mut self,
        rows: Option<(u64, u64)>,
        cols: Option<(u64, u64)>,
        out: &dyn AsRef<Path>,
    ) -> VanadiumResult<()>;
    fn rgb_batched(
        &mut self,
        colormap: &mut dyn FnMut(&mut Array2<T>) -> Array2<u8>
    ) -> VanadiumResult<RgbImage>;
}