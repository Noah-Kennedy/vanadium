use std::error::Error;
use std::fs::{File, read_to_string};
use std::str::FromStr;

use image::{GrayImage, RgbImage};

use vanadium_core::container::{ColorFlag, ImageDims, IndexImage, IterableImage, LockImage, Render};
use vanadium_core::container::mapped::{Bip, Bsq};
use vanadium_core::header::{Headers, Interleave};

use crate::cli::{Color, RenderOpt, RenderRGBOptions, RenderSingleBandOpt, RenderSubcommand};

pub fn normalize(opt: RenderOpt) -> Result<(), Box<dyn Error>> {
    let RenderOpt { input, header, .. } = &opt;

    let input_file = File::open(input.clone())?;

    let headers_str = read_to_string(header.clone())?;
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

fn helper<'a, I>(
    input: &LockImage<f32, I>,
    opt: &RenderOpt,
)
    -> Result<(), Box<dyn Error>>
    where I: IterableImage<'a, f32> + Sync + IndexImage<f32> + 'static,
{
    let guard = input.read();
    let ImageDims { samples, lines, .. } = guard.inner.dims();

    let subcommand = &opt.subcommand;

    let path = &opt.output;

    match subcommand {
        RenderSubcommand::Rgb(rgb) => {
            let RenderRGBOptions { bands, minimums, maximums } = &rgb;
            let mut out = RgbImage::from_raw(
                samples as u32,
                lines as u32,
                vec![0; lines * samples * 3],
            ).unwrap();

            println!("Applying color map");
            input.rgb(&mut out, minimums, maximums, bands);

            println!("Saving...");
            out.save(path)?;
        }
        RenderSubcommand::Solid(solid) => {
            let RenderSingleBandOpt { band, min, max, color } = &solid;
            if *color == Color::Gray {
                let mut out = GrayImage::from_raw(
                    samples as u32,
                    lines as u32,
                    vec![0; lines * samples],
                ).unwrap();

                println!("Applying color map");
                input.gray(&mut out, *min, *max, *band);

                println!("Saving...");
                out.save(path)?;
            } else {
                println!("Allocating output buffer");
                let mut out = RgbImage::from_raw(
                    samples as u32,
                    lines as u32,
                    vec![0; lines * samples * 3],
                ).unwrap();

                let flag = match solid.color {
                    Color::Red => ColorFlag::Red,
                    Color::Blue => ColorFlag::Blue,
                    Color::Green => ColorFlag::Green,
                    Color::Teal => ColorFlag::Teal,
                    Color::Purple => ColorFlag::Purple,
                    Color::Yellow => ColorFlag::Yellow,
                    Color::Gray => unreachable!()
                };

                println!("Applying color map");
                input.solid(&mut out, *min, *max, *band, flag);

                println!("Saving...");
                out.save(path)?;
            }
        }
        RenderSubcommand::Mask(_) => {
            unimplemented!()
        }
    }

    Ok(())
}
