//! Agent-neutral checks and grader commands after tree settlement.
//! Reads stay inside the candidate copy; links and special files are scope
//! evidence and are never followed. No provider or agent state is inspected.
//! Contracts: ../../docs/design/benchmarks/README.md, section 3.2, and ../../docs/design/benchmarks/results.md, section 2, and ../../docs/design/benchmarks/isolation.md, section 1.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use skein_lib::Duration;
use skein_shell::Clock;
use skein_world::end_to_end::Command;

use crate::files::{Files, Kind, Native};
use crate::guard::Guard;
use crate::isolate::{Machine, Private};
use crate::process::{Bounds, Capture, execute};
use crate::{CheckVerdict, Exit, Fault, Forced, Grade, Input, Measure, OutcomeCheck, input, refuse_if};

/// Safe materialised file observations, supplied to outcome and scope checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// An ordinary directory, retained so an empty directory is still present.
    Directory,
    /// A regular file's bytes and executable permission, supplied by the walk.
    File { sha256: String, executable: bool },
    /// An unfollowed link target, supplied to the scope gate.
    Link { target: PathBuf },
    /// A special node, which the grader never opens.
    Special,
}

/// A sorted candidate view; bytes are hashed only for regular files.
pub type Snapshot = BTreeMap<PathBuf, Entry>;

/// Inspect a settled directory without following links or opening special nodes.
pub fn snapshot(root: &Path) -> Result<Snapshot, Fault> {
    snapshot_with(&Native, root)
}

/// Inspect a settled directory on the supplied filesystem, excluding git metadata.
pub fn snapshot_with(files: &dyn Files, root: &Path) -> Result<Snapshot, Fault> {
    let mut entries = BTreeMap::new();
    walk(files, root, root, &mut entries, true)?;
    Ok(entries)
}

/// Inspect every node, including cache and git metadata, without following links.
pub fn snapshot_all(root: &Path) -> Result<Snapshot, Fault> {
    snapshot_all_with(&Native, root)
}

/// Inspect every node on the supplied filesystem without following links.
pub fn snapshot_all_with(files: &dyn Files, root: &Path) -> Result<Snapshot, Fault> {
    let mut entries = BTreeMap::new();
    walk(files, root, root, &mut entries, false)?;
    Ok(entries)
}

fn walk(files: &dyn Files, root: &Path, directory: &Path, entries: &mut Snapshot, skip_git: bool) -> Result<(), Fault> {
    let mut children =
        files.read_dir(directory)?.map(|child| child.map(|child| child.path())).collect::<Result<Vec<_>, _>>()?;
    children.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    for path in children {
        if skip_git && path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        let metadata = files.symlink_metadata(&path)?;
        let entry = match metadata.kind {
            Kind::Directory => Entry::Directory,
            Kind::Link => Entry::Link { target: files.read_link(&path)? },
            Kind::File => Entry::File {
                sha256: crate::arm::binary_digest_with(files, &path)?,
                executable: metadata.mode & 0o111 != 0,
            },
            Kind::Other => Entry::Special,
        };
        entries.insert(path.strip_prefix(root).expect("walk stays under root").into(), entry);
        if metadata.kind == Kind::Directory {
            walk(files, root, &path, entries, skip_git)?;
        }
    }
    Ok(())
}

pub(crate) fn file(root: &Path, relative: &Path) -> Result<PathBuf, Fault> {
    file_with(&Native, root, relative)
}

fn file_with(files: &dyn Files, root: &Path, relative: &Path) -> Result<PathBuf, Fault> {
    if relative.is_absolute() || relative.components().any(|part| !matches!(part, Component::Normal(_))) {
        return Err(input("check path must be relative without traversal"));
    }
    let mut path = root.to_path_buf();
    for part in relative.components() {
        path.push(part);
        let metadata = files.symlink_metadata(&path)?;
        refuse_if(metadata.kind == Kind::Link, Input, "check path traverses a link")?;
    }
    refuse_if(files.metadata(&path)?.kind != Kind::File, Input, "check path is not a regular file")?;
    Ok(path)
}

