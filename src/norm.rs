use std::error::Error;
use std::fs::{File, read_to_string};
use std::path::PathBuf;
use std::str::FromStr;

use image::{GrayImage, RgbImage};

use crate::bin_formats::{FileIndex, FileInner, Mat};
use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
use crate::cli::ColorOpt;
use crate::headers::{Headers, Interleave};

pub fn normalize(opt: ColorOpt) -> Result<(), Box<dyn Error>> {
    println!("Opening input file");
    let input_file = File::open(opt.input)?;

    println!("Reading headers");
    let headers_str = read_to_string(opt.input_header)?;
    let parsed_headers = Headers::from_str(&headers_str)?;

    println!("Mapping input file");
    let inner: FileInner<_, f32> = unsafe { FileInner::headers(&parsed_headers, &input_file)? };

    match parsed_headers.interleave {
        Interleave::Bip => {
            let input = Mat::from(Bip::from(inner));
            helper(&input, opt.output, &opt.color_map, &opt.min, &opt.max, &opt.bands)
        }
        Interleave::Bil => {
            let input = Mat::from(Bil::from(inner));
            helper(&input, opt.output, &opt.color_map, &opt.min, &opt.max, &opt.bands)
        }
        Interleave::Bsq => {
            let input = Mat::from(Bsq::from(inner));
            helper(&input, opt.output, &opt.color_map, &opt.min, &opt.max, &opt.bands)
        }
    }
}

fn helper<F>(input: &Mat<f32, F>, path: PathBuf, f: &str, min: &[f32], max: &[f32], bands: &[usize])
             -> Result<(), Box<dyn Error>>
    where F: 'static + FileIndex<f32> + Sync + Send
{
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
            input.cool_warm(&mut out, min[0], max[0], bands[0]);

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
            input.rgb(&mut out,
                      [min[0], min[1], min[2]],
                      [max[0], max[1], max[2]],
                      [bands[0], bands[1], bands[2]],
            );

            println!("Saving...");
            out.save(path)?;
        }
        "gray" | "grey" => {
            println!("Allocating output buffer");
            let mut out = GrayImage::from_raw(Moved benchmark files to new dir.
                width as u32,
                height as u32,
                vec![0; height * width],
            ).unwrap();

            println!("Applying color map");
            input.gray(&mut out, min[0], max[0], bands[0]);

            println!("Saving...");
            out.save(path)?;
        }
        _ => unimplemented!()
    };

    Ok(())
}
