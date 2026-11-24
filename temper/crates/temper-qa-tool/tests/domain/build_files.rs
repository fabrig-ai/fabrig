//! The default checksum methods over the QA world's reads (temper/docs/design/testing/qa/builds.md, 3 and 4).
use sha2::{Digest, Sha256};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    faces::{DirectoryMode, FilesFace, RemoveMode, WriteOptions},
    testing::{FileOp, Tree},
};

#[test]
fn real_checksums_write_sha256sum_lines_for_plain_and_escaped_paths() {
    // More than one streaming buffer must contribute to the digest.
    const ASSET_BYTES: usize = 150_000;
    let mut tree = Tree::default();
    let directory = Path::new("/");
    tree.directory(Path::new("/bin"), DirectoryMode::One).expect("bin");
    tree.directory(Path::new("/bundle"), DirectoryMode::One).expect("bundle");
    let plain = PathBuf::from("bin/x");
    let escaped = PathBuf::from("bundle/a\\b\nc\r");
    let asset = vec![b'a'; ASSET_BYTES];
    tree.write(&directory.join(&plain), b"x", WriteOptions::REPLACE, false).expect("plain file");
    tree.write(&directory.join(&escaped), &asset, WriteOptions::REPLACE, false).expect("escaped file");
    let answer = tree.hash(directory, &[plain, escaped]).expect("checksums");
    let expected = format!(
        "2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881  bin/x\n\\{:x}  bundle/a\\\\b\\nc\\r\n",
        Sha256::digest(&asset),
    );
    assert_eq!(answer, expected.as_bytes());
}

#[test]
fn a_real_checksum_refuses_a_file_that_does_not_read() {
    let mut tree = Tree::default();
    let directory = Path::new("/");
    let missing = PathBuf::from("bundle/absent");
    tree.inject(directory.join(&missing), FileOp::OpenRead, ErrorKind::NotFound);
    let why = tree.hash(directory, std::slice::from_ref(&missing)).expect_err("absent file");
    assert!(why.0.starts_with("checksum: "), "{why}");
    assert!(why.0.contains(&missing.display().to_string()), "{why}");
}

#[test]
fn a_real_checksum_check_names_every_failed_path_in_order() {
    let mut tree = Tree::default();
    let place = Path::new("/");
    tree.directory(&place.join("bundle"), DirectoryMode::One).expect("bundle");
    let changed = PathBuf::from("bundle/a\\b\nc\r");
    let unchanged = PathBuf::from("bundle/m");
    let removed = PathBuf::from("bundle/z");
    for path in [&changed, &unchanged, &removed] {
        tree.write(&place.join(path), b"asset", WriteOptions::REPLACE, false).expect("asset");
    }
    // The manifest's order does not decide the refusal's path order.
    let lines = tree.hash(place, &[removed.clone(), unchanged, changed.clone()]).expect("checksums");
    let manifest = place.join("manifest.tmp");
    tree.write(&manifest, &lines, WriteOptions::REPLACE, false).expect("manifest");
    tree.check(place, &manifest).expect("matching checksums, including escaped path");
    tree.write(&place.join(&changed), b"changed", WriteOptions::REPLACE, false).expect("change bytes");
    tree.remove(&place.join(&removed), RemoveMode::File).expect("remove file");
    let io_error = tree.open_read(&place.join(&removed)).expect_err("absent file");
    let why = tree.check(place, &manifest).expect_err("both failures");
    assert_eq!(why.0, format!("checksum check: {}: FAILED; {}: {io_error}", changed.display(), removed.display()));
}

#[test]
fn a_real_checksum_check_refuses_an_invalid_manifest_with_the_readers_words() {
    let mut tree = Tree::default();
    let directory = Path::new("/");
    let manifest = directory.join("manifest.tmp");
    tree.write(&manifest, b"not a checksum\n", WriteOptions::REPLACE, false).expect("invalid manifest");
    let why = tree.check(directory, &manifest).expect_err("invalid checksum row");
    assert_eq!(why.0, "checksum check: invalid checksum line");
}