/// Compare a protected path and every descendant with the frozen seed.
#[must_use]
pub fn changed(before: &Snapshot, after: &Snapshot, path: &Path) -> bool {
    before.iter().filter(|(name, _)| name.starts_with(path)).ne(after.iter().filter(|(name, _)| name.starts_with(path)))
}

mod outcomes;

/// Settled process evidence alongside the files an outcome check reads.
#[derive(Clone, Debug)]
pub struct Ended {
    pub end: crate::End,
    pub exit: Measure<Exit>,
    /// A bounded complete UTF-8 stderr line; never parsed for observations.
    pub last_line: Option<String>,
    /// The native result text or Codex's final root message.
    pub answer: Option<String>,
    /// Pair settlement is supplied by the dispatcher; an isolated attempt cannot prove it.
    pub credential_refresh: Measure<bool>,
}

/// Judge outcome bytes and protected files after the agent's tree has ended.
/// A pattern is a Rust regular expression, supplied by the task author.
#[must_use]
pub fn checks(
    workspace: &Path,
    before: &Snapshot,
    after: &Snapshot,
    checks: &[OutcomeCheck],
    ended: &Ended,
) -> Vec<CheckVerdict> {
    checks_with(&Native, workspace, before, after, checks, ended)
}

/// Judge outcomes and protected files on the supplied settled filesystem.
#[must_use]
pub fn checks_with(
    files: &dyn Files,
    workspace: &Path,
    before: &Snapshot,
    after: &Snapshot,
    checks: &[OutcomeCheck],
    ended: &Ended,
) -> Vec<CheckVerdict> {
    checks
        .iter()
        .map(|check| {
            let (name, path) = match check {
                OutcomeCheck::FileContent { path, .. } => ("file-content", path),
                OutcomeCheck::FileDigest { path, .. } => ("file-digest", path),
                OutcomeCheck::FileSize { path, .. } => ("file-size", path),
                OutcomeCheck::Absent { path } => ("absent", path),
                OutcomeCheck::Pattern { path, .. } => ("pattern", path),
                OutcomeCheck::Protected { path } => ("protected", path),
                OutcomeCheck::End { .. }
                | OutcomeCheck::ExitCode { .. }
                | OutcomeCheck::LastLine { .. }
                | OutcomeCheck::Answer { .. }
                | OutcomeCheck::CredentialRefresh => return outcomes::check(check, ended),
            };
            let value = match check {
                OutcomeCheck::FileContent { path, content } => file_with(files, workspace, path)
                    .and_then(|path| files.read(&path).map_err(Fault::from))
                    .map(|bytes| bytes == content.as_bytes()),
                OutcomeCheck::FileDigest { path, sha256 } => file_with(files, workspace, path)
                    .and_then(|path| crate::arm::binary_digest_with(files, &path))
                    .map(|got| got == *sha256),
                OutcomeCheck::FileSize { path, bytes_at_least } => file_with(files, workspace, path)
                    .and_then(|path| files.metadata(&path).map_err(Fault::from))
                    .map(|metadata| metadata.size >= *bytes_at_least),
                OutcomeCheck::Absent { path } => {
                    Ok(!after.contains_key(path) && !after.keys().any(|item| item.starts_with(path)))
                }
                OutcomeCheck::Pattern { path, pattern } => {
                    regex::Regex::new(pattern).map_err(|error| input(error.to_string())).and_then(|pattern| {
                        file_with(files, workspace, path)
                            .and_then(|path| files.read_to_string(&path).map_err(Fault::from))
                            .map(|text| pattern.is_match(&text))
                    })
                }
                OutcomeCheck::Protected { path } => Ok(before.contains_key(path) && !changed(before, after, path)),
                OutcomeCheck::End { .. }
                | OutcomeCheck::ExitCode { .. }
                | OutcomeCheck::LastLine { .. }
                | OutcomeCheck::Answer { .. }
                | OutcomeCheck::CredentialRefresh => unreachable!("neutral checks return above"),
            };
            let (passed, detail) = match value {
                Ok(passed) => (passed, Vec::new()),
                Err(error) => (false, vec![error.detail]),
            };
            CheckVerdict {
                check: format!("{name}:{}", path.display()),
                passed: Measure::Observed { value: passed },
                evidence: detail,
            }
        })
        .collect()
}

