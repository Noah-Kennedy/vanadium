use std::fs::File;
use std::io;

use ndarray::{Array1, Array2};

use vanadium_core::headers::Header;
use vanadium_core::image_formats::bsq::Bsq;

use crate::{GenericResult, ImageStats};
use std::path::Path;

pub struct SyncBsq<T> {
    _file: File,
    _bsq: Bsq<T>,
}

impl<T> SyncBsq<T> {
    pub fn _new(header: Header) -> io::Result<Self> {
        let bsq = Bsq {
            dims: header.dims,
            phantom: Default::default(),
        };

        let file = File::open(header.path)?;

        Ok(Self {
            _file: file,
            _bsq: bsq,
        })
    }
}

impl ImageStats<f32> for SyncBsq<f32> {
    fn means(&mut self) -> GenericResult<Array1<f32>> {
        todo!()
        // let mut means = Vec::with_capacity(self.bsq.dims.channels);
        // let mut buffer = vec![0.0; self.bsq.channel_length()];
        //
        // for _ in 0..self.bsq.dims.channels {
        //     self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
        //     let channel_mean = self.bsq.find_channel_mean(&buffer);
        //     means.push(channel_mean);
        // }
        //
        // Ok(means)
    }

    fn std_deviations(&mut self, _means: &Array1<f32>) -> GenericResult<Array1<f32>> {
        todo!()
        // let mut std_devs = Vec::with_capacity(self.bsq.dims.channels);
        // let mut buffer = vec![0.0; self.bsq.channel_length()];
        //
        // for m in means.iter() {
        //     self.file.read_f32_into::<LittleEndian>(&mut buffer)?;
        //     let channel_std_dev = self.bsq.find_channel_std_dev(&buffer, *m);
        //     std_devs.push(channel_std_dev);
        // }
        //
        // Ok(std_devs)
    }

    fn covariance_matrix(&mut self, _means: Option<&Array1<f32>>, _std_devs: Option<&Array1<f32>>) -> GenericResult<Array2<f32>> {
        todo!()
        // let mut cov_mat = Array2::zeros((self.bsq.dims.channels, self.bsq.dims.channels));
        // let mut major_buffer = vec![0.0; self.bsq.channel_length()];
        // let mut minor_buffer = vec![0.0; self.bsq.channel_length()];
        //
        // let major_iter = means.iter().zip(std_devs.iter()).enumerate();
        //
        // for (major_idx, (major_mean, major_std_dev)) in major_iter {
        //     let off = self.bsq.index_channel(major_idx) * mem::size_of::<f32>();
        //
        //     self.file.seek(SeekFrom::Start(off as u64))?;
        //     self.file.read_f32_into::<LittleEndian>(&mut major_buffer)?;
        //     self.bsq.standardize_channel(&mut major_buffer, *major_mean, *major_std_dev);
        //
        //     cov_mat[(major_idx, major_idx)] = self.bsq.variance(&major_buffer);
        //
        //     let minor_iter = means.iter()
        //         .zip(std_devs.iter())
        //         .enumerate()
        //         .skip(major_idx + 1);
        //
        //     for (minor_idx, (minor_mean, minor_std_dev)) in minor_iter {
        //         self.file.read_f32_into::<LittleEndian>(&mut minor_buffer)?;
        //
        //         cov_mat[(major_idx, minor_idx)] = self.bsq.covariance(
        //             &mut major_buffer,
        //             &mut minor_buffer,
        //             *minor_mean,
        //             *minor_std_dev,
        //         );
        //     }
        // }
        //
        // Ok(cov_mat)
    }

    fn write_transformed(&mut self, _transform: &Array2<f32>, _out: &dyn AsRef<Path>, _means: Option<&Array1<f32>>, _std_devs: Option<&Array1<f32>>) -> GenericResult<()> {
        todo!()
    }
}
