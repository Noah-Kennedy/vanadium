use std::error::Error;
use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{AddAssign, DivAssign, SubAssign};

use ndarray::{Array1, Array2};
use num_traits::{Float, FromPrimitive};

use sync_syscall::bip::SyncBip;

use crate::backends::glommio::bip::GlommioBip;
use crate::headers::{Header, ImageFormat};
use crate::specialization::bip::Bip;

#[cfg(feature = "progress")]
const UPDATE_FREQ: u64 = 8;

pub(crate) const BATCH_SIZE: usize = 256;

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
                $i.set_draw_rate($crate::backends::UPDATE_FREQ);
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

// #[cfg(feature = "tokio-uring-support")]
// pub mod tokio_uring;

#[cfg(feature = "glommio-support")]
pub mod glommio;

pub mod sync_syscall;

pub type GenericResult<T> = Result<T, Box<dyn Error>>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum BackendSelector {
    Syscall,
    Glommio,
}

impl Default for BackendSelector {
    fn default() -> Self {
        Self::Syscall
    }
}

pub fn get_image_f32(backend: BackendSelector, header: Header)
                     -> Result<Box<dyn Image<f32>>, Box<dyn Error>>
{
    Ok(match (backend, header.format) {
        (BackendSelector::Glommio, ImageFormat::Bip) => Box::new(GlommioBip::new(header)),
        (BackendSelector::Syscall, ImageFormat::Bip) => Box::new(SyncBip::new(header)?),
        _ => unimplemented!("Image backend configuration not supported yet!")
    })
}

pub trait Image<T> {
    fn means(&mut self) -> GenericResult<Array1<T>>;
    fn std_deviations(&mut self, means: &Array1<T>) -> GenericResult<Array1<T>>;
    fn covariance_matrix(&mut self, means: Option<&Array1<T>>, std_devs: Option<&Array1<T>>) -> GenericResult<Array2<T>>;
}

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
            Bip::map_mean(pixels, acc)
        })?;

        self.bip().reduce_mean(&mut res);

        Ok(res)
    }

    fn std_deviations(&mut self, means: &Array1<T>) -> GenericResult<Array1<T>> {
        let accumulator = Array1::zeros(self.bip().pixel_length());

        let mut res = self.reduce_pixels_batched("std", accumulator, |pixels, acc| {
            Bip::map_std_dev(pixels, means, acc)
        })?;

        self.bip().reduce_std_dev(&mut res);

        Ok(res)
    }

    fn covariance_matrix(&mut self, means: Option<&Array1<T>>, std_devs: Option<&Array1<T>>) -> GenericResult<Array2<T>> {
        let accumulator = Array2::zeros((self.bip().dims.channels, self.bip().dims.channels));

        let mut res = self.reduce_pixels_batched("cov", accumulator, |pixels, acc| {
            Bip::map_covariance(pixels, means, std_devs, acc)
        })?;

        self.bip().reduce_covariance(&mut res);

        Ok(res)
    }
}