//! Attempt preparation keeps private paths and an exact environment, no login.
//! `prepare` verifies and copies the frozen seed before inference;
//! `environment` gives commands only machine tools, private homes and explicit
//! task variables (../../docs/design/benchmarks/agents.md, section 1.1, and ../../docs/design/benchmarks/isolation.md, section 1). Setup and measurement
//! belong to the runner; this module reads no clock or agent configuration.

use crate::{Fault, FaultKind, Input, input, io, refuse_if};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use crate::files::{Files, Kind, Native};
use crate::guard::Guard;

/// One user-owned temporary entry, without following links or reading contents.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TemporaryEntry {
    kind: fs::FileType,
    bytes: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
}

/// List the user's entries directly under the shared directory, outside this attempt's private root.
pub(crate) fn temporary_entries(shared: &Path, private: &Path) -> Result<BTreeMap<PathBuf, TemporaryEntry>, Fault> {
    let owner = fs::metadata("/proc/self")?.uid();
    let mut entries = BTreeMap::new();
    for entry in fs::read_dir(shared)? {
        let path = entry?.path();
        if path.starts_with(private) {
            continue;
        }
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if metadata.uid() == owner {
            entries.insert(
                path,
                TemporaryEntry {
                    kind: metadata.file_type(),
                    bytes: metadata.len(),
                    modified_seconds: metadata.mtime(),
                    modified_nanoseconds: metadata.mtime_nsec(),
                },
            );
        }
    }
    Ok(entries)
}

/// Retain the path and both metadata records for each new or changed user entry.
pub(crate) fn temporary_changes(
    before: Result<BTreeMap<PathBuf, TemporaryEntry>, Fault>,
    shared: &Path,
    private: &Path,
) -> Result<Vec<(PathBuf, String)>, Fault> {
    let before = before?;
    let after = temporary_entries(shared, private)?;
    Ok(after
        .into_iter()
        .filter(|(path, entry)| before.get(path) != Some(entry))
        .map(|(path, entry)| {
            let evidence = format!("{}: before {:?}, after {entry:?}", path.display(), before.get(&path));
            (path, evidence)
        })
        .collect())
}

/// The preparation step that refused an attempt before inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStep {
    /// The fresh private scratch paths could not be made.
    Scratch,
    /// The frozen seed could not be validated or materialised.
    Seed,
    /// The author's warm-up or private cache copy failed.
    Warm,
    /// Available disk space could not satisfy the attempt's estimate.
    Space,
    /// The pinned invocation or its exact environment was refused.
    Configuration,
    /// The adapter found an absent or lapsed existing login.
    Credentials,
}

/// The machine's tool paths, supplied once to attempt preparation.
#[derive(Clone, Debug)]
pub struct Machine {
    pub path: OsString,
    pub rustup_home: Option<PathBuf>,
}

