use std::fs::File;
use std::{io, mem};
use std::io::{Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{GenericResult, Image};
use crate::headers::Header;
use crate::specialization::bip::Bip;
use ndarray::{Array2, Array1};
use crate::backends::BATCH_SIZE;

pub struct SyncBip<T> {
    file: File,
    bip: Bip<T>,
}

impl <T> SyncBip<T> {
    pub fn new(header: Header) -> io::Result<Self> {
        let bip = Bip {
            dims: header.dims,
            phantom: Default::default()
        };

        let file = File::open(header.path)?;

        Ok(Self {
            file,
            bip,
        })
    }
}

impl Image<f32> for SyncBip<f32> {
    fn means(&mut self) -> GenericResult<Array1<f32>> {
        self.file.seek(SeekFrom::Start(0))?;

        make_bar!(pb, self.bip.num_pixels() as u64, "mean");

        let mut accumulator = Array1::zeros(self.bip.pixel_length());
        let mut buffer = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        while self.file.read_f32_into::<LittleEndian>(&mut buffer).is_ok() {
            let mut tmp = Vec::new();
            mem::swap(&mut tmp, &mut buffer);

            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp).unwrap();

            self.bip.map_mean(&mut pixel, &mut accumulator);

            tmp = pixel.into_raw_vec();
            mem::swap(&mut tmp, &mut buffer);

            inc_bar!(pb, BATCH_SIZE as u64);
        }

        self.bip.reduce_mean(&mut accumulator);

        Ok(accumulator)
    }

    fn std_deviations(&mut self, means: &Array1<f32>) -> GenericResult<Array1<f32>> {
        self.file.seek(SeekFrom::Start(0))?;

        make_bar!(pb, self.bip.num_pixels() as u64, "std");

        let mut accumulator = Array1::zeros(self.bip.pixel_length());
        let mut buffer = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        while self.file.read_f32_into::<LittleEndian>(&mut buffer).is_ok() {
            let mut tmp = Vec::new();
            mem::swap(&mut tmp, &mut buffer);

            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp).unwrap();

            self.bip.map_std_dev(&mut pixel, means, &mut accumulator);

            tmp = pixel.into_raw_vec();
            mem::swap(&mut tmp, &mut buffer);

            inc_bar!(pb, BATCH_SIZE as u64);
        }

        self.bip.reduce_std_dev(&mut accumulator);

        Ok(accumulator)
    }

    fn covariance_matrix(&mut self, means: &Array1<f32>, std_devs: &Array1<f32>) -> GenericResult<Array2<f32>> {
        self.file.seek(SeekFrom::Start(0))?;

        let mut accumulator = Array2::zeros((self.bip.dims.channels, self.bip.dims.channels));
        let mut buffer: Vec<f32> = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        make_bar!(pb, self.bip.num_pixels() as u64, "cov");

        while self.file.read_f32_into::<LittleEndian>(&mut buffer).is_ok() {
            let mut tmp = Vec::new();
            mem::swap(&mut tmp, &mut buffer);

            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp).unwrap();

            self.bip.map_covariance(&mut pixel, &means, &std_devs, &mut accumulator);

            tmp = pixel.into_raw_vec();
            mem::swap(&mut tmp, &mut buffer);

            inc_bar!(pb, BATCH_SIZE as u64);
        }

        self.bip.reduce_covariance(&mut accumulator);

        Ok(accumulator)
    }
}
