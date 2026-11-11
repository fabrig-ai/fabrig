//! Jobs by owner and their lifecycle (temper/docs/design/domain/worker.md, 4.4).

use alloc::boxed::Box;
use jig_domain_link::{JobBound, JobKind};
use skein_lib::bytes::copy_of;
use skein_lib::{Duration, Env, List, Map, Queue, Token, Writer};

use crate::{End, Event, Exit, Invalid, Kind, Limits, Request, Spec, valid};

/// The capability's entries and jobs in flight, keyed by the parent's owner.
#[derive(Debug)]
pub struct Domain {
    kinds: Box<[Kind]>,
    jobs: Map<Token, Job>,
}

impl Domain {
    /// Refuses kinds not ascending and distinct, or past the limits.
    ///
    /// # Errors
    /// Returns which entry or limit is invalid.
    pub fn new(limits: &Limits, kinds: Box<[Kind]>) -> Result<Domain, Invalid> {
        if !valid(limits) {
            return Err(Invalid::Limits);
        }
        if u16::try_from(kinds.len()).is_err() {
            return Err(Invalid::Kinds);
        }
        let mut previous = None;
        for kind in &kinds {
            if let Some(prior) = previous
                && prior >= kind.bound.kind
            {
                return Err(Invalid::Kinds);
            }
            previous = Some(kind.bound.kind);
            if kind.verb.is_empty() || kind.verb.contains(&0) || kind.verb.contains(&b'/') {
                return Err(Invalid::Verb);
            }
            if kind.bound.spec > limits.file_bytes {
                return Err(Invalid::Spec);
            }
            if kind.bound.result > limits.file_bytes {
                return Err(Invalid::Result);
            }
        }
        Ok(Domain { kinds, jobs: Map::with_capacity(limits.jobs) })
    }

    /// The declared verb for an assignment's kind, or closed words for an unknown kind (worker.md, 4.4).
    #[must_use]
    pub fn verb(&self, number: JobKind) -> Box<[u8]> {
        for kind in &self.kinds {
            if kind.bound.kind == number {
                return kind.verb.clone();
            }
        }
        copy_of(b"unknown")
    }

    /// The kinds' bounds, for the hello.
    #[must_use]
    pub fn bounds(&self) -> Box<[JobBound]> {
        let count = u32::try_from(self.kinds.len()).expect("the kind count fits a u16");
        let mut bounds = List::with_capacity(count);
        for kind in &self.kinds {
            bounds.push(kind.bound).expect("room for every kind's bound");
        }
        bounds.into_boxed()
    }
}

#[derive(Debug)]
struct Job {
    kind: Kind,
    directory: Box<[u8]>,
    state: State,
    tail: Box<[u8]>,
    process_gone: bool,
}

#[derive(Debug)]
enum State {
    Preparing,
    Writing { program: Box<[u8]>, deadline: Duration },
    Spawning,
    Running,
    Reading { deliver: bool, most: u32, code: u8 },
    Ending,
    Stopped,
}

struct Run {
    owner: Token,
    kind: JobKind,
    spec: Spec,
    deadline: Duration,
    directory: Box<[u8]>,
}

/// Handles one event, emitting file and process operations or root answers.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let owner = match &event {
        Event::Run { owner, .. }
        | Event::Prepared { owner, .. }
        | Event::Stop { owner }
        | Event::Written { owner, .. }
        | Event::Spawned { owner }
        | Event::Exited { owner, .. }
        | Event::Read { owner, .. }
        | Event::Gone { owner } => *owner,
    };
    match event {
        Event::Run { owner, kind, spec, deadline, directory } => {
            start(domain, env, Run { owner, kind, spec, deadline, directory }, out);
        }
        event @ (Event::Prepared { .. }
        | Event::Stop { .. }
        | Event::Written { .. }
        | Event::Spawned { .. }
        | Event::Exited { .. }
        | Event::Read { .. }
        | Event::Gone { .. }) => {
            let Some(mut job) = domain.jobs.remove(&owner) else { return };
            advance(&mut job, env, event, out);
            let ended = match job.state {
                State::Ending | State::Stopped => job.process_gone,
                State::Preparing | State::Writing { .. } | State::Spawning | State::Running | State::Reading { .. } => false,
            };
            if ended {
                out.push(Request::Gone { owner, detail: job.tail });
            } else {
                let prior = domain.jobs.insert(owner, job).expect("the removed job's slot is free");
                assert!(prior.is_none(), "the owner was removed for this step");
            }
        }
    }
}

