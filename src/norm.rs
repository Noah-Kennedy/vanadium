use std::error::Error;
use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;

use image::{GrayImage, RgbImage};

use crate::bin_formats::{FileIndex, FileIndexMut, FileInner, Mat};
use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::cli::NormOpt;
use crate::headers::{Headers, Interleave};

pub fn normalize(opt: NormOpt) -> Result<(), Box<dyn Error>> {
    println!("Opening input file");
    let input_file = File::open(opt.input)?;

    println!("Reading headers");
    let headers_str = read_to_string(opt.input_header)?;
    let parsed_headers = Headers::from_str(&headers_str)?;

    println!("Mapping input file");
    let inner: FileInner<_, f32> = unsafe { FileInner::headers_copy(&parsed_headers, &input_file)? };

    match parsed_headers.interleave {
        Interleave::Bip => {
            let mut input = Mat::from(Bip::from(inner));
            helper(&mut input, opt.output, &opt.format, &opt.min, &opt.max, &opt.bands)
        }
        Interleave::Bil => {
            let mut input = Mat::from(Bil::from(inner));
            helper(&mut input, opt.output, &opt.format, &opt.min, &opt.max, &opt.bands)
        }
        Interleave::Bsq => {
            let mut input = Mat::from(Bsq::from(inner));
            helper(&mut input, opt.output, &opt.format, &opt.min, &opt.max, &opt.bands)
        }
    }
}

fn helper<F>(input: &mut Mat<F>, path: PathBuf, f: &str, min: &[f32], max: &[f32], bands: &[usize])
             -> Result<(), Box<dyn Error>>
    where F: 'static + FileIndex<f32> + FileIndexMut<f32> + Sync + Send
{
    println!("Normalizing");
    input.norm_between(min, max, bands);

    let (height, width, _) = input.inner.size();

    match f {
        "coolwarm" => {
            println!("Allocating output buffer");
            let mut out = RgbImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width * 3],
            ).unwrap();

            println!("Applying color map");
            input.cool_warm(&mut out, bands[0]);

            println!("Saving...");
            out.save(path)?;
        }
        "rgb" => {
            println!("Allocating output buffer");
            let mut out = RgbImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width * 3],
            ).unwrap();

            println!("Applying color map");
            input.rgb(&mut out, [bands[0], bands[1], bands[2]]);

            println!("Saving...");
            out.save(path)?;
        }
        "gray" | "grey" => {
            println!("Allocating output buffer");
            let mut out = GrayImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width],
            ).unwrap();

            println!("Applying color map");
            input.gray(&mut out, bands[0]);

            println!("Saving...");
            out.save(path)?;
        }
        _ => unimplemented!()
    };

    Ok(())
}
