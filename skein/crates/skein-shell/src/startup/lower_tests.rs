//! PEM and source decisions with in-memory input (shell.md, 6.2).

use std::path::Path;

use super::{RootBounds, Roots, RootsRefusal, Unread, pem, select_source};

const BOUNDS: RootBounds = RootBounds { bundle_bytes: 4_096, certificates: 8, certificate_bytes: 32 };
const VALID: &[u8] = b"-----BEGIN CERTIFICATE-----\nTWFu\n-----END CERTIFICATE-----\n";
const OTHER: &[u8] = b"-----BEGIN CERTIFICATE-----\nTWE=\n-----END CERTIFICATE-----\n";

fn parse(bytes: &[u8], bounds: RootBounds) -> Result<Roots, RootsRefusal> {
    pem(bytes, Path::new("bundle"), &bounds)
}

#[test]
fn pem_preserves_certificate_order_and_skips_other_blocks_without_decoding() {
    let bytes = b"explanatory text\r\n-----BEGIN PRIVATE KEY-----\ninvalid!\n-----END PRIVATE KEY-----\n\
        -----BEGIN CERTIFICATE-----\n T W\tF\x0bu\x0c \n-----END CERTIFICATE-----\n\
        -----BEGIN TRUSTED CERTIFICATE-----\nbroken!\n-----END TRUSTED CERTIFICATE-----\n\
        -----BEGIN CERTIFICATE-----\nAP8=\n-----END CERTIFICATE-----\n";
    let roots = parse(bytes, BOUNDS).expect("well-framed certificates");
    assert_eq!(roots.certificates.as_ref(), [Box::from(b"Man".as_slice()), Box::from([0, 255].as_slice())]);
    assert_eq!((roots.other_blocks, roots.malformed), (2, 0));
    assert_eq!(roots.path, Path::new("bundle"));
}

#[test]
fn pem_recovers_malformed_blocks_without_spending_the_certificate_count() {
    for bad in [
        b"-----BEGIN CERTIFICATE-----\nTW!u\n-----END CERTIFICATE-----\n".as_slice(),
        b"-----BEGIN CERTIFICATE-----\nTWFu\n".as_slice(),
        b"-----BEGIN CERTIFICATE-----\nTWFu\n-----END KEY-----\n".as_slice(),
        b"-----BEGIN CERTIFICATE-----\nProc-Type: 4,ENCRYPTED\nTWFu\n-----END CERTIFICATE-----\n".as_slice(),
        b"-----BEGIN CERTIFICATE-----\nTW=Fu\n-----END CERTIFICATE-----\n".as_slice(),
        b"-----BEGIN CERTIFICATE-----\nT\n-----END CERTIFICATE-----\n".as_slice(),
    ] {
        let mut bytes = Vec::from(bad);
        bytes.extend_from_slice(VALID);
        let roots = parse(&bytes, RootBounds { certificates: 1, ..BOUNDS }).expect("recover next root");
        assert_eq!(roots.certificates.as_ref(), [Box::from(b"Man".as_slice())]);
        assert_eq!((roots.malformed, roots.other_blocks), (1, 0), "one bad certificate block: {bad:?}");
    }
    let mut bytes = Vec::from(b"-----BEGIN CERTIFICATE-----\n!\n-----END CERTIFICATE-----\n".as_slice());
    bytes.extend_from_slice(b"-----BEGIN CERTIFICATE-----\nTWFu\n");
    bytes.extend_from_slice(VALID);
    let roots = parse(&bytes, RootBounds { certificates: 1, ..BOUNDS }).expect("two malformed blocks then a root");
    assert_eq!(roots.certificates.as_ref(), [Box::from(b"Man".as_slice())]);
    assert_eq!((roots.malformed, roots.other_blocks), (2, 0));
    let roots = parse(b"-----BEGIN CERTIFICATE-----\nTWFu", BOUNDS).expect("truncated block is skipped");
    assert!(roots.certificates.is_empty(), "none recovered");
    assert_eq!((roots.malformed, roots.other_blocks), (1, 0));
}