fn start(domain: &mut Domain, env: &Env<Limits>, run: Run, out: &mut Queue<Request>) {
    let refusal = if domain.jobs.contains_key(&run.owner) {
        Some(b"owner already in flight".as_slice())
    } else if domain.jobs.len() >= env.limits.jobs {
        Some(b"jobs bound".as_slice())
    } else {
        None
    };
    if let Some(why) = refusal {
        refuse(run.owner, why, out);
        return;
    }
    let mut entry = None;
    for kind in &domain.kinds {
        if kind.bound.kind == run.kind {
            entry = Some(kind);
            break;
        }
    }
    let Some(kind) = entry else {
        refuse(run.owner, b"kind not entered", out);
        return;
    };
    let own = run.kind == JobKind(1) && run.spec.command.is_empty();
    let limits = &env.limits;
    let why = if !within(&run.spec.request, limits.file_bytes) || !within(&run.spec.request, kind.bound.spec) {
        Some(b"request bound".as_slice())
    } else if run.deadline > kind.bound.deadline {
        Some(b"deadline bound".as_slice())
    } else if !own && (!command_valid(&run.spec.command) || !within(&run.spec.command, limits.path_bytes)) {
        Some(b"command path".as_slice())
    } else if run.directory.is_empty() || run.directory.contains(&0) || !within(&run.directory, limits.path_bytes) {
        Some(b"directory path".as_slice())
    } else {
        None
    };
    if let Some(why) = why {
        refuse(run.owner, why, out);
        return;
    }
    if own {
        let job = Job { kind: kind.clone(), directory: run.directory, state: State::Preparing, tail: Box::new([]), process_gone: true };
        domain.jobs.insert(run.owner, job).expect("admitted own job");
        out.push(Request::Started { owner: run.owner });
        return;
    }
    let Some(program) = path(&run.directory, b"/", &run.spec.command, limits.path_bytes) else {
        refuse(run.owner, b"command path bound", out);
        return;
    };
    let Some(request_path) = path(&run.directory, b"/request.json", b"", limits.path_bytes) else {
        refuse(run.owner, b"request path bound", out);
        return;
    };
    if path(&run.directory, b"/result.json", b"", limits.path_bytes).is_none() {
        refuse(run.owner, b"result path bound", out);
        return;
    }
    let job = Job {
        kind: kind.clone(),
        directory: run.directory,
        state: State::Writing { program, deadline: run.deadline },
        tail: Box::new([]),
        process_gone: false,
    };
    let prior = domain.jobs.insert(run.owner, job).expect("admission checked the jobs bound");
    assert!(prior.is_none(), "admission checked the owner is free");
    out.push(Request::Write { owner: run.owner, path: request_path, bytes: run.spec.request });
}

fn advance(job: &mut Job, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Run { .. } => unreachable!("Run is handled at admission"),
        Event::Prepared { owner, result, code } => {
            if !match job.state { State::Preparing => true, State::Writing { .. } | State::Spawning | State::Running | State::Reading { .. } | State::Ending | State::Stopped => false } { return; }
            let end = if (code == 0 || code == 3) && within(&result, job.kind.bound.result) {
                End::Finished { result, deliver: code == 0 && job.kind.delivers }
            } else { End::Failed { detail: Box::from(b"own prepare result".as_slice()) } };
            job.state = State::Ending;
            out.push(Request::Ended { owner, end });
        }
        Event::Stop { owner } => {
            match job.state {
                State::Preparing | State::Writing { .. } => job.process_gone = true,
                State::Spawning | State::Running | State::Reading { .. } | State::Ending => {
                    out.push(Request::Kill { owner });
                }
                State::Stopped => return,
            }
            job.state = State::Stopped;
        }
        Event::Written { owner, written } => written_job(job, env, owner, written, out),
        Event::Spawned { owner } => match job.state {
            State::Spawning => {
                job.state = State::Running;
                out.push(Request::Started { owner });
            }
            State::Preparing | State::Writing { .. } | State::Running | State::Reading { .. } | State::Ending | State::Stopped => {}
        },
        Event::Exited { owner, exit, tail } => {
            job.tail = bounded_tail(tail, env.limits.tail_bytes);
            match job.state {
                State::Spawning | State::Running => exited_job(job, env, owner, exit, out),
                State::Preparing | State::Writing { .. } | State::Reading { .. } | State::Ending | State::Stopped => {}
            }
        }
        Event::Read { owner, bytes } => match job.state {
            State::Reading { deliver, most, code } => {
                let end = match bytes {
                    Some(result) if within(&result, most) => End::Finished { result, deliver },
                    Some(_) | None => End::Failed {
                        detail: diagnostic(
                            &exit_words(b"read result.json failed or past bound after exit ", code),
                            &job.tail,
                        ),
                    },
                };
                job.state = State::Ending;
                out.push(Request::Ended { owner, end });
            }
            State::Preparing | State::Writing { .. } | State::Spawning | State::Running | State::Ending | State::Stopped => {}
        },
        Event::Gone { owner } => {
            match job.state {
                State::Preparing | State::Writing { .. } => return,
                State::Spawning | State::Running => {
                    job.state = State::Ending;
                    out.push(Request::Ended {
                        owner,
                        end: End::Failed { detail: diagnostic(b"process gone without exit status", &job.tail) },
                    });
                }
                State::Reading { .. } | State::Ending | State::Stopped => {}
            }
            job.process_gone = true;
        }
    }
}

