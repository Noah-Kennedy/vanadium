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
    /// Specifies the IO backend to use. Currently, glommio (io-uring) and syscall are supported.
    #[structopt(long)]
    pub backend: IoBackend,
    /// Subcommand to invoke.
    #[structopt(subcommand)]
    pub op: Operation,
}

#[derive(Debug, StructOpt)]
pub enum Operation {
    /// Calculate the spectral means for all bands.
    Means {
        /// The path to the header file.
        /// Header files must be JSON and follow the JSON header format used by the program.
        #[structopt(long)]
        header: PathBuf,
        /// Output JSON file to store the spectral means in.
        #[structopt(short, long)]
        output: PathBuf,
    },
    /// Calculate the standard deviations for all bands.
    StandardDeviations {
        /// The path to the header file.
        ///
        /// Header files must be JSON and follow the JSON header format used by the program.
        #[structopt(long)]
        header: PathBuf,
        /// Output JSON file to store the spectral standard deviations in.
        #[structopt(short, long)]
        output: PathBuf,
        /// Optional path to a file containing cached spectral means.
        ///
        /// If not present, means will be calculated first.
        #[structopt(short, long)]
        means: Option<PathBuf>,
    },
    /// Calculate the covariance matrix of the bands.
    Covariances {
        /// The path to the header file.
        ///
        /// Header files must be JSON and follow the JSON header format used by the program.
        #[structopt(long)]
        header: PathBuf,
        /// Output JSON file to store the spectral covariances in.
        #[structopt(short, long)]
        output: PathBuf,
        /// Optional path to a file containing spectral means.
        ///
        /// If not present, means will be assumed to be zero.
        /// You should always calculate the means and standard deviations first if your data is not
        /// standardized.
        /// This is an optimization for pre-standardized data.
        #[structopt(short, long)]
        means: Option<PathBuf>,
        /// Optional path to a file containing spectral standard deviations.
        ///
        /// If not present, means will be assumed to be one.
        /// You should always calculate the means and standard deviations first if your data is not
        /// standardized.
        /// This is an optimization for pre-standardized data.
        #[structopt(short, long)]
        std_devs: Option<PathBuf>,
    },
    /// Construct a new header file.
    NewHeader {
        /// Output path for the new JSON header.
        ///
        /// Currently, only BIP is supported, so no format tag is present.
        /// This will change soon.
        #[structopt(short, long)]
        output: PathBuf,
        /// Path of the data file covered by the header.
        #[structopt(short, long)]
        data_path: PathBuf,
        /// Number of channels of the image in the data file.
        #[structopt(short, long)]
        channels: usize,
        /// Number of lines in the image in the data file.
        #[structopt(short, long)]
        lines: usize,
        /// Number of pixels per line in the image in the data file.
        #[structopt(short, long)]
        pixels: usize,
    },
    /// Crop an image.
    Crop {
        /// The path to the header file.
        ///
        /// Header files must be JSON and follow the JSON header format used by the program.
        #[structopt(long)]
        header: PathBuf,
        /// Output path for the new data file.
        #[structopt(short, long)]
        output: PathBuf,
        /// Optional range of rows to be kept.
        ///
        /// Defaults to keep all.
        #[structopt(short, long, number_of_values = 2)]
        rows: Option<Vec<u64>>,
        /// Optional range of columns to be kept.
        ///
        /// Defaults to keep all.
        #[structopt(short, long, number_of_values = 2)]
        cols: Option<Vec<u64>>,
    },
}