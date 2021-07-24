extern crate blas_src;

#[cfg(feature = "header-parsing")]
#[macro_use]
extern crate serde;

pub mod headers;

mod specialization;

mod backends;

mod util;

pub mod ops {
    pub use crate::backends::{
        get_image_f32,
        BackendSelector,
        GenericResult,
        BatchedPixelReduce,
        Image
    };
}
