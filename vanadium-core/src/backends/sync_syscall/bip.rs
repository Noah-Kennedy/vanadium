use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use nalgebra::{DMatrix, Dynamic, SymmetricEigen};

use crate::{GenericResult, Image};
use crate::headers::Header;
use crate::specialization::bip::Bip;

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
    fn means(&mut self) -> GenericResult<Vec<f32>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut accumulator = vec![0.0; self.bip.pixel_length()];
        let mut buffer = vec![0.0; self.bip.pixel_length()];

        for _ in 0..self.bip.num_pixels() {
            self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
            self.bip.map_mean(&buffer, &mut accumulator);
        }

        self.bip.reduce_mean(&mut accumulator);

        Ok(accumulator)
    }

    fn std_deviations(&mut self, means: &[f32]) -> GenericResult<Vec<f32>> {
        self.file.seek(SeekFrom::Start(0))?;

        let mut accumulator = vec![0.0; self.bip.pixel_length()];
        let mut buffer = vec![0.0; self.bip.pixel_length()];

        for _ in 0..self.bip.num_pixels() {
            self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
            self.bip.map_std_dev(&buffer, means, &mut accumulator);
        }

        self.bip.reduce_std_dev(&mut accumulator);

        Ok(accumulator)
    }

    fn covariance_matrix(&mut self, means: &[f32], std_devs: &[f32]) -> GenericResult<DMatrix<f32>> {
        self.file.seek(SeekFrom::Start(0))?;

        let mut accumulator = DMatrix::zeros(self.bip.dims.channels, self.bip.dims.channels);
        let mut buffer = vec![0.0; self.bip.pixel_length()];

        for _ in 0..self.bip.num_pixels() {
            self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
            self.bip.map_covariance(&mut buffer, means, std_devs, &mut accumulator);
        }

        self.bip.reduce_covariance(&mut accumulator);

        Ok(accumulator)
    }

    fn write_standardized(&mut self, _path: &str, _means: &[f32], _std_devs: &[f32], _eigen: &SymmetricEigen<f32, Dynamic>) -> GenericResult<()> {
        todo!()
    }
}
