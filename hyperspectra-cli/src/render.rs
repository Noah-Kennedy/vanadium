use std::error::Error;
use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Div, Sub};

use num::{Bounded, FromPrimitive, ToPrimitive};
use num::traits::NumAssign;

use hyperspectra::container::{ImageDims, IndexImage, IterableImage, LockImage, ColorFlag, Render};

use crate::cli::ColorOpt;
use image::RgbImage;
use nalgebra::RealField;
use std::fs::{read_to_string, File};
use hyperspectra::header::{Headers, Interleave};
use std::str::FromStr;
use hyperspectra::container::mapped::{Bip, Bsq};

pub fn normalize(opt: ColorOpt) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(opt.input.clone())?;

    let headers_str = read_to_string(opt.header.clone())?;
    let parsed_headers = Headers::from_str(&headers_str)?;

    match parsed_headers.interleave {
        Interleave::Bip => {
            let b: Bip<_, f32> = Bip::headers(&parsed_headers, &input_file)?;

            let l = LockImage::new(b);

            helper(&l, &opt)
        }
        Interleave::Bil => {
            todo!()
        }
        Interleave::Bsq => {
            let b: Bsq<_, f32> = Bsq::headers(&parsed_headers, &input_file)?;

            let l = LockImage::new(b);

            helper(&l, &opt)
        }
    }

}

fn helper<'a, T, I>(
    input: &LockImage<T, I>,
    opt: &ColorOpt,
)
    -> Result<(), Box<dyn Error>>
    where I: IterableImage<'a, T> + Sync + IndexImage<T> + 'static,
          T: NumAssign + Copy + PartialOrd + 'static + Debug + Send + Sync + Bounded + Sum + Div
          + Sub + FromPrimitive + ToPrimitive + RealField
{
    let guard = input.read();
    let ImageDims { samples, lines, .. } = guard.inner.dims();
    let height = lines;
    let width = samples;
    let min = &opt.minimums;
    let max = &opt.maximums;
    let bands = &opt.bands;
    let path = &opt.output;
    let color = opt.color_map.as_str();
    let _reds = &opt.red_bands;
    let _greens = &opt.green_bands;
    let _blues = &opt.blue_bands;

    match color {
        "rgb" => {
            todo!()

            // let mut out = RgbImage::from_raw(
            //     width as u32,
            //     height as u32,
            //     vec![0; height * width * 3],
            // ).unwrap();
            //
            // println!("Applying color map");
            // input.rgb(&mut out, min, max, bands, [reds, greens, blues]);
            //
            // println!("Saving...");
            // out.save(path)?;
        }
        "gray" | "grey" => {
            todo!()

            // let mut out = GrayImage::from_raw(
            //     width as u32,
            //     height as u32,
            //     vec![0; height * width],
            // ).unwrap();
            //
            // println!("Applying color map");
            // input.gray(&mut out, min[0], max[0], bands[0]);
            //
            // println!("Saving...");
            // out.save(path)?;
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
            input.solid(&mut out, T::from_f32(min[0]).unwrap(),
                        T::from_f32(max[0]).unwrap(), bands[0], flag);

            println!("Saving...");
            out.save(path)?;
        }
        "mask" => {
            todo!()
            // let mut out = GrayImage::from_raw(
            //     width as u32,
            //     height as u32,
            //     vec![0; height * width],
            // ).unwrap();
            //
            // println!("Applying color map");
            // input.mask(&mut out, min[0]);
            //
            // println!("Saving...");
            // out.save(path)?;
        }
        _ => unimplemented!()
    };

    Ok(())
}
