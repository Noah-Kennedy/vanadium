use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::str::FromStr;

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, Mat, MatType};
use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::{ConversionError, ConversionErrorKind, SizeMismatchError};
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
            let input = Mat::from(Bip::from(inner));
            continue_from_input(&parsed_headers, &input, &output_file, output_type)
        }
        Interleave::Bil => {
            let input = Mat::from(Bil::from(inner));
            continue_from_input(&parsed_headers, &input, &output_file, output_type)
        }
        Interleave::Bsq => {
            let input = Mat::from(Bsq::from(inner));
            continue_from_input(&parsed_headers, &input, &output_file, output_type)
        }
    }
}

fn continue_from_input<T>(headers: &Headers, input: &Mat<T>, out: &File, out_type: Interleave)
                          -> Result<(), Box<dyn Error>>
    where T: 'static + FileIndex<f32> + Sync + Send,
{
    println!("Mapping output file");
    let inner = unsafe {
        FileInner::headers_mut(&headers, &out)?
    };

    match out_type {
        Interleave::Bip => {
            let mut out = Mat::from(Bip::from(inner));
            finish_conversion(&input, &mut out)
        }
        Interleave::Bil => {
            let mut out = Mat::from(Bil::from(inner));
            finish_conversion(&input, &mut out)
        }
        Interleave::Bsq => {
            let mut out = Mat::from(Bil::from(inner));
            finish_conversion(&input, &mut out)
        }
    }?;

    Ok(())
}

fn finish_conversion<I, O>(input: &Mat<I>, output: &mut Mat<O>) -> Result<(), ConversionError>
    where I: 'static + FileIndex<f32> + Sync + Send,
          O: 'static + FileIndexMut<f32> + Sync + Send,
{
    if input.inner.size() == output.inner.size() {
        println!("Performing conversion");
        input.convert(output);
        println!("finished");
        Ok(())
    } else {
        Err(ConversionError {
            input_type: match input.inner.order() {
                MatType::Bip => "bip",
                MatType::Bil => "bil",
                MatType::Bsq => "bsq",
            },
            output_type: "",
            kind: ConversionErrorKind::SizeMismatch(
                SizeMismatchError {
                    input_size: input.inner.size(),
                    output_size: output.inner.size(),
                }
            ),
        })
    }
}