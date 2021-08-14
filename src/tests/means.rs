use std::mem::MaybeUninit;
use std::sync::Once;

use ndarray::Array1;

use super::*;

static mut GLOMMIO_VAL: MaybeUninit<Array1<f32>> = MaybeUninit::uninit();
static mut MAPPED_VAL: MaybeUninit<Array1<f32>> = MaybeUninit::uninit();
static mut SYS_VAL: MaybeUninit<Array1<f32>> = MaybeUninit::uninit();

fn glommio_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());

        unsafe {
            GLOMMIO_VAL = MaybeUninit::new(bip.means().unwrap());
        }
    });
}

fn mapped_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();

        unsafe {
            MAPPED_VAL = MaybeUninit::new(bip.means().unwrap());
        }
    });
}

fn sys_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

        unsafe {
            SYS_VAL = MaybeUninit::new(bip.means().unwrap());
        }
    });
}

#[test]
fn means_check_eq_sys_gl() {
    glommio_init();
    sys_init();

    unsafe {
        assert_eq!(*GLOMMIO_VAL.as_ptr(), *SYS_VAL.as_ptr());
    }
}

#[test]
fn means_check_eq_map_gl() {
    mapped_init();
    glommio_init();

    unsafe {
        assert_eq!(*MAPPED_VAL.as_ptr(), *GLOMMIO_VAL.as_ptr());
    }
}

#[test]
fn means_check_eq_sys_map() {
    sys_init();
    mapped_init();

    unsafe {
        assert_eq!(*SYS_VAL.as_ptr(), *MAPPED_VAL.as_ptr());
    }
}
