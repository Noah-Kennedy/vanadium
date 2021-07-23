use std::{mem, slice};

use futures::AsyncReadExt;
use glommio::io::{DmaFile, DmaStreamReaderBuilder};
use glommio::LocalExecutorBuilder;
use nalgebra::{DMatrix, Dynamic, SymmetricEigen};

use crate::backends::{GenericResult, Image};
use crate::headers::Header;
use crate::specialization::bip::Bip;

pub struct GlommioBip<T> {
    headers: Header,
    bip: Bip<T>,
}

impl Image<f32> for GlommioBip<f32> {
    fn means(&mut self) -> GenericResult<Vec<f32>> {
        let ex = LocalExecutorBuilder::new()
            .name("means")
            .pin_to_cpu(1)
            .make()?;

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "mean");

            let file = DmaFile::open(&self.headers.path).await?;
            let mut accumulator = vec![0.0; self.bip.pixel_length()];
            let mut raw_buffer = vec![0; self.bip.pixel_length() * mem::size_of::<f32>()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(self.buffer_size())
                .with_read_ahead(self.read_ahead())
                .build();

            for _ in 0..self.bip.num_pixels() {
                reader.read_exact(&mut raw_buffer).await?;

                let buffer = unsafe {
                    slice::from_raw_parts(
                        raw_buffer.as_ptr() as *const f32,
                        raw_buffer.len() / mem::size_of::<f32>(),
                    )
                };

                self.bip.map_mean(&buffer, &mut accumulator);

                inc_bar!(pb);
            }

            self.bip.reduce_mean(&mut accumulator);

            Ok(accumulator)
        })
    }

    fn std_deviations(&mut self, means: &[f32]) -> GenericResult<Vec<f32>> {
        let ex = LocalExecutorBuilder::new()
            .name("std-devs")
            .pin_to_cpu(1)
            .make()?;


        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "std");

            let file = DmaFile::open(&self.headers.path).await?;
            let mut accumulator = vec![0.0; self.bip.pixel_length()];
            let mut raw_buffer = vec![0; self.bip.pixel_length() * mem::size_of::<f32>()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(self.buffer_size())
                .with_read_ahead(self.read_ahead())
                .build();

            for _ in 0..self.bip.num_pixels() {
                reader.read_exact(&mut raw_buffer).await?;

                let buffer = unsafe {
                    slice::from_raw_parts(
                        raw_buffer.as_ptr() as *const f32,
                        raw_buffer.len() / mem::size_of::<f32>(),
                    )
                };

                self.bip.map_std_dev(&buffer, means, &mut accumulator);

                inc_bar!(pb);
            }

            self.bip.reduce_std_dev(&mut accumulator);

            Ok(accumulator)
        })
    }

    fn covariance_matrix(&mut self, means: &[f32], std_devs: &[f32]) -> GenericResult<DMatrix<f32>> {
        let ex = LocalExecutorBuilder::new()
            .name("covariance")
            .pin_to_cpu(1)
            .make()?;

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "cov");

            let file = DmaFile::open(&self.headers.path).await?;
            let mut accumulator = DMatrix::zeros(self.bip.dims.channels, self.bip.dims.channels);
            let mut raw_buffer = vec![0; self.bip.pixel_length() * mem::size_of::<f32>()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(self.buffer_size())
                .with_read_ahead(self.read_ahead())
                .build();

            for _ in 0..self.bip.num_pixels() {
                reader.read_exact(&mut raw_buffer).await?;

                let buffer = unsafe {
                    slice::from_raw_parts_mut(
                        raw_buffer.as_mut_ptr() as *mut f32,
                        raw_buffer.len() / mem::size_of::<f32>(),
                    )
                };

                self.bip.map_covariance(buffer, means, std_devs, &mut accumulator);

                inc_bar!(pb);
            }

            self.bip.reduce_covariance(&mut accumulator);

            Ok(accumulator)
        })
    }

    fn write_standardized(&mut self, _path: &str, _means: &[f32], _std_devs: &[f32], _eigen: &SymmetricEigen<f32, Dynamic>) -> GenericResult<()> {
        todo!()
    }
}

impl<T> GlommioBip<T> {
    pub fn new(headers: Header) -> Self {
        let bip = Bip {
            dims: headers.dims.clone(),
            phantom: Default::default(),
        };

        Self { headers, bip }
    }
    fn buffer_size(&self) -> usize {
        131072
    }
    fn read_ahead(&self) -> usize {
        16
    }
}