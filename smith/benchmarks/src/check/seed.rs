//! Frozen seed files and their digest (smith/docs/design/benchmarks/README.md, 5.2).

use std::{
    fs, io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::Refusal;

pub(crate) mod testing {
    pub use super::{SeedEntry, SeedFiles, SeedKind, seed_digest_with};
}

/// The walk distinguishes ordinary files, directories and everything it refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedKind {
    /// An ordinary directory whose entries join the walk.
    Directory,
    /// A regular file whose relative name, mode and bytes enter the digest.
    File,
    /// A link or special node refused by the walk.
    Other,
}

/// A child entry from a seed directory.
pub trait SeedEntry {
    fn path(&self) -> PathBuf;
    fn kind(&self) -> io::Result<SeedKind>;
}

/// Files consulted by the seed walk and digest.
pub trait SeedFiles {
    type Entry: SeedEntry;
    type Children: Iterator<Item = io::Result<Self::Entry>>;

    fn ordinary_directory(&self, path: &Path) -> io::Result<bool>;
    fn children(&self, path: &Path) -> io::Result<Self::Children>;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn executable(&self, path: &Path) -> io::Result<bool>;
}

struct Native;

impl SeedEntry for fs::DirEntry {
    fn path(&self) -> PathBuf {
        self.path()
    }

    fn kind(&self) -> io::Result<SeedKind> {
        let kind = self.file_type()?;
        Ok(if kind.is_dir() {
            SeedKind::Directory
        } else if kind.is_file() {
            SeedKind::File
        } else {
            SeedKind::Other
        })
    }
}

impl SeedFiles for Native {
    type Entry = fs::DirEntry;
    type Children = fs::ReadDir;

    fn ordinary_directory(&self, path: &Path) -> io::Result<bool> {
        let metadata = fs::symlink_metadata(path)?;
        Ok(metadata.is_dir() && !metadata.is_symlink())
    }

    fn children(&self, path: &Path) -> io::Result<Self::Children> {
        fs::read_dir(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn executable(&self, path: &Path) -> io::Result<bool> {
        Ok(fs::metadata(path)?.permissions().mode() & 0o111 != 0)
    }
}

/// Hash a frozen seed by sorted relative names, executable bits and file bytes.
pub fn seed_digest(directory: &Path) -> Result<String, Refusal> {
    seed_digest_with(directory, &Native)
}

/// Hash a seed using the entry kinds, bytes and executable bits supplied by its filesystem.
pub fn seed_digest_with(directory: &Path, source: &impl SeedFiles) -> Result<String, Refusal> {
    let mut files = Vec::new();
    seed_files(directory, directory, source, &mut files)?;
    files.sort();
    let mut digest = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(directory).expect("walked files remain under seed");
        let name = relative.to_str().ok_or_else(|| Refusal::new(&path, "seed", "file name must be UTF-8"))?;
        let bytes = source.read(&path).map_err(|error| Refusal::new(&path, "seed", error.to_string()))?;
        let executable = source.executable(&path).map_err(|error| Refusal::new(&path, "seed", error.to_string()))?;
        digest.update(skein_lib::len_u64(name.len()).to_be_bytes());
        digest.update(name.as_bytes());
        digest.update([u8::from(executable)]);
        digest.update(skein_lib::len_u64(bytes.len()).to_be_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn seed_files(root: &Path, directory: &Path, source: &impl SeedFiles, files: &mut Vec<PathBuf>) -> Result<(), Refusal> {
    let ordinary =
        source.ordinary_directory(directory).map_err(|error| Refusal::new(directory, "seed", error.to_string()))?;
    if !ordinary {
        return Err(Refusal::new(directory, "seed", "expected an ordinary seed directory"));
    }
    for entry in source.children(directory).map_err(|error| Refusal::new(directory, "seed", error.to_string()))? {
        let entry = entry.map_err(|error| Refusal::new(directory, "seed", error.to_string()))?;
        let path = entry.path();
        let relative = path.strip_prefix(root).expect("walk remains under seed");
        if relative.components().any(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some("benchmarks" | "grader" | "grading" | "reference" | "reference-solutions" | ".git")
            )
        }) {
            return Err(Refusal::new(
                &path,
                "seed",
                "benchmark, grader, reference and git files stay outside the seed",
            ));
        }
        match entry.kind().map_err(|error| Refusal::new(&path, "seed", error.to_string()))? {
            SeedKind::Directory => seed_files(root, &path, source, files)?,
            SeedKind::File => files.push(path),
            SeedKind::Other => {
                return Err(Refusal::new(&path, "seed", "symlinks and special files are not frozen inputs"));
            }
        }
    }
    Ok(())
}
