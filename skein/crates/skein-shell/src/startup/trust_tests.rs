use super::{RootBounds, RootRead, Roots, RootsRefusal, Trust, Unread, roots_with};
use std::path::Path;

const BOUNDS: RootBounds = RootBounds { bundle_bytes: 4_096, certificates: 8, certificate_bytes: 32 };

fn one_read(
    trust: &Trust,
    bounds: &RootBounds,
    expected: RootRead<'_>,
    reply: Result<Box<[u8]>, Unread>,
) -> Result<Roots, RootsRefusal> {
    let mut reply = Some(reply);
    let roots = roots_with(trust, bounds, |request| {
        assert_eq!(request, expected, "path and byte bound handed to the reader");
        reply.take().expect("only a single read of the configured path")
    });
    assert!(reply.is_none(), "asked the reader for the configured path");
    roots
}

#[test]
fn startup_takes_the_der_file_whole_as_one_certificate() {
    let path = Path::new("root.der");
    let bytes = b"DER bytes";
    let roots = one_read(
        &Trust::Der(path.to_path_buf()),
        &BOUNDS,
        RootRead { path, max: BOUNDS.bundle_bytes },
        Ok(Box::from(bytes.as_slice())),
    )
    .expect("the DER source is read, not parsed as PEM");
    assert_eq!(roots.certificates.as_ref(), [Box::from(bytes.as_slice())]);
    assert_eq!((roots.other_blocks, roots.malformed), (0, 0));
    assert_eq!(roots.path, path);
}

#[test]
fn startup_passes_the_explicit_bundle_path_and_byte_bound_to_the_reader() {
    let path = Path::new("bundle");
    let trust = Trust::Machine { ssl_cert_file: Some(path.to_path_buf()) };
    let bundle = b"-----BEGIN CERTIFICATE-----\nTWFu\n-----END CERTIFICATE-----\n";
    let length = u32::try_from(bundle.len()).expect("fixture fits");
    let exact = RootBounds { bundle_bytes: length, ..BOUNDS };
    let roots = one_read(&trust, &exact, RootRead { path, max: length }, Ok(Box::from(bundle.as_slice())))
        .expect("exact bundle bytes");
    assert_eq!(roots.certificates.as_ref(), [Box::from(b"Man".as_slice())]);
    assert_eq!(roots.path, path);

    let max = length - 1;
    let past = RootBounds { bundle_bytes: max, ..BOUNDS };
    let why = Unread::TooLarge { max };
    assert_eq!(
        one_read(&trust, &past, RootRead { path, max }, Err(why)).expect_err("bundle past bound"),
        RootsRefusal::Unread { path: path.to_path_buf(), why }
    );
}
