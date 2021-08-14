use std::{io, mem};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use ndarray::{Array2, ArrayViewMut2};

use crate::error::{VanadiumError, VanadiumResult};
use crate::headers::{Header, ImageFormat};
use crate::image_formats::bip::BipDims;
use crate::io::BATCH_SIZE;
use crate::io::bip::Bip;
use crate::util::make_raw_mut;

pub struct SyscallBip<T> {
    file: File,
    bip: BipDims<T>,
}

impl<T> SyscallBip<T> {
    pub fn new<P>(header: Header<P>) -> io::Result<Self> where P: AsRef<Path> {
        assert_eq!(ImageFormat::Bip, header.format);
        let bip = BipDims {
            dims: header.dims,
            phantom: Default::default(),
        };

        let file = File::open(header.path)?;

        Ok(Self {
            file,
            bip,
        })
    }
}

impl Bip<f32> for SyscallBip<f32> {
    fn fold_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> VanadiumResult<A>
        where F: FnMut(&mut Array2<f32>, &mut A)
    {
        self.file.seek(SeekFrom::Start(0)).map_err(|_| VanadiumError::IoError)?;

        let name = name.to_owned();

        make_bar!(pb, self.bip.num_pixels() as u64, name);

        let mut buffer = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        let mut seek = 0;
        let byte_len = buffer.len() * mem::size_of::<f32>();

        while self.file.read_f32_into::<LittleEndian>(&mut buffer).is_ok() {
            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), buffer)
                .unwrap();

            f(&mut pixel, &mut accumulator);

            buffer = pixel.into_raw_vec();

            inc_bar!(pb, BATCH_SIZE as u64);

            seek += byte_len;
        }

        self.file.seek(SeekFrom::Start(seek as u64)).map_err(|_| VanadiumError::IoError)?;

        let mut raw_buf = Vec::new();

        let n_bytes = self.file.read_to_end(&mut raw_buf)
            .map_err(|_| VanadiumError::IoError)?;

        assert_eq!(0, n_bytes % 4);

        if n_bytes > 0 {
            let n_elements = n_bytes / mem::size_of::<f32>();
            let b = &mut buffer[..n_elements];

            raw_buf.as_slice().read_f32_into::<LittleEndian>(b);

            let shape = (n_elements / self.bip.pixel_length(), self.bip.pixel_length());
            let mut pixel = Array2::from_shape_vec(shape, b.to_vec()).unwrap();

            f(&mut pixel, &mut accumulator);
        }

        Ok(accumulator)
    }

    fn bip(&self) -> &BipDims<f32> {
        &self.bip
    }

    fn map_and_write_batched<F>(
        &mut self, _name: &str, _out: &dyn AsRef<Path>, _n_output_channels: usize, _f: F,
    ) -> VanadiumResult<()>
        where F: FnMut(&mut ArrayViewMut2<f32>, &mut Array2<f32>)
    {
        todo!()
    }

    fn crop_map<F>(&mut self, _name: &str, _rows: Option<(u64, u64)>, _cols: Option<(u64, u64)>,
                   _n_output_channels: usize, _out: &dyn AsRef<Path>, _f: F) -> VanadiumResult<()>
        where F: FnMut(&mut ArrayViewMut2<f32>, &mut Array2<f32>) {
        todo!()
    }
}