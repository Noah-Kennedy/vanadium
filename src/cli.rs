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

    #[structopt(short, long, parse(from_os_str))]
    pub header: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,

    #[structopt(short, long, parse(try_from_str))]
    pub output_type: Interleave,
}