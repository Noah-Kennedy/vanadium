use std::error::Error;
use std::fs::{OpenOptions, read_to_string};
use std::str::FromStr;

use hyperspectra::header::{Headers, Interleave};

use crate::cli::PcaOpt;
use hyperspectra::container::mapped::Bip;
use hyperspectra::container::{LockImage, PCA};

pub fn execute_pca(op: PcaOpt) -> Result<(), Box<dyn Error>> {
    // unpack PCA cli options
    let PcaOpt {
        input,
        header,
        output,
        dims,
        verbose,
        max,
        min,
    } = op;

    // parse headers
    let headers_str = read_to_string(header)?;
    let mut headers = Headers::from_str(&headers_str)?;

    let input_file = OpenOptions::new()
        .read(true)
        .open(input)?;

    // validate preconditions
    assert_eq!(headers.interleave, Interleave::Bsq,
               "Only BSQ files are supported, please use the 'convert' subcommand to convert your \
                file into a BSQ file."
    );

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
            let output = Bip::<_, f32>::headers_mut(&headers, &input_file)?;

            let input_image = LockImage::new(input);
            let output_image = LockImage::new(output);

            input_image.pca(&output_image, dims as usize, verbose, min, max);
        }
        Interleave::Bil => {}
        Interleave::Bsq => {}
    }

    Ok(())
}