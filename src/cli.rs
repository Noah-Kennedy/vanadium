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

#[derive(StructOpt, Debug)]
#[structopt(name = "convert")]
pub struct ConvertOpt {
    #[structopt(short, long, parse(from_str))]
    pub input: PathBuf,

    #[structopt(short = "n", long, parse(from_os_str))]
    pub input_header: PathBuf,

    #[structopt(short = "o", long, parse(from_os_str))]
    pub output: PathBuf,

    #[structopt(short = "u", long, parse(from_os_str))]
    pub output_header: PathBuf,

    #[structopt(short = "t", long, parse(try_from_str))]
    pub output_type: Interleave,
}