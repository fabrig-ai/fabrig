//! Linked installs and checked cross-device copies (qa/places.md, 3).
use crate::{Refusal, faces::BuildFilesFace, manifest, place::PlacePaths, private, refused};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// Install a kept build, reusing a completed install with the same manifest.
pub fn install(build: &Path, commit: &str, place: &Path, face: &mut impl BuildFilesFace) -> Result<(), Refusal> {
    PlacePaths::new(place).ensure()?;
    let manifest_path = build.join("manifest");
    let bytes = io(&manifest_path, fs::read(&manifest_path))?;
    let manifest =
        manifest::parse(&bytes, commit).map_err(|why| refused(format!("{}: {why}", manifest_path.display())))?;
    let receipt = place.join("manifest");
    match fs::read(&receipt) {
        Ok(installed) if installed == bytes => return Ok(()),
        Ok(_) => (),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
        Err(why) => return Err(refused(format!("{}: {why}", receipt.display()))),
    }
    remove_file(&receipt)?;
    let mut installed = BTreeSet::new();
    let mut copied = false;
    for name in ["bin", "bundle"] {
        remove(&place.join(name))?;
        install_tree(build, place, Path::new(name), &manifest, face, &mut installed, &mut copied)?;
    }
    for path in manifest.keys().filter(|path| installed_path(path)) {
        if !installed.contains(path) {
            return Err(refused(format!("{}: manifest file was not installed", path.display())));
        }
    }
    let temporary = place.join("manifest.tmp");
    if copied {
        // Check only installed paths; other kept directories are not part of a place.
        let mut checks = Vec::new();
        for line in bytes.split_inclusive(|byte| *byte == b'\n').skip(1) {
            let mut row = format!("commit {commit}\n").into_bytes();
            row.extend_from_slice(line);
            if manifest::parse(&row, commit)?.keys().any(|path| installed_path(path)) {
                checks.extend_from_slice(line);
            }
        }
        write_private(&temporary, &checks)?;
        let result = face.check(place, &temporary);
        remove_file(&temporary)?;
        result?;
    }
    remove_file(&place.join("setup"))?;
    write_private(&temporary, &bytes)?;
    io(&receipt, fs::rename(&temporary, &receipt))
}

fn installed_path(path: &Path) -> bool {
    path.starts_with("bin") || path.starts_with("bundle")
}

fn install_tree(
    build: &Path,
    place: &Path,
    path: &Path,
    manifest: &BTreeMap<PathBuf, String>,
    face: &mut impl BuildFilesFace,
    installed: &mut BTreeSet<PathBuf>,
    copied: &mut bool,
) -> Result<(), Refusal> {
    let source = build.join(path);
    let destination = place.join(path);
    let metadata = io(&source, fs::symlink_metadata(&source))?;
    if metadata.is_dir() {
        private::directory(&destination)?;
        for child in io(&source, fs::read_dir(&source))? {
            let child = io(&source, child)?;
            install_tree(build, place, &path.join(child.file_name()), manifest, face, installed, copied)?;
        }
    } else if metadata.is_file() {
        if !manifest.contains_key(path) {
            return Err(refused(format!("{}: file is absent from manifest", path.display())));
        }
        // A bundle's files are public documents, which the box serves with one link alone, so they are copied.
        let linked = path.starts_with("bin")
            && match face.link(&source, &destination) {
                Ok(()) => true,
                Err(why) if why.kind() == std::io::ErrorKind::CrossesDevices => false,
                Err(why) => return Err(refused(format!("{}: {why}", destination.display()))),
            };
        if !linked {
            let (temporary, mut output) = temporary(&destination)?;
            let mut input = io(&source, fs::File::open(&source))?;
            io(&temporary, std::io::copy(&mut input, &mut output))?;
            io(&temporary, output.flush())?;
            io(&temporary, output.set_permissions(metadata.permissions()))?;
            io(&destination, fs::rename(&temporary, &destination))?;
            *copied = true;
        }
        installed.insert(path.into());
    } else {
        return Err(refused(format!("{}: neither a file nor a directory", source.display())));
    }
    Ok(())
}

fn temporary(destination: &Path) -> Result<(PathBuf, fs::File), Refusal> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mut number = 0_u64;
    loop {
        let mut name = destination.as_os_str().to_os_string();
        name.push(format!(".qa-tmp-{number}"));
        let path = PathBuf::from(name);
        match OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path) {
            Ok(file) => {
                io(&path, file.set_permissions(fs::Permissions::from_mode(0o600)))?;
                return Ok((path, file));
            }
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => number = number.saturating_add(1),
            Err(why) => return Err(refused(format!("{}: {why}", path.display()))),
        }
    }
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), Refusal> {
    let mut file = private::file(path, OpenOptions::new().write(true).create(true).truncate(true))?;
    io(path, file.write_all(bytes))?;
    io(path, file.sync_all())
}
fn remove_file(path: &Path) -> Result<(), Refusal> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(refused(format!("{}: {why}", path.display()))),
    }
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
