//! Linked installs and checked cross-device copies (qa/places.md, 3).
use crate::{
    Refusal,
    faces::{FilesFace, Mode, PathKind, RemoveMode, WriteMode, WriteOptions, remove_missing},
    manifest,
    place::PlacePaths,
    private, refused,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

/// Install a kept build, reusing a completed install with the same manifest.
pub fn install(
    build: &Path,
    commit: &str,
    place: &Path,
    kind: crate::Kind,
    face: &mut impl FilesFace,
) -> Result<(), Refusal> {
    PlacePaths::new(place, kind).ensure(face)?;
    let manifest_path = build.join("manifest");
    let bytes = io(&manifest_path, face.read(&manifest_path))?;
    let manifest =
        manifest::parse(&bytes, commit).map_err(|why| refused(format!("{}: {why}", manifest_path.display())))?;
    let receipt = place.join("manifest");
    match face.read(&receipt) {
        Ok(installed) if installed == bytes => return Ok(()),
        Ok(_) => (),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
        Err(why) => return Err(refused(format!("{}: {why}", receipt.display()))),
    }
    io(&receipt, remove_missing(face, &receipt, RemoveMode::File))?;
    let mut installed = BTreeSet::new();
    let mut copied = false;
    for name in ["bin", "bundle"] {
        let path = place.join(name);
        io(&path, remove_missing(face, &path, RemoveMode::Tree))?;
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
        write_private(&temporary, &checks, face)?;
        let result = face.check(place, &temporary);
        io(&temporary, remove_missing(face, &temporary, RemoveMode::File))?;
        result?;
    }
    let setup = place.join("setup");
    io(&setup, remove_missing(face, &setup, RemoveMode::File))?;
    write_private(&temporary, &bytes, face)?;
    io(&receipt, face.rename(&temporary, &receipt))
}

fn installed_path(path: &Path) -> bool {
    path.starts_with("bin") || path.starts_with("bundle")
}

fn install_tree(
    build: &Path,
    place: &Path,
    path: &Path,
    manifest: &BTreeMap<PathBuf, String>,
    face: &mut impl FilesFace,
    installed: &mut BTreeSet<PathBuf>,
    copied: &mut bool,
) -> Result<(), Refusal> {
    let source = build.join(path);
    let destination = place.join(path);
    let metadata = io(&source, face.facts(&source, false))?;
    if metadata.kind == PathKind::Directory {
        private::directory(&destination, face)?;
        for name in io(&source, face.names(&source))? {
            let name = io(&source, name)?;
            install_tree(
                build,
                place,
                &path.join(std::ffi::OsStr::from_bytes(&name)),
                manifest,
                face,
                installed,
                copied,
            )?;
        }
    } else if metadata.kind == PathKind::File {
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
            let (temporary, mut output) = temporary(&destination, face)?;
            let mut input = io(&source, face.open_read(&source))?;
            io(&temporary, std::io::copy(&mut input, &mut output))?;
            io(&temporary, output.flush())?;
            io(&temporary, face.writer_permissions(&output, metadata.mode))?;
            io(&destination, face.rename(&temporary, &destination))?;
            *copied = true;
        }
        installed.insert(path.into());
    } else {
        return Err(refused(format!("{}: neither a file nor a directory", source.display())));
    }
    Ok(())
}

fn temporary<F: FilesFace>(destination: &Path, face: &mut F) -> Result<(PathBuf, F::Writer), Refusal> {
    let options = WriteOptions { way: WriteMode::New, mode: Mode::Create(0o600), read: false };
    let mut number = 0_u64;
    loop {
        let mut name = destination.as_os_str().to_os_string();
        name.push(format!(".qa-tmp-{number}"));
        let path = PathBuf::from(name);
        match face.open_write(&path, options) {
            Ok(file) => {
                io(&path, face.writer_permissions(&file, 0o600))?;
                return Ok((path, file));
            }
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => number = number.saturating_add(1),
            Err(why) => return Err(refused(format!("{}: {why}", path.display()))),
        }
    }
}

fn write_private(path: &Path, bytes: &[u8], files: &mut impl FilesFace) -> Result<(), Refusal> {
    io(path, files.write(path, bytes, WriteOptions::private(WriteMode::Replace), true))
}
fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
