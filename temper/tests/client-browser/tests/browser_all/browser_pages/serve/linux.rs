//! Retain a scratch serve's identity for the reading fault (temper/docs/design/client/web/testing.md, 6).
use skein_fake_person::{Find, Next, Why};
use skein_fake_person_browser::{BrowserFace, Play, Script, Setup};
use skein_io::kernel::{Error, Fd, Signal, Target};
use skein_lib::Time;
use skein_shell::keeper;
use std::{
    collections::VecDeque,
    ffi::OsString,
    fs,
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
};
use temper_client_browser_world::Tier;

pub(super) struct Reading {
    script: Script,
    switches: VecDeque<(u32, Signal)>,
    program: PathBuf,
    box_args: Vec<OsString>,
    serve_args: Vec<OsString>,
    serve: Option<Serve>,
}

impl Reading {
    pub fn new(script: Script, switches: VecDeque<(u32, Signal)>, setup: &Setup<'_>, tier: &Tier) -> Self {
        Self {
            script,
            switches,
            program: setup.server.clone(),
            box_args: setup.args.clone(),
            serve_args: tier.box_serve(),
            serve: None,
        }
    }

    fn signal(&mut self, signal: Signal) -> Result<(), String> {
        if self.serve.is_none() {
            let program = self.program.canonicalize().map_err(|error| format!("serve program: {error}"))?;
            let (pid, host) =
                child(Path::new("/proc/thread-self/children"), std::process::id(), &program, &self.box_args)?;
            let children = PathBuf::from(format!("/proc/{pid}/task/{pid}/children"));
            let (_, process) = child(&children, pid, &program, &self.serve_args)?;
            if host.exited()? {
                return Err("the reading box exited during serve discovery".into());
            }
            self.serve = Some(Serve { process, held: false });
        }
        let serve = self.serve.as_mut().expect("serve identity acquired");
        serve.process.signal(signal).map_err(|error| format!("reading serve {signal:?}: {error:?}"))?;
        serve.held = signal == Signal::Stop;
        Ok(())
    }

    fn release_on_failure(&mut self) {
        if let Some(serve) = &mut self.serve {
            serve.release();
        }
    }

    fn failed(step: u32, report: &str) -> Next<Box<[u8]>> {
        Next::Failed { step, why: Why::Unexpected { find: Find::text(report.as_bytes()) } }
    }
}

impl Play for Reading {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        let step = self.script.at();
        if let Some(&(at, signal)) = self.switches.front()
            && at == step
        {
            self.switches.pop_front();
            if let Err(report) = self.signal(signal) {
                self.release_on_failure();
                return Self::failed(step, &report);
            }
        }
        let next = self.script.next(face, now);
        let held = self.serve.as_ref().is_some_and(|serve| serve.held);
        if matches!(next, Next::Done) && (held || !self.switches.is_empty()) {
            self.release_on_failure();
            return Self::failed(step, "reading script ended with unfinished serve holds");
        }
        if held {
            if matches!(next, Next::Failed { .. }) || face.expired().is_some() {
                self.release_on_failure();
            } else if let Some(report) = face.failure() {
                // An immediate script finding still fails, never a deadline that could starve.
                self.release_on_failure();
                return Self::failed(step, &report);
            }
        }
        next
    }

    fn at(&self) -> u32 {
        self.script.at()
    }

    fn next_wake(&self, face: &BrowserFace<'_>, now: Time, runnable: bool) -> Option<Time> {
        self.script.next_wake(face, now, runnable)
    }

    fn check_deadline(&mut self, face: &mut BrowserFace<'_>, now: Time) {
        self.script.check_deadline(face, now);
        // While held the script only sends keys and queries DOM state. An
        // unanswered check or a refused browser operation ends the hold.
        if face.expired().is_some() || face.refusal().is_some() {
            self.release_on_failure();
        }
    }
}

struct Pidfd(Fd);

impl Pidfd {
    fn exited(&self) -> Result<bool, String> {
        keeper::pidfd_exited(self.0).map_err(|error| format!("reading process identity: {error:?}"))
    }

    fn signal(&self, signal: Signal) -> Result<(), Error> {
        keeper::signal_kept_child(self.0, signal, Target::Child)
    }
}

impl Drop for Pidfd {
    fn drop(&mut self) {
        keeper::close_keeper_fd(self.0);
    }
}

struct Serve {
    process: Pidfd,
    held: bool,
}

impl Serve {
    fn release(&mut self) {
        if self.held && self.process.signal(Signal::Continue).is_ok() {
            self.held = false;
        }
    }
}

impl Drop for Serve {
    fn drop(&mut self) {
        self.release();
    }
}

fn child(children: &Path, parent: u32, program: &Path, args: &[OsString]) -> Result<(u32, Pidfd), String> {
    let children = fs::read_to_string(children).map_err(|error| format!("reading children of {parent}: {error}"))?;
    let mut found = None;
    for pid in children.split_whitespace() {
        let pid = pid.parse::<u32>().map_err(|error| format!("reading child pid: {error}"))?;
        let process = match keeper::open_child_pidfd(pid, parent) {
            Ok(fd) => Pidfd(fd),
            Err(Error::NotFound | Error::Other(3)) => continue, // Child exited during discovery.
            Err(error) => return Err(format!("reading child identity: {error:?}")),
        };
        if fs::read_link(format!("/proc/{pid}/exe")).as_deref().ok() != Some(program) {
            continue;
        }
        if let Ok(command) = fs::read(format!("/proc/{pid}/cmdline"))
            && let Some(command) = command.strip_suffix(b"\0")
            && command.split(|byte| *byte == 0).skip(1).eq(args.iter().map(|arg| arg.as_bytes()))
            && !process.exited()?
            && found.replace((pid, process)).is_some()
        {
            return Err(format!("ambiguous reading child in {parent}'s tree"));
        }
    }
    found.ok_or_else(|| format!("scratch box or serve missing in {parent}'s children"))
}
