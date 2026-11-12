//! Pass preparation and changed recipes (qa/places.md, 2–3; qa/passes.md, 2 and 6).
use crate::{Home, Refusal, faces::GitFace, refused};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File, OpenOptions, TryLockError},
    io::{Read, Write},
    os::unix::{ffi::OsStringExt, fs::OpenOptionsExt},
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
    io(place, fs::create_dir_all(place))?;
    let path = place.join("drive.lock");
    let file =
        io(&path, OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(&path))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(why)) => Err(refused(format!("{}: {why}", path.display()))),
    }
}

pub(crate) fn install(build: &Path, commit: &str, place: &Path) -> Result<(), Refusal> {
    let manifest_path = build.join("manifest");
    let bytes = io(&manifest_path, fs::read(&manifest_path))?;
    let manifest = manifest(&bytes, commit).map_err(|why| refused(format!("{}: {why}", manifest_path.display())))?;
    let mut copied = BTreeSet::new();
    for name in ["bin", "bundle"] {
        remove(&place.join(name))?;
        copy(build, place, Path::new(name), &manifest, &mut copied)?;
    }
    for path in manifest.keys().filter(|path| path.starts_with("bin") || path.starts_with("bundle")) {
        if !copied.contains(path) {
            return Err(refused(format!("{}: manifest file was not copied", path.display())));
        }
    }
    let setup = place.join("setup");
    match fs::remove_file(&setup) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(refused(format!("{}: {why}", setup.display()))),
    }
}

fn copy(
    build: &Path,
    place: &Path,
    path: &Path,
    manifest: &BTreeMap<PathBuf, String>,
    copied: &mut BTreeSet<PathBuf>,
) -> Result<(), Refusal> {
    let source = build.join(path);
    let destination = place.join(path);
    let metadata = io(&source, fs::symlink_metadata(&source))?;
    if metadata.is_dir() {
        io(&destination, fs::create_dir_all(&destination))?;
        for child in children(&source)? {
            copy(build, place, &path.join(child), manifest, copied)?;
        }
    } else if metadata.is_file() {
        let expected =
            manifest.get(path).ok_or_else(|| refused(format!("{}: file is absent from manifest", path.display())))?;
        let (temporary, mut output) = temporary(&destination)?;
        let mut result = || -> Result<(), Refusal> {
            let mut input = io(&source, File::open(&source))?;
            io(&temporary, std::io::copy(&mut input, &mut output))?;
            io(&temporary, output.flush())?;
            io(&temporary, fs::set_permissions(&temporary, metadata.permissions()))?;
            if checksum(&temporary)? != *expected {
                return Err(refused(format!("{}: checksum differs from manifest", path.display())));
            }
            io(&destination, fs::rename(&temporary, &destination))
        };
        if let Err(why) = result() {
            let _ = fs::remove_file(&temporary);
            return Err(why);
        }
        copied.insert(path.into());
    } else {
        return Err(refused(format!("{}: neither a file nor a directory", source.display())));
    }
    Ok(())
}

fn temporary(destination: &Path) -> Result<(PathBuf, File), Refusal> {
    let mut number = 0_u64;
    loop {
        let mut name = destination.as_os_str().to_os_string();
        name.push(format!(".qa-tmp-{number}"));
        let path = PathBuf::from(name);
        match OpenOptions::new().read(true).write(true).create_new(true).mode(0o600).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => number = number.saturating_add(1),
            Err(why) => return Err(refused(format!("{}: {why}", path.display()))),
        }
    }
}

fn checksum(path: &Path) -> Result<String, Refusal> {
    let mut input = io(path, File::open(path))?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 16384];
    loop {
        let count = io(path, input.read(&mut buffer))?;
        if count == 0 {
            return Ok(format!("{:x}", hash.finalize()));
        }
        hash.update(&buffer[..count]);
    }
}

/// The build writer uses sha256sum's escaped, byte-oriented path form.
fn manifest(bytes: &[u8], commit: &str) -> Result<BTreeMap<PathBuf, String>, Refusal> {
    let mut lines = bytes.split(|byte| *byte == b'\n');
    if lines.next() != Some(format!("commit {commit}").as_bytes()) {
        return Err(refused("commit differs from build"));
    }
    let mut files = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (escaped, line) = line.strip_prefix(b"\\").map_or((false, line), |rest| (true, rest));
        let (hash, path) = line.split_at_checked(64).ok_or_else(|| refused("invalid checksum line"))?;
        let path = path.strip_prefix(b"  ").ok_or_else(|| refused("invalid checksum line"))?;
        if !hash.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
            return Err(refused("invalid checksum"));
        }
        let mut name = Vec::new();
        let mut bytes = path.iter();
        while let Some(byte) = bytes.next() {
            name.push(if escaped && *byte == b'\\' {
                match bytes.next() {
                    Some(b'\\') => b'\\',
                    Some(b'n') => b'\n',
                    Some(b'r') => b'\r',
                    Some(_) | None => return Err(refused("invalid escaped path")),
                }
            } else {
                *byte
            });
        }
        let path = PathBuf::from(OsString::from_vec(name));
        let hash = String::from_utf8(hash.to_vec()).map_err(|why| refused(why.to_string()))?;
        if files.insert(path.clone(), hash).is_some() {
            return Err(refused(format!("{}: duplicate manifest path", path.display())));
        }
    }
    Ok(files)
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
