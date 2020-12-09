use std::mem;

use memmap2::MmapMut;

use crate::bin_formats::{FileDims, ImageIndex, SpectralImageContainer, SpectralImage};

mod mat_tests;

const BIP: [u8; 108] = unsafe {
    mem::transmute::<[f32; 27], [u8; 108]>
        (
            [
                00., 00., 00., 01., 00., 01., 02., 00., 02.,
                10., 01., 00., 11., 01., 01., 12., 01., 02.,
                20., 02., 00., 21., 02., 01., 22., 02., 02.
            ]
        )
};

const BSQ: [u8; 108] = unsafe {
    mem::transmute::<[f32; 27], [u8; 108]>
        (
            [
                00., 01., 02., 10., 11., 12., 20., 21., 22.,
                00., 00., 00., 01., 01., 01., 02., 02., 02.,
                00., 01., 02., 00., 01., 02., 00., 01., 02.,
            ]
        )
};

const BIL: [u8; 108] = unsafe {
    mem::transmute::<[f32; 27], [u8; 108]>
        (
            [
                00., 01., 02., 00., 00., 00., 00., 01., 02.,
                10., 11., 12., 01., 01., 01., 00., 01., 02.,
                20., 21., 22., 02., 02., 02., 00., 01., 02.,
            ]
        )
};

unsafe fn make_mat<I>(bytes: &[u8; 108]) -> SpectralImage<MmapMut, f32, I>
    where I: ImageIndex + From<FileDims>
{
    let mut inner = SpectralImageContainer::_from_dims_anon(&FileDims {
        bands: vec![0, 1, 2],
        samples: 3,
        lines: 3,
    }).unwrap();

    inner.container.clone_from_slice(&mut bytes.to_vec());

    let index = I::from(inner.dims.clone());

    SpectralImage {
        inner,
        index
    }
}