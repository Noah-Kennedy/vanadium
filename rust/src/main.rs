use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::io::Write;
use std::str::FromStr;

use memmap2::{Mmap, MmapMut};
use structopt::StructOpt;

use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::cli::{ConvertOpt, Opt, SubcommandOpt};
use crate::headers::Headers;

mod headers;
mod bin_formats;
mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();

    match opt.subcommand {
        SubcommandOpt::Convert(cvt) => {
            let ConvertOpt {
                input,
                input_type,
                header,
                output,
                output_type
            } = cvt;

            println!("Converting bip {:?} into bsq {:?}", input.as_os_str(), output.as_os_str());

            println!("Opening input file");
            let input_file = File::open(input)?;

            println!("Opening output file");
            let mut output_file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .read(true)
                .open(output)?;

            println!("Reading headers");
            let headers_str = read_to_string(header)?;
            let parsed_headers = Headers::from_str(&headers_str)?;

            println!("Mapping input file");
            let bip: Bip<Mmap, f32> = unsafe {
                Bip::with_headers(&parsed_headers, &input_file)?
            };

            println!("Mapping output file");
            let mut bsq: Bsq<MmapMut, f32> = unsafe {
                Bsq::with_headers_anon(&parsed_headers)?
            };

            println!("Performing conversion");
            bip.convert_bsq(&mut bsq);

            println!("Writing file");
            output_file.write_all(bsq.container.as_ref())?;

            println!("finished")
        }
    }

    Ok(())
}