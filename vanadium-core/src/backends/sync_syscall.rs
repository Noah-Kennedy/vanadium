use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::{mem, io};

use byteorder::{LittleEndian, ReadBytesExt};
use nalgebra::{DMatrix, Dynamic, SymmetricEigen};

use crate::backends::{GenericResult, Image};
use crate::specialization::bip::Bip;
use crate::specialization::bsq::Bsq;
use crate::headers::Header;

pub struct SyncBip<T> {
    pub (crate) file: File,
    pub (crate) bip: Bip<T>,
}

pub struct SyncBsq<T> {
    pub (crate) file: File,
    pub (crate) bsq: Bsq<T>,
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

impl Image<f32> for SyncBsq<f32> {
    fn means(&mut self) -> GenericResult<Vec<f32>> {
        let mut means = Vec::with_capacity(self.bsq.dims.channels);
        let mut buffer = vec![0.0; self.bsq.channel_length()];

        for _ in 0..self.bsq.dims.channels {
            self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
            let channel_mean = self.bsq.find_channel_mean(&buffer);
            means.push(channel_mean);
        }

        Ok(means)
    }

    fn std_deviations(&mut self, means: &[f32]) -> GenericResult<Vec<f32>> {
        let mut std_devs = Vec::with_capacity(self.bsq.dims.channels);
        let mut buffer = vec![0.0; self.bsq.channel_length()];

        for m in means.iter() {
            self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
            let channel_std_dev = self.bsq.find_channel_std_dev(&buffer, *m);
            std_devs.push(channel_std_dev);
        }

        Ok(std_devs)
    }

    fn covariance_matrix(&mut self, means: &[f32], std_devs: &[f32]) -> GenericResult<DMatrix<f32>> {
        let mut cov_mat = DMatrix::zeros(self.bsq.dims.channels, self.bsq.dims.channels);
        let mut major_buffer = vec![0.0; self.bsq.channel_length()];
        let mut minor_buffer = vec![0.0; self.bsq.channel_length()];

        let major_iter = means.iter().zip(std_devs.iter()).enumerate();

        for (major_idx, (major_mean, major_std_dev)) in major_iter {
            let off = self.bsq.index_channel(major_idx) * mem::size_of::<f32>();

            self.file.seek(SeekFrom::Start(off as u64))?;
            self.file.read_f32_into::<LittleEndian>(&mut major_buffer)?;
            self.bsq.standardize_channel(&mut major_buffer, *major_mean, *major_std_dev);

            cov_mat[(major_idx, major_idx)] = self.bsq.variance(&major_buffer);

            let minor_iter = means.iter()
                .zip(std_devs.iter())
                .enumerate()
                .skip(major_idx + 1);

            for (minor_idx, (minor_mean, minor_std_dev)) in minor_iter {
                self.file.read_f32_into::<LittleEndian>(&mut minor_buffer)?;

                cov_mat[(major_idx, minor_idx)] = self.bsq.covariance(
                    &mut major_buffer,
                    &mut minor_buffer,
                    *minor_mean,
                    *minor_std_dev,
                );
            }
        }

        Ok(cov_mat)
    }

    fn write_standardized(&mut self, _path: &str, _means: &[f32], _std_devs: &[f32], _eigen: &SymmetricEigen<f32, Dynamic>) -> GenericResult<()> {
        todo!()
    }
}