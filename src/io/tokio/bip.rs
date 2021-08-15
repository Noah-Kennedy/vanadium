use std::{io, mem};
use std::fmt::Debug;
use std::io::SeekFrom;
use std::iter::Sum;
use std::ops::{AddAssign, DivAssign, SubAssign};
use std::path::Path;
use std::sync::Arc;

use ndarray::{Array2, ArrayViewMut2};
use num_traits::{Float, FromPrimitive};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeek, AsyncSeekExt};
use tokio::runtime;

use crate::error::VanadiumResult;
use crate::headers::{Header, ImageFormat};
use crate::image_formats::bip::BipDims;
use crate::io::BATCH_SIZE;
use crate::io::bip::Bip;
use crate::util::make_raw_mut;

pub struct TokioBip<T> {
    file: File,
    rt: Arc<runtime::Runtime>,
    dims: BipDims<T>,
}

impl<T> TokioBip<T> {
    pub fn new<P>(header: Header<P>) -> io::Result<Self> where P: AsRef<Path> {
        assert_eq!(ImageFormat::Bip, header.format);

        let dims = BipDims {
            dims: header.dims,
            phantom: Default::default(),
        };

        let rt = runtime::Builder::new_current_thread()
            .build()?;

        let file = rt.block_on(File::open(header.path))?;

        Ok(Self {
            file,
            rt: Arc::new(rt),
            dims,
        })
    }
}

impl<T> Bip<T> for TokioBip<T>
    where T: Float + Clone + Copy + FromPrimitive + Sum + AddAssign + SubAssign + DivAssign +
    'static + Debug,
{
    fn fold_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> VanadiumResult<A>
        where F: FnMut(&mut Array2<T>, &mut A)
    {
        let name = name.to_owned();

        self.rt.clone().block_on(async {
            make_bar!(pb, self.dims.num_pixels() as u64, name);

            let mut buffer = Array2::from_shape_vec(
                (BATCH_SIZE, self.dims.pixel_length()),
                vec![T::zero(); BATCH_SIZE * self.dims.pixel_length()],
            ).unwrap();

            let mut seek = 0;
            let byte_len = buffer.len() * mem::size_of::<f32>();

            while unsafe {
                let raw_buffer = make_raw_mut(buffer.as_slice_mut().unwrap());
                self.file.read_exact(raw_buffer).await.is_ok()
            } {
                f(&mut buffer, &mut accumulator);

                inc_bar!(pb, BATCH_SIZE as u64);

                seek += byte_len;
            }

            self.file.seek(SeekFrom::Start(seek as u64)).await.unwrap();

            let n_elements = unsafe {
                let raw_buffer = make_raw_mut(buffer.as_slice_mut().unwrap());
                let mut v = Vec::new();
                let n_bytes = self.file.read_to_end(&mut v).await.unwrap();

                raw_buffer[..v.len()].clone_from_slice(&v);

                assert_eq!(0, n_bytes % mem::size_of::<T>());
                n_bytes / mem::size_of::<T>()
            };

            if n_elements > 0 {
                let shape = (((n_elements - 1) / self.dims.pixel_length()) + 1, self.dims.pixel_length());

                let mut pixel = Array2::from_shape_vec(
                    shape,
                    buffer.as_slice().unwrap()[..n_elements].to_vec(),
                ).unwrap();

                f(&mut pixel, &mut accumulator);
            }

            Ok(accumulator)
        })
    }

    fn dims(&self) -> &BipDims<T> {
        &self.dims
    }

    fn map_and_write_batched<F>(&mut self, name: &str, out: &dyn AsRef<Path>, n_output_channels: usize, f: F) -> VanadiumResult<()> where F: FnMut(&mut ArrayViewMut2<T>, &mut Array2<T>) {
        todo!()
    }

    fn crop_map<F>(&mut self, name: &str, rows: Option<(u64, u64)>, cols: Option<(u64, u64)>, n_output_channels: usize, out: &dyn AsRef<Path>, f: F) -> VanadiumResult<()> where F: FnMut(&mut ArrayViewMut2<T>, &mut Array2<T>) {
        todo!()
    }
}