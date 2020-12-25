use std::error::Error;
use std::fs::{File, read_to_string};
use std::ops::Deref;
use std::str::FromStr;

use image::{GrayImage, RgbImage};

use envi_header::{Headers, Interleave};
use envi_mapped_image::{ColorFlag, SpectralImage, SpectralImageContainer};

use crate::cli::ColorOpt;
use envi_image::{ImageIndex, FileDims, Bip, Bil, Bsq};

pub fn normalize(opt: ColorOpt) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(opt.input.clone())?;

    let headers_str = read_to_string(opt.header.clone())?;
    let parsed_headers = Headers::from_str(&headers_str)?;

    let inner: SpectralImageContainer<_, f32> = SpectralImageContainer::headers(&parsed_headers, &input_file)?;

    match parsed_headers.interleave {
        Interleave::Bip => {
            let index = Bip::from(inner.dims.clone());
            let input = SpectralImage {
                inner,
                index,
            };
            helper(&input, &opt)
        }
        Interleave::Bil => {
            let index = Bil::from(inner.dims.clone());
            let input = SpectralImage {
                inner,
                index,
            };
            helper(&input, &opt)
        }
        Interleave::Bsq => {
            let index = Bsq::from(inner.dims.clone());
            let input = SpectralImage {
                inner,
                index,
            };
            helper(&input, &opt)
        }
    }
}

fn helper<C, I>(
    input: &SpectralImage<C, f32, I>,
    opt: &ColorOpt,
)
    -> Result<(), Box<dyn Error>>
    where I: 'static + ImageIndex + Sync + Send + Copy + Clone,
          C: Deref<Target=[u8]> + Sync + Send,
{
    let FileDims { samples, lines, .. } = input.inner.size();
    let height = lines;
    let width = samples;
    let min = &opt.minimums;
    let max = &opt.maximums;
    let bands = &opt.bands;
    let path = &opt.output;
    let color = opt.color_map.as_str();
    let reds = &opt.red_bands;
    let greens = &opt.green_bands;
    let blues = &opt.blue_bands;

    match color {
        "rgb" => {
            let mut out = RgbImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width * 3],
            ).unwrap();

            println!("Applying color map");
            input.rgb(&mut out, min, max, bands, [reds, greens, blues]);

            println!("Saving...");
            out.save(path)?;
        }
        "gray" | "grey" => {
            let mut out = GrayImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width],
            ).unwrap();

            println!("Applying color map");
            input.gray(&mut out, min[0], max[0], bands[0]);

            println!("Saving...");
            out.save(path)?;
        }
        "green" | "red" | "blue" | "purple" | "yellow" | "teal" => {
            println!("Allocating output buffer");
            let mut out = RgbImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width * 3],
            ).unwrap();

            let flag = match color {
                "green" => ColorFlag::Green,
                "red" => ColorFlag::Red,
                "blue" => ColorFlag::Blue,
                "purple" => ColorFlag::Purple,
                "yellow" => ColorFlag::Yellow,
                "teal" => ColorFlag::Teal,
                _ => unreachable!()
            };

            println!("Applying color map");
            input.solid(&mut out, min[0], max[0], bands[0], flag);

            println!("Saving...");
            out.save(path)?;
        }
        "mask" => {
            let mut out = GrayImage::from_raw(
                width as u32,
                height as u32,
                vec![0; height * width],
            ).unwrap();

            println!("Applying color map");
            input.mask(&mut out, min[0]);

            println!("Saving...");
            out.save(path)?;
        }
        _ => unimplemented!()
    };

    Ok(())
}
