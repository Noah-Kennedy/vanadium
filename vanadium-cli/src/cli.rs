use std::path::PathBuf;

use vanadium_core::header::Interleave;

#[derive(Clap, Debug)]
#[clap(version = "1.0.0-alpha", author = "Noah M. Kennedy <noah.kennedy.professional@gmail.com>")]
pub struct Opt {
    #[clap(subcommand)]
    pub subcommand: SubcommandOpt
}

#[derive(Clap, Debug)]
pub enum SubcommandOpt {
    Convert(ConvertOpt),
    Color(RenderOpt),
    Pca(PcaOpt),
}

/// Subcommand for converting between any one of the following supported file types: BIP, BSQ, BIL.
#[derive(Clap, Debug)]
#[clap(name = "convert")]
pub struct ConvertOpt {
    /// The path to the input binary file.
    #[clap(short, long, parse(from_str))]
    pub input: PathBuf,

    /// The path to the input header file.
    #[clap(short = 'd', long, parse(from_os_str))]
    pub input_header: PathBuf,

    /// The path to the output binary file.
    #[clap(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// The output file type to use.
    /// Must be bip, bsq or bil.
    #[clap(short = 't', long, parse(try_from_str))]
    pub output_type: Interleave,
}

/// Perform Principal Component Analysis on an image.
///
/// This will write out a BSQ file with the top PCA dims.
///
/// The output will be standardized to z-score to maintain a uniform scale between bands.
/// As a result, the output will contain negative values.
///
/// The background of the image will be negative infinity.
#[derive(Clap, Debug)]
#[clap(name = "pca")]
pub struct PcaOpt {
    /// The path to the input binary file.
    #[clap(short, long, parse(from_str))]
    pub input: PathBuf,

    /// The path to the input header file.
    #[clap(short = 'd', long, parse(from_os_str))]
    pub header: PathBuf,

    #[clap(short, long)]
    pub verbose: bool,

    /// Optional argument, anything greater less than or equal to this value will be excluded from
    /// PCA calculations.
    #[clap(short = 'n', long)]
    pub min: Option<f32>,

    /// Optional argument, anything greater than this value will be excluded from PCA calculations.
    #[clap(short = 'm', long)]
    pub max: Option<f32>,

    #[clap(subcommand)]
    pub subcommand: PCASubcommand,
}

#[derive(Clap, Debug)]
pub enum PCASubcommand {
    Transform(PCAWriteOut),
    Solve(PCASolve),
}

/// Write out then PCA transformed results to a binary file.
/// This will generate a new file of the same type used for the input file.
#[derive(Clap, Debug)]
#[clap(name = "transform")]
pub struct PCAWriteOut {
    /// The path to the output binary file.
    #[clap(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// The number of bands to keep.
    #[clap(short, long)]
    pub dims: u64,
}

/// Write out the eigenvalues and eigenvectors found by PCA to a CSV file.
#[derive(Clap, Debug)]
#[clap(name = "solve")]
pub struct PCASolve {
    /// The path to the output CSV file.
    #[clap(short, long, parse(from_os_str))]
    pub output: PathBuf,
}

/// Render a viewable image from raw data.
#[derive(Clap, Debug)]
#[clap(name = "render")]
pub struct RenderOpt {
    /// The path to the input binary file.
    #[clap(short, long, parse(from_os_str))]
    pub input: PathBuf,

    /// The path to the input header file.
    #[clap(short = 'd', long, parse(from_os_str))]
    pub header: PathBuf,

    /// The path to the output image file.
    /// Supported file extensions include jpg, png, tiff, tga, bmp, and pnm.
    #[clap(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// Subcommand for describing the rendering scheme to use.
    #[clap(subcommand)]
    pub subcommand: RenderSubcommand,
}

/// The rendering scheme to use.
#[derive(Clap, Debug)]
pub enum RenderSubcommand {
    Rgb(RenderRGBOptions),
    Solid(RenderSingleBandOpt),
    Mask(RenderMaskOpt),
}

/// Take three channels of a file and render them together as R, G, and B bands in an image.
#[derive(Clap, Debug)]
#[clap(name = "rgb")]
pub struct RenderRGBOptions {
    /// The bands to be used for rendering a file.
    /// These should be in R,G,B order
    #[clap(short, long)]
    pub bands: Vec<usize>,

    /// The minimums to be used for rendering a file.
    /// These should be in R,G,B order
    #[clap(short = 'n', long)]
    pub minimums: Vec<f32>,

    /// The maximums to be used for rendering a file.
    /// These should be in R,G,B order
    #[clap(short = 'm', long)]
    pub maximums: Vec<f32>,
}

#[derive(Clap, Debug)]
pub struct RenderMaskOpt {}

/// Render a single band of a file.
#[derive(Clap, Debug)]
#[clap(name = "single-band")]
pub struct RenderSingleBandOpt {
    #[clap(long)]
    pub band: usize,

    /// The minimum value to be used for rendering a file.
    /// This will be applied as a floor for our calculations
    #[clap(short = 'n', long)]
    pub min: f32,

    /// The maximum value to be used for rendering a file.
    /// This will be applied as a ceiling for our calculations
    #[clap(short = 'm', long)]
    pub max: f32,

    /// The color map to output the image to.
    #[clap(subcommand)]
    pub color: Color,
}

/// The colors that you can choose between for rendering a single band.
#[derive(Clap, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Color {
    Red,
    Blue,
    Green,
    Teal,
    Purple,
    Yellow,
    Gray,
}