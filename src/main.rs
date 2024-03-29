#[macro_use]
extern crate ndarray;
#[cfg(feature = "netlib")]
extern crate netlib_src;
#[cfg(feature = "openblas")]
extern crate openblas_src;
#[macro_use]
extern crate serde;

use std::error::Error;
use std::fs::{File, OpenOptions};

use structopt::StructOpt;

use crate::error::VanadiumError;
use crate::headers::{Header, ImageDims, ImageFormat};
use crate::io::BasicImage;
#[cfg(feature = "glommio")]
use crate::io::bip::GlommioBip;
#[cfg(feature = "syscall-backend")]
use crate::io::bip::SyscallBip;
#[cfg(feature = "memmap2")]
use crate::io::mapped::bip::MappedBip;
use crate::opt::{IoBackend, Operation, VanadiumArgs};
use crate::io::tokio::bip::TokioBip;

#[cfg(not(tarpaulin_include))]
mod headers;

mod image_formats;

#[cfg(not(tarpaulin_include))]
mod error;

mod io;

mod util;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests;

#[cfg(not(tarpaulin_include))]
mod opt;

#[cfg(not(tarpaulin_include))]
fn get_image(backend: IoBackend, headers: Header<String>) -> Box<dyn BasicImage<f32>> {
    assert_eq!(ImageFormat::Bip, headers.format);
    match backend {
        #[cfg(feature = "glommio-backend")]
        IoBackend::Glommio => Box::new(GlommioBip::new(headers).unwrap()),
        #[cfg(feature = "tokio-backend")]
        IoBackend::Tokio => Box::new(TokioBip::new(headers).unwrap()),
        #[cfg(feature = "syscall-backend")]
        IoBackend::Syscall => Box::new(SyscallBip::new(headers).unwrap()),
        #[cfg(feature = "mapped-backend")]
        IoBackend::Mapped => Box::new(MappedBip::new(headers).unwrap()),
        #[cfg(not(all(
        feature = "mapped-backend",
        feature = "glommio-backend",
        feature = "syscall-backend",
        feature = "tokio-backend"
        )))]
        _ => panic!("Unknown backend!")
    }
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Box<dyn Error>> {
    let args: VanadiumArgs = VanadiumArgs::from_args();

    match args.op {
        Operation::Means { header, output } => {
            let header = serde_json::from_reader(File::open(header).unwrap()).map_err(|_|
                VanadiumError::InvalidHeader)?;
            let mut image = get_image(args.backend, header);

            let means = image.means()?;

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .unwrap();

            serde_json::to_writer(file, &means).unwrap();
        }
        Operation::StandardDeviations { header, output, means } => {
            let header = serde_json::from_reader(File::open(header).unwrap()).map_err(|_|
                VanadiumError::InvalidHeader)?;
            let mut image = get_image(args.backend, header);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .unwrap();

            let means = if let Some(m) = means {
                serde_json::from_reader(File::open(m)?)?
            } else {
                image.means()?
            };

            let std_devs = image.std_deviations(&means)?;

            serde_json::to_writer(file, &std_devs).unwrap();
        }
        Operation::Covariances { header, output, means, std_devs } => {
            let header = serde_json::from_reader(File::open(header).unwrap()).map_err(|_|
                VanadiumError::InvalidHeader)?;
            let mut image = get_image(args.backend, header);

            let means = means.map(|x| serde_json::from_reader(File::open(x).unwrap()).unwrap());
            let std_devs = std_devs.map(|x| serde_json::from_reader(File::open(x).unwrap()).unwrap());

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .unwrap();

            let cov = image.covariance_matrix(means.as_ref(), std_devs.as_ref())?;

            serde_json::to_writer(file, &cov).unwrap();
        }
        Operation::NewHeader { output, data_path, channels, lines, pixels } => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .unwrap();

            let header = Header {
                dims: ImageDims {
                    channels,
                    lines,
                    pixels,
                },
                format: ImageFormat::Bip,
                path: data_path,
            };

            serde_json::to_writer(file, &header).unwrap();
        }
        Operation::Crop { header, output, rows, cols } => {
            let rows = rows.map(|x| (x[0], x[1]));
            let cols = cols.map(|x| (x[0], x[1]));

            let header = serde_json::from_reader(File::open(header)?)?;

            let mut image = get_image(args.backend, header);

            image.crop(rows, cols, &output)?;
        }
    }

    Ok(())
}