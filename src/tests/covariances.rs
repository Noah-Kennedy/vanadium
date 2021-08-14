use std::fs::File;
use std::mem::MaybeUninit;
use std::sync::Once;

use approx::assert_relative_eq;
use ndarray::{Array2};

use super::*;

static mut GLO_VAL: MaybeUninit<Array2<f32>> = MaybeUninit::uninit();
static mut MAP_VAL: MaybeUninit<Array2<f32>> = MaybeUninit::uninit();
static mut SYS_VAL: MaybeUninit<Array2<f32>> = MaybeUninit::uninit();

fn glommio_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let means = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();
        let sd = serde_json::from_reader(File::open("data/small/std-devs.json").unwrap()).unwrap();

        let mut bip: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());

        unsafe {
            GLO_VAL = MaybeUninit::new(bip.covariance_matrix(Some(&means), Some(&sd)).unwrap());
        }
    });
}

fn mapped_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let means = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();
        let sd = serde_json::from_reader(File::open("data/small/std-devs.json").unwrap()).unwrap();

        let mut bip: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();

        unsafe {
            MAP_VAL = MaybeUninit::new(bip.covariance_matrix(Some(&means), Some(&sd)).unwrap());
        }
    });
}

fn sys_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let means = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();
        let sd = serde_json::from_reader(File::open("data/small/std-devs.json").unwrap()).unwrap();

        let mut bip: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

        unsafe {
            SYS_VAL = MaybeUninit::new(bip.covariance_matrix(Some(&means), Some(&sd)).unwrap());
        }
    });
}

#[test]
fn cov_check_eq_sys_gl() {
    glommio_init();
    sys_init();

    unsafe {
        assert_relative_eq!(
            GLO_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            SYS_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            epsilon = f32::EPSILON
        );
    }
}

#[test]
fn cov_check_eq_map_gl() {
    mapped_init();
    glommio_init();

    unsafe {
        assert_relative_eq!(
            MAP_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            GLO_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            epsilon = f32::EPSILON
        );
    }
}

#[test]
fn cov_check_eq_sys_map() {
    sys_init();
    mapped_init();

    unsafe {
        assert_relative_eq!(
            SYS_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            MAP_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            epsilon = f32::EPSILON
        );
    }
}
