//! Checksum defaults over file readers (temper/docs/design/testing/qa/builds.md, 3–4).
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};
use temper_qa_tool::faces::FilesFace;

struct ByteReader(&'static [u8]);
impl Read for ByteReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if let Some(byte) = buffer.first_mut() { self.0.read(std::slice::from_mut(byte)) } else { Ok(0) }
    }
}

struct Files(&'static [(&'static str, &'static [u8])]);
impl FilesFace for Files {
    type Lock = ();
    type Listener = ();
    type Connection = io::Cursor<Vec<u8>>;
    type Reader = ByteReader;
    type Writer = io::Sink;

    fn open_read(&mut self, path: &Path) -> io::Result<ByteReader> {
        self.0
            .iter()
            .find(|(name, _)| path == Path::new(name))
            .map(|(_, bytes)| ByteReader(bytes))
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }
}

const ESCAPED_PATH: &str = "x\\y";

#[test]
fn streamed_hash_matches_sha256sum_including_the_escaped_path() {
    let paths = [PathBuf::from(ESCAPED_PATH), PathBuf::from("pieces")];
    let lines = Files(&[("build/x\\y", b"a"), ("build/pieces", b"abc")])
        .hash(Path::new("build"), &paths)
        .expect("checksum lines");
    assert_eq!(
        lines,
        concat!(
            "\\ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb  x\\\\y\n",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  pieces\n",
        )
        .as_bytes()
    );
}

#[test]
fn check_names_only_mismatches_in_manifest_order() {
    // The kept manifest lists files in path order; all three held b"a" at keep.
    const MANIFEST: &[u8] = concat!(
        "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb  a\n",
        "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb  unchanged\n",
        "\\ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb  x\\\\y\n",
    )
    .as_bytes();
    let why = Files(&[("manifest", MANIFEST), ("build/a", b"b"), ("build/unchanged", b"a"), ("build/x\\y", b"c")])
        .check(Path::new("build"), Path::new("manifest"))
        .expect_err("mismatches");
    assert_eq!(why.0, "checksum check: a: FAILED; x\\y: FAILED");
}