fn written_job(job: &mut Job, env: &Env<Limits>, owner: Token, written: bool, out: &mut Queue<Request>) {
    match &job.state {
        State::Writing { program, deadline } => {
            if !written {
                job.state = State::Ending;
                job.process_gone = true;
                job.tail = copy_of(b"write request.json failed");
                out.push(Request::Ended { owner, end: End::Failed { detail: copy_of(b"write request.json failed") } });
                return;
            }
            let request_path = path(&job.directory, b"/request.json", b"", env.limits.path_bytes)
                .expect("admission checked the request path");
            let request = Request::Spawn {
                owner,
                program: program.clone(),
                args: Box::new([program.clone(), job.kind.verb.clone(), request_path]),
                env: Box::new([
                    (copy_of(b"PATH"), env.limits.path.clone()),
                    (copy_of(b"HOME"), job.directory.clone()),
                    (copy_of(b"TEMPER_JOB"), job.directory.clone()),
                    (copy_of(b"LANG"), copy_of(b"C.UTF-8")),
                ]),
                dir: job.directory.clone(),
                deadline: env.now.saturating_add(*deadline),
            };
            job.state = State::Spawning;
            out.push(request);
        }
        State::Preparing | State::Spawning | State::Running | State::Reading { .. } | State::Ending | State::Stopped => {}
    }
}

fn exited_job(job: &mut Job, env: &Env<Limits>, owner: Token, exit: Exit, out: &mut Queue<Request>) {
    let end = match exit {
        Exit::Code(code @ (0 | 3)) => {
            let most = env.limits.file_bytes.min(job.kind.bound.result);
            let path = path(&job.directory, b"/result.json", b"", env.limits.path_bytes)
                .expect("admission checked the result path");
            job.state = State::Reading { deliver: code == 0 && job.kind.delivers, most, code };
            out.push(Request::Read { owner, path, most });
            return;
        }
        Exit::Code(code) => End::Failed { detail: diagnostic(&exit_words(b"exit ", code), &job.tail) },
        Exit::Signal => End::Failed { detail: diagnostic(b"signal", &job.tail) },
        Exit::Deadline => End::Deadline,
    };
    job.state = State::Ending;
    out.push(Request::Ended { owner, end });
}

fn exit_words(prefix: &[u8], code: u8) -> Box<[u8]> {
    let number = skein_lib::Decimal::of(u64::from(code));
    let mut words = Writer::new(prefix.len().checked_add(number.as_bytes().len()).expect("bounded exit code"));
    words.put(prefix).expect("exit prefix");
    words.put(number.as_bytes()).expect("exit code");
    words.finish()
}

fn diagnostic(why: &[u8], tail: &[u8]) -> Box<[u8]> {
    let size = why.len().checked_add(2).expect("fixed reason").checked_add(tail.len()).expect("bounded stderr tail");
    let mut text = Writer::new(size);
    for part in [why, b": ", tail] {
        text.put(part).expect("measured job diagnostic");
    }
    text.finish()
}

fn refuse(owner: Token, why: &[u8], out: &mut Queue<Request>) {
    out.push(Request::StartRefused { owner, why: copy_of(why) });
}

fn within(bytes: &[u8], most: u32) -> bool {
    bytes.len() <= usize::try_from(most).expect("u32 lengths fit usize")
}

fn path(directory: &[u8], middle: &[u8], end: &[u8], most: u32) -> Option<Box<[u8]>> {
    let len = directory.len().checked_add(middle.len())?.checked_add(end.len())?;
    if len > usize::try_from(most).expect("u32 lengths fit usize") {
        return None;
    }
    let mut writer = Writer::new(len);
    writer.put(directory).expect("the path length includes the directory");
    writer.put(middle).expect("the path length includes the separator");
    writer.put(end).expect("the path length includes the suffix");
    Some(writer.finish())
}

fn command_valid(command: &[u8]) -> bool {
    if command.is_empty() || command.first() == Some(&b'/') || command.contains(&0) {
        return false;
    }
    let mut start = 0;
    for (index, byte) in command.iter().enumerate() {
        if *byte == b'/' {
            if command.get(start..index) == Some(b"..".as_slice()) {
                return false;
            }
            start = index.checked_add(1).expect("a byte follows its index");
        }
    }
    command.get(start..) != Some(b"..".as_slice())
}

fn bounded_tail(tail: Box<[u8]>, most: u32) -> Box<[u8]> {
    let start = tail.len().saturating_sub(usize::try_from(most).expect("u32 lengths fit usize"));
    if start == 0 { tail } else { copy_of(tail.get(start..).expect("the tail begins within the bytes")) }
}
