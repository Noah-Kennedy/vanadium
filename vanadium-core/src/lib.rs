#[cfg(feature = "header-parsing")]
#[macro_use]
extern crate serde;

pub mod headers;

mod image_formats;

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
