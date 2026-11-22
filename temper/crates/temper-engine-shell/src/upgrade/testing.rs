//! Scratch operations exercised on the real kernel.
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub fn make(cache: &Path) -> Result<(PathBuf, File), String> {
    super::scratch::make(cache)
}

#[must_use]
pub fn sweep(binary: &str, cache: &Path) -> Vec<String> {
    super::scratch::sweep(binary, cache)
}

#[must_use]
pub fn remove(binary: &str, path: &Path, held: File) -> Option<String> {
    super::scratch::remove(binary, path, held)
}
