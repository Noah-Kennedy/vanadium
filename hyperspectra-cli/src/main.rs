use std::error::Error;

use structopt::StructOpt;

use crate::cli::{Opt, SubcommandOpt};
use crate::convert::execute_conversion;
use crate::pca::execute_pca;
use crate::render::normalize;

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
    let opt: Opt = Opt::from_args();

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => execute_conversion(cvt),
        SubcommandOpt::Color(norm_opt) => normalize(norm_opt),
        SubcommandOpt::Pca(pca) => execute_pca(pca),
    }
}