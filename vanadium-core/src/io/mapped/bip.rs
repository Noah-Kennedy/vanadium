use std::{io, mem};
use std::fs::{OpenOptions};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use memmap2::MmapMut;
use ndarray::{Array2, ArrayViewMut2};

use crate::error::VanadiumResult;
use crate::headers::{Header, ImageFormat};
use crate::image_formats::bip::Bip;
use crate::io::BATCH_SIZE;
use crate::io::bip::SequentialPixels;

pub struct MappedBip<T> {
    map: MmapMut,
    bip: Bip<T>,
}

impl<T> MappedBip<T> {
    pub fn new(header: Header) -> io::Result<Self> {
        assert_eq!(ImageFormat::Bip, header.format);

        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(header.path)?;

        let bip = Bip {
            dims: header.dims,
            phantom: Default::default(),
        };

        let map = unsafe { MmapMut::map_mut(&file).unwrap() };

        Ok(Self {
            map,
            bip,
        })
    }
}

impl SequentialPixels<f32> for MappedBip<f32> {
    fn fold_batched<F, A>(&mut self, name: &str, mut accumulator: A, mut f: F) -> VanadiumResult<A>
        where F: FnMut(&mut Array2<f32>, &mut A)
    {
        let mut seek = 0;

        let name = name.to_owned();

        make_bar!(pb, self.bip.num_pixels() as u64, name);

        let mut buffer = vec![0.0; BATCH_SIZE * self.bip.pixel_length()];

        let byte_len = buffer.len()  * mem::size_of::<f32>();

        while {
            let end = seek + byte_len;

            if end < self.map.len() {
                let mut d = &self.map[seek..(seek + byte_len)];
                d.read_f32_into::<LittleEndian>(&mut buffer).unwrap();
                true
            } else {
                false
            }
        } {
            seek += byte_len;

            let mut pixel = Array2::from_shape_vec((BATCH_SIZE, self.bip.pixel_length()), buffer)
                .unwrap();

            f(&mut pixel, &mut accumulator);

            buffer = pixel.into_raw_vec();

            inc_bar!(pb, BATCH_SIZE as u64);
        }

        let d = &self.map[seek..];

        let n_bytes = d.len();

        // todo use a proper error handling approach, this can be triggered by user error
        assert_eq!(0, n_bytes % mem::size_of::<f32>());
        let n_elements = n_bytes / mem::size_of::<f32>();

        if n_elements > 0 {
            let shape = (n_elements / self.bip.pixel_length(), self.bip.pixel_length());
            let mut pixel = Array2::from_shape_vec(shape, buffer[..n_elements].to_vec()).unwrap();

            f(&mut pixel, &mut accumulator);
        }

        Ok(accumulator)
    }

    fn bip(&self) -> &Bip<f32> {
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