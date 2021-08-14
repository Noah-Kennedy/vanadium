use crate::headers::{Header, ImageDims, ImageFormat};
use crate::io::BasicImage;
use crate::io::bip::{GlommioBip, SyscallBip};
use crate::io::mapped::bip::MappedBip;
use crate::util::{make_raw, make_raw_mut};

const TEST_HEADER: Header<&str> = Header {
    dims: ImageDims {
        channels: 5,
        lines: 21954,
        pixels: 28740,
    },
    format: ImageFormat::Bip,
    path: "/data/undergrad-research/bench-data/small-bip",
};

#[cfg(test)]
#[cfg_attr(miri, ignore)]
mod means;

#[cfg(test)]
#[cfg_attr(miri, ignore)]
mod std_devs;

#[cfg(test)]
#[cfg_attr(miri, ignore)]
mod covariances;

#[test]
fn test_raw() {
    let mut v: Vec<f32> = vec![0., 5., 2., 3., 4.];

    unsafe {
        let r = make_raw(&v);

        assert_eq!(&r[4..8], (5f32).to_ne_bytes());
    }

    unsafe {
        let w = make_raw_mut(&mut v);

        let x = &mut w[4..8];

        let y = (7f32).to_ne_bytes();

        for (a, b) in x.iter_mut().zip(y.iter()) {
            *a = *b;
        }
    }

    unsafe {
        let r = make_raw(&v);

        assert_eq!(&r[4..8], (7f32).to_ne_bytes());
    }
}