use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::str::FromStr;

use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::bin_formats::{FileInner, Mat};
use crate::cli::NormOpt;
use crate::file_alloc::allocate_file;
use crate::headers::{Headers, Interleave};

pub fn norm(cvt: NormOpt) -> Result<(), Box<dyn Error>> {
    println!("Opening input file");
    let input_file = File::open(cvt.input)?;

    println!("Opening output file");
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(cvt.output)?;

    println!("Allocating output file");
    allocate_file(&output_file, input_file.metadata()?.len() as usize)?;

    println!("Reading headers");
    let headers_str = read_to_string(cvt.input_header)?;
    let parsed_headers = Headers::from_str(&headers_str)?;

    println!("Mapping input file");
    let inner: FileInner<_, f32> = unsafe { FileInner::headers(&parsed_headers, &input_file)? };
    println!("Mapping output file");
    let outer: FileInner<_, f32> = unsafe { FileInner::headers_mut(&parsed_headers, &output_file)? };
    match parsed_headers.interleave {
        Interleave::Bip => {
            let input = Mat::from(Bip::from(inner));
            let mut out = Mat::from(Bip::from(outer));
            input.clamp_between(&mut out, cvt.min, cvt.max, &cvt.bands);
        }
        Interleave::Bil => {
            let input = Mat::from(Bil::from(inner));
            let mut out = Mat::from(Bil::from(outer));
            input.clamp_between(&mut out, cvt.min, cvt.max, &cvt.bands);
        }
        Interleave::Bsq => {
            let input = Mat::from(Bsq::from(inner));
            let mut out = Mat::from(Bil::from(outer));
            input.clamp_between(&mut out, cvt.min, cvt.max, &cvt.bands);
        }
    }

    Ok(())
}