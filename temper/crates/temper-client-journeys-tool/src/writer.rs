//! The journeys' one writer (temper/docs/design/client/journey-writer.md, 4).
use std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
};

use crate::{
    host::{Answer, Ask, Host, Program, Ran, Stream},
    table::{BUNDLE, TABLE, WRITER},
};

mod checks;
mod print;
mod state;
#[cfg(test)]
mod tests;

#[derive(Default)]
struct Options {
    project: Box<[u8]>,
    budget: Box<[u8]>,
    allowed: bool,
    prepared: Box<[u8]>,
    branch: Box<[u8]>,
    push: Box<[u8]>,
    within: Box<[u8]>,
}

impl Options {
    fn parse(mut arguments: &[Box<[u8]>]) -> Result<Option<Self>, Stop> {
        let mut options = Self::default();
        while let Some((option, rest)) = arguments.split_first() {
            arguments = rest;
            let slot = match option.as_ref() {
                b"--project" => &mut options.project,
                b"--budget" => &mut options.budget,
                b"--prepared" => &mut options.prepared,
                b"--branch" => &mut options.branch,
                b"--push" => &mut options.push,
                b"--within" => &mut options.within,
                b"--help" => return Ok(None),
                b"--as-allowed" => {
                    if options.allowed {
                        return Err(refuse(&[b"repeated --as-allowed"]));
                    }
                    options.allowed = true;
                    continue;
                }
                _ => return Err(refuse(&[b"unknown argument: ", option])),
            };
            let Some((value, rest)) = rest.split_first() else { return Err(refuse(&[option, b" needs a value"])) };
            if value.is_empty() || value.starts_with(b"--") {
                return Err(refuse(&[option, b" needs a value"]));
            }
            if !slot.is_empty() {
                return Err(refuse(&[b"repeated ", option]));
            }
            slot.clone_from(value);
            arguments = rest;
        }
        match (options.project.is_empty(), options.budget.is_empty(), options.allowed) {
            (true, true, false) | (false, false, false) | (false, true, true) => (),
            (true, _, _) => return Err(refuse(&[b"--budget and --as-allowed need --project"])),
            (false, true, false) => return Err(refuse(&[b"needs --budget AMOUNT or --as-allowed"])),
            (false, false, true) => return Err(refuse(&[b"use --budget or --as-allowed, not both"])),
        }
        if let Some((suffix, digits)) = options.within.split_last()
            && (!matches!(suffix, b's' | b'm' | b'h') || !decimal(digits))
        {
            return Err(refuse(&[b"--within needs digits then s, m or h"]));
        }
        Ok(Some(options))
    }
}

/// A child normally says why it failed itself; only the writer's own decisions add a diagnostic.
enum Stop {
    Child,
    Refused(Vec<u8>),
    Failed(Vec<u8>),
    Host(String),
}

fn refuse(words: &[&[u8]]) -> Stop {
    Stop::Refused(words.concat())
}

fn fail(words: &[&[u8]]) -> Stop {
    Stop::Failed(words.concat())
}

pub(crate) fn run(arguments: &[Box<[u8]>], host: &mut Host, pid: u32) -> u8 {
    match start(arguments, host, pid) {
        Ok(code) => code,
        Err(Stop::Child) => 1,
        Err(Stop::Host(why)) => {
            diagnostic(host, why.as_bytes());
            1
        }
        Err(Stop::Refused(why)) => {
            diagnostic(host, &why);
            2
        }
        Err(Stop::Failed(why)) => {
            diagnostic(host, &why);
            1
        }
    }
}

fn diagnostic(host: &mut Host, words: &[u8]) {
    host.err(&[b"journey-pages: ".as_slice(), words, b"\n"].concat());
}

