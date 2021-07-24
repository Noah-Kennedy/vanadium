use std::mem;

use byteorder::{ByteOrder, LittleEndian};
use futures::AsyncReadExt;
use glommio::io::{DmaFile, DmaStreamReaderBuilder};
use glommio::LocalExecutorBuilder;
use ndarray::Array2;

use crate::backends::{BATCH_SIZE, BatchedPixelReduce, GenericResult};
use crate::headers::Header;
use crate::specialization::bip::Bip;

pub struct GlommioBip<T> {
    headers: Header,
    bip: Bip<T>,
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

impl BatchedPixelReduce<f32> for GlommioBip<f32> {
    fn reduce_pixels_batched<F, A>(&mut self, mut accumulator: A, mut f: F) -> GenericResult<A>
        where F: FnMut(&mut Array2<f32>, &mut A)
    {
        let ex = LocalExecutorBuilder::new()
            .name("means")
            .pin_to_cpu(1)
            .make()?;

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, "mean");

            let file = DmaFile::open(&self.headers.path).await?;
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

                f(&mut pixel, &mut accumulator);

                tmp = pixel.into_raw_vec();

                mem::swap(&mut tmp, &mut buffer);

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            Ok(accumulator)
        })
    }

    fn bip(&self) -> &Bip<f32> {
        &self.bip
    }
}