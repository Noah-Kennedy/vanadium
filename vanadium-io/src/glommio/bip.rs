use std::fmt::Debug;
use std::iter::Sum;
use std::mem;
use std::ops::{AddAssign, DivAssign, SubAssign};
use std::path::PathBuf;

use futures::AsyncReadExt;
use glommio::io::{DmaFile, DmaStreamReaderBuilder};
use glommio::LocalExecutorBuilder;
use ndarray::{Array2, ArrayViewMut2};
use num_traits::{Float, FromPrimitive};

use vanadium_core::headers::{Header, ImageFormat};
use vanadium_core::image_formats::bip::Bip;

use crate::{BATCH_SIZE, GenericResult};
use crate::bip::SequentialPixels;

const READ_AHEAD: usize = 16;
const PIN_CPU: usize = 1;

// todo make variable
// not everyone has exactly this much locked memory
// maybe make it static or part of structure?
const LOCKED_MEMORY: usize = 524_288;

pub struct GlommioBip<T> {
    headers: Header,
    bip: Bip<T>,
}

impl<T> GlommioBip<T> {
    pub fn new(headers: Header) -> Self {
        assert_eq!(ImageFormat::Bip, headers.format);
        let bip = Bip {
            dims: headers.dims.clone(),
            phantom: Default::default(),
        };

        Self { headers, bip }
    }
}

impl<T> SequentialPixels<T> for GlommioBip<T>
    where T: Float + Clone + Copy + FromPrimitive + Sum + AddAssign + SubAssign + DivAssign +
    'static + Debug
{
    fn fold_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> GenericResult<A>
        where F: FnMut(&mut Array2<T>, &mut A)
    {
        let ex = LocalExecutorBuilder::new()
            .name(name)
            .pin_to_cpu(PIN_CPU)
            .make()?;

        let name = name.to_owned();

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let file = DmaFile::open(&self.headers.path).await?;
            let mut buffer: Vec<T> = vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()];

            let mut reader = DmaStreamReaderBuilder::new(file)
                .with_buffer_size(LOCKED_MEMORY)
                .with_read_ahead(READ_AHEAD)
                .build();

            while {
                // "do" part of while loop, evaluates loop continuation condition

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

                    // loop condition
                    reader.read_exact(raw_buffer).await.is_ok()
                }
            } {
                // main loop body
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

                // todo use a proper error handling approach, this can be triggered by user error
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

    fn map_and_write_batched<F>(
        &mut self,
        name: &str,
        out: PathBuf,
        n_output_channels: usize,
        mut f: F,
    ) -> GenericResult<()>
        where F: FnMut(&mut ArrayViewMut2<T>, &mut Array2<T>)
    {
        let ex = LocalExecutorBuilder::new()
            .name(name)
            .pin_to_cpu(PIN_CPU)
            .make()?;

        let name = name.to_owned();

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let shape = (BATCH_SIZE, self.bip.pixel_length());

            let read_file = DmaFile::open(&self.headers.path).await?;

            let mut read_buffer: Vec<T> = vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()];
            let mut write_array = Array2::from_shape_vec(
                shape,
                vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()],
            ).unwrap();

            let mut reader = DmaStreamReaderBuilder::new(read_file)
                .with_buffer_size(LOCKED_MEMORY)
                .with_read_ahead(READ_AHEAD)
                .build();

            while {
                // "do" part of while loop, evaluates loop continuation condition

                // Safety: here, we are effectively just taking a slice from the vec but as bytes
                // rather than floats.
                //
                // Since any byte pattern is a valid float, this is safe.
                //
                // We do not attempt to account for endianness here, we assume that the data is
                // already in LE form, as further support will be added in the next MVP.
                unsafe {
                    let raw_read_buffer = std::slice::from_raw_parts_mut(
                        read_buffer.as_mut_ptr() as *mut u8,
                        BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<T>(),
                    );

                    // loop condition
                    reader.read_exact(raw_read_buffer).await.is_ok()
                }
            } {
                // main loop body
                let mut pixel = Array2::from_shape_vec(shape, read_buffer).unwrap();

                f(&mut pixel, &mut write_array);

                read_buffer = pixel.into_raw_vec();

                // todo write

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            // Safety: similar to other section, but we need to double check that we read a valid
            // amount of bytes.
            let n_elements = unsafe {
                let raw_buffer = std::slice::from_raw_parts_mut(
                    read_buffer.as_mut_ptr() as *mut u8,
                    BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<T>(),
                );

                let n_bytes = reader.read(raw_buffer).await?;

                // todo use a proper error handling approach, this can be triggered by user error
                assert_eq!(0, n_bytes % mem::size_of::<T>());
                n_bytes / mem::size_of::<T>()
            };

            if n_elements > 0 {
                let shape = (n_elements / self.bip.pixel_length(), self.bip.pixel_length());
                let mut pixel = Array2::from_shape_vec(shape, read_buffer[..n_elements].to_vec()).unwrap();

                f(&mut pixel, &mut write_array);

                // todo write
            }

            Ok(())
        })
    }
}