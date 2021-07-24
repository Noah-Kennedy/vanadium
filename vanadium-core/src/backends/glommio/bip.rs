use std::{mem};

use byteorder::{ByteOrder, LittleEndian};
use futures::AsyncReadExt;
use glommio::io::{DmaFile, DmaStreamReaderBuilder};
use glommio::LocalExecutorBuilder;
use ndarray::{Array1, Array2};

use crate::backends::{BATCH_SIZE, GenericResult, Image};
use crate::headers::Header;
use crate::specialization::bip::Bip;

pub struct GlommioBip<T> {
    headers: Header,
    bip: Bip<T>,
}

impl Image<f32> for GlommioBip<f32> {
    fn means(&mut self) -> GenericResult<Array1<f32>> {
        let ex = LocalExecutorBuilder::new()
            .name("means")
            .pin_to_cpu(1)
            .make()?;

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "mean");

            let file = DmaFile::open(&self.headers.path).await?;
            let mut accumulator = Array1::zeros(self.bip.pixel_length());
            let mut raw_buffer = vec![0; BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<f32>()];
            let mut buffer: Vec<f32> = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(self.buffer_size())
                .with_read_ahead(self.read_ahead())
                .build();

            while reader.read_exact(&mut raw_buffer).await.is_ok() {
                LittleEndian::read_f32_into(&raw_buffer, &mut buffer);

                let mut tmp = Vec::new();

                mem::swap(&mut tmp, &mut buffer);

                let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp)
                    .unwrap();

                self.bip.map_mean(&mut pixel, &mut accumulator);

                tmp = pixel.into_raw_vec();

                mem::swap(&mut tmp, &mut buffer);

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            self.bip.reduce_mean(&mut accumulator);

            Ok(accumulator)
        })
    }

    fn std_deviations(&mut self, means: &Array1<f32>) -> GenericResult<Array1<f32>> {
        let ex = LocalExecutorBuilder::new()
            .name("std-devs")
            .pin_to_cpu(1)
            .make()?;


        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "std");

            let file = DmaFile::open(&self.headers.path).await?;
            let mut accumulator = Array1::zeros(self.bip.pixel_length());
            let mut raw_buffer = vec![0; BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<f32>()];
            let mut buffer: Vec<f32> = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(self.buffer_size())
                .with_read_ahead(self.read_ahead())
                .build();

            while reader.read_exact(&mut raw_buffer).await.is_ok() {
                LittleEndian::read_f32_into(&raw_buffer, &mut buffer);

                let mut tmp = Vec::new();

                mem::swap(&mut tmp, &mut buffer);

                let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp)
                    .unwrap();

                self.bip.map_std_dev(&mut pixel, means, &mut accumulator);

                tmp = pixel.into_raw_vec();

                mem::swap(&mut tmp, &mut buffer);

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            self.bip.reduce_std_dev(&mut accumulator);

            Ok(accumulator)
        })
    }

    fn covariance_matrix(&mut self, means: &Array1<f32>, std_devs: &Array1<f32>) -> GenericResult<Array2<f32>> {
        let ex = LocalExecutorBuilder::new()
            .name("covariance")
            .pin_to_cpu(1)
            .make()?;

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "cov");

            let file = DmaFile::open(&self.headers.path).await?;
            let mut accumulator = Array2::zeros((self.bip.dims.channels, self.bip.dims.channels));
            let mut raw_buffer = vec![0; BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<f32>()];
            let mut buffer: Vec<f32> = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(self.buffer_size())
                .with_read_ahead(self.read_ahead())
                .build();

            while reader.read_exact(&mut raw_buffer).await.is_ok() {
                LittleEndian::read_f32_into(&raw_buffer, &mut buffer);

                let mut tmp = Vec::new();

                mem::swap(&mut tmp, &mut buffer);

                let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), tmp)
                    .unwrap();

                self.bip.map_covariance(&mut pixel, &means, &std_devs, &mut accumulator);

                tmp = pixel.into_raw_vec();

                mem::swap(&mut tmp, &mut buffer);

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            self.bip.reduce_covariance(&mut accumulator);

            Ok(accumulator)
        })
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