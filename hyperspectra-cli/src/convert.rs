use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::str::FromStr;

use hyperspectra::container::{convert, IterableImage, IterableImageMut, LockImage};
use hyperspectra::container::mapped::{Bip, Bsq};
use hyperspectra::header::{Headers, Interleave};

use crate::cli::ConvertOpt;

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
    let mut parsed_headers = Headers::from_str(&headers_str)?;

    println!("Mapping input file");
    match parsed_headers.interleave {
        Interleave::Bip => {
            let index = Bip::<_, f32>::headers(&parsed_headers, &input_file)?;
            let input = LockImage::new(index);
            continue_from_input(&mut parsed_headers, &input, &output_file, output_type)
        }
        Interleave::Bil => {
            todo!()
        }
        Interleave::Bsq => {
            let index = Bsq::<_, f32>::headers(&parsed_headers, &input_file)?;
            let input = LockImage::new(index);
            continue_from_input(&mut parsed_headers, &input, &output_file, output_type)
        }
    }
}

fn continue_from_input<'a, I>(
    headers: &mut Headers, input: &LockImage<f32, I>, out: &File, out_type: Interleave,
)
    -> Result<(), Box<dyn Error>>
    where I: 'static + IterableImage<'a, f32> + Sync + Send,
{
    println!("Mapping output file");

    match out_type {
        Interleave::Bip => {
            headers.interleave = Interleave::Bip;
            let index = Bip::<_, f32>::headers_mut(&headers, &out)?;
            let mut out = LockImage::new(index);
            finish_conversion(&input, &mut out)
        }
        Interleave::Bil => {
            todo!()
        }
        Interleave::Bsq => {
            headers.interleave = Interleave::Bsq;
            let index = Bsq::<_, f32>::headers_mut(&headers, &out)?;
            let mut out = LockImage::new(index);
            finish_conversion(&input, &mut out)
        }
    };

    Ok(())
}

fn finish_conversion<'a, I1, I2>(
    input: &LockImage<f32, I1>, output: &mut LockImage<f32, I2>)
    where I1: 'static + IterableImage<'a, f32> + Sync + Send,
          I2: 'static + IterableImageMut<'a, f32> + Sync + Send,
{
    convert(&input, output)
}