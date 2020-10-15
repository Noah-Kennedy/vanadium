use super::*;
use crate::cli::ConvertOpt;
use std::process::Command;

#[test]
fn test_convert() {
    let opt = ConvertOpt {
        input: "data/raw/unnormalized/unnorm.bip".into(),
        input_header: "data/raw/unnormalized/unnorm.bip.hdr".into(),
        output: "out.bsq".into(),
        output_header: None,
        output_type: Interleave::Bsq
    };

    execute_conversion(opt).unwrap();

    let diff = Command::new("diff")
        .arg("out.bsq")
        .arg("data/raw/unnormalized/unnorm.bsq")
        .status()
        .unwrap();

    assert!(diff.success())
}