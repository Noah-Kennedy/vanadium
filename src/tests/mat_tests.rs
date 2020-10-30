use crate::bin_formats::bil::Bil;
use crate::bin_formats::bip::Bip;
use crate::bin_formats::bsq::Bsq;
pub use crate::bin_formats::Mat;
use crate::tests::{BIL, BIP, BSQ, make_mat};

#[test]
fn equality() {
    let (bip, bil, bsq) = unsafe {
        let bip = make_mat::<Bip<_, _>>(&BIP);
        let bil = make_mat::<Bil<_, _>>(&BIL);
        let bsq = make_mat::<Bsq<_, _>>(&BSQ);

        (bip, bil, bsq)
    };

    assert!(bip == bip);
    assert!(bip == bil);
    assert!(bip == bsq);

    assert!(bil == bip);
    assert!(bil == bil);
    assert!(bil == bsq);

    assert!(bsq == bip);
    assert!(bsq == bil);
    assert!(bsq == bsq);
}

#[test]
fn conversion() {
    let (bip, bil, bsq) = unsafe {
        let bip = make_mat::<Bip<_, _>>(&BIP);
        let bil = make_mat::<Bil<_, _>>(&BIL);
        let bsq = make_mat::<Bsq<_, _>>(&BSQ);

        (bip, bil, bsq)
    };

    let (mut bip1, mut bil1, mut bsq1) = unsafe {
        let bip = make_mat::<Bip<_, _>>(&BIP);
        let bil = make_mat::<Bil<_, _>>(&BIL);
        let bsq = make_mat::<Bsq<_, _>>(&BSQ);

        (bip, bil, bsq)
    };

    let (mut bip2, mut bil2, mut bsq2) = unsafe {
        let bip = make_mat::<Bip<_, _>>(&BIP);
        let bil = make_mat::<Bil<_, _>>(&BIL);
        let bsq = make_mat::<Bsq<_, _>>(&BSQ);

        (bip, bil, bsq)
    };

    let (mut bip3, mut bil3, mut bsq3) = unsafe {
        let bip = make_mat::<Bip<_, _>>(&BIP);
        let bil = make_mat::<Bil<_, _>>(&BIL);
        let bsq = make_mat::<Bsq<_, _>>(&BSQ);

        (bip, bil, bsq)
    };

    bip.convert(&mut bip1);
    bip.convert(&mut bil1);
    bip.convert(&mut bsq1);

    bil.convert(&mut bip2);
    bil.convert(&mut bil2);
    bil.convert(&mut bsq2);

    bsq.convert(&mut bip3);
    bsq.convert(&mut bil3);
    bsq.convert(&mut bsq3);

    assert!(bip == bip1);
    assert!(bip == bil2);
    assert!(bip == bsq3);

    assert!(bil == bip1);
    assert!(bil == bil2);
    assert!(bil == bsq3);

    assert!(bsq == bip1);
    assert!(bsq == bil2);
    assert!(bsq == bsq3);
}