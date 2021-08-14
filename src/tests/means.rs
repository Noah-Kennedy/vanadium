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
