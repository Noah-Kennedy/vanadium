use std::path::PathBuf;

use structopt::StructOpt;

use crate::headers::Interleave;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcommand: SubcommandOpt
}

#[derive(StructOpt, Debug)]
pub enum SubcommandOpt {
    Convert(ConvertOpt),
    Color(ColorOpt),
}

/// Subcommand for converting between any one of the following supported file types: BIP, BSQ, BIL.
#[derive(StructOpt, Debug)]
#[structopt(name = "convert")]
pub struct ConvertOpt {
    /// The path to the input binary file.
    #[structopt(short, long, parse(from_str))]
    pub input: PathBuf,

    /// The path to the input header file.
    #[structopt(short = "n", long, parse(from_os_str))]
    pub input_header: PathBuf,

    /// The path to the output binary file.
    #[structopt(short = "o", long, parse(from_os_str))]
    pub output: PathBuf,

    /// The path to the output header file to be generated.
    /// Currently this flag does nothing.
    #[structopt(short = "u", long, parse(from_os_str))]
    pub output_header: Option<PathBuf>,

    /// The output file type to use.
    /// Must be bip, bsq or bil.
    #[structopt(short = "t", long, parse(try_from_str))]
    pub output_type: Interleave,
}

/// Subcommand for outputting color images.
///
/// # Examples
///
/// ## RGB
///
/// ```sh
/// hyperspectra color -i input.bsq -n input.hdr -o rgb.png -m 0 0 0 -x 0.5 0.5 1 -b 1 3 4 -c rgb
/// ```
///
/// ## Grayscale
///
/// ```sh
/// hyperspectra color -i input.bsq -n input.hdr -o gray.png -m 0 -x 0.5 -b 3 -c gray
/// ```
///
/// ## Coolwarm
///
/// ```sh
/// hyperspectra color -i input.bsq -n input.hdr -o coolwarm.png -m 0 -x 0.5 -b 3 -c coolwarm
/// ```
#[derive(StructOpt, Debug)]
#[structopt(name = "color")]
pub struct ColorOpt {
    /// The path to the input binary file.
    #[structopt(short, long, parse(from_os_str))]
    pub input: PathBuf,

    /// The path to the input header file.
    #[structopt(short = "d", long, parse(from_os_str))]
    pub header: PathBuf,

    /// The path to the output image file.
    /// The file should have a .png, .jpg, or .jpeg extension
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// The floor to clamp to for each band.
    ///
    /// If the colormap is 'gray', 'grey', or 'coolwarm', 1 value should be provided.
    /// If the colormap is 'rgb', 3 values should be provided.
    #[structopt(long)]
    pub minimums: Vec<f32>,

    /// The ceiling to clamp to for each band.
    ///
    /// If the colormap is 'gray', 'grey', or 'coolwarm', 1 value should be provided.
    /// If the colormap is 'rgb', 3 values should be provided.
    #[structopt(long)]
    pub maximums: Vec<f32>,

    /// The bands to work with.
    ///
    /// If the colormap is 'gray', 'grey', or 'coolwarm', 1 value should be provided.
    /// If the colormap is 'rgb', 3 values should be provided.
    #[structopt(long)]
    pub bands: Vec<usize>,

    #[structopt(short = "r", long)]
    pub red_bands: Vec<usize>,

    #[structopt(short = "b", long)]
    pub blue_bands: Vec<usize>,

    #[structopt(short = "g", long)]
    pub green_bands: Vec<usize>,

    /// The color map of the image.
    ///
    /// If 'gray', 'grey', 'mask', or 'coolwarm', 1 band should be provided.
    /// If 'rgb', 3 bands should be provided.
    ///
    /// If 'mask', the max flag will be ignored.
    #[structopt(short = "c", long)]
    pub color_map: String,
}