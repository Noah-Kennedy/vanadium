use std::error::Error;

use nalgebra::{ComplexField, DMatrix, Dynamic, SymmetricEigen};

use sync_syscall::bip::SyncBip;
use sync_syscall::bsq::SyncBsq;

use crate::backends::glommio::bip::GlommioBip;
use crate::headers::{Header, ImageFormat};

macro_rules! make_bar {
    ($i:ident, $x:expr) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "progress")] {
                let mut $i = pbr::ProgressBar::new($x);
                $i.set_max_refresh_rate(Some(UPDATE_FREQ));
            }
        }
    }
}

macro_rules! inc_bar {
    ($b:expr, $x:expr) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "progress")] {
                if ($x & UPDATE_MASK) > 0 {
                    $b.inc();
                }
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

pub fn get_image_f32(backend: Option<&str>, header: Header)
                     -> Result<Box<dyn Image<f32>>, Box<dyn Error>>
{
    Ok(if let Some(s) = backend {
        match (s, header.format) {
            ("glommio", ImageFormat::Bip) => Box::new(GlommioBip::new(header)),
            ("sync", ImageFormat::Bip) => Box::new(SyncBip::new(header)?),
            ("sync", ImageFormat::Bsq) => Box::new(SyncBsq::new(header)?),
            _ => unimplemented!()
        }
    } else {
        todo!()
    })
}

pub trait Image<T> where T: ComplexField {
    fn means(&mut self) -> GenericResult<Vec<T>>;
    fn std_deviations(&mut self, means: &[T]) -> GenericResult<Vec<T>>;
    fn covariance_matrix(&mut self, means: &[T], std_devs: &[T]) -> GenericResult<DMatrix<T>>;
    fn write_standardized(
        &mut self,
        path: &str,
        means: &[T], std_devs: &[T],
        eigen: &SymmetricEigen<T, Dynamic>,
    ) -> GenericResult<()>;
}