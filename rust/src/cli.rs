use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
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

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "bip2bsq")]
pub struct ConvertOpt {
    #[structopt(short, long, parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    pub header: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
}