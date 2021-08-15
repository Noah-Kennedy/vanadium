use std::fs::File;
use std::result::Result::Ok;
use std::sync::Once;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::io::BasicImage;
use crate::io::bip::{GlommioBip, SyscallBip};
use crate::tests::{CROP_HEADER};

const GLO_PATH: &str = "data/tiny/glo-bip";
const SYS_PATH: &str = "data/tiny/sys-bip";

const FILE_SIZE: u64 = 1000 * 1000 * 4 * 5;

fn glommio_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: GlommioBip<&str, f32> = GlommioBip::new(CROP_HEADER.clone()).unwrap();
        bip.crop(Some((0, 1000)), Some((0, 1000)), &GLO_PATH).unwrap();
    });
}

fn syscall_init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut bip: SyscallBip<f32> = SyscallBip::new(CROP_HEADER.clone()).unwrap();
        bip.crop(Some((0, 1000)), Some((0, 1000)), &SYS_PATH).unwrap();
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

    while let (Ok(gf), Ok(sf)) = (g.read_f32::<LittleEndian>(), s.read_f32::<LittleEndian>()) {
        assert_eq!(gf, sf, "EQ failed at {}: {} != {}", counter, gf, sf);
        counter += 1;
    }

    assert_eq!((FILE_SIZE / 4), counter, "Did not make through file, made it to {}", counter);
}