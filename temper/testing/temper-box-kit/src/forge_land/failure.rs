//! Failure evidence for the owned-forge sitting (testing/live-tier.md).
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::fs::{DirBuilderExt, symlink},
    path::{Path, PathBuf},
};

/// Failed and refused run records, oldest plain-text rotation first, redacted for notices.
#[must_use]
pub fn run_failure_lines(data: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(data.join("box/logs")) else { return Vec::new() };
    let mut logs = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let age = if name == "serve.log" {
            0
        } else if let Some(age) = name.strip_prefix("serve.log.").and_then(|age| age.parse::<u64>().ok()) {
            age
        } else {
            // Compressed and unrecognized siblings aren't plain-text serve logs.
            continue;
        };
        logs.push((age, entry.path()));
    }
    logs.sort_by_key(|(age, _)| std::cmp::Reverse(*age));
    let mut lines = Vec::new();
    for (_, path) in logs {
        let Ok(file) = fs::File::open(path) else { continue };
        let mut reader = BufReader::new(file);
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

/// Keep a private copy without the box's credential-bearing entries, never following links.
/// A partial copy may remain on error; an existing destination is never changed.
pub fn keep_box_data(data: &Path, scratch: &Path) -> Result<PathBuf, String> {
    let destination = scratch.join("data");
    let metadata = fs::symlink_metadata(data).map_err(|error| format!("{}: {error}", data.display()))?;
    if !metadata.is_dir() {
        return Err(format!("{}: box data is not a directory", data.display()));
    }
    copy_directory(data, &destination, true)?;
    Ok(destination)
}

fn copy_directory(source: &Path, destination: &Path, root: bool) -> Result<(), String> {
    fs::DirBuilder::new()
        .mode(0o700)
        .create(destination)
        .map_err(|error| format!("{}: {error}", destination.display()))?;
    let entries = fs::read_dir(source).map_err(|error| format!("{}: {error}", source.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", source.display()))?;
        let name = entry.file_name();
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
        let from = entry.path();
        let to = destination.join(name);
        let kind = entry.file_type().map_err(|error| format!("{}: {error}", from.display()))?;
        let result = if kind.is_symlink() {
            fs::read_link(&from).and_then(|target| symlink(target, &to))
        } else if kind.is_dir() {
            copy_directory(&from, &to, false)?;
            continue;
        } else if kind.is_file() {
            fs::copy(&from, &to).map(|_| ())
        } else {
            return Err(format!("{}: unsupported box data entry", from.display()));
        };
        result.map_err(|error| format!("{}: {error}", from.display()))?;
    }
    Ok(())
}