impl Machine {
    /// Read only the allow-listed directory choices, never an inherited login.
    pub fn from_environment() -> Result<Self, Fault> {
        let path =
            std::env::var_os("PATH").filter(|value| !value.is_empty()).ok_or_else(|| input("PATH is required"))?;
        let rustup_home = std::env::var_os("RUSTUP_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".rustup")));
        Ok(Self { path, rustup_home })
    }
}

/// Resolve an author's command through the exact machine PATH before spawn.
/// Skein executes the supplied path directly; no inherited PATH is consulted.
pub fn program(machine: &Machine, working_directory: &Path, name: &Path) -> Result<PathBuf, Fault> {
    program_with(&Native, machine, working_directory, name)
}

pub(crate) fn program_with(
    files: &dyn Files,
    machine: &Machine,
    working_directory: &Path,
    name: &Path,
) -> Result<PathBuf, Fault> {
    let candidates = if name.components().count() > 1 || name.is_absolute() {
        vec![working_directory.join(name)]
    } else {
        std::env::split_paths(&machine.path).map(|directory| working_directory.join(directory).join(name)).collect()
    };
    for path in candidates {
        if files.metadata(&path).is_ok_and(|facts| facts.kind == Kind::File) {
            // Tool proxies such as rustup choose their command from argv[0].
            return Ok(path);
        }
    }
    Err(input(format!("command not found in the exact PATH: {}", name.display())))
}

/// The writable paths supplied to one agent and its commands.
#[derive(Clone, Debug)]
pub struct Private {
    pub root: PathBuf,
    pub workspace: PathBuf,
    pub home: PathBuf,
    pub temporary: PathBuf,
    pub cargo_home: PathBuf,
    pub target: PathBuf,
    pub warm_cargo: PathBuf,
    pub warm_target: PathBuf,
    pub config: PathBuf,
    pub state: PathBuf,
}

/// Resolve an authored seed-relative working directory before credentials.
pub fn working_directory(guard: &Guard, private: &Private, task: &crate::Task) -> Result<PathBuf, Fault> {
    working_directory_with(&Native, guard, private, task)
}

/// Resolve an authored working directory against the supplied filesystem in a step test.
pub fn working_directory_with(
    files: &dyn Files,
    guard: &Guard,
    private: &Private,
    task: &crate::Task,
) -> Result<PathBuf, Fault> {
    let Some(path) = &task.working_directory else {
        return Ok(private.workspace.clone());
    };
    if path.as_os_str().is_empty() || path.components().any(|part| !matches!(part, Component::Normal(_))) {
        return Err(input("task working_directory must be a seed-relative path without traversal"));
    }
    let workspace = files.canonicalize(&private.workspace)?;
    let directory = files.canonicalize(&guard.write_path_with(files, &workspace.join(path))?)?;
    if !files.metadata(&directory).is_ok_and(|facts| facts.kind == Kind::Directory)
        || !directory.starts_with(&workspace)
    {
        return Err(input("task working_directory must be an existing directory inside the private seed"));
    }
    Ok(directory)
}

/// Delete settled owned scratch, restoring read-only directory modes without following links.
pub fn remove_owned_tree(guard: &Guard, root: &Path) -> Result<(), Fault> {
    remove_owned_tree_with(&Native, guard, root)
}

/// Restore owner access and remove a settled owned tree on the supplied filesystem.
pub fn remove_owned_tree_with(files: &dyn Files, guard: &Guard, root: &Path) -> Result<(), Fault> {
    fn writable(files: &dyn Files, guard: &Guard, directory: &Path) -> Result<(), Fault> {
        let metadata = files.symlink_metadata(directory)?;
        if metadata.kind != Kind::Directory {
            return Err(input("owned scratch root must be an ordinary directory"));
        }
        guard.write_path_with(files, directory)?;
        files.set_permissions(directory, metadata.mode | 0o700)?;
        for entry in files.read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()? == Kind::Directory {
                writable(files, guard, &entry.path())?;
            }
        }
        Ok(())
    }
    writable(files, guard, root)?;
    guard.write_path_with(files, root)?;
    Ok(files.remove_dir_all(root)?)
}

impl Private {
    pub(crate) fn paths(root: &Path) -> Self {
        Self {
            root: root.into(),
            workspace: root.join("workspace"),
            home: root.join("home"),
            temporary: root.join("tmp"),
            cargo_home: root.join("cargo"),
            target: root.join("target"),
            warm_cargo: root.join("warm-cargo"),
            warm_target: root.join("warm-target"),
            config: root.join("config"),
            state: root.join("state"),
        }
    }
}

fn directory(files: &dyn Files, guard: &Guard, path: &Path) -> Result<(), Fault> {
    let destination = guard.write_path_with(files, path)?;
    files.create_dir(&destination, 0o700).map_err(|error| io(format!("private directory: {error}")))
}

fn copy_seed(files: &dyn Files, guard: &Guard, source: &Path, destination: &Path) -> Result<(), Fault> {
    let mut entries =
        files.read_dir(source)?.map(|entry| entry.map(|entry| entry.path())).collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    for path in entries {
        let name = path.file_name().expect("directory child has a name");
        if name == ".git" {
            continue;
        }
        let metadata = files.symlink_metadata(&path)?;
        let destination = guard.write_path_with(files, &destination.join(name))?;
        match metadata.kind {
            Kind::Directory => {
                directory(files, guard, &destination)?;
                copy_seed(files, guard, &path, &destination)?;
            }
            Kind::File => {
                files.copy(&path, &destination)?;
                let mode = if metadata.mode & 0o111 == 0 { 0o600 } else { 0o700 };
                files.set_permissions(&destination, mode)?;
            }
            Kind::Link | Kind::Other => return Err(input("seed contains a link or special file")),
        }
    }
    Ok(())
}

/// Materialise a fresh guarded attempt after verifying the frozen seed digest.
/// The caller supplies an existing private scratch parent outside repositories.
pub fn prepare(guard: &Guard, seed: &Path, sha256: &str, root: &Path) -> Result<Private, Fault> {
    prepare_with(&Native, guard, seed, sha256, root)
}