fn start(arguments: &[Box<[u8]>], host: &mut Host, pid: u32) -> Result<u8, Stop> {
    let Some(mut options) = Options::parse(arguments)? else {
        host.out(TABLE.command_help(&WRITER).as_bytes());
        return Ok(0);
    };
    for (name, value) in [(b"prepared".as_slice(), &options.prepared), (b"within", &options.within)] {
        if !value.is_empty() {
            host.err(
                &[b"journey-pages.sh: --".as_slice(), name, b" has no effect: temper-client-walk pages walks no box\n"]
                    .concat(),
            );
        }
    }
    let force = !options.project.is_empty() && options.branch.is_empty();
    if options.branch.is_empty() {
        options.branch = b"journey-pages".as_slice().into();
    }
    if !quiet_git(host, &[b"check-ref-format", b"--branch", &options.branch]) {
        return Err(refuse(&[b"invalid branch: ", &options.branch]));
    }
    if !options.push.is_empty() {
        if !quiet_git(host, &[b"remote", b"get-url", &options.push]) {
            return Err(refuse(&[b"not a configured remote: ", &options.push]));
        }
    } else if !options.project.is_empty() {
        options.push = b"origin".as_slice().into();
    }

    let root = checked(host, Program::new(b"git", &[b"rev-parse", b"--show-toplevel"]))
        .map_err(|_| refuse(&[b"run in the writer checkout"]))?;
    let root = path(chomp(&root));
    let offer = git_path(host, &root, b"journey-pages")?;
    let lock = git_path(host, &root, b"journey-pages.lock")?;
    let finished = git_path(host, &root, b"journey-pages.finished")?;
    let checkout = Checkout { offer: root.join(&offer), lock: root.join(&lock), finished: root.join(finished), root };
    checkout.take_lock(host, &lock)?;
    let result = match changed(host, Ask::Write(checkout.lock.join("pid"), format!("{pid}\n").into_bytes().into())) {
        Ok(()) => checkout.write_pages(host, &options, force, &offer),
        Err(stop) => Err(stop),
    };
    if let Answer::Failed(why) = host.ask(Ask::Remove(checkout.lock)) {
        diagnostic(host, why.as_bytes());
    }
    result
}

struct Checkout {
    root: PathBuf,
    offer: PathBuf,
    lock: PathBuf,
    finished: PathBuf,
}

impl Checkout {
    fn take_lock(&self, host: &mut Host, shown: &Path) -> Result<(), Stop> {
        if host.ask(Ask::Directory(self.lock.clone())) == Answer::Done {
            return Ok(());
        }
        let shown = shown.as_os_str().as_bytes();
        let Answer::Bytes(holder) = host.ask(Ask::Read(self.lock.join("pid"))) else {
            return Err(refuse(&[b"lock ", shown, b" has no process number"]));
        };
        let holder = chomp(&holder);
        let pid = if decimal(holder) {
            std::str::from_utf8(holder).ok().and_then(|text| text.parse::<u32>().ok())
        } else {
            None
        };
        let Some(pid) = pid.filter(|&pid| pid != 0 && i32::try_from(pid).is_ok()) else {
            return Err(refuse(&[b"lock ", shown, b" has invalid process number: ", holder]));
        };
        // The keeper does not probe PID 1. Never replace a lock unless its holder is known gone.
        if pid == 1 || host.ask(Ask::Alive(pid)) != Answer::Is(false) {
            return Err(refuse(&[b"lock ", shown, b" held by process ", holder]));
        }
        if host.ask(Ask::Remove(self.lock.clone())) != Answer::Done {
            return Err(refuse(&[b"cannot take over lock ", shown, b" from process ", holder]));
        }
        if host.ask(Ask::Directory(self.lock.clone())) != Answer::Done {
            return Err(refuse(&[b"lock ", shown, b" taken while replacing process ", holder]));
        }
        Ok(())
    }

