use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::str::FromStr;

use crate::bin_formats::{convert, FileIndex, FileIndexMut, FileInner};
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::cli::ConvertOpt;
use crate::file_alloc::allocate_file;
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
    let inner = unsafe { FileInner::headers(&parsed_headers, &input_file)? };
    match parsed_headers.interleave {
        Interleave::Bip => {
            let input = Bip::from(inner);
            continue_from_input(&parsed_headers, input, &output_file, output_type)
        }
        Interleave::Bil => {
            todo!()
        }
        Interleave::Bsq => {
            let input = Bsq::from(inner);
            continue_from_input(&parsed_headers, input, &output_file, output_type)
        }
    }
}

fn continue_from_input<T>(headers: &Headers, input: T, out: &File, out_type: Interleave)
                          -> Result<(), Box<dyn Error>>
    where T: 'static + FileIndex<f32> + Sync + Send,
{
    println!("Mapping output file");
    let output_inner = unsafe {
        FileInner::headers_mut(&headers, &out)?
    };

    match out_type {
        Interleave::Bip => {
            let mut out: Bip<_, f32> = Bip::from(output_inner);
            finish_conversion(&input, &mut out)
        }
        Interleave::Bil => todo!(),
        Interleave::Bsq => {
            let mut out: Bsq<_, f32> = Bsq::from(output_inner);
            finish_conversion(&input, &mut out)
        }
    }

    Ok(())
}

fn finish_conversion<I, O>(input: &I, output: &mut O)
    where I: 'static + FileIndex<f32> + Sync + Send,
          O: 'static + FileIndexMut<f32> + Sync + Send,
{
    println!("Performing conversion");
    convert(input, output);
    println!("finished")
}