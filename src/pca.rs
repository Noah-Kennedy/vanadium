use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::bin_formats::{FileIndex, FileInner, Mat};
use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::error::ConversionError;
use crate::cli::PcaOpt;
use crate::headers::{Headers, Interleave};

pub fn execute_pca(op: PcaOpt) -> Result<(), Box<dyn Error>> {
    let PcaOpt {
        input,
        input_header: header,
        output,
        output_header: _output_header,
        bands,
    } = op;

    println!("{:?}->{:?}", input.as_os_str(), output.as_os_str());

    println!("Opening input file");
    let input_file = File::open(input)?;

    println!("Opening output file");
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(output)?;

    println!("Reading headers");
    let headers_str = read_to_string(header)?;
    let mut parsed_headers = Headers::from_str(&headers_str)?;

    println!("Mapping input file");
    let inner = unsafe { FileInner::headers(&parsed_headers, &input_file)? };
    match parsed_headers.interleave {
        Interleave::Bip => {
            let index = Bip::from(inner.dims.clone());
            let input = Mat {
                inner,
                index,
            };
            continue_from_input(&mut parsed_headers, &input, &output_file, bands)
        }
        Interleave::Bil => {
            let index = Bil::from(inner.dims.clone());
            let input = Mat {
                inner,
                index,
            };
            continue_from_input(&mut parsed_headers, &input, &output_file, bands)
        }
        Interleave::Bsq => {
            let index = Bsq::from(inner.dims.clone());
            let input = Mat {
                inner,
                index,
            };
            continue_from_input(&mut parsed_headers, &input, &output_file, bands)
        }
    }
}

fn continue_from_input<C, I>(
    headers: &mut Headers, input: &Mat<C, f32, I>, out: &File, bands: u64,
)
    -> Result<(), Box<dyn Error>>
    where I: 'static + FileIndex + Sync + Send + Copy + Clone,
          C: Deref<Target=[u8]> + Sync + Send,
{
    println!("Mapping output file");

    headers.bands = bands as usize;

    println!("Allocating output file");
    out.set_len(headers.bands as u64 * headers.lines as u64 * headers.samples as u64 * 4)?;

    let inner = unsafe {
        FileInner::headers_mut(&headers, &out)?
    };


    let index = Bsq::from(inner.dims.clone());
    let mut out = Mat {
        inner,
        index,
    };

    finish_pca(&input, &mut out, bands)?;

    Ok(())
}

fn finish_pca<C1, C2, I1>(input: &Mat<C1, f32, I1>, output: &mut Mat<C2, f32, Bsq>, bands: u64)
                          -> Result<(), ConversionError>
    where I1: 'static + FileIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
          C2: DerefMut<Target=[u8]> + Sync + Send
{
    println!("Performing PCA");
    unsafe {
        input.pca(output, bands);
    }
    println!("finished");
    Ok(())
}