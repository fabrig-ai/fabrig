//! Real checksums preserve a non-UTF-8 file name (temper/docs/design/testing/qa/README.md, 6).
use sha2::{Digest, Sha256};
use skein_scratch::Scratch;
use std::{fs, path::PathBuf};
use temper_qa_tool::faces::{FilesFace, RealFiles};

#[test]
fn real_checksums_keep_a_non_utf8_path_as_its_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let scratch = Scratch::new("qa-checksum-byte-path");
    fs::create_dir(scratch.path().join("bundle")).expect("bundle");
    let path = PathBuf::from(OsString::from_vec(b"bundle/weird\\\n\r\xff".to_vec()));
    fs::write(scratch.path().join(&path), b"asset").expect("byte path");
    let lines = RealFiles.hash(scratch.path(), &[path]).expect("checksum");
    let mut expected = format!("\\{:x}  bundle/weird\\\\\\n\\r", Sha256::digest(b"asset")).into_bytes();
    expected.extend_from_slice(b"\xff\n");
    assert_eq!(lines, expected);
    let manifest = scratch.path().join("manifest.tmp");
    fs::write(&manifest, &lines).expect("manifest");
    RealFiles.check(scratch.path(), &manifest).expect("raw byte path checked");
}
