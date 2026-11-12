//! Publishing staged builds (temper/docs/design/testing/qa/builds.md, 3 and 4).
use crate::{Home, Refusal, cli::Environment, ledger::short, refused};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

pub(crate) fn keep(home: &Home, commit: &str, directory: &Path, environment: &Environment) -> Result<String, Refusal> {
    if !(commit.len() == 40 || commit.len() == 64)
        || !commit.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(refused(format!("{commit} is no commit id")));
    }
    home.transact(|ledger| {
        ledger.check_build_keep(commit)?;
        let files = check_files(directory)?;
        for path in &files {
            if path.parent() == Some(Path::new("bin")) {
                check_version(home, directory, path, commit, environment)?;
            }
        }
        io(&home.builds, fs::create_dir_all(&home.builds))?;
        let parent = directory.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
        if io(parent, fs::metadata(parent))?.dev() != io(&home.builds, fs::metadata(&home.builds))?.dev() {
            return Err(refused(format!(
                "{} is not on the filesystem of {}",
                directory.display(),
                home.builds.display()
            )));
        }
        let partial = home.builds.join(format!(".{commit}.partial"));
        match fs::symlink_metadata(&partial) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    io(&partial, fs::remove_dir_all(&partial))?;
                } else {
                    io(&partial, fs::remove_file(&partial))?;
                }
            }
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(refused(format!("{}: {why}", partial.display()))),
        }
        io(directory, fs::rename(directory, &partial))?;
        manifest(&partial, commit, &files)?;
        io(&partial, fs::rename(&partial, home.builds.join(commit)))?;
        ledger.record_build_kept(commit, skein_shell::Clock::new().now().wall.as_secs())?;
        Ok(format!("build {} kept: {} files\n", short(commit), files.len()))
    })
}

fn check_files(directory: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let mut missing = Vec::new();
    for path in ["bin/temper-box", "bundle/index.html", "bundle-record/index.html"] {
        if !fs::symlink_metadata(directory.join(path)).is_ok_and(|metadata| metadata.is_file()) {
            missing.push(format!("{} lacks {path}", directory.display()));
        }
    }
    if !missing.is_empty() {
        return Err(refused(missing.join("; ")));
    }
    let mut files = Vec::new();
    walk(directory, Path::new(""), &mut files)?;
    files.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
    files.retain(|path| path != Path::new("manifest"));
    Ok(files)
}

fn walk(directory: &Path, path: &Path, files: &mut Vec<PathBuf>) -> Result<(), Refusal> {
    let full = directory.join(path);
    let metadata = io(&full, fs::symlink_metadata(&full))?;
    if metadata.is_file() {
        files.push(path.into());
    } else if metadata.is_dir() {
        let mut children = io(&full, fs::read_dir(&full))?
            .map(|entry| entry.map(|entry| path.join(entry.file_name())))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|why| refused(format!("{}: {why}", full.display())))?;
        children.sort();
        for child in children {
            walk(directory, &child, files)?;
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
) -> Result<(), Refusal> {
    let executable = directory.join(path);
    if io(&executable, fs::metadata(&executable))?.permissions().mode() & 0o111 == 0 {
        return Err(refused(format!("{} is not executable", path.display())));
    }
    let line = version(home, &executable, environment)?;
    if !line.ends_with(&format!(" ({})", short(commit))) {
        return Err(refused(format!("{} prints {line}, not {}", path.display(), short(commit))));
    }
    Ok(())
}

// An unlinked capture file lets the one-threaded wait enforce the bound even
// when a program fills stdout or never closes it. No capture enters the build.
fn version(home: &Home, executable: &Path, environment: &Environment) -> Result<String, Refusal> {
    let capture = home.directory.join("build-version.tmp");
    let mut output =
        io(&capture, OpenOptions::new().read(true).write(true).create_new(true).mode(0o600).open(&capture))?;
    io(&capture, fs::remove_file(&capture))?;
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let mut child = io(
        executable,
        Command::new(executable)
            .arg("--version")
            .env_clear()
            .envs(environment.iter().map(|(key, value)| (OsStr::from_bytes(key), OsStr::from_bytes(value))))
            .stdin(Stdio::null())
            .stdout(io(&capture, output.try_clone())?)
            .stderr(Stdio::null())
            .spawn(),
    )?;
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {}
            Err(why) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(refused(format!("{}: {why}", executable.display())));
            }
        }
        if clock.now().now.as_nanos().saturating_sub(started.as_nanos()) >= 30_000_000_000 {
            let _ = child.kill();
            io(executable, child.wait())?;
            return Err(refused(format!("{}: --version did not answer in 30 seconds", executable.display())));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    io(&capture, output.rewind())?;
    let mut line = Vec::new();
    io(&capture, BufReader::new(output).read_until(b'\n', &mut line))?;
    Ok(String::from_utf8_lossy(&line).lines().next().unwrap_or("").into())
}

fn manifest(directory: &Path, commit: &str, files: &[PathBuf]) -> Result<(), Refusal> {
    let path = directory.join("manifest");
    let mut output = io(&path, File::create(&path))?;
    io(&path, writeln!(output, "commit {commit}"))?;
    for name in files {
        let full = directory.join(name);
        let mut file = io(&full, File::open(&full))?;
        let mut hash = Sha256::new();
        let mut buffer = [0; 16384];
        loop {
            let count = io(&full, file.read(&mut buffer))?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        let bytes = name.as_os_str().as_bytes();
        let escaped = bytes.contains(&b'\\') || bytes.contains(&b'\n') || bytes.contains(&b'\r');
        io(&path, write!(output, "{}{:x}  ", if escaped { "\\" } else { "" }, hash.finalize()))?;
        for byte in bytes {
            let escaped: &[u8] = match byte {
                b'\\' => b"\\\\",
                b'\n' => b"\\n",
                b'\r' => b"\\r",
                _ => std::slice::from_ref(byte),
            };
            io(&path, output.write_all(escaped))?;
        }
        io(&path, output.write_all(b"\n"))?;
    }
    io(&path, output.sync_all())
}

fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
