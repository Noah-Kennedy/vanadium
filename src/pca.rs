use std::error::Error;
use std::fs::{OpenOptions, read_to_string};
use std::str::FromStr;

use crate::bin_formats::{SpectralImage, SpectralImageContainer};
use crate::bin_formats::bsq::Bsq;
use crate::cli::PcaOpt;
use crate::headers::{Headers, Interleave};

pub fn execute_pca(op: PcaOpt) -> Result<(), Box<dyn Error>> {
    // unpack PCA cli options
    let PcaOpt {
        input,
        header,
        output,
        bands,
        verbose,
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

    let inner = SpectralImageContainer::headers(&headers, &input_file)?;

    let index = Bsq::from(inner.dims.clone());
    let input = SpectralImage {
        inner,
        index,
    };

    headers.bands = bands as usize;

    output_file.set_len(headers.bands as u64 * headers.lines as u64 * headers.samples as u64 * 4)?;

    headers.interleave = Interleave::Bsq;

    let inner = SpectralImageContainer::headers_mut(&headers, &output_file)?;

    let index = Bsq::from(inner.dims.clone());
    let mut out = SpectralImage {
        inner,
        index,
    };

    input.pca(&mut out, bands, verbose);

    Ok(())
}