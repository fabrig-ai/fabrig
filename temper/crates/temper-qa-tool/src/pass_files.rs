//! Pass preparation and changed recipes (qa/places.md, 2–3; qa/passes.md, 2 and 6).
use crate::faces::{DirectoryMode, FilesFace, LockMode, PathKind, RemoveMode, WriteOptions, remove_missing};
use crate::{Home, Refusal, faces::GitFace, place::PlacePaths, private, refused};
use std::{
    ffi::OsString,
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};

pub(crate) const SETUPS: &str = "temper/docs/development/qa/setups";

pub(crate) fn setups(home: &Home, pass: u64) -> PathBuf {
    home.directory.join("passes").join(pass.to_string()).join("setups")
}

pub(crate) fn files(home: &Home, pass: u64) -> PathBuf {
    home.directory.join("passes").join(pass.to_string()).join("files")
}

pub(crate) fn prepare_files(home: &Home, pass: u64, fs: &mut impl FilesFace) -> Result<(), Refusal> {
    let directory = files(home, pass);
    io(&directory, fs.directory(&directory, DirectoryMode::Parents))
}

/// Keep the free-place test's lock through preparation and recording.
pub(crate) fn free_lock<F: FilesFace>(place: &Path, kind: crate::Kind, fs: &mut F) -> Result<Option<F::Lock>, Refusal> {
    let paths = PlacePaths::new(place, kind);
    for path in [&paths.directory, &paths.scratch, &paths.copies] {
        private::directory(path, fs)?;
    }
    let path = place.join("drive.lock");
    io(&path, fs.lock(&path, LockMode::Try, true))
}

pub(crate) fn write_setups(
    git: &mut impl GitFace,
    commit: &str,
    directory: &Path,
    fs: &mut impl FilesFace,
) -> Result<(), Refusal> {
    io(directory, remove_missing(fs, directory, RemoveMode::Tree))?;
    write_tree(git, commit, SETUPS, directory, fs)
}

fn write_tree(
    git: &mut impl GitFace,
    commit: &str,
    source: &str,
    directory: &Path,
    fs: &mut impl FilesFace,
) -> Result<(), Refusal> {
    io(directory, fs.directory(directory, DirectoryMode::Parents))?;
    for entry in git.entries(commit, source)? {
        let path = format!("{source}/{}", entry.name);
        let destination = directory.join(entry.name);
        if entry.directory {
            write_tree(git, commit, &path, &destination, fs)?;
        } else {
            let bytes = git.file(commit, &path)?.ok_or_else(|| refused(format!("{commit}: missing {path}")))?;
            io(&destination, fs.write(&destination, &bytes, WriteOptions::REPLACE, false))?;
        }
    }
    Ok(())
}

pub(crate) fn changed_setups(
    git: &mut impl GitFace,
    commit: &str,
    directory: &Path,
    fs: &mut impl FilesFace,
) -> Result<Vec<String>, Refusal> {
    if fs.facts(directory, true).is_err() {
        return Ok(Vec::new());
    }
    let mut changed = Vec::new();
    for name in children(directory, fs)? {
        let recipe = directory.join(&name);
        if io(&recipe, fs.facts(&recipe, true))?.kind != PathKind::Directory {
            continue;
        }
        let name = name.to_str().ok_or_else(|| refused(format!("{}: recipe name is not UTF-8", recipe.display())))?;
        let mut differs = git.entries(commit, &format!("{SETUPS}/{name}"))?.is_empty();
        for file in ["recipe", "scripts.json", "README.md"] {
            let path = recipe.join(file);
            let local = match fs.read(&path) {
                Ok(bytes) => Some(bytes),
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => None,
                Err(why) => return Err(refused(format!("{}: {why}", path.display()))),
            };
            differs |= local != git.file(commit, &format!("{SETUPS}/{name}/{file}"))?;
        }
        if differs {
            changed.push(name.into());
        }
    }
    Ok(changed)
}

fn children(directory: &Path, fs: &mut impl FilesFace) -> Result<Vec<OsString>, Refusal> {
    let mut names = io(directory, fs.names(directory))?
        .map(|name| name.map(|name| OsString::from_vec(name.into_vec())))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    names.sort();
    Ok(names)
}

fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
