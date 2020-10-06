use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::ops::DerefMut;
use std::str::FromStr;

use memmap2::{Mmap, MmapMut};
use structopt::StructOpt;

use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::OperableFile;
use crate::cli::{ConvertOpt, Opt, SubcommandOpt};
use crate::file_alloc::allocate_file;
use crate::headers::{Headers, Interleave};

mod headers;
mod bin_formats;
mod cli;
mod file_alloc;

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
                .truncate(true)
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

            match output_type {
                Interleave::Bip => todo!(),
                Interleave::Bil => todo!(),
                Interleave::Bsq => {
                    println!("Mapping output file");
                    let mut bsq: Bsq<MmapMut, f32> = unsafe {
                        Bsq::with_headers_mut(&parsed_headers, output_file)?
                    };

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
                              -> Result<Box<dyn OperableFile<T, C>>, Box<dyn Error>>
    where T: Copy + Send + Sync + 'static,
          C: Sync + Send + DerefMut<Target=[u8]>
{
    match headers.interleave {
        Interleave::Bip => Ok(Box::new(Bip::with_headers(&headers, &file)?)),
        Interleave::Bil => todo!(),
        Interleave::Bsq => todo!(),
    }
}