pub(crate) fn prepare_with(
    files: &dyn Files,
    guard: &Guard,
    seed: &Path,
    sha256: &str,
    root: &Path,
) -> Result<Private, Fault> {
    verify_seed_digest(crate::seed_digest_with(files, seed), sha256)?;
    let paths = empty_with(files, guard, root).map_err(Fault::at(SetupStep::Scratch))?;
    copy_seed(files, guard, seed, &paths.workspace).map_err(Fault::at(SetupStep::Seed))?;
    if crate::seed_digest_with(files, &paths.workspace).map_err(Fault::at(SetupStep::Seed))? != sha256 {
        return Err(Fault::new(FaultKind::Setup(SetupStep::Seed), "materialised seed digest differs; setup refused"));
    }
    Ok(paths)
}

pub(crate) fn verify_seed_digest(digest: Result<String, crate::Refusal>, expected: &str) -> Result<(), Fault> {
    if digest.map_err(Fault::at(SetupStep::Seed))? != expected {
        return Err(Fault::new(
            FaultKind::Setup(SetupStep::Seed),
            "seed digest differs; setup refused before materialisation",
        ));
    }
    Ok(())
}

/// Create fresh private command paths with an initially empty workspace.
pub fn empty(guard: &Guard, root: &Path) -> Result<Private, Fault> {
    empty_with(&Native, guard, root)
}

fn empty_with(files: &dyn Files, guard: &Guard, root: &Path) -> Result<Private, Fault> {
    let paths = Private::paths(&guard.write_path_with(files, root)?);
    directory(files, guard, &paths.root)?;
    for path in [
        &paths.workspace,
        &paths.home,
        &paths.temporary,
        &paths.cargo_home,
        &paths.target,
        &paths.warm_cargo,
        &paths.warm_target,
        &paths.config,
        &paths.state,
    ] {
        directory(files, guard, path)?;
    }
    Ok(paths)
}

/// Populate a caller-created private workspace without following links.
pub fn copy_into(guard: &Guard, source: &Path, destination: &Path) -> Result<(), Fault> {
    guard.write_path(destination)?;
    refuse_if(fs::read_dir(destination)?.next().is_some(), Input, "copy destination must be empty")?;
    copy_seed(&Native, guard, source, destination)
}

/// Restore the machine PATH when a login shell reads this private home.
/// No inherited profile or instruction file is copied (../../docs/design/benchmarks/agents.md, section 1.1).
pub(crate) fn login_profile(guard: &Guard, machine: &Machine, private: &Private) -> Result<(), Fault> {
    let path = machine.path.as_bytes();
    refuse_if(path.contains(&0), Input, "machine PATH contains a NUL")?;
    let mut contents = b"export PATH='".to_vec();
    for byte in path {
        if *byte == b'\'' {
            contents.extend_from_slice(b"'\\''");
        } else {
            contents.push(*byte);
        }
    }
    contents.extend_from_slice(b"'\n");
    let destination = guard.write_path(&private.home.join(".profile"))?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)
        .and_then(|mut file| file.write_all(&contents))
        .map_err(|error| io(format!("private login profile: {error}")))
}

/// Construct an exact allow-listed environment, refusing private-path overrides.
pub fn environment(
    machine: &Machine,
    private: &Private,
    task: &BTreeMap<String, String>,
) -> Result<Vec<(OsString, OsString)>, Fault> {
    let mut values = BTreeMap::new();
    values.insert(OsString::from("PATH"), machine.path.clone());
    values.insert("LANG".into(), "C.UTF-8".into());
    values.insert("LC_ALL".into(), "C.UTF-8".into());
    for (name, path) in [
        ("HOME", &private.home),
        ("TMPDIR", &private.temporary),
        ("CARGO_HOME", &private.cargo_home),
        ("CARGO_TARGET_DIR", &private.target),
        ("XDG_CONFIG_HOME", &private.config),
        ("XDG_STATE_HOME", &private.state),
    ] {
        values.insert(name.into(), path.as_os_str().into());
    }
    if let Some(path) = &machine.rustup_home {
        values.insert("RUSTUP_HOME".into(), path.as_os_str().into());
    }
    for (name, value) in task {
        if name.is_empty() || name.contains(['=', '\0']) || value.contains('\0') {
            return Err(input("task environment names/values must be valid process entries"));
        }
        if values.contains_key(&OsString::from(name))
            || matches!(name.as_str(), "CODEX_HOME" | "CLAUDE_CONFIG_DIR" | "RUSTUP_HOME")
            || name.starts_with("GIT_CONFIG_")
        {
            return Err(input(format!("task variable {name} would override harness isolation")));
        }
        values.insert(name.into(), value.into());
    }
    Ok(values.into_iter().collect())
}

