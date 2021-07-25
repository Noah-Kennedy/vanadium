use std::fmt::Debug;
use std::iter::Sum;
use std::mem;
use std::ops::{AddAssign, DivAssign, SubAssign};

use futures::AsyncReadExt;
use glommio::io::{DmaFile, DmaStreamReaderBuilder};
use glommio::LocalExecutorBuilder;
use ndarray::Array2;
use num_traits::{Float, FromPrimitive};

use vanadium_core::headers::Header;
use vanadium_core::image_formats::bip::Bip;

use crate::{BATCH_SIZE, GenericResult};
use crate::bip::BatchedPixelReduce;

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
}

impl<T> BatchedPixelReduce<T> for GlommioBip<T>
    where T: Float + Clone + Copy + FromPrimitive + Sum + AddAssign + SubAssign + DivAssign +
    'static + Debug
{
    fn reduce_pixels_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> GenericResult<A>
        where F: FnMut(&mut Array2<T>, &mut A)
    {
        let ex = LocalExecutorBuilder::new()
            .name("means")
            .pin_to_cpu(1)
            .make()?;

        let name = name.to_owned();

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let file = DmaFile::open(&self.headers.path).await?;
            let mut buffer: Vec<T> = vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(131072)
                .with_read_ahead(16)
                .build();

            while {
                // Safety: here, we are effectively just taking a slice from the vec but as bytes
                // rather than floats.
                //
                // Since any byte pattern is a valid float, this is safe.
                //
                // We do not attempt to account for endianness here, we assume that the data is
                // already in LE form, as further support will be added in the next MVP.
                unsafe {
                    let raw_buffer = std::slice::from_raw_parts_mut(
                        buffer.as_mut_ptr() as *mut u8,
                        BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<T>(),
                    );
                    reader.read_exact(raw_buffer).await.is_ok()
                }
            } {
                let shape = (BATCH_SIZE, self.bip.pixel_length());
                let mut pixel = Array2::from_shape_vec(shape, buffer).unwrap();

                f(&mut pixel, &mut accumulator);

                buffer = pixel.into_raw_vec();

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            // Safety: similar to other section, but we need to double check that we read a valid
            // amount of bytes.
            let n_elements = unsafe {
                let raw_buffer = std::slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<T>(),
                );

                let n_bytes = reader.read(raw_buffer).await?;

                assert_eq!(0, n_bytes % mem::size_of::<T>());
                n_bytes / mem::size_of::<T>()
            };

            if n_elements > 0 {
                let shape = (n_elements / self.bip.pixel_length(), self.bip.pixel_length());
                let mut pixel = Array2::from_shape_vec(shape, buffer[..n_elements].to_vec()).unwrap();

                f(&mut pixel, &mut accumulator);
            }

            Ok(accumulator)
        })
    }

    fn bip(&self) -> &Bip<T> {
        &self.bip
    }
}