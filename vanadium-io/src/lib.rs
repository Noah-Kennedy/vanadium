#[macro_use]
extern crate ndarray;

use std::error::Error;

use ndarray::{Array1, Array2};

#[cfg(feature = "progress")]
const UPDATE_FREQ: u64 = 8;

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
                $i.set_draw_rate($crate::UPDATE_FREQ);
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

pub type GenericResult<T> = Result<T, Box<dyn Error>>;

pub trait ImageStats<T> {
    fn means(&mut self) -> GenericResult<Array1<T>>;
    fn std_deviations(&mut self, means: &Array1<T>) -> GenericResult<Array1<T>>;
    fn covariance_matrix(&mut self, means: Option<&Array1<T>>, std_devs: Option<&Array1<T>>) -> GenericResult<Array2<T>>;
}