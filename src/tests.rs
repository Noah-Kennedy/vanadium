use crate::headers::{Header, ImageDims, ImageFormat};
use crate::io::BasicImage;
use crate::io::bip::{GlommioBip, SyscallBip};
use crate::io::mapped::bip::MappedBip;
use crate::util::{make_raw, make_raw_mut};

const TEST_HEADER: Header<&str> = Header {
    dims: ImageDims {
        channels: 5,
        lines: 1000,
        pixels: 1000,
    },
    format: ImageFormat::Bip,
    path: "./data/tiny/bip",
};

#[cfg(test)]
#[cfg_attr(miri, ignore)]
mod means {
    use super::*;

    #[test]
    fn means_check_eq_sys_gl() {
        let mut gl: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());
        let mut syscall: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

        let means_gl = gl.means().unwrap();
        let means_sys = syscall.means().unwrap();

        assert_eq!(means_gl, means_sys);
    }

    #[test]
    fn means_check_eq_map_gl() {
        let mut gl: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());
        let mut mapped: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();

        let means_gl = gl.means().unwrap();
        let means_map = mapped.means().unwrap();

        assert_eq!(means_gl, means_map);
    }

    #[test]
    fn means_check_eq_sys_map() {
        let mut mapped: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();
        let mut syscall: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

        let means_map = mapped.means().unwrap();
        let means_sys = syscall.means().unwrap();

        assert_eq!(means_map, means_sys);
    }
}

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