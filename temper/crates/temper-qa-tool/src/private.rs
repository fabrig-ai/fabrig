//! Exact modes for the tool's own place paths (qa/places.md, 2).
use crate::{
    Refusal,
    faces::{FilesFace, WriteMode, WriteOptions, private_directory},
    refused,
};
use std::path::Path;

pub(crate) fn directory(path: &Path, files: &mut impl FilesFace) -> Result<(), Refusal> {
    private_directory(files, path).map_err(|(path, why)| refused(format!("{}: {why}", path.display())))
}

pub(crate) fn file<F: FilesFace>(path: &Path, files: &mut F, way: WriteMode, read: bool) -> Result<F::Writer, Refusal> {
    files
        .open_write(path, WriteOptions { read, ..WriteOptions::private(way) })
        .map_err(|why| refused(format!("{}: {why}", path.display())))
}
