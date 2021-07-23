use std::error::Error;

use nalgebra::{ComplexField, DMatrix, Dynamic, SymmetricEigen};

#[cfg(feature = "tokio-support")]
pub mod tokio;

#[cfg(feature = "tokio-uring-support")]
pub mod tokio_uring;

#[cfg(feature = "glommio-support")]
pub mod glommio;

pub mod sync_syscall;

pub type GenericResult<T> = Result<T, Box<dyn Error>>;

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