#[test]
fn pem_checks_both_sides_of_bounds_and_keeps_first_error_precedence() {
    let roots = parse(VALID, RootBounds { certificates: 1, certificate_bytes: 3, ..BOUNDS }).expect("exact bounds");
    assert_eq!(roots.certificates.as_ref(), [Box::from(b"Man".as_slice())]);
    assert_eq!(
        parse(VALID, RootBounds { certificate_bytes: 2, ..BOUNDS }).expect_err("one byte past"),
        RootsRefusal::TooLong { index: 0, bound: 2 }
    );
    assert_eq!(
        parse(VALID, RootBounds { certificates: 0, ..BOUNDS }).expect_err("one certificate past"),
        RootsRefusal::TooMany { bound: 0 }
    );
    assert!(parse(b"", RootBounds { certificates: 0, ..BOUNDS }).expect("empty fits zero").certificates.is_empty());
    let mut two = Vec::from(VALID);
    two.extend_from_slice(OTHER);
    assert_eq!(
        parse(&two, RootBounds { certificates: 1, ..BOUNDS }).expect_err("second certificate past"),
        RootsRefusal::TooMany { bound: 1 }
    );
    let bad_first = b"-----BEGIN CERTIFICATE-----\n!TWFu\n-----END CERTIFICATE-----\n";
    let roots = parse(bad_first, RootBounds { certificate_bytes: 0, ..BOUNDS }).expect("first defect wins");
    assert!(roots.certificates.is_empty());
    assert_eq!(roots.malformed, 1);
    let bound_first = b"-----BEGIN CERTIFICATE-----\nTWFu!\n-----END CERTIFICATE-----\n";
    assert_eq!(
        parse(bound_first, RootBounds { certificate_bytes: 0, ..BOUNDS }).expect_err("first bound wins"),
        RootsRefusal::TooLong { index: 0, bound: 0 }
    );
    let mut after_bad = Vec::from(bad_first.as_slice());
    after_bad.extend_from_slice(VALID);
    assert_eq!(
        parse(&after_bad, RootBounds { certificate_bytes: 2, ..BOUNDS }).expect_err("second certificate long"),
        RootsRefusal::TooLong { index: 1, bound: 2 }
    );
}

#[test]
fn supplied_sources_keep_the_first_existing_bundle() {
    let mut selected = None;
    select_source(&mut selected, Path::new("absent"), Err(Unread::Absent), &BOUNDS);
    assert!(selected.is_none(), "only an absent path permits fallback");
    select_source(&mut selected, Path::new("first"), Ok(Box::from(VALID)), &BOUNDS);
    select_source(&mut selected, Path::new("second"), Ok(Box::from(OTHER)), &BOUNDS);
    let roots = selected.expect("an existing source").expect("valid PEM");
    assert_eq!(roots.path, Path::new("first"));
    assert_eq!(roots.certificates.as_ref(), [Box::from(b"Man".as_slice())]);
}

#[test]
fn supplied_source_errors_prevent_fallback() {
    for why in [Unread::NotAFile, Unread::TooLarge { max: 4_096 }, Unread::Kernel(skein_io::kernel::Error::Permission)]
    {
        let mut selected = None;
        select_source(&mut selected, Path::new("first"), Err(why), &BOUNDS);
        select_source(&mut selected, Path::new("second"), Ok(Box::from(VALID)), &BOUNDS);
        assert_eq!(
            selected.expect("refused source is final").expect_err("no fallback"),
            RootsRefusal::Unread { path: Path::new("first").to_path_buf(), why }
        );
    }
    let mut selected = None;
    let bounds = RootBounds { certificate_bytes: 2, ..BOUNDS };
    select_source(&mut selected, Path::new("first"), Ok(Box::from(VALID)), &bounds);
    select_source(&mut selected, Path::new("second"), Ok(Box::from(OTHER)), &bounds);
    assert_eq!(
        selected.expect("existing source is final").expect_err("no fallback on PEM error"),
        RootsRefusal::TooLong { index: 0, bound: 2 }
    );
}

#[test]
fn an_existing_empty_bundle_prevents_fallback() {
    let mut selected = None;
    select_source(&mut selected, Path::new("first"), Ok(Box::from([])), &BOUNDS);
    select_source(&mut selected, Path::new("second"), Ok(Box::from(VALID)), &BOUNDS);
    let roots = selected.expect("an empty source still exists").expect("PEM leaves empty-root policy to TLS");
    assert_eq!(roots.path, Path::new("first"));
    assert!(roots.certificates.is_empty());
}