    fn write_pages(&self, host: &mut Host, options: &Options, force: bool, offer: &Path) -> Result<u8, Stop> {
        let Some(run) = self.plan(host, offer)? else { return Ok(0) };
        let tip = run.tip.as_bytes();
        self.git(host, &[b"checkout", b"-B", &options.branch, tip], Stream::Err)?;

        let mut program = Program::new(b"cargo", &[b"run", b"--locked", b"-p", b"temper-client-walk", b"--", b"pages"]);
        program.environment.push((BUNDLE.name.as_bytes().into(), b"none".as_slice().into()));
        program.stdout = Stream::Out;
        let printed = child(host, self.here(program))?;
        let status = match printed.code {
            Some(0) => 0,
            Some(1) => 1,
            _ => return Err(Stop::Child),
        };
        let check_lines = self.check_requests(host, &run.tip, &printed.stdout)?;
        let changes = self.git(
            host,
            &[b"status", b"--porcelain", b"-z", b"--untracked-files=all", b"--no-renames"],
            Stream::Keep,
        )?;
        let paths = print::paths(&changes)?;
        if paths.is_empty() {
            self.finish_run(host, &run)?;
            state::current(host, tip);
            return Ok(status);
        }

        let subject = [b"Journey pages printed at ".as_slice(), &tip[..tip.len().min(12)]].concat();
        let body = print::body(&check_lines, &printed.stdout, tip);
        let commit = self.commit(host, paths, &subject, &body, tip)?;
        let commit = chomp(&commit);
        if !options.push.is_empty() {
            self.push(host, options, force, commit)?;
        }
        let mut kept = if options.project.is_empty() {
            [b"commit ".as_slice(), commit, b"\n"].concat()
        } else {
            self.open(host, options, commit, &subject, &body)?
        };
        kept.extend_from_slice(&[b"run ", tip, b" ", commit, format!(" {}\n", run.seconds).as_bytes()].concat());
        changed(host, Ask::Write(self.offer.clone(), kept.into_boxed_slice()))?;
        host.out(&[b"wrote ".as_slice(), &options.branch, b" ", commit, b"\n"].concat());
        Ok(status)
    }

    fn fetch(&self, host: &mut Host) -> Result<(), Stop> {
        self.git(host, &[b"fetch", b"origin", b"main"], Stream::Err)?;
        Ok(())
    }

    fn commit(
        &self,
        host: &mut Host,
        paths: Vec<u8>,
        subject: &[u8],
        body: &[u8],
        tip: &[u8],
    ) -> Result<Box<[u8]>, Stop> {
        // Git needs files for its NUL pathspecs and verbatim message. Both go with the lock.
        let paths_file = self.lock.join("paths");
        let message = self.lock.join("message");
        changed(host, Ask::Write(paths_file.clone(), paths.into_boxed_slice()))?;
        changed(host, Ask::Write(message.clone(), [subject, b"\n\n", body, b"\n"].concat().into_boxed_slice()))?;
        let pathspec = [b"--pathspec-from-file=".as_slice(), paths_file.as_os_str().as_bytes()].concat();
        self.git(host, &[b"--literal-pathspecs", b"add", b"-A", &pathspec, b"--pathspec-file-nul"], Stream::Err)?;
        let date = self.git(host, &[b"show", b"-s", b"--format=%cI", tip], Stream::Keep)?;
        let arguments =
            [b"-c".as_slice(), b"commit.cleanup=verbatim", b"commit", b"-F", message.as_os_str().as_bytes()];
        let mut commit = self.here(Program::new(b"git", &arguments));
        commit.stdout = Stream::Err;
        for name in [b"GIT_AUTHOR_DATE".as_slice(), b"GIT_COMMITTER_DATE"] {
            commit.environment.push((name.into(), chomp(&date).into()));
        }
        checked(host, commit)?;
        self.git(host, &[b"rev-parse", b"HEAD"], Stream::Keep)
    }

    fn push(&self, host: &mut Host, options: &Options, force: bool, commit: &[u8]) -> Result<(), Stop> {
        let remote = &options.push;
        let branch = &options.branch;
        let reference = [b"refs/heads/".as_slice(), branch].concat();
        let refspec = [b"HEAD:".as_slice(), &reference].concat();
        if force {
            self.git(host, &[b"push", b"--force", remote, &refspec], Stream::Err)?;
        } else {
            let at = self.git(host, &[b"ls-remote", remote, &reference], Stream::Keep)?;
            let mut remote_commit = Vec::new();
            for (i, line) in chomp(&at).split(|&b| b == b'\n').enumerate() {
                if i != 0 {
                    remote_commit.push(b'\n');
                }
                remote_commit.extend_from_slice(line.split(|&b| b == b'\t').next().unwrap_or_default());
            }
            let remote_commit = chomp(&remote_commit);
            if !remote_commit.is_empty() && remote_commit != commit {
                return Err(fail(&[remote, b" holds ", branch, b" at ", remote_commit]));
            }
            self.git(host, &[b"push", remote, &refspec], Stream::Err)?;
        }
        Ok(())
    }

