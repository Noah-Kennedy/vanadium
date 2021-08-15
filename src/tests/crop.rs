use std::sync::Once;
use crate::io::bip::{GlommioBip, SyscallBip};
use crate::tests::TEST_HEADER;
use crate::io::BasicImage;
use std::fs::File;
use std::io::Seek;
use std::result::Result::Ok;
use byteorder::ReadBytesExt;

const GLO_PATH: &str = "data/tiny/glo-bip";
const SYS_PATH: &str = "data/tiny/sys-bip";

const FILE_SIZE: u64 = 1000 * 1000 * 4 * 5;

fn glommio_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: GlommioBip<&str, f32> = GlommioBip::new(TEST_HEADER.clone());
        bip.crop(Some((0, 1000)), Some((0, 1000)), GLO_PATH).unwrap();
    });
}

fn syscall_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: SyscallBip<f32> = SyscallBip::new(TEST_HEADER.clone()).unwrap();
        bip.crop(Some((0, 1000)), Some((0, 1000)), GLO_PATH).unwrap();
    });
}

#[test]
fn check_glommio_crop_size() {
    glommio_init();

    let f = File::open(GLO_PATH).unwrap();

    assert_eq!(FILE_SIZE, f.metadata().unwrap().len());
}

#[test]
fn check_syscall_crop_size() {
    syscall_init();

    let f = File::open(SYS_PATH).unwrap();

    assert_eq!(FILE_SIZE, f.metadata().unwrap().len());
}

#[test]
fn check_glommio_syscall_equivalence() {
    let mut counter = 0;

    glommio_init();
    syscall_init();

    let mut g = File::open(GLO_PATH).unwrap();
    let mut s = File::open(SYS_PATH).unwrap();

    while let (Ok(gf), Ok(sf)) = (g.read_f32(), s.read_f32()) {
        assert_eq!(gf, sf, "EQ failed at {}: {} != {}", counter, gf, sf);
        counter += 1;
    }

    assert_eq!(FILE_SIZE - 1, counter, "Did not make through file, made it to {}", counter);
}