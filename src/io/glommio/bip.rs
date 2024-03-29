use std::fmt::Debug;
use std::iter::Sum;
use std::mem;
use std::ops::{AddAssign, DivAssign, SubAssign};
use std::path::Path;

use futures::{AsyncReadExt, AsyncWriteExt};
use glommio::{LocalExecutor, LocalExecutorBuilder};
use glommio::io::{DmaFile, DmaStreamReader, DmaStreamReaderBuilder, DmaStreamWriter, DmaStreamWriterBuilder};
use ndarray::{Array2, ArrayViewMut2};
use num_traits::{Float, FromPrimitive};

use crate::error::{VanadiumError, VanadiumResult};
use crate::headers::{Header, ImageFormat};
use crate::image_formats::bip::BipDims;
use crate::io::BATCH_SIZE;
use crate::io::bip::Bip;
use crate::util::{make_raw, make_raw_mut};

const READ_AHEAD: usize = 16;
const PIN_CPU: usize = 1;

// todo make variable
// not everyone has exactly this much locked memory
// maybe make it static or part of structure?
const LOCKED_MEMORY: usize = 524_288;

pub struct GlommioBip<P, T> where P: AsRef<Path> {
    headers: Header<P>,
    executor: LocalExecutor,
    bip: BipDims<T>,
}

impl<P, T> GlommioBip<P, T> where P: AsRef<Path> + ToString {
    pub fn new(headers: Header<P>) -> VanadiumResult<Self <>> {
        assert_eq!(ImageFormat::Bip, headers.format);

        let executor = LocalExecutorBuilder::new()
            .pin_to_cpu(PIN_CPU)
            .make()
            .map_err(|_| VanadiumError::Unknown).unwrap();

        let bip = BipDims {
            dims: headers.dims.clone(),
            phantom: Default::default(),
        };

        Ok(Self { headers, executor, bip })
    }

    async fn open_input_file(&self) -> VanadiumResult<DmaFile> {
        let f = DmaFile::open(&self.headers.path)
            .await
            .map_err(|_| VanadiumError::FileNotFound(self.headers.path.to_string()))?;

        assert_eq!(self.bip.get_image_size() as u64, f.file_size().await.unwrap());

        Ok(f)
    }

    async fn open_input_reader(&self) -> VanadiumResult<DmaStreamReader> {
        let file = self.open_input_file().await?;

        Ok(DmaStreamReaderBuilder::new(file)
            .with_buffer_size(LOCKED_MEMORY)
            .with_read_ahead(READ_AHEAD)
            .build())
    }

    async fn open_output_file(&self, out: &dyn AsRef<Path>) -> VanadiumResult<DmaFile> {
        glommio::io::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .dma_open(out)
            .await
            .map_err(|_| VanadiumError::FileNotFound(self.headers.path.to_string()))
    }

    async fn open_output_writer(&self, out: &dyn AsRef<Path>) -> VanadiumResult<DmaStreamWriter> {
        let file = self.open_output_file(out).await?;
        Ok(DmaStreamWriterBuilder::new(file)
            .with_buffer_size(LOCKED_MEMORY)
            .with_write_behind(READ_AHEAD)
            .build())
    }
}

