//! Exact modes for the tool's own place paths (qa/places.md, 2).
use crate::{Refusal, refused};
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::Path,
};

pub(crate) fn directory(path: &Path) -> Result<(), Refusal> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty() && !parent.exists()) {
        directory(parent)?;
    }
    match DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => {}
        Err(why) => return Err(refused(format!("{}: {why}", path.display()))),
    }
    io(path, fs::set_permissions(path, fs::Permissions::from_mode(0o700)))
}

pub(crate) fn file(path: &Path, options: &mut OpenOptions) -> Result<File, Refusal> {
    let file = io(path, options.mode(0o600).open(path))?;
    io(path, file.set_permissions(fs::Permissions::from_mode(0o600)))?;
    Ok(file)
}

fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
