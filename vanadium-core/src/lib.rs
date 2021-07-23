#[cfg(feature = "header-parsing")]
#[macro_use]
extern crate serde;

pub use crate::backends::{GenericResult, Image};

pub mod headers;

mod specialization;

mod backends;

mod util;

pub mod image_backends {
    pub use crate::backends::get_image_f32;
}