/// The author-command evidence retained beside the attempt, outside its timer.
#[derive(Debug)]
pub struct GradeResult {
    pub end: GradeEnd,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub forced: Forced,
}

/// The grader's terminal, supplied after its tree has settled.
#[derive(Debug, PartialEq, Eq)]
pub enum GradeEnd {
    /// The author's command exited successfully.
    Passed,
    /// The author's command rejected the candidate with an exit code.
    Failed,
    /// The grader crashed, exceeded its deadline or lost bounded output.
    Fault { reason: String },
}

/// The settled exit and captured streams supplied by the command runner to grading.
#[derive(Debug)]
pub struct CommandReceipt {
    pub exit: Exit,
    pub forced: Forced,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub error: Option<String>,
}

pub(crate) fn run_command(clock: &Clock, command: Command, bounds: Bounds) -> Result<CommandReceipt, Fault> {
    let result = execute(clock, command, Box::new([]), Capture::Raw, bounds)?;
    Ok(CommandReceipt {
        exit: result.exit,
        forced: result.forced,
        stdout: result.stdout,
        stderr: result.stderr,
        error: result.error,
    })
}

/// Grade a separate copy with the author's deadline and bounded logs.
pub fn command(
    clock: &Clock,
    machine: &Machine,
    private: &Private,
    directory: &Path,
    grade: &Grade,
    bounds: Bounds,
    environment: &BTreeMap<String, String>,
) -> Result<GradeResult, Fault> {
    command_with(&Native, machine, private, directory, grade, bounds, environment, &mut |command, bounds| {
        run_command(clock, command, bounds)
    })
}

#[expect(clippy::too_many_arguments, reason = "grade inputs plus the file and command boundaries")]
pub(crate) fn command_with(
    files: &dyn Files,
    machine: &Machine,
    private: &Private,
    directory: &Path,
    grade: &Grade,
    bounds: Bounds,
    environment: &BTreeMap<String, String>,
    run: &mut impl FnMut(Command, Bounds) -> Result<CommandReceipt, Fault>,
) -> Result<GradeResult, Fault> {
    let (program, arguments) = grade.command.split_first().ok_or_else(|| input("empty grading command"))?;
    let maximum = usize::try_from(grade.max_output_bytes)?;
    let command = Command {
        program: crate::isolate::program_with(files, machine, directory, Path::new(program))?,
        arguments: arguments.iter().map(Into::into).collect(),
        environment: crate::isolate::environment(machine, private, environment)?,
        directory: directory.into(),
    };
    let result = run(
        command,
        Bounds {
            hard: Duration::from_secs(grade.deadline_seconds),
            teardown: Duration::ZERO,
            grace: bounds.grace,
            settlement: bounds.settlement,
            stdout: maximum,
            stderr: maximum,
            force_walk: bounds.force_walk,
        },
    )?;
    let end = match (result.error, result.forced, result.exit) {
        (Some(reason), _, _) => GradeEnd::Fault { reason },
        (None, Forced::Terminated | Forced::Killed, _) => {
            GradeEnd::Fault { reason: "grader exceeded its deadline".into() }
        }
        (None, Forced::No, Exit::Signal(signal)) => {
            GradeEnd::Fault { reason: format!("grader crashed with signal {signal}") }
        }
        (None, Forced::No, Exit::Code(0)) => GradeEnd::Passed,
        (None, Forced::No, Exit::Code(_)) => GradeEnd::Failed,
    };
    Ok(GradeResult { end, stdout: result.stdout, stderr: result.stderr, forced: result.forced })
}

/// Archive exactly these bytes as a new private file, retaining its digest.
pub fn archive(guard: &Guard, directory: &Path, name: &str, bytes: &[u8]) -> Result<crate::Artifact, Fault> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    if Path::new(name).components().any(|part| !matches!(part, Component::Normal(_))) {
        return Err(input("archive name must be a relative ordinary path"));
    }
    let path = guard.write_path(&directory.join(name))?;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    file.write_all(bytes).and_then(|()| file.sync_all())?;
    Ok(crate::Artifact { path: name.into(), sha256: format!("{:x}", Sha256::digest(bytes)) })
}
