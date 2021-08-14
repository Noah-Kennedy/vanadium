use std::fs::File;
use std::mem::MaybeUninit;
use std::sync::Once;

use approx::assert_relative_eq;
use ndarray::Array1;

use super::*;

static mut GLO_VAL: MaybeUninit<Array1<f32>> = MaybeUninit::uninit();
static mut MAP_VAL: MaybeUninit<Array1<f32>> = MaybeUninit::uninit();
static mut SYS_VAL: MaybeUninit<Array1<f32>> = MaybeUninit::uninit();

fn glommio_init(means: &Array1<f32>) {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());

        unsafe {
            GLO_VAL = MaybeUninit::new(bip.std_deviations(&means).unwrap());
        }
    });
}

fn mapped_init(means: &Array1<f32>) {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();

        unsafe {
            MAP_VAL = MaybeUninit::new(bip.std_deviations(&means).unwrap());
        }
    });
}

fn sys_init(means: &Array1<f32>) {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

        unsafe {
            SYS_VAL = MaybeUninit::new(bip.std_deviations(&means).unwrap());
        }
    });
}

#[test]
fn std_devs_check_eq_sys_gl() {
    let means = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();

    glommio_init(&means);
    sys_init(&means);

    unsafe {
        assert_relative_eq!(
            GLO_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            SYS_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            epsilon = f32::EPSILON
        );
    }
}

#[test]
fn std_devs_check_eq_map_gl() {
    let means = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();

    mapped_init(&means);
    glommio_init(&means);

    unsafe {
        assert_relative_eq!(
            MAP_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            GLO_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            epsilon = f32::EPSILON
        );
    }
}

#[test]
fn std_devs_check_eq_sys_map() {
    let means = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();

    sys_init(&means);
    mapped_init(&means);

    unsafe {
        assert_relative_eq!(
            SYS_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            MAP_VAL.as_ptr().as_ref().unwrap().as_slice().unwrap(),
            epsilon = f32::EPSILON
        );
    }
}
