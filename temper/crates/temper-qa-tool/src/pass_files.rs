//! Pass preparation and changed recipes (qa/places.md, 2–3; qa/passes.md, 2 and 6).
use crate::{Home, Refusal, faces::GitFace, place::PlacePaths, private, refused};
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions, TryLockError},
    path::{Path, PathBuf},
};

pub(crate) const SETUPS: &str = "temper/docs/development/qa/setups";

pub(crate) fn setups(home: &Home, pass: u64) -> PathBuf {
    home.directory.join("passes").join(pass.to_string()).join("setups")
}

pub(crate) fn files(home: &Home, pass: u64) -> PathBuf {
    home.directory.join("passes").join(pass.to_string()).join("files")
}

pub(crate) fn prepare_files(home: &Home, pass: u64) -> Result<(), Refusal> {
    let directory = files(home, pass);
    io(&directory, fs::create_dir_all(&directory))
}

/// Keep the free-place test's lock through preparation and recording.
pub(crate) fn free_lock(place: &Path) -> Result<Option<File>, Refusal> {
    let paths = PlacePaths::new(place);
    for path in [&paths.directory, &paths.scratch, &paths.copies] {
        private::directory(path)?;
    }
    let path = place.join("drive.lock");
    let file = private::file(&path, OpenOptions::new().read(true).write(true).create(true).truncate(false))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(why)) => Err(refused(format!("{}: {why}", path.display()))),
    }
}

pub(crate) fn write_setups(git: &mut impl GitFace, commit: &str, directory: &Path) -> Result<(), Refusal> {
    remove(directory)?;
    write_tree(git, commit, SETUPS, directory)
}

fn write_tree(git: &mut impl GitFace, commit: &str, source: &str, directory: &Path) -> Result<(), Refusal> {
    io(directory, fs::create_dir_all(directory))?;
    for entry in git.entries(commit, source)? {
        let path = format!("{source}/{}", entry.name);
        let destination = directory.join(entry.name);
        if entry.directory {
            write_tree(git, commit, &path, &destination)?;
        } else {
            let bytes = git.file(commit, &path)?.ok_or_else(|| refused(format!("{commit}: missing {path}")))?;
            io(&destination, fs::write(&destination, bytes))?;
        }
    }
    Ok(())
}

pub(crate) fn changed_setups(git: &mut impl GitFace, commit: &str, directory: &Path) -> Result<Vec<String>, Refusal> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut changed = Vec::new();
    for name in children(directory)? {
        let recipe = directory.join(&name);
        if !io(&recipe, fs::metadata(&recipe))?.is_dir() {
            continue;
        }
        let name = name.to_str().ok_or_else(|| refused(format!("{}: recipe name is not UTF-8", recipe.display())))?;
        let mut differs = git.entries(commit, &format!("{SETUPS}/{name}"))?.is_empty();
        for file in ["recipe", "scripts.json", "README.md"] {
            let path = recipe.join(file);
            let local = match fs::read(&path) {
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

fn children(directory: &Path) -> Result<Vec<OsString>, Refusal> {
    let mut names = io(directory, fs::read_dir(directory))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    names.sort();
    Ok(names)
}

fn remove(path: &Path) -> Result<(), Refusal> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(refused(format!("{}: {why}", path.display()))),
    }
}

fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
