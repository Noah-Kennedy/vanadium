#[macro_use]
extern crate clap;

use std::error::Error;

use clap::Clap;
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

use crate::cli::{Opt, SubcommandOpt};
use crate::convert::execute_conversion;
use crate::pca::execute_pca;
use crate::render::normalize;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(not(tarpaulin_include))]
mod cli;
#[cfg(not(tarpaulin_include))]
mod convert;
#[cfg(not(tarpaulin_include))]
mod pca;
#[cfg(not(tarpaulin_include))]
mod render;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::parse();

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => execute_conversion(cvt),
        SubcommandOpt::Color(norm_opt) => normalize(norm_opt),
        SubcommandOpt::Pca(pca) => execute_pca(pca),
    }
}