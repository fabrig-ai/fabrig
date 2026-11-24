//! Publishing staged builds (temper/docs/design/testing/qa/builds.md, 3 and 4).
use crate::{
    Home, Refusal,
    cli::Environment,
    faces::{ClockFace, DirectoryMode, FilesFace, PathKind, Processes, WriteOptions, remove_any_if_present},
    ledger::short,
    refused,
};
use std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

/// Publish a staged build after checking its files and executable versions.
pub fn keep_with(
    home: &Home,
    commit: &str,
    directory: &Path,
    environment: &Environment,
    face: &mut impl FilesFace,
    processes: &mut impl Processes,
    clock: &impl ClockFace,
) -> Result<String, Refusal> {
    if !(commit.len() == 40 || commit.len() == 64)
        || !commit.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(refused(format!("{commit} is no commit id")));
    }
    home.transact(face, |ledger, face| {
        ledger.check_build_keep(commit)?;
        let files = check_files(directory, face)?;
        for path in &files {
            if path.parent() == Some(Path::new("bin")) {
                check_version(home, directory, path, commit, environment, face, processes)?;
            }
        }
        io(&home.builds, face.directory(&home.builds, DirectoryMode::Parents))?;
        let parent = directory.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
        if io(parent, face.facts(parent, true))?.device != io(&home.builds, face.facts(&home.builds, true))?.device {
            return Err(refused(format!(
                "{} is not on the filesystem of {}",
                directory.display(),
                home.builds.display()
            )));
        }
        let checksums = face.hash(directory, &files)?;
        let mut bytes = format!("commit {commit}\n").into_bytes();
        bytes.extend_from_slice(&checksums);
        let parsed = crate::manifest::parse(&bytes, commit)?;
        for path in &files {
            if !parsed.contains_key(path) {
                return Err(refused(format!("{}: file is absent from sha256sum answer", path.display())));
            }
        }
        if parsed.len() != files.len() {
            return Err(refused("sha256sum answer names an unexpected file"));
        }
        let partial = home.builds.join(format!(".{commit}.partial"));
        io(&partial, remove_any_if_present(face, &partial))?;
        io(directory, face.rename(directory, &partial))?;
        let manifest = partial.join("manifest");
        io(&manifest, face.write(&manifest, &bytes, WriteOptions::REPLACE, true))?;
        io(&partial, face.rename(&partial, &home.builds.join(commit)))?;
        ledger.record_build_kept(commit, clock.now().wall)?;
        Ok(format!("build {} kept: {} files\n", short(commit), files.len()))
    })
}

fn check_files(directory: &Path, face: &mut impl FilesFace) -> Result<Vec<PathBuf>, Refusal> {
    let mut missing = Vec::new();
    for path in [
        "bin/temper-box",
        "bin/fake-claude",
        "bin/temper",
        "bin/temper-client-journeys",
        "bin/temper-engine",
        "bin/temper-worker",
        "bin/temper-agent",
        "bundle/index.html",
    ] {
        if !face.facts(&directory.join(path), false).is_ok_and(|facts| facts.kind == PathKind::File) {
            missing.push(format!("{} lacks {path}", directory.display()));
        }
    }
    if !missing.is_empty() {
        return Err(refused(missing.join("; ")));
    }
    let mut files = Vec::new();
    walk(directory, Path::new(""), &mut files, face)?;
    files.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
    files.retain(|path| path != Path::new("manifest"));
    Ok(files)
}

fn walk(directory: &Path, path: &Path, files: &mut Vec<PathBuf>, face: &mut impl FilesFace) -> Result<(), Refusal> {
    let full = directory.join(path);
    let metadata = io(&full, face.facts(&full, false))?;
    if metadata.kind == PathKind::File {
        files.push(path.into());
    } else if metadata.kind == PathKind::Directory {
        let mut children = io(&full, face.names(&full))?
            .map(|name| name.map(|name| path.join(OsStr::from_bytes(&name))))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|why| refused(format!("{}: {why}", full.display())))?;
        children.sort();
        for child in children {
            walk(directory, &child, files, face)?;
        }
    } else {
        return Err(refused(format!(
            "{} holds {}, neither a file nor a directory",
            directory.display(),
            path.display()
        )));
    }
    Ok(())
}

fn check_version(
    home: &Home,
    directory: &Path,
    path: &Path,
    commit: &str,
    environment: &Environment,
    face: &mut impl FilesFace,
    processes: &mut impl Processes,
) -> Result<(), Refusal> {
    let executable = directory.join(path);
    if io(&executable, face.facts(&executable, true))?.mode & 0o111 == 0 {
        return Err(refused(format!("{} is not executable", path.display())));
    }
    if path.file_name() == Some(OsStr::new("fake-claude")) {
        return Ok(());
    }
    let line = processes.version(&executable, &home.directory.join("build-version.tmp"), environment)?;
    if !line.ends_with(&format!(" ({})", short(commit))) {
        return Err(refused(format!("{} prints {line}, not {}", path.display(), short(commit))));
    }
    Ok(())
}

fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
