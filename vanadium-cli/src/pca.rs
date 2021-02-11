use std::error::Error;
use std::fs::{OpenOptions, read_to_string};
use std::str::FromStr;

use vanadium_core::container::{LockImage, PCA};
use vanadium_core::container::mapped::{Bip, Bsq};
use vanadium_core::header::{Headers, Interleave};

use crate::cli::PcaOpt;

pub fn execute_pca(op: PcaOpt) -> Result<(), Box<dyn Error>> {
    // unpack PCA cli options
    let PcaOpt {
        input,
        header,
        output,
        csv: csv_out,
        dims,
        verbose,
        max,
        min,
    } = op;

    // parse headers
    let headers_str = read_to_string(header)?;
    let mut headers = Headers::from_str(&headers_str)?;

    let input_file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(input)?;

    if csv_out {
        let eigen = match headers.interleave {
            Interleave::Bip => {
                let input = Bip::<_, f32>::headers_mut(&headers, &input_file)?;
                let input_image = LockImage::new(input);

                input_image.pca_eigen(verbose, min, max)
            }
            Interleave::Bil => unimplemented!(),
            Interleave::Bsq => {
                let input = Bsq::<_, f32>::headers_mut(&headers, &input_file)?;
                let input_image = LockImage::new(input);

                input_image.pca_eigen(verbose, min, max)
            }
        };

        let mut writer = csv::Writer::from_path(output)?;

        for (value, vector) in eigen.eigenvalues.iter().zip(eigen.eigenvectors.row_iter()) {
            let mut record = vec![*value];

            for v in vector.iter() {
                record.push(*v);
            }

            writer.serialize(record)?;
        }
    } else {
        let output_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(output)?;

        match headers.interleave {
            Interleave::Bip => {
                let input = Bip::<_, f32>::headers_mut(&headers, &input_file)?;
                headers.bands = dims as usize;
                output_file.set_len(headers.bands as u64 * headers.lines as u64 * headers.samples as u64 * 4)?;
                let output = Bip::<_, f32>::headers_mut(&headers, &output_file)?;

                let input_image = LockImage::new(input);
                let output_image = LockImage::new(output);

                input_image.pca(&output_image, verbose, min, max);
            }
            Interleave::Bil => unimplemented!(),
            Interleave::Bsq => {
                let input = Bsq::<_, f32>::headers_mut(&headers, &input_file)?;

                headers.bands = dims as usize;
                output_file.set_len(headers.bands as u64 * headers.lines as u64 * headers.samples as u64 * 4)?;

                let output = Bsq::<_, f32>::headers_mut(&headers, &output_file)?;

                let input_image = LockImage::new(input);
                let output_image = LockImage::new(output);

                input_image.pca(&output_image, verbose, min, max);
            }
        }
    }

    Ok(())
}