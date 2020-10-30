use std::error::Error;

use structopt::StructOpt;

use crate::bin_formats::WORK_UNIT_SIZE;
use crate::cli::{Opt, SubcommandOpt};
use crate::convert::execute_conversion;
use crate::norm::normalize;

mod headers;
mod bin_formats;
mod cli;
mod convert;
mod norm;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();

    if let Ok(s) = std::env::var("HYPERSPECTRA_WORK_UNIT_SIZE") {
        let size = s.parse()?;

        unsafe {
            WORK_UNIT_SIZE = size;
        }
    }

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => execute_conversion(cvt),
        SubcommandOpt::Color(norm_opt) => normalize(norm_opt),
    }
}