use std::error::Error;

use structopt::StructOpt;

use crate::cli::{Opt, SubcommandOpt};
use crate::convert::execute_conversion;
use crate::pca::execute_pca;
use crate::render::normalize;

mod headers;
mod bin_formats;
mod cli;
mod convert;
mod pca;
mod render;
mod util;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => execute_conversion(cvt),
        SubcommandOpt::Color(norm_opt) => normalize(norm_opt),
        SubcommandOpt::Pca(pca) => execute_pca(pca),
    }
}