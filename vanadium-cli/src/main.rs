use std::error::Error;
use std::fs::{File, OpenOptions};

use structopt::StructOpt;

use vanadium_core::headers::{Header, ImageFormat};
use vanadium_core::io::bip::{GlommioBip, SyscallBip};
use vanadium_core::io::ImageStats;

use crate::opt::{IoBackend, Operation, VanadiumArgs};

mod opt;

fn get_image(backend: IoBackend, headers: Header) -> Box<dyn ImageStats<f32>> {
    assert_eq!(ImageFormat::Bip, headers.format);
    match backend {
        IoBackend::Glommio => Box::new(GlommioBip::new(headers)),
        IoBackend::Syscall => Box::new(SyscallBip::new(headers).unwrap())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: VanadiumArgs = VanadiumArgs::from_args();

    match args.op {
        Operation::Means { header, output } => {
            let header: Header = serde_json::from_reader(File::open(header).unwrap()).unwrap();
            let mut image = get_image(args.backend, header);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .unwrap();

            let means = image.means()?;

            serde_json::to_writer(file, &means).unwrap();
        }
        Operation::StandardDeviations { header, output, means } => {
            let header: Header = serde_json::from_reader(File::open(header)?)?;
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
            let header: Header = serde_json::from_reader(File::open(header)?)?;
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
    }

    Ok(())
}