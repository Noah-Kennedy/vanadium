#[cfg(feature = "header-parsing")]
#[macro_use]
extern crate serde;

pub use crate::backends::{GenericResult, Image};

pub mod headers;

mod specialization;

mod backends;

mod util;

#[cfg(feature = "async")]
pub mod asynchronous_ops {
    #[cfg(feature = "glommio-support")]
    pub use crate::backends::glommio;
    #[cfg(feature = "tokio-uring-support")]
    pub use crate::backends::tokio_uring;
}

pub mod sync_syscall {
    pub use crate::backends::sync_syscall::*;
}
