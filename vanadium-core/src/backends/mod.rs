use std::error::Error;

use sync_syscall::bip::SyncBip;
use sync_syscall::bsq::SyncBsq;

use crate::backends::glommio::bip::GlommioBip;
use crate::headers::{Header, ImageFormat};
use ndarray::{Array2, Array1};

#[cfg(feature = "progress")]
const UPDATE_FREQ: u64 = 8;

pub (crate) const BATCH_SIZE: usize = 256;

// #[cfg(feature = "progress")]
// const UPDATE_MASK: usize = 0xFF_FF;

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

pub trait Image<T> {
    fn means(&mut self) -> GenericResult<Array1<T>>;
    fn std_deviations(&mut self, means: &Array1<T>) -> GenericResult<Array1<T>>;
    fn covariance_matrix(&mut self, means: &Array1<T>, std_devs: &Array1<T>) -> GenericResult<Array2<T>>;
}