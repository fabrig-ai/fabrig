//! Failure evidence for the owned-forge sitting (temper/docs/design/testing/box/README.md, 3.16).
use super::files::FileKind;
use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead, BufReader},
    os::unix::fs::{DirBuilderExt, symlink},
    path::{Path, PathBuf},
};

/// A listed entry's name and its no-follow kind, read separately so exclusions need no type read.
pub trait DirectoryEntry {
    fn name(&self) -> OsString;
    fn kind(&self) -> io::Result<FileKind>;
}

impl DirectoryEntry for fs::DirEntry {
    fn name(&self) -> OsString {
        self.file_name()
    }

    fn kind(&self) -> io::Result<FileKind> {
        self.file_type().map(FileKind::from)
    }
}

/// Failed and refused run records, oldest plain-text rotation first, redacted for notices.
#[must_use]
pub fn run_failure_lines(data: &Path) -> Vec<String> {
    run_failure_lines_with(data, |path| fs::read_dir(path), |path| fs::File::open(path).map(BufReader::new))
}

#[must_use]
pub fn run_failure_lines_with<E: DirectoryEntry, L: IntoIterator<Item = io::Result<E>>, R: BufRead>(
    data: &Path,
    list: impl FnOnce(&Path) -> io::Result<L>,
    mut open: impl FnMut(&Path) -> io::Result<R>,
) -> Vec<String> {
    let directory = data.join("box/logs");
    let Ok(entries) = list(&directory) else { return Vec::new() };
    let mut logs = Vec::new();
    for entry in entries.into_iter().flatten() {
        if !entry.kind().is_ok_and(|kind| kind == FileKind::File) {
            continue;
        }
        let name = entry.name();
        let Some(name) = name.to_str() else { continue };
        let age = if name == "serve.log" {
            0
        } else if let Some(age) = name.strip_prefix("serve.log.").and_then(|age| age.parse::<u64>().ok()) {
            age
        } else {
            // Compressed and unrecognized siblings aren't plain-text serve logs.
            continue;
        };
        logs.push((age, directory.join(name)));
    }
    logs.sort_by_key(|(age, _)| std::cmp::Reverse(*age));
    let mut lines = Vec::new();
    for (_, path) in logs {
        let Ok(mut reader) = open(&path) else { continue };
        let mut bytes = Vec::new();
        while reader.read_until(b'\n', &mut bytes).is_ok_and(|read| read != 0) {
            let line = String::from_utf8_lossy(&bytes);
            let line = line.trim_end_matches(['\r', '\n']);
            if is_run_failure(line) {
                lines.push(super::super::live::redact(line));
            }
            bytes.clear();
        }
    }
    lines
}

fn number(rest: &str) -> Option<&str> {
    let (digits, rest) = rest.split_once(' ')?;
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())).then_some(rest)
}

fn is_run_failure(line: &str) -> bool {
    let rest = if let Some((binary, rest)) = line.split_once(" serve: task ") {
        if binary.is_empty() || binary.contains(char::is_whitespace) {
            return false;
        }
        number(rest).and_then(|rest| rest.strip_prefix("run "))
    } else if let Some((binary, rest)) = line.split_once(" worker: run ") {
        if binary.is_empty() || binary.contains(char::is_whitespace) {
            return false;
        }
        Some(rest)
    } else {
        None
    };
    let Some(rest) = rest.and_then(number).and_then(|rest| rest.strip_prefix("attempt ")).and_then(number) else {
        return false;
    };
    if let Some(class) = rest.strip_prefix("failed ") {
        return class.split_once(':').is_some_and(|(class, _)| !class.is_empty());
    }
    // Version refusals have no separate class in serve's operator line.
    line.contains(" serve: task ")
        && (rest.starts_with("refused:")
            || rest.strip_prefix("refused ").is_some_and(|rest| rest.split_once(':').is_some()))
}

/// The file effects of keeping data; the traversal and exclusions are decided by the kit.
pub trait CopyFiles {
    type Entry: DirectoryEntry;
    type Entries: IntoIterator<Item = io::Result<Self::Entry>>;

    fn root_is_directory(&mut self, path: &Path) -> io::Result<bool>;
    fn create_exclusive(&mut self, path: &Path, mode: u32) -> io::Result<()>;
    fn entries(&mut self, path: &Path) -> io::Result<Self::Entries>;
    fn read_link(&mut self, path: &Path) -> io::Result<PathBuf>;
    fn link(&mut self, target: &Path, path: &Path) -> io::Result<()>;
    fn copy(&mut self, source: &Path, destination: &Path) -> io::Result<()>;
}

struct Disk;

impl CopyFiles for Disk {
    type Entry = fs::DirEntry;
    type Entries = fs::ReadDir;

    fn root_is_directory(&mut self, path: &Path) -> io::Result<bool> {
        fs::symlink_metadata(path).map(|metadata| metadata.is_dir())
    }

    fn create_exclusive(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        fs::DirBuilder::new().mode(mode).create(path)
    }

    fn entries(&mut self, path: &Path) -> io::Result<Self::Entries> {
        fs::read_dir(path)
    }

    fn read_link(&mut self, path: &Path) -> io::Result<PathBuf> {
        fs::read_link(path)
    }

    fn link(&mut self, target: &Path, path: &Path) -> io::Result<()> {
        symlink(target, path)
    }

    fn copy(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        fs::copy(source, destination).map(|_| ())
    }
}

/// Keep a private copy without the box's credential-bearing entries, never following links.
/// A partial copy may remain on error; an existing destination is never changed.
pub fn keep_box_data(data: &Path, scratch: &Path) -> Result<PathBuf, String> {
    keep_box_data_with(data, scratch, &mut Disk)
}

pub fn keep_box_data_with(data: &Path, scratch: &Path, files: &mut impl CopyFiles) -> Result<PathBuf, String> {
    let destination = scratch.join("data");
    let is_directory = files.root_is_directory(data).map_err(|error| format!("{}: {error}", data.display()))?;
    if !is_directory {
        return Err(format!("{}: box data is not a directory", data.display()));
    }
    copy_directory(data, &destination, true, files)?;
    Ok(destination)
}

fn copy_directory(source: &Path, destination: &Path, root: bool, files: &mut impl CopyFiles) -> Result<(), String> {
    files.create_exclusive(destination, 0o700).map_err(|error| format!("{}: {error}", destination.display()))?;
    let entries = files.entries(source).map_err(|error| format!("{}: {error}", source.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", source.display()))?;
        let name = entry.name();
        if name == "secrets" || name == "checkpoints" {
            continue;
        }
        if root
            && (name == "worker.secret"
                || name == "provider.key"
                || name == "programs"
                || name.as_encoded_bytes().starts_with(b"forge-"))
        {
            continue;
        }
        let from = source.join(&name);
        let to = destination.join(name);
        let failed = |error: io::Error| format!("{}: {error}", from.display());
        let kind = entry.kind().map_err(failed)?;
        match kind {
            FileKind::Symlink => {
                let target = files.read_link(&from).map_err(failed)?;
                files.link(&target, &to).map_err(failed)?;
            }
            FileKind::Directory => copy_directory(&from, &to, false, files)?,
            FileKind::File => files.copy(&from, &to).map_err(failed)?,
            FileKind::Other => return Err(format!("{}: unsupported box data entry", from.display())),
        }
    }
    Ok(())
}
