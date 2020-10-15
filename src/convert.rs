use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::str::FromStr;

use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::FileInner;
use crate::cli::ConvertOpt;
use crate::file_alloc::allocate_file;
use crate::get_input_map;
use crate::headers::{Headers, Interleave};

pub fn execute_conversion(cvt: ConvertOpt) -> Result<(), Box<dyn Error>> {
    let ConvertOpt {
        input,
        input_header: header,
        output,
        output_header: _output_header,
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
            println!("Mapping output file");
            let mut bip: Bip<_, f32> = Bip::from(output_inner);


            println!("Performing conversion");
            input_mat.to_bip(&mut bip)?;
            println!("finished")
        }
        Interleave::Bil => todo!(),
        Interleave::Bsq => {
            let mut bsq: Bsq<_, f32> = Bsq::from(output_inner);

            println!("Performing conversion");
            input_mat.to_bsq(&mut bsq)?;
            println!("finished")
        }
    }

    Ok(())
}