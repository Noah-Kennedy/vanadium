use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use ndarray::Array2;

use vanadium_core::headers::Header;
use vanadium_core::image_formats::bip::Bip;

use crate::{BATCH_SIZE, GenericResult};
use crate::bip::BatchedPixelReduce;

pub struct SyscallBip<T> {
    file: File,
    bip: Bip<T>,
}

impl<T> SyscallBip<T> {
    pub fn new(header: Header) -> io::Result<Self> {
        let bip = Bip {
            dims: header.dims,
            phantom: Default::default(),
        };

        let file = File::open(header.path)?;

        Ok(Self {
            file,
            bip,
        })
    }
}

impl BatchedPixelReduce<f32> for SyscallBip<f32> {
    fn reduce_pixels_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> GenericResult<A>
        where F: FnMut(&mut Array2<f32>, &mut A)
    {
        self.file.seek(SeekFrom::Start(0))?;

        let name = name.to_owned();

        make_bar!(pb, self.bip.num_pixels() as u64, name);

        let mut buffer = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        while self.file.read_f32_into::<LittleEndian>(&mut buffer).is_ok() {
            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), buffer)
                .unwrap();

            f(&mut pixel, &mut accumulator);

            buffer = pixel.into_raw_vec();

            inc_bar!(pb, BATCH_SIZE as u64);
        }

        Ok(accumulator)
    }

    fn bip(&self) -> &Bip<f32> {
        &self.bip
    }
}