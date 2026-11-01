//! The routing's box link: serve's side of the box's pipe (protocol/forge.md, 2.8).

use alloc::boxed::Box;
use skein_lib::{Decimal, List, Queue, Token, Writer, bytes};

/// The box's word for the step it runs (protocol/setup.md, 7.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Fetch,
    Prepare,
    Run,
}
/// Why the link refuses an ask (protocol/forge.md, 2.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refused {
    Busy,
}
/// One outcome of a link input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Out {
    /// A line for fd 3, its line feed included.
    Write(Box<[u8]>),
    /// The ask's one answer: setting up, or its refusal.
    Answered { forge: u16, request: Token, answer: Result<(), Refused> },
    /// The forge setting up: `None` at the ask, then each `step` line's step.
    SettingUp { forge: u16, step: Option<Step> },
    /// `started FORGE`: the ask ended, the group started.
    Started { forge: u16 },
    /// `removed FORGE`: the group and its files are gone.
    Removed { forge: u16 },
    /// `failed FORGE WHY`.
    Failed { forge: u16, why: Box<[u8]> },
    /// One log line naming what the link dropped, or the pipe's end.
    Named(Box<[u8]>),
    /// The pipe ended; the root decides what it means for the forges.
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    Start(u16),
    Stop(u16),
    Remove(u16),
}

pub(crate) struct Link {
    pub(crate) write: Option<Box<[u8]>>,
    ended: bool,
    piped: bool,
    removes: Queue<u16>,
    asked: Option<Ask>,
    // which the box link holds while another group starts and sends once that
    // one answers (protocol/setup.md, 8.4).
    held: Option<u16>,
    stop: Option<u16>,
    line_bytes: u32,
    partial: List<u8>,
    skipping: bool,
}
impl Link {
    /// The routing only sends asks admitted by the domain.
    pub(crate) fn new(line_bytes: u32, piped: bool, forges: u32) -> Link {
        Link {
            write: None,
            ended: false,
            piped,
            removes: Queue::with_capacity(forges),
            asked: None,
            held: None,
            stop: None,
            line_bytes,
            partial: List::with_capacity(line_bytes),
            skipping: false,
        }
    }
    /// The caller reserves three output slots before asking.
    pub(crate) fn ask(&mut self, forge: u16, request: Token, out: &mut Queue<Out>) {
        if self.held.is_some()
            || self.asked == Some(Ask::Start(forge))
            || self.ended
            || self.asked.is_none() && self.write.is_some()
        {
            out.push(Out::Named(Box::from(b"box link: fault: a setup ask while busy or closed".as_slice())));
            out.push(Out::Answered { forge, request, answer: Err(Refused::Busy) });
            return;
        }
        if self.asked.is_some() {
            self.held = Some(forge);
        } else {
            self.asked = Some(Ask::Start(forge));
            out.push(Out::Write(start(forge)));
        }
        out.push(Out::SettingUp { forge, step: None });
        out.push(Out::Answered { forge, request, answer: Ok(()) });
    }
    /// No request waits on the demo's stop. Keep it even when a start is held.
    pub(crate) fn stop(&mut self, forge: u16, out: &mut Queue<Out>) {
        if self.ended {
            return;
        }
        if self.held == Some(forge) {
            self.held = None;
        }
        if self.asked.is_some() {
            self.stop = Some(forge);
        } else {
            self.asked = Some(Ask::Stop(forge));
            out.push(Out::Write(stop(forge)));
        }
    }
    /// A removal stays a kept act until its matching `removed` reply.
    pub(crate) fn remove(&mut self, forge: u16, out: &mut Queue<Out>) {
        if self.ended || !self.piped {
            out.push(Out::Named(joined(&[
                b"box link: dropped remove for forge ",
                Decimal::of(u64::from(forge)).as_bytes(),
                b", link ended",
            ])));
        } else if self.asked.is_some() {
            if self.removes.try_push(forge).is_err() {
                out.push(Out::Named(Box::from(b"box link: dropped remove, queue full".as_slice())));
            }
        } else {
            self.asked = Some(Ask::Remove(forge));
            out.push(Out::Write(remove(forge)));
        }
    }
    /// Any chunking is accepted. The caller reserves one output slot per LF in
    /// this chunk (including an LF completing a previous partial line), plus
    /// one for a held ask sent after the active group's answer.
    /// No output is emitted before LF, so a bounded read bounds its outputs.
    pub(crate) fn read(&mut self, input: &[u8], out: &mut Queue<Out>) {
        if self.ended {
            return;
        }
        for byte in input {
            if self.skipping || self.partial.room() == 0 {
                self.skipping = true;
                if *byte == b'\n' {
                    self.skipping = false;
                    out.push(Out::Named(joined(&[
                        b"box link: dropped a line past box.line_bytes (",
                        Decimal::of(u64::from(self.line_bytes)).as_bytes(),
                        b")",
                    ])));
                }
                continue;
            }
            // As in temper-box's Reader, the LF is inside line_bytes. The
            // partial buffer is allocated once at new, and never grows.
            self.partial.push(*byte).expect("room checked per byte");
            if *byte == b'\n' {
                line(&mut self.asked, self.partial.as_slice(), out);
                self.partial.clear();
                if self.asked.is_none() {
                    if let Some(forge) = self.stop.take() {
                        self.asked = Some(Ask::Stop(forge));
                        out.push(Out::Write(stop(forge)));
                    } else if let Some(forge) = self.held.take() {
                        self.asked = Some(Ask::Start(forge));
                        out.push(Out::Write(start(forge)));
                    } else if let Some(forge) = self.removes.pop() {
                        self.asked = Some(Ask::Remove(forge));
                        out.push(Out::Write(remove(forge)));
                    }
                }
            } else if self.partial.room() == 0 {
                self.partial.clear();
                self.skipping = true;
            }
        }
    }
    /// fd 4 ended, or a read/write failed. The caller reserves two output slots.
    pub(crate) fn closed(&mut self, out: &mut Queue<Out>) {
        if self.ended {
            return;
        }
        self.ended = true;
        self.write = None;
        self.partial.clear();
        self.skipping = false;
        out.push(Out::Named(Box::from(b"box link: the box closed the pipe".as_slice())));
        self.asked = None;
        self.held = None;
        self.stop = None;
        for _ in 0..self.removes.len() {
            let _forge = self.removes.pop();
        }
        out.push(Out::Closed);
    }
}

