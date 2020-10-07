use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::ops::DerefMut;
use std::str::FromStr;

use num::traits::NumAssign;
use structopt::StructOpt;

use crate::bin_formats::{FileConvert, FileInner};
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::cli::{ConvertOpt, Opt, SubcommandOpt};
use crate::file_alloc::allocate_file;
use crate::headers::{Headers, Interleave};

pub mod headers;
pub mod bin_formats;
pub mod cli;
pub mod file_alloc;

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => {
            let ConvertOpt {
                input,
                header,
                output,
                output_type
            } = cvt;

            println!("{:?}->{:?}", input.as_os_str(), output.as_os_str());

            println!("Opening input file");
            let input_file = File::open(input)?;

            println!("Opening output file");
            let output_file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(output)?;

            println!("Allocating output file");
            allocate_file(&output_file, input_file.metadata()?.len() as usize)?;

            println!("Reading headers");
            let headers_str = read_to_string(header)?;
            let parsed_headers = Headers::from_str(&headers_str)?;

            println!("Mapping input file");
            let input_mat = unsafe { get_input_map(&parsed_headers, &input_file)? };

            println!("Mapping output file");
            let output_inner = unsafe {
                FileInner::headers_mut(&parsed_headers, &output_file)?
            };

            match output_type {
                Interleave::Bip => {
                    // println!("Mapping output file");
                    // let mut bip: Bip<_, f32> = unsafe {
                    //     Bip::with_headers_mut(&parsed_headers, output_file)?
                    // };
                    //
                    // println!("Performing conversion");
                    // input_mat.to_bsq(&mut bip)?;
                    // println!("finished")
                }
                Interleave::Bil => todo!(),
                Interleave::Bsq => {
                    let mut bsq: Bsq<_, f32> = Bsq::from(output_inner);

                    println!("Performing conversion");
                    input_mat.to_bsq(&mut bsq)?;
                    println!("finished")
                }
            }
        }
    }

    Ok(())
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