impl<P, T> Bip<T> for GlommioBip<P, T>
    where T: Float + Clone + Copy + FromPrimitive + Sum + AddAssign + SubAssign + DivAssign +
    'static + Debug,
          P: AsRef<Path> + ToString
{
    fn fold_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> VanadiumResult<A>
        where F: FnMut(&mut Array2<T>, &mut A)
    {
        let name = name.to_owned();

        self.executor.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let mut reader = self.open_input_reader().await?;

            let mut buffer: Vec<T> = vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()];

            let mut seek = 0;
            let byte_len = buffer.len() * mem::size_of::<T>();

            while {
                unsafe {
                    let raw_buffer = make_raw_mut(&mut buffer);
                    reader.read_exact(raw_buffer).await.is_ok()
                }
            } {
                let shape = (BATCH_SIZE, self.bip.pixel_length());
                let mut pixel = Array2::from_shape_vec(shape, buffer).unwrap();

                f(&mut pixel, &mut accumulator);

                buffer = pixel.into_raw_vec();

                inc_bar!(pb, BATCH_SIZE as u64);

                seek += byte_len;
            }

            let mut reader = self.open_input_reader().await?;
            reader.skip(seek as u64);

            let n_elements = unsafe {
                let raw_buffer = make_raw_mut(&mut buffer);

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

    fn dims(&self) -> &BipDims<T> {
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
        let name = name.to_owned();

        self.executor.run(async {
            make_bar!(pb, self.bip.num_pixels() as u64, name);

            let mut read_array = Array2::from_shape_vec(
                (BATCH_SIZE, self.bip.pixel_length()),
                vec![T::zero(); BATCH_SIZE * self.bip.pixel_length()],
            ).unwrap();

            let mut write_array = Array2::from_shape_vec(
                (BATCH_SIZE, n_output_channels),
                vec![T::zero(); BATCH_SIZE * n_output_channels],
            ).unwrap();

            let mut reader = self.open_input_reader().await?;

            let mut writer = self.open_output_writer(out).await?;

            while {
                unsafe {
                    let raw_read_buffer = make_raw_mut(read_array.as_slice_mut().unwrap());
                    reader.read_exact(raw_read_buffer).await.is_ok()
                }
            } {
                f(&mut read_array.view_mut(), &mut write_array);

                unsafe {
                    let raw_write_buffer = make_raw(write_array.as_slice().unwrap());
                    writer.write(raw_write_buffer).await.map_err(|_| VanadiumError::IoError)?;
                }

                inc_bar!(pb, BATCH_SIZE as u64);
            }

            let n_elements = unsafe {
                let raw_buffer = make_raw_mut(read_array.as_slice_mut().unwrap());

                let n_bytes = reader.read(raw_buffer).await.map_err(|_| VanadiumError::IoError)?;

                // todo use a proper error handling approach, this can be triggered by user error
                assert_eq!(0, n_bytes % mem::size_of::<T>());
                n_bytes / mem::size_of::<T>()
            };

            if n_elements > 0 {
                let mut pixel = read_array.slice_mut(
                    s![..n_elements / self.bip.pixel_length(),..]);

                f(&mut pixel, &mut write_array);

                unsafe {
                    let write_slice = write_array.as_slice().unwrap();

                    let ws = &write_slice[..((n_elements / self.bip.pixel_length()) * n_output_channels)];

                    let raw_write_buffer = make_raw(ws);

                    writer.write_all(raw_write_buffer).await.map_err(|_| VanadiumError::IoError)?;
                }
            }

            writer.flush().await.unwrap();

            Ok(())
        })
    }

    fn crop_map<F>(
        &mut self,
        name: &str,
        rows: Option<(u64, u64)>,
        cols: Option<(u64, u64)>,
        n_output_channels: usize,
        out: &dyn AsRef<Path>,
        mut f: F,
    ) -> VanadiumResult<()>
        where F: FnMut(&mut ArrayViewMut2<T>, &mut Array2<T>)
    {
        let (start_col, end_col) = cols.unwrap_or((0, self.headers.dims.pixels as u64));
        let (start_row, end_row) = rows.unwrap_or((0, self.headers.dims.lines as u64));

        let row_length = end_col - start_col;

        let initial_skip = (start_row * self.headers.dims.pixels as u64)
            * self.bip.pixel_length() as u64
            * mem::size_of::<T>() as u64;

        let start_row_skip = start_col
            * self.bip.pixel_length() as u64
            * mem::size_of::<T>() as u64;

        let end_row_skip = (self.bip.dims.pixels as u64 - end_col)
            * self.bip.pixel_length() as u64
            * mem::size_of::<T>() as u64;

        let name = name.to_owned();

        self.executor.run(async {
            let mut reader = self.open_input_reader().await?;
            let mut writer = self.open_output_writer(out).await?;

            make_bar!(pb, end_row - start_row, name);

            let mut read_array = Array2::from_shape_vec(
                (row_length as usize, self.bip.pixel_length()),
                vec![T::zero(); row_length as usize * self.bip.pixel_length()],
            ).unwrap();

            let mut write_array = Array2::from_shape_vec(
                (row_length as usize, n_output_channels),
                vec![T::zero(); row_length as usize * n_output_channels],
            ).unwrap();

            reader.skip(initial_skip);

            let mut row = start_row;

            while row < end_row {
                reader.skip(start_row_skip);

                unsafe {
                    let raw_read_buffer = make_raw_mut(read_array.as_slice_mut().unwrap());
                    reader.read_exact(raw_read_buffer).await.unwrap();
                }

                f(&mut read_array.view_mut(), &mut write_array);

                unsafe {
                    let raw_write_buffer = make_raw(write_array.as_slice().unwrap());
                    writer.write_all(raw_write_buffer)
                        .await
                        .map_err(|_| VanadiumError::IoError)?;
                }

                inc_bar!(pb, 1);
                row += 1;

                reader.skip(end_row_skip);
            }

            writer.flush().await.unwrap();

            Ok(())
        })
    }
}