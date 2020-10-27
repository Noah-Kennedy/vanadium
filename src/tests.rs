use super::*;
use crate::cli::ConvertOpt;
use std::process::Command;
use crate::headers::Interleave;
use subprocess::Exec;

#[test]
fn test_convert_bip_2_bsq() {
    Command::new("sync").output().unwrap();
    Exec::shell("pkexec bash -c 'echo 3 > /proc/sys/vm/drop_caches'").join().unwrap();

    let out = "bip.bsq";
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bip".into(),
        input_header: "data/raw/unnormalized/unnorm.bip.hdr".into(),
        output: out.into(),
        output_header: None,
        output_type: Interleave::Bsq
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg(out)
        .arg("data/raw/unnormalized/unnorm.bsq")
        .status()
        .unwrap();

    assert!(diff.success());

    Command::new("rm")
        .arg(out)
        .output()
        .unwrap();
}

#[test]
fn test_convert_bip_2_bil() {
    Command::new("sync").output().unwrap();
    Exec::shell("pkexec bash -c 'echo 3 > /proc/sys/vm/drop_caches'").join().unwrap();

    let out = "bip.bil";
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bip".into(),
        input_header: "data/raw/unnormalized/unnorm.bip.hdr".into(),
        output: out.into(),
        output_header: None,
        output_type: Interleave::Bil
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg(out)
        .arg("data/raw/unnormalized/unnorm.bil")
        .status()
        .unwrap();

    assert!(diff.success());

    Command::new("rm")
        .arg(out)
        .output()
        .unwrap();
}

#[test]
fn test_convert_bil_2_bip() {
    Command::new("sync").output().unwrap();
    Exec::shell("pkexec bash -c 'echo 3 > /proc/sys/vm/drop_caches'").join().unwrap();

    let out = "bil.bip";
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bil".into(),
        input_header: "data/raw/unnormalized/unnorm.bil.hdr".into(),
        output: out.into(),
        output_header: None,
        output_type: Interleave::Bip
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg(out)
        .arg("data/raw/unnormalized/unnorm.bip")
        .status()
        .unwrap();

    assert!(diff.success());

    Command::new("rm")
        .arg(out)
        .output()
        .unwrap();
}

#[test]
fn test_convert_bil_2_bsq() {
    Command::new("sync").output().unwrap();
    Exec::shell("pkexec bash -c 'echo 3 > /proc/sys/vm/drop_caches'").join().unwrap();

    let out = "bil.bsq";
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bil".into(),
        input_header: "data/raw/unnormalized/unnorm.bil.hdr".into(),
        output: out.into(),
        output_header: None,
        output_type: Interleave::Bsq
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg(out)
        .arg("data/raw/unnormalized/unnorm.bsq")
        .status()
        .unwrap();

    assert!(diff.success());

    Command::new("rm")
        .arg(out)
        .output()
        .unwrap();
}

#[test]
fn test_convert_bsq_2_bil() {
    Command::new("sync").output().unwrap();
    Exec::shell("pkexec bash -c 'echo 3 > /proc/sys/vm/drop_caches'").join().unwrap();

    let out = "bsq.bil";
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bsq".into(),
        input_header: "data/raw/unnormalized/unnorm.bsq.hdr".into(),
        output: out.into(),
        output_header: None,
        output_type: Interleave::Bil
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg(out)
        .arg("data/raw/unnormalized/unnorm.bil")
        .status()
        .unwrap();

    assert!(diff.success());

    Command::new("rm")
        .arg(out)
        .output()
        .unwrap();
}

#[test]
fn test_convert_bsq_2_bip() {
    Command::new("sync").output().unwrap();
    Exec::shell("pkexec bash -c 'echo 3 > /proc/sys/vm/drop_caches'").join().unwrap();

    let out = "bsq.bip";
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bsq".into(),
        input_header: "data/raw/unnormalized/unnorm.bsq.hdr".into(),
        output: out.into(),
        output_header: None,
        output_type: Interleave::Bip
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg(out)
        .arg("data/raw/unnormalized/unnorm.bip")
        .status()
        .unwrap();

    assert!(diff.success());

    Command::new("rm")
        .arg(out)
        .output()
        .unwrap();
}