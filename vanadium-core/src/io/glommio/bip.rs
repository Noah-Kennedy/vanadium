use std::fmt::Debug;
use std::iter::Sum;
use std::mem;
use std::ops::{AddAssign, DivAssign, SubAssign};
use std::path::Path;

use futures::{AsyncReadExt, AsyncWriteExt};
use glommio::io::{DmaFile, DmaStreamReaderBuilder, DmaStreamWriterBuilder};
use glommio::LocalExecutorBuilder;
use ndarray::{Array2, ArrayViewMut2};
use num_traits::{Float, FromPrimitive};

use crate::error::{VanadiumError, VanadiumResult};
use crate::headers::{Header, ImageFormat};
use crate::image_formats::bip::Bip;
use crate::io::BATCH_SIZE;
use crate::io::bip::SequentialPixels;

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
    fn fold_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> VanadiumResult<A>
        where F: FnMut(&mut Array2<T>, &mut A)
    {
        // todo handle errors

        let ex = LocalExecutorBuilder::new()
            .name(name)
            .pin_to_cpu(PIN_CPU)
            .make()
            .map_err(|_| VanadiumError::Unknown)?;

        let name = name.to_owned();

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let file = DmaFile::open(&self.headers.path).await
                .map_err(|_| VanadiumError::FileNotFound(self.headers.path.to_str().unwrap().to_string()))?;

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

                let n_bytes = reader.read(raw_buffer).await.map_err(|_| VanadiumError::IoError)?;

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
        out: &dyn AsRef<Path>,
        n_output_channels: usize,
        mut f: F,
    ) -> VanadiumResult<()>
        where F: FnMut(&mut ArrayViewMut2<T>, &mut Array2<T>)
    {
        let ex = LocalExecutorBuilder::new()
            .name(name)
            .pin_to_cpu(PIN_CPU)
            .make()
            .map_err(|_| VanadiumError::Unknown)?;

        let name = name.to_owned();

        ex.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let read_file = DmaFile::open(&self.headers.path).await
                .map_err(|_| VanadiumError::FileNotFound(self.headers.path.to_str().unwrap().to_string()))?;

            let write_file = glommio::io::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .dma_open(out).await
                .map_err(|_| VanadiumError::FileNotFound(self.headers.path.to_str().unwrap().to_string()))?;

            let mut read_array = Array2::from_shape_vec(
                (BATCH_SIZE, self.bip.pixel_length()),
                vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()],
            ).unwrap();

            let mut write_array = Array2::from_shape_vec(
                (BATCH_SIZE, n_output_channels),
                vec![T::zero(); BATCH_SIZE * n_output_channels],
            ).unwrap();

            let mut reader = DmaStreamReaderBuilder::new(read_file)
                .with_buffer_size(LOCKED_MEMORY)
                .with_read_ahead(READ_AHEAD)
                .build();

            let mut writer = DmaStreamWriterBuilder::new(write_file)
                .with_buffer_size(LOCKED_MEMORY)
                .with_write_behind(READ_AHEAD)
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
                    let raw_read_buffer = std::slice::from_raw_parts_mut(
                        read_array.as_mut_ptr() as *mut u8,
                        BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<T>(),
                    );

                    // loop condition
                    reader.read_exact(raw_read_buffer).await.is_ok()
                }
            } {
                f(&mut read_array.view_mut(), &mut write_array);

                // Safety: Same as above
                unsafe {
                    let raw_write_buffer = std::slice::from_raw_parts(
                        write_array.as_ptr() as *const u8,
                        write_array.len() * mem::size_of::<T>(),
                    );

                    writer.write(raw_write_buffer).await.map_err(|_| VanadiumError::IoError)?;
                }

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            // warning: We are really making some assumptions about how read_array stores data
            // internally

            // Safety: similar to other section, but we need to double check that we read a valid
            // amount of bytes.
            let n_elements = unsafe {
                let raw_buffer = std::slice::from_raw_parts_mut(
                    read_array.as_mut_ptr() as *mut u8,
                    BATCH_SIZE * self.bip.pixel_length() * mem::size_of::<T>(),
                );

                let n_bytes = reader.read(raw_buffer).await.map_err(|_| VanadiumError::IoError)?;

                // todo use a proper error handling approach, this can be triggered by user error
                assert_eq!(0, n_bytes % mem::size_of::<T>());
                n_bytes / mem::size_of::<T>()
            };

            if n_elements > 0 {
                let mut pixel = read_array.slice_mut(
                    s![..n_elements / self.bip.pixel_length(),..]);

                f(&mut pixel, &mut write_array);

                // Safety: Same as above
                unsafe {
                    let raw_write_buffer = std::slice::from_raw_parts(
                        write_array.as_ptr() as *const u8,
                        (n_elements / self.bip.pixel_length()) * n_output_channels * mem::size_of::<T>(),
                    );

                    writer.write_all(raw_write_buffer).await.map_err(|_| VanadiumError::IoError)?;
                }
            }

            Ok(())
        })
    }
}