    fn open(
        &self,
        host: &mut Host,
        options: &Options,
        commit: &[u8],
        subject: &[u8],
        body: &[u8],
    ) -> Result<Vec<u8>, Stop> {
        let mut arguments: Vec<&[u8]> = vec![
            b"ctl",
            b"change",
            b"open",
            b"--project",
            &options.project,
            b"--branch",
            &options.branch,
            b"--at",
            commit,
            b"--title",
            subject,
            b"--body",
            body,
        ];
        if options.allowed {
            arguments.push(b"--as-allowed");
        } else {
            arguments.extend_from_slice(&[b"--budget", &options.budget]);
        }
        let key = [b"journey-pages-".as_slice(), commit].concat();
        arguments.extend_from_slice(&[b"--key", &key]);
        let answer = checked(host, self.here(Program::new(b"temper", &arguments)))?;
        let task = task_number(chomp(&answer)).ok_or_else(|| fail(&[b"cannot read change open answer"]))?;
        Ok([b"task ".as_slice(), task, b"\n"].concat())
    }

    fn git(&self, host: &mut Host, words: &[&[u8]], stdout: Stream) -> Result<Box<[u8]>, Stop> {
        let mut program = self.here(Program::new(b"git", words));
        program.stdout = stdout;
        checked(host, program)
    }

    fn here(&self, mut program: Program) -> Program {
        program.directory = Some(self.root.clone());
        program
    }
}

fn git_path(host: &mut Host, root: &Path, name: &[u8]) -> Result<PathBuf, Stop> {
    let mut program = Program::new(b"git", &[b"rev-parse", b"--git-path", name]);
    program.directory = Some(root.to_owned());
    let bytes = checked(host, program)?;
    Ok(path(chomp(&bytes)))
}

fn quiet_git(host: &mut Host, words: &[&[u8]]) -> bool {
    let mut program = Program::new(b"git", words);
    program.stderr = Stream::Keep;
    // Both streams are discarded, including a failure to start the command.
    matches!(host.ask(Ask::Run(program)), Answer::Ran(Ran { code: Some(0), .. }))
}

fn child(host: &mut Host, program: Program) -> Result<Ran, Stop> {
    let stderr = program.stderr;
    match host.ask(Ask::Run(program)) {
        Answer::Ran(ran) => {
            if ran.code.is_none() && stderr == Stream::Err && !ran.stderr.is_empty() {
                host.err(&ran.stderr);
            }
            Ok(ran)
        }
        Answer::Failed(why) => Err(Stop::Host(why)),
        _ => unreachable!("host answers Run with Ran or Failed"),
    }
}

fn checked(host: &mut Host, program: Program) -> Result<Box<[u8]>, Stop> {
    let ran = child(host, program)?;
    if ran.code != Some(0) {
        return Err(Stop::Child);
    }
    Ok(ran.stdout)
}

fn changed(host: &mut Host, ask: Ask) -> Result<(), Stop> {
    match host.ask(ask) {
        Answer::Done => Ok(()),
        Answer::Failed(why) => Err(Stop::Host(why)),
        _ => unreachable!("host answers a file change with Done or Failed"),
    }
}

fn path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsStr::from_bytes(bytes))
}

fn chomp(mut bytes: &[u8]) -> &[u8] {
    while let Some(line) = bytes.strip_suffix(b"\n") {
        bytes = line;
    }
    bytes
}

fn decimal(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().all(u8::is_ascii_digit)
}

fn task_number(line: &[u8]) -> Option<&[u8]> {
    line.strip_prefix(b"task ").filter(|number| decimal(number))
}

enum Phase {
    Ended,
    Live,
}

fn phase(bytes: &[u8]) -> Option<Phase> {
    use skein_json::{Kind, document, tokenizer};
    let size = u32::try_from(bytes.len()).ok()?;
    let doc = skein_json::parse(
        bytes,
        &tokenizer::Limits { depth: 1024, string: size, number: size, chunk: 4096, length: size },
        &document::Limits { tokens: size, text: size },
    )
    .ok()?;
    let task = doc.member(0, b"task")?;
    let chip = doc.member(task, b"chip")?;
    let phase = doc.token(doc.member(chip, b"phase")?)?;
    if phase.kind != Kind::String {
        return None;
    }
    match doc.text(phase)? {
        b"ended" | b"done" | b"failed" | b"cancelled" => Some(Phase::Ended),
        b"waiting" | b"idle" | b"due" | b"preparing" | b"claimed" | b"running" | b"backing_off" | b"active"
        | b"closing" | b"held" => Some(Phase::Live),
        _ => None,
    }
}
