use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;

use vanadium_core::error::VanadiumError;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum IoBackend {
    Glommio,
    Syscall,
}

impl FromStr for IoBackend {
    type Err = VanadiumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "glommio" => Ok(IoBackend::Glommio),
            "syscall" => Ok(IoBackend::Syscall),
            _ => Err(VanadiumError::InvalidArgs("Invalid IO backend".to_owned()))
        }
    }
}


#[derive(Debug, StructOpt)]
#[structopt(name = "Vanadium", about = "A tool for fast hyperspectral image processing.")]
pub struct VanadiumArgs {
    #[structopt(long)]
    pub backend: IoBackend,
    #[structopt(subcommand)]
    pub op: Operation,
}

#[derive(Debug, StructOpt)]
pub enum Operation {
    Means {
        #[structopt(long)]
        header: PathBuf,
        #[structopt(short, long)]
        output: PathBuf,
    },
    StandardDeviations {
        #[structopt(long)]
        header: PathBuf,
        #[structopt(short, long)]
        output: PathBuf,
        #[structopt(short, long)]
        means: Option<PathBuf>,
    },
    Covariances {
        #[structopt(long)]
        header: PathBuf,
        #[structopt(short, long)]
        output: PathBuf,
        #[structopt(short, long)]
        means: Option<PathBuf>,
        #[structopt(short, long)]
        std_devs: Option<PathBuf>,
    },
    NewHeader {
        #[structopt(short, long)]
        output: PathBuf,
        #[structopt(short, long)]
        data_path: PathBuf,
        #[structopt(short, long)]
        channels: usize,
        #[structopt(short, long)]
        lines: usize,
        #[structopt(short, long)]
        pixels: usize,
    }
}