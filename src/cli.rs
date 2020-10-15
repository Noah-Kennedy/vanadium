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
    Convert(ConvertOpt)
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