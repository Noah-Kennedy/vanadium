use std::error::Error;
use std::fs::File;
use std::ops::DerefMut;

use num::traits::NumAssign;
use structopt::StructOpt;

use crate::bin_formats::{FileConvert, FileInner};
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::cli::{Opt, SubcommandOpt};
use crate::convert::execute_conversion;
use crate::headers::{Headers, Interleave};

pub mod headers;
pub mod bin_formats;
pub mod cli;
pub mod file_alloc;
pub mod convert;

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => execute_conversion(cvt),
    }
}

unsafe fn get_input_map<T, C>(headers: &Headers, file: &File)
                              -> Result<Box<dyn FileConvert<T, C>>, Box<dyn Error>>
    where T: Copy + Send + Sync + NumAssign + 'static,
          C: Sync + Send + DerefMut<Target=[u8]>
{
    let inner = FileInner::headers(&headers, &file)?;
    match headers.interleave {
        Interleave::Bip => Ok(Box::new(Bip::from(inner))),
        Interleave::Bil => todo!(),
        Interleave::Bsq => Ok(Box::new(Bsq::from(inner))),
    }
}