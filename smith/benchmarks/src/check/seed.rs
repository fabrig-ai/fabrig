//! Frozen seed files and their digest (smith/docs/design/benchmarks/README.md, 5.2).

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::Refusal;
use crate::files::{Files, Kind, Native};

/// Hash a frozen seed by sorted relative names, executable bits and file bytes.
pub fn seed_digest(directory: &Path) -> Result<String, Refusal> {
    seed_digest_with(&Native, directory)
}

/// Hash a frozen seed on the supplied filesystem in a step test.
pub fn seed_digest_with(files: &dyn Files, directory: &Path) -> Result<String, Refusal> {
    let mut paths = Vec::new();
    seed_files(files, directory, directory, &mut paths)?;
    paths.sort();
    let mut digest = Sha256::new();
    for path in paths {
        let relative = path.strip_prefix(directory).expect("walked files remain under seed");
        let name = relative.to_str().ok_or_else(|| Refusal::new(&path, "seed", "file name must be UTF-8"))?;
        let bytes = files.read(&path).map_err(|error| Refusal::new(&path, "seed", error.to_string()))?;
        let facts = files.metadata(&path).map_err(|error| Refusal::new(&path, "seed", error.to_string()))?;
        let executable = facts.mode & 0o111 != 0;
        digest.update(skein_lib::len_u64(name.len()).to_be_bytes());
        digest.update(name.as_bytes());
        digest.update([u8::from(executable)]);
        digest.update(skein_lib::len_u64(bytes.len()).to_be_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn seed_files(files: &dyn Files, root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), Refusal> {
    let facts =
        files.symlink_metadata(directory).map_err(|error| Refusal::new(directory, "seed", error.to_string()))?;
    if facts.kind != Kind::Directory {
        return Err(Refusal::new(directory, "seed", "expected an ordinary seed directory"));
    }
    for entry in files.read_dir(directory).map_err(|error| Refusal::new(directory, "seed", error.to_string()))? {
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
        match entry.file_type().map_err(|error| Refusal::new(&path, "seed", error.to_string()))? {
            Kind::Directory => seed_files(files, root, &path, paths)?,
            Kind::File => paths.push(path),
            Kind::Link | Kind::Other => {
                return Err(Refusal::new(&path, "seed", "symlinks and special files are not frozen inputs"));
            }
        }
    }
    Ok(())
}
