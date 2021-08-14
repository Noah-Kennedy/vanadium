use super::*;
use std::fs::File;

#[test]
fn std_devs_check_eq_sys_gl() {
    let mut gl: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());
    let mut syscall: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

    let sd = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();

    let val_gl = gl.std_deviations(&sd).unwrap();
    let val_sys = syscall.means().unwrap();

    assert_eq!(val_gl, val_sys);
}

#[test]
fn std_devs_check_eq_map_gl() {
    let mut gl: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());
    let mut mapped: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();

    let sd = serde_json::from_reader(File::open("data/tiny/means.json").unwrap()).unwrap();

    let val_gl = gl.std_deviations(&sd).unwrap();
    let val_map = mapped.std_deviations(&sd).unwrap();

    assert_eq!(val_gl, val_map);
}

#[test]
fn std_devs_check_eq_sys_map() {
    let mut mapped: MappedBip<f32> = MappedBip::new(TEST_HEADER.clone()).unwrap();
    let mut syscall: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();

    let sd = serde_json::from_reader(File::open("data/small/means.json").unwrap()).unwrap();

    let val_map = mapped.std_deviations(&sd).unwrap();
    let val_sys = syscall.std_deviations(&sd).unwrap();

    assert_eq!(val_map, val_sys);
}
