use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::bin_formats::{ImageIndex, MatType, SpectralImage, SpectralImageContainer};
use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::{ConversionError, ConversionErrorKind, SizeMismatchError};
use crate::cli::ConvertOpt;
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
    output_file.set_len(input_file.metadata()?.len())?;

    println!("Reading headers");
    let headers_str = read_to_string(header)?;
    let parsed_headers = Headers::from_str(&headers_str)?;

    println!("Mapping input file");
    let inner = SpectralImageContainer::headers(&parsed_headers, &input_file)?;
    match parsed_headers.interleave {
        Interleave::Bip => {
            let index = Bip::from(inner.dims.clone());
            let input = SpectralImage {
                inner,
                index,
            };
            continue_from_input(&parsed_headers, &input, &output_file, output_type)
        }
        Interleave::Bil => {
            let index = Bil::from(inner.dims.clone());
            let input = SpectralImage {
                inner,
                index,
            };
            continue_from_input(&parsed_headers, &input, &output_file, output_type)
        }
        Interleave::Bsq => {
            let index = Bsq::from(inner.dims.clone());
            let input = SpectralImage {
                inner,
                index,
            };
            continue_from_input(&parsed_headers, &input, &output_file, output_type)
        }
    }
}

fn continue_from_input<C, I>(
    headers: &Headers, input: &SpectralImage<C, f32, I>, out: &File, out_type: Interleave,
)
    -> Result<(), Box<dyn Error>>
    where I: 'static + ImageIndex + Sync + Send + Copy + Clone,
          C: Deref<Target=[u8]> + Sync + Send,
{
    println!("Mapping output file");
    let inner = SpectralImageContainer::headers_mut(&headers, &out)?;

    match out_type {
        Interleave::Bip => {
            let index = Bip::from(inner.dims.clone());
            let mut out = SpectralImage {
                inner,
                index,
            };
            finish_conversion(&input, &mut out)
        }
        Interleave::Bil => {
            let index = Bil::from(inner.dims.clone());
            let mut out = SpectralImage {
                inner,
                index,
            };
            finish_conversion(&input, &mut out)
        }
        Interleave::Bsq => {
            let index = Bsq::from(inner.dims.clone());
            let mut out = SpectralImage {
                inner,
                index,
            };
            finish_conversion(&input, &mut out)
        }
    }?;

    Ok(())
}

fn finish_conversion<C1, C2, I1, I2>(input: &SpectralImage<C1, f32, I1>, output: &mut SpectralImage<C2, f32, I2>)
                                     -> Result<(), ConversionError>
    where I1: 'static + ImageIndex + Sync + Send + Copy + Clone,
          I2: 'static + ImageIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
          C2: DerefMut<Target=[u8]> + Sync + Send
{
    if input.inner.size() == output.inner.size() {
        println!("Performing conversion");
        input.convert(output);
        println!("finished");
        Ok(())
    } else {
        Err(ConversionError {
            input_type: match input.index.order() {
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