/// Copy a settled directory into a fresh guarded destination, without links.
/// Grading and cache copies are private writable files, never hard links.
pub fn copy_directory(guard: &Guard, source: &Path, destination: &Path) -> Result<(), Fault> {
    copy_directory_with(&Native, guard, source, destination)
}

pub(crate) fn copy_directory_with(
    files: &dyn Files,
    guard: &Guard,
    source: &Path,
    destination: &Path,
) -> Result<(), Fault> {
    directory(files, guard, destination)?;
    copy_seed(files, guard, source, destination)
}

/// Create a new private archive or scratch parent behind the common guard.
pub fn create_directory(guard: &Guard, path: &Path) -> Result<PathBuf, Fault> {
    create_directory_with(&Native, guard, path)
}

/// Create a guarded private directory on the supplied filesystem.
pub fn create_directory_with(files: &dyn Files, guard: &Guard, path: &Path) -> Result<PathBuf, Fault> {
    let path = guard.write_path_with(files, path)?;
    directory(files, guard, &path)?;
    Ok(path)
}

/// Make a settled writable copy private, rejecting links before changing modes.
pub fn restrict_tree(guard: &Guard, path: &Path) -> Result<(), Fault> {
    let path = guard.write_path(path)?;
    let metadata = fs::symlink_metadata(&path)?;
    let mode = if metadata.is_symlink() {
        return Err(input("private copy contains a link"));
    } else if metadata.is_dir() {
        0o700
    } else if metadata.is_file() {
        if metadata.permissions().mode() & 0o111 == 0 { 0o600 } else { 0o700 }
    } else {
        return Err(input("private copy contains a special node"));
    };
    fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
    if metadata.is_dir() {
        for child in fs::read_dir(&path)? {
            restrict_tree(guard, &child?.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_keeps_the_name_of_its_tool_proxy() {
        let scratch = skein_scratch::Scratch::new("bench-isolate");
        let root = scratch.path().to_path_buf().join(format!("smith-bench-proxy-{}", std::process::id()));
        fs::create_dir(&root).expect("fresh proxy directory");
        std::os::unix::fs::symlink("/bin/sh", root.join("cargo")).expect("tool proxy");
        let machine = Machine { path: root.as_os_str().into(), rustup_home: None };
        assert_eq!(program(&machine, &root, Path::new("cargo")).expect("proxy"), root.join("cargo"));
    }

    #[test]
    fn exact_environment_has_private_caches_and_only_explicit_task_variables() {
        let machine = Machine { path: "/usr/bin:/bin".into(), rustup_home: Some("/tools/rustup".into()) };
        let private = Private::paths(Path::new("/attempt"));
        let task = BTreeMap::from([("BENCH_MARKER".into(), "exact".into())]);
        let values: BTreeMap<_, _> = environment(&machine, &private, &task).expect("allow-list").into_iter().collect();
        assert_eq!(values[&OsString::from("HOME")], OsString::from("/attempt/home"));
        assert_eq!(values[&OsString::from("CARGO_TARGET_DIR")], OsString::from("/attempt/target"));
        assert_eq!(values[&OsString::from("BENCH_MARKER")], OsString::from("exact"));
        assert!(!values.contains_key(&OsString::from("CODEX_HOME")));
        for name in ["HOME", "PATH", "TMPDIR", "CODEX_HOME", "CARGO_HOME", "GIT_CONFIG_GLOBAL"] {
            assert!(environment(&machine, &private, &BTreeMap::from([(name.into(), "/outside".into())])).is_err());
        }
    }

    #[test]
    fn seed_digest_mismatch_is_refused_before_writing_any_attempt() {
        let seed = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/tasks/valid/seed");
        let scratch = skein_scratch::Scratch::new("bench-isolate");
        let fake_home = scratch.path().to_path_buf().join(format!("smith-bench-isolation-{}", std::process::id()));
        let guard = Guard::new(&fake_home, &fake_home.join("config"), &fake_home.join("state"), &[]).expect("guard");
        let attempt = fake_home.join("attempt");
        let refused = prepare(&guard, &seed, &"0".repeat(64), &attempt).expect_err("digest");
        assert_eq!(refused.kind, crate::FaultKind::Setup(SetupStep::Seed));
        assert!(refused.detail.contains("digest"));
        assert!(!fake_home.exists());
    }
}
