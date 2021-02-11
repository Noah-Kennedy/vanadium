#[cfg_attr(feature = "serde", macro_use)]
#[cfg(feature = "serde")]
extern crate serde;

pub mod header;

pub mod container;

#[cfg(not(tarpaulin_include))]
pub mod bar;