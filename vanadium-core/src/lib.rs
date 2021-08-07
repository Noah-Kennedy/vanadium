#[cfg(feature = "header-parsing")]
#[macro_use]
extern crate serde;

pub mod headers;

pub mod image_formats;

pub mod error;

mod util;