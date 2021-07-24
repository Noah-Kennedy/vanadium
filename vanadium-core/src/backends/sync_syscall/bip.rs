use std::{io, mem};
use std::fs::File;
use std::io::{Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use ndarray::Array2;

use crate::backends::{BATCH_SIZE, BatchedPixelReduce, GenericResult};
use crate::headers::Header;
use crate::specialization::bip::Bip;

pub struct SyncBip<T> {
    file: File,
    bip: Bip<T>,
}

impl<T> SyncBip<T> {
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

impl BatchedPixelReduce<f32> for SyncBip<f32> {
    fn reduce_pixels_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> GenericResult<A>
        where F: FnMut(&mut Array2<f32>, &mut A)
    {
        self.file.seek(SeekFrom::Start(0))?;

        let name = name.to_owned();

        make_bar!(pb, self.bip.num_pixels() as u64, name);

        let mut buffer = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        while self.file.read_f32_into::<LittleEndian>(&mut buffer).is_ok() {
            let mut tmp = Vec::new();
            mem::swap(&mut tmp, &mut buffer);

            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp).unwrap();

            f(&mut pixel, &mut accumulator);

            tmp = pixel.into_raw_vec();
            mem::swap(&mut tmp, &mut buffer);

            inc_bar!(pb, BATCH_SIZE as u64);
        }

        Ok(accumulator)
    }

    fn bip(&self) -> &Bip<f32> {
        &self.bip
    }
}