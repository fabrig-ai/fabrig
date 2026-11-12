//! The loop's world (qa/landings.md, 6; qa/triage.md, 5; testing/tools.md, 4.3).
use crate::{Exit, Refusal, refused};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::Duration,
};

pub type Environment = Vec<(Box<[u8]>, Box<[u8]>)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    pub name: String,
    pub parents: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub directory: bool,
}

pub trait GitFace {
    fn fetch(&mut self, remote: &str, branch: &str) -> Result<(), Refusal>;
    fn head(&mut self, reference: &str) -> Result<String, Refusal>;
    fn commits(&mut self, old: &str, new: &str) -> Result<Vec<Commit>, Refusal>;
    fn message(&mut self, commit: &str) -> Result<String, Refusal>;
    fn contains(&mut self, ancestor: &str, descendant: &str) -> Result<bool, Refusal>;
    fn count(&mut self, commit: &str) -> Result<u64, Refusal>;
    fn entries(&mut self, commit: &str, directory: &str) -> Result<Vec<Entry>, Refusal>;
    fn file(&mut self, commit: &str, path: &str) -> Result<Option<Vec<u8>>, Refusal>;
    fn diff(&mut self, parent: &str, commit: &str) -> Result<Vec<u8>, Refusal>;
    fn stat(&mut self, parent: &str, commit: &str) -> Result<String, Refusal>;
    fn changed_paths(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal>;
    fn qa_trailers(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal>;
    fn add_tree(&mut self, path: &Path, commit: &str) -> Result<(), Refusal>;
    fn remove_tree(&mut self, path: &Path) -> Result<(), Refusal>;
    fn prune(&mut self) -> Result<(), Refusal>;
}

/// A rev-list line is a commit followed by all of its parents, in git's order.
pub fn parse_commits(text: &str) -> Result<Vec<Commit>, Refusal> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next().ok_or_else(|| refused("git rev-list: no commit"))?;
            let parents: Vec<String> = words.map(str::to_owned).collect();
            if std::iter::once(name)
                .chain(parents.iter().map(String::as_str))
                .any(|word| ![40, 64].contains(&word.len()) || !word.bytes().all(|byte| byte.is_ascii_hexdigit()))
            {
                return Err(refused("git rev-list: invalid commit"));
            }
            Ok(Commit { name: name.into(), parents })
        })
        .collect()
}

/// Only the final paragraph made entirely of Key: value lines is a trailer block.
#[must_use]
pub fn trailers(message: &str) -> Vec<(String, String)> {
    let mut paragraph: Vec<_> = message
        .lines()
        .rev()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| !line.trim().is_empty())
        .collect();
    paragraph.reverse();
    let mut result = Vec::new();
    for line in paragraph {
        let Some((key, value)) = line.split_once(':') else { return Vec::new() };
        if key.is_empty() || !key.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') {
            return Vec::new();
        }
        result.push((key.into(), value.trim().into()));
    }
    result
}