fn remove(forge: u16) -> Box<[u8]> {
    joined(&[b"remove ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])
}

fn stop(forge: u16) -> Box<[u8]> {
    joined(&[b"stop ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])
}

fn start(forge: u16) -> Box<[u8]> {
    joined(&[b"start ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])
}

enum Reply {
    Step(u16, Step),
    Started(u16),
    Stopped(u16),
    Removed(u16),
    Failed(u16, Box<[u8]>),
}
impl Reply {
    fn forge(&self) -> u16 {
        match self {
            Self::Step(forge, _)
            | Self::Started(forge)
            | Self::Stopped(forge)
            | Self::Removed(forge)
            | Self::Failed(forge, _) => *forge,
        }
    }
    fn word(&self) -> &'static [u8] {
        match self {
            Self::Step(..) => b"step",
            Self::Started(_) => b"started",
            Self::Stopped(_) => b"stopped",
            Self::Removed(_) => b"removed",
            Self::Failed(..) => b"failed",
        }
    }
}
fn line(asked: &mut Option<Ask>, line: &[u8], out: &mut Queue<Out>) {
    let Some(reply) = decoded(line) else {
        out.push(Out::Named(Box::from(b"box link: dropped an unknown line".as_slice())));
        return;
    };
    let forge = reply.forge();
    let matches = match (*asked, &reply) {
        (Some(Ask::Start(active)), Reply::Step(..) | Reply::Started(_) | Reply::Failed(..))
        | (Some(Ask::Stop(active)), Reply::Stopped(_) | Reply::Failed(..))
        | (Some(Ask::Remove(active)), Reply::Removed(_) | Reply::Failed(..)) => active == forge,
        _ => false,
    };
    if !matches {
        out.push(Out::Named(joined(&[
            b"box link: dropped ",
            reply.word(),
            b" for forge ",
            Decimal::of(u64::from(forge)).as_bytes(),
            b", not asked",
        ])));
        return;
    }
    match reply {
        Reply::Step(_, step) => out.push(Out::SettingUp { forge, step: Some(step) }),
        Reply::Started(_) => {
            *asked = None;
            out.push(Out::Started { forge });
        }
        Reply::Stopped(_) => *asked = None,
        Reply::Removed(_) => {
            *asked = None;
            out.push(Out::Removed { forge });
        }
        Reply::Failed(_, why) => {
            let operation = match *asked {
                Some(Ask::Stop(_)) => Some(b"stop".as_slice()),
                Some(Ask::Remove(_)) => Some(b"remove".as_slice()),
                Some(Ask::Start(_)) | None => None,
            };
            *asked = None;
            if let Some(operation) = operation {
                out.push(Out::Named(joined(&[
                    b"box link: ",
                    operation,
                    b" failed for forge ",
                    Decimal::of(u64::from(forge)).as_bytes(),
                    b": ",
                    &why,
                ])));
            } else {
                out.push(Out::Failed { forge, why });
            }
        }
    }
}
fn decoded(line: &[u8]) -> Option<Reply> {
    let line = line.strip_suffix(b"\n")?;
    if let Some(digits) = line.strip_prefix(b"started ") {
        return Some(Reply::Started(number(digits)?));
    }
    if let Some(digits) = line.strip_prefix(b"stopped ") {
        return Some(Reply::Stopped(number(digits)?));
    }
    if let Some(digits) = line.strip_prefix(b"removed ") {
        return Some(Reply::Removed(number(digits)?));
    }
    let (word, rest) = if let Some(rest) = line.strip_prefix(b"step ") {
        (b"step".as_slice(), rest)
    } else {
        (b"failed".as_slice(), line.strip_prefix(b"failed ")?)
    };
    let mut space = None;
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b' ' {
            space = Some(index);
            break;
        }
    }
    let (digits, tail) = rest.split_at_checked(space?)?;
    let forge = number(digits)?;
    let tail = tail.strip_prefix(b" ")?;
    if word == b"step" {
        let step = match tail {
            b"fetch" => Step::Fetch,
            b"prepare" => Step::Prepare,
            b"run" => Step::Run,
            _ => return None,
        };
        Some(Reply::Step(forge, step))
    } else if tail.is_empty() {
        None
    } else {
        Some(Reply::Failed(forge, bytes::copy_of(tail)))
    }
}
fn number(digits: &[u8]) -> Option<u16> {
    if digits.is_empty() {
        return None;
    }
    let mut number = 0_u16;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded line pieces");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("computed line length");
    }
    writer.finish()
}

/// One drain for asks, pipe lines and pipe terminals (protocol/forge.md, 2.8).
pub(crate) fn drain(engine: &mut crate::engine::Engine, out: &mut Queue<Out>) {
    use crate::setup::lifecycle::{set_up_refused, state};
    use temper_engine_domain_forge::{Connection, SetUpRefusal, SetUpStep};
    for _ in 0..out.len() {
        match out.pop().expect("box link outputs") {
            Out::Write(line) => {
                assert!(engine.box_link.write.is_none(), "one box line pending");
                engine.box_link.write = Some(line);
            }
            Out::SettingUp { forge, step } => {
                let step = match step {
                    None => None,
                    Some(Step::Fetch) => Some(SetUpStep::Fetch),
                    Some(Step::Prepare) => Some(SetUpStep::Prepare),
                    Some(Step::Run) => Some(SetUpStep::Run),
                };
                state(engine, forge, Connection::SettingUp { step });
            }
            Out::Failed { forge, why } => state(engine, forge, Connection::SetUpFailed { why }),
            Out::Removed { forge } => engine.events.push(temper_engine_domain::Event::Forge(Box::new(
                temper_engine_domain_forge::Event::GroupRemoved { forge },
            ))),
            Out::Started { forge } => {
                crate::setup::lifecycle::started(engine, forge);
            }
            Out::Answered { forge, request, answer } => match answer {
                Ok(()) => engine.events.push(temper_engine_domain::Event::Forge(Box::new(
                    temper_engine_domain_forge::Event::SetUpAnswered {
                        forge,
                        request,
                        answer: temper_engine_domain_forge::SetUpAnswer::SettingUp,
                    },
                ))),
                Err(refused) => {
                    let why = match refused {
                        Refused::Busy => SetUpRefusal::Busy,
                    };
                    set_up_refused(engine, forge, request, why);
                }
            },
            Out::Closed => engine
                .events
                .push(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::BoxClosed))),
            Out::Named(line) => engine.notes.push(crate::Note::BoxLink { line }),
        }
    }
}

/// Stage the host's whole group write before admitting its start (contracts T3).
pub(crate) fn set_up_group(engine: &mut crate::engine::Engine, forge: u16, request: Token, entry: &[u8]) {
    if engine.group_starts.get(&forge).is_some() {
        crate::setup::lifecycle::set_up_refused(engine, forge, request, temper_engine_domain_forge::SetUpRefusal::Busy);
    } else if entry.is_empty() {
        let mut out = Queue::with_capacity(3);
        engine.box_link.ask(forge, request, &mut out);
        drain(engine, &mut out);
    } else if let Some(text) = temper_forge_setup::setup::group_settings(entry, forge) {
        engine.group_starts.insert(forge, request).expect("one group write per admitted forge");
        engine.group_writes.push(crate::GroupWrite { forge, text });
    } else {
        group_failed(engine, forge, request, b"group settings write: invalid entry".as_slice().into());
    }
}
pub(crate) fn group_written(engine: &mut crate::engine::Engine, forge: u16, result: Result<(), i32>) {
    let Some(request) = engine.group_starts.remove(&forge) else { return };
    if engine.end.is_some() || engine.drain.is_some() {
        return;
    }
    match result {
        Ok(()) => {
            let mut out = Queue::with_capacity(3);
            engine.box_link.ask(forge, request, &mut out);
            drain(engine, &mut out);
        }
        Err(errno) => group_failed(
            engine,
            forge,
            request,
            joined(&[b"group settings write failed: errno ", Decimal::of(u64::from(errno.unsigned_abs())).as_bytes()]),
        ),
    }
}
fn group_failed(engine: &mut crate::engine::Engine, forge: u16, request: Token, why: Box<[u8]>) {
    let mut out = Queue::with_capacity(2);
    out.push(Out::Answered { forge, request, answer: Ok(()) });
    out.push(Out::Failed { forge, why });
    drain(engine, &mut out);
}
