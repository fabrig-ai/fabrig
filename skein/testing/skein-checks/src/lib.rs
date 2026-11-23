//! Counts of checks performed by test kits (`skein/docs/design/testing/checks.md`, 1).

use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Write},
    path::PathBuf,
    sync::OnceLock,
};

mod mutation;
pub use mutation::{Mutation, MutationBatch, mutation};

struct Output {
    path: PathBuf,
    file: File,
}

static OUTPUT: OnceLock<Option<Output>> = OnceLock::new();

/// Append one count for a check performed by a test kit.
#[inline]
pub fn count(kind: &'static str, n: u64) {
    assert!(
        kind.len() <= 32
            && kind.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
            && kind.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "invalid check kind: {kind:?}"
    );
    assert!(n > 0, "check count must be positive: {n}");
    if let Some(output) = OUTPUT.get_or_init(open) {
        output.append(kind, n);
    }
}

#[expect(clippy::disallowed_methods, reason = "the test runner supplies one output path, read once per process")]
fn open() -> Option<Output> {
    let name = std::env::var_os("TIERS_CHECKS_FILE").filter(|name| !name.is_empty())?;
    let path = PathBuf::from(name);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|error| panic!("open check counts {}: {error}", path.display()));
    Some(Output { path, file })
}

impl Output {
    fn append(&self, kind: &str, n: u64) {
        // One append write keeps a line whole across test threads and forked children.
        let mut line = Cursor::new([0; 54]);
        writeln!(line, "{kind} {n}").expect("a check count line fits 54 bytes");
        let length = usize::try_from(line.position()).expect("check count line length fits usize");
        let bytes = line.get_ref().get(..length).expect("formatted check count line");
        match (&self.file).write(bytes) {
            Ok(written) if written == length => {}
            Ok(written) => {
                panic!("append check counts {}: wrote {written} of {length} bytes", self.path.display())
            }
            Err(error) => panic!("append check counts {}: {error}", self.path.display()),
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