pub struct RealGit {
    pub directory: PathBuf,
    pub environment: Environment,
}
impl RealGit {
    fn output(&self, arguments: &[OsString]) -> Result<Output, Refusal> {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.directory).args(arguments).stdin(Stdio::null());
        captured(&mut command, &self.environment);
        command.output().map_err(|why| refused(format!("git: {why}")))
    }
    fn bytes(&self, arguments: &[&str]) -> Result<Vec<u8>, Refusal> {
        let output = self.output(&arguments.iter().map(OsString::from).collect::<Vec<_>>())?;
        checked(output, "git")
    }
    fn text(&self, arguments: &[&str]) -> Result<String, Refusal> {
        String::from_utf8(self.bytes(arguments)?).map_err(|why| refused(format!("git: {why}")))
    }
    fn worktree(&self, arguments: &[&str], path: &Path, tail: &[&str]) -> Result<(), Refusal> {
        let mut args: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        args.push(path.as_os_str().into());
        args.extend(tail.iter().map(OsString::from));
        checked(self.output(&args)?, "git worktree")?;
        Ok(())
    }
}
impl GitFace for RealGit {
    fn fetch(&mut self, remote: &str, branch: &str) -> Result<(), Refusal> {
        self.bytes(&["fetch", "--quiet", remote, &format!("+refs/heads/{branch}:refs/temper-qa/{branch}")])?;
        Ok(())
    }
    fn head(&mut self, reference: &str) -> Result<String, Refusal> {
        Ok(self.text(&["rev-parse", "--verify", &format!("{reference}^{{commit}}")])?.trim().into())
    }
    fn commits(&mut self, old: &str, new: &str) -> Result<Vec<Commit>, Refusal> {
        parse_commits(&self.text(&["rev-list", "--topo-order", "--reverse", "--parents", new, &format!("^{old}")])?)
    }
    fn message(&mut self, commit: &str) -> Result<String, Refusal> {
        self.text(&["show", "-s", "--format=%B", commit])
    }
    fn contains(&mut self, ancestor: &str, descendant: &str) -> Result<bool, Refusal> {
        let output = self.output(&["merge-base", "--is-ancestor", ancestor, descendant].map(OsString::from))?;
        if output.status.success() {
            return Ok(true);
        }
        if output.status.code() == Some(1) {
            return Ok(false);
        }
        checked(output, "git merge-base")?;
        Ok(false)
    }
    fn count(&mut self, commit: &str) -> Result<u64, Refusal> {
        self.text(&["rev-list", "--count", commit])?.trim().parse().map_err(|why| refused(format!("git count: {why}")))
    }
    fn entries(&mut self, commit: &str, directory: &str) -> Result<Vec<Entry>, Refusal> {
        let prefix = format!("{directory}/");
        let text = self.text(&["ls-tree", "-z", commit, "--", &prefix])?;
        text.split('\0')
            .filter(|row| !row.is_empty())
            .map(|row| {
                let (header, path) = row.split_once('\t').ok_or_else(|| refused("git ls-tree: no path"))?;
                Ok(Entry {
                    name: path.strip_prefix(&prefix).unwrap_or(path).into(),
                    directory: header.split_whitespace().nth(1) == Some("tree"),
                })
            })
            .collect()
    }
    fn file(&mut self, commit: &str, path: &str) -> Result<Option<Vec<u8>>, Refusal> {
        if self.bytes(&["ls-tree", "-z", commit, "--", path])?.is_empty() {
            return Ok(None);
        }
        self.bytes(&["show", &format!("{commit}:{path}")]).map(Some)
    }
    fn diff(&mut self, parent: &str, commit: &str) -> Result<Vec<u8>, Refusal> {
        self.bytes(&["diff", parent, commit, "--"])
    }
    fn stat(&mut self, parent: &str, commit: &str) -> Result<String, Refusal> {
        self.text(&["diff", "--stat", parent, commit, "--"])
    }
    fn changed_paths(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        Ok(self
            .text(&["diff", "--name-only", "-z", parent, commit, "--"])?
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(str::to_owned)
            .collect())
    }
    fn qa_trailers(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        let text = self.text(&["log", "--reverse", "--format=%B%x00", &format!("{parent}..{commit}"), "--"])?;
        let mut values = Vec::new();
        for message in text.split('\0') {
            for (key, value) in trailers(message) {
                if key == "QA" && !values.contains(&value) {
                    values.push(value);
                }
            }
        }
        Ok(values)
    }
    fn add_tree(&mut self, path: &Path, commit: &str) -> Result<(), Refusal> {
        self.worktree(&["worktree", "add", "--detach"], path, &[commit])
    }
    fn remove_tree(&mut self, path: &Path) -> Result<(), Refusal> {
        if !path.exists() {
            return Ok(());
        }
        self.worktree(&["worktree", "remove", "--force"], path, &[])
    }
    fn prune(&mut self) -> Result<(), Refusal> {
        self.bytes(&["worktree", "prune"])?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ProcessCommand {
    pub arguments: Vec<String>,
    pub directory: PathBuf,
    pub stdin: PathBuf,
    pub output: PathBuf,
    pub environment: Environment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Term,
    Kill,
}
/// A bounded deployment command's answer, separated from its diagnostic stderr.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandAnswer {
    pub exit: Exit,
    pub last_stdout: Option<String>,
}
pub trait Processes {
    /// Run to exit with closed stdin, keeping both streams at `command.output`.
    fn run(&mut self, command: &ProcessCommand, seconds: u64) -> Result<CommandAnswer, Refusal>;
    fn start(&mut self, command: &ProcessCommand) -> Result<u64, Refusal>;
    fn poll(&mut self, process: u64) -> Result<Option<Exit>, Refusal>;
    fn signal(&mut self, process: u64, signal: Signal) -> Result<(), Refusal>;
}
#[derive(Default)]
pub struct RealProcesses {
    children: BTreeMap<u64, Child>,
    pub environment: Environment,
}
impl RealProcesses {
    #[must_use]
    pub fn new(environment: Environment) -> Self {
        Self { environment, ..Self::default() }
    }
}
impl Processes for RealProcesses {
    fn run(&mut self, request: &ProcessCommand, seconds: u64) -> Result<CommandAnswer, Refusal> {
        let (program, arguments) = request.arguments.split_first().ok_or_else(|| refused("command needs a program"))?;
        // Files drain both streams without pipes filling while we wait. Only stdout
        // supplies the answer; the output keeps stderr followed by stdout.
        let stdout_path = request.output.with_extension("stdout");
        let mut run = || -> Result<CommandAnswer, Refusal> {
            let stdout = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&stdout_path)
                .map_err(|why| refused(format!("stdout: {why}")))?;
            let mut output = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&request.output)
                .map_err(|why| refused(format!("output: {why}")))?;
            let mut command = Command::new(program);
            captured(&mut command, &request.environment);
            let mut child = command
                .args(arguments)
                .current_dir(&request.directory)
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(output.try_clone().map_err(|why| refused(format!("output: {why}")))?)
                .process_group(0)
                .spawn()
                .map_err(|why| refused(format!("{program}: {why}")))?;
            let clock = skein_shell::Clock::default();
            let started = clock.now().now.as_nanos();
            let exit = loop {
                if let Some(status) = child.try_wait().map_err(|why| refused(format!("{program}: {why}")))? {
                    break status.code().map_or_else(|| Exit::Signal(status.signal().unwrap_or(0)), Exit::Code);
                }
                if clock.now().now.as_nanos().saturating_sub(started) >= seconds.saturating_mul(1_000_000_000) {
                    self.signal(u64::from(child.id()), Signal::Kill)?;
                    child.wait().map_err(|why| refused(format!("{program}: {why}")))?;
                    break Exit::Bound;
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            let mut stdout = BufReader::new(File::open(&stdout_path).map_err(|why| refused(format!("stdout: {why}")))?);
            let mut last = None;
            let mut line = Vec::new();
            loop {
                line.clear();
                if stdout.read_until(b'\n', &mut line).map_err(|why| refused(format!("stdout: {why}")))? == 0 {
                    break;
                }
                output.write_all(&line).map_err(|why| refused(format!("output: {why}")))?;
                if !String::from_utf8_lossy(&line).trim().is_empty() {
                    last = Some(line.clone());
                }
            }
            let last_stdout = last.map(|line| String::from_utf8_lossy(&line).trim_end_matches(['\r', '\n']).to_owned());
            Ok(CommandAnswer { exit, last_stdout })
        };
        let result = run();
        let cleanup = fs::remove_file(&stdout_path);
        match cleanup {
            Ok(()) => (),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
            Err(why) => return Err(refused(format!("stdout: {why}"))),
        }
        result
    }
    fn start(&mut self, request: &ProcessCommand) -> Result<u64, Refusal> {
        let (program, arguments) = request.arguments.split_first().ok_or_else(|| refused("command needs a program"))?;
        let input = File::open(&request.stdin).map_err(|why| refused(format!("stdin: {why}")))?;
        let output = File::create(&request.output).map_err(|why| refused(format!("output: {why}")))?;
        let error = output.try_clone().map_err(|why| refused(format!("output: {why}")))?;
        let mut command = Command::new(program);
        captured(&mut command, &request.environment);
        let child = command
            .args(arguments)
            .current_dir(&request.directory)
            .stdin(input)
            .stdout(output)
            .stderr(error)
            .process_group(0)
            .spawn()
            .map_err(|why| refused(format!("{program}: {why}")))?;
        let number = u64::from(child.id());
        self.children.insert(number, child);
        Ok(number)
    }
    fn poll(&mut self, process: u64) -> Result<Option<Exit>, Refusal> {
        let child = self.children.get_mut(&process).ok_or_else(|| refused(format!("no process {process}")))?;
        let Some(status) = child.try_wait().map_err(|why| refused(format!("process {process}: {why}")))? else {
            return Ok(None);
        };
        self.children.remove(&process);
        Ok(Some(status.code().map_or_else(|| Exit::Signal(status.signal().unwrap_or(0)), Exit::Code)))
    }
    fn signal(&mut self, process: u64, signal: Signal) -> Result<(), Refusal> {
        let mut command = Command::new("kill");
        captured(&mut command, &self.environment);
        // A group can already be gone; the command's spawn failure is still an error.
        command
            .args([
                "-s",
                match signal {
                    Signal::Term => "TERM",
                    Signal::Kill => "KILL",
                },
                "--",
                &format!("-{process}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|why| refused(format!("kill: {why}")))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moment {
    pub wall: u64,
    pub monotonic: u64,
}
pub trait ClockFace {
    fn now(&self) -> Moment;
    fn sleep(&mut self, seconds: u64);
}
#[derive(Default)]
pub struct RealClock(skein_shell::Clock);
impl ClockFace for RealClock {
    fn now(&self) -> Moment {
        let now = self.0.now();
        Moment { wall: now.wall.as_secs(), monotonic: now.now.as_nanos() / 1_000_000_000 }
    }
    fn sleep(&mut self, seconds: u64) {
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }
}

fn captured(command: &mut Command, environment: &Environment) {
    command.env_clear().envs(environment.iter().map(|(key, value)| (OsStr::from_bytes(key), OsStr::from_bytes(value))));
}
fn checked(output: Output, name: &str) -> Result<Vec<u8>, Refusal> {
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(refused(format!("{name}: {}: {}", output.status, String::from_utf8_lossy(&output.stderr).trim())))
    }
}
