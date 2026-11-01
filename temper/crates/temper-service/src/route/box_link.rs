//! The routing's box link: serve's side of the box's pipe (protocol/forge.md, 2.8).
//! Removal: temper/docs/plan/playground-reset-r4/notes/contracts.md, 3.

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
    /// Write the group file before sending its start.
    GroupFile { forge: u16 },
    /// The ask's one answer: setting up, or its refusal.
    Answered { forge: u16, request: Token, answer: Result<(), Refused> },
    /// The forge setting up: `None` at the ask, then each `step` line's step.
    SettingUp { forge: u16, step: Option<Step> },
    /// `started FORGE`: the ask ended, the group started.
    Started { forge: u16 },
    /// `failed FORGE WHY`.
    Failed { forge: u16, why: Box<[u8]> },
    /// The box answered the kept removal ask.
    Removed { forge: u16, removed: bool },
    /// One log line naming what the link dropped, or the pipe's end.
    Named(Box<[u8]>),
    /// The pipe ended; the root decides what it means for the forges.
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    Start(u16),
    Writing(u16),
    Stop(u16),
    Remove(u16),
}

pub(crate) struct Link {
    pub(crate) files: Option<super::group_file::Files>,
    pub(crate) write: Option<Box<[u8]>>,
    ended: bool,
    piped: bool,
    asked: Option<Ask>,
    // Starts and removals wait in order for the active ask (protocol/setup.md,
    // 7.1; contracts, 3), bounded by forge.forges when removing.
    held: Queue<Ask>,
    stop: Option<u16>,
    line_bytes: u32,
    partial: List<u8>,
    skipping: bool,
}
impl Link {
    /// The routing only sends asks admitted by the domain.
    pub(crate) fn new(line_bytes: u32, piped: bool) -> Link {
        Link {
            files: None,
            write: None,
            ended: false,
            piped,
            asked: None,
            held: Queue::with_capacity(1),
            stop: None,
            line_bytes,
            partial: List::with_capacity(line_bytes),
            skipping: false,
        }
    }
    /// The caller reserves three output slots before asking.
    pub(crate) fn ask(&mut self, forge: u16, request: Token, out: &mut Queue<Out>) {
        let mut held_start = false;
        for ask in &self.held {
            match ask {
                Ask::Start(_) => held_start = true,
                Ask::Writing(_) | Ask::Stop(_) | Ask::Remove(_) => {}
            }
        }
        if self.held.room() == 0
            || held_start
            || (self.asked == Some(Ask::Start(forge)) || self.asked == Some(Ask::Writing(forge)))
            || self.ended
            || self.asked.is_none() && self.write.is_some()
        {
            out.push(Out::Named(Box::from(b"box link: fault: a setup ask while busy or closed".as_slice())));
            out.push(Out::Answered { forge, request, answer: Err(Refused::Busy) });
            return;
        }
        if self.asked.is_some() {
            self.held.push(Ask::Start(forge));
        } else {
            self.begin_start(forge, out);
        }
        out.push(Out::SettingUp { forge, step: None });
        out.push(Out::Answered { forge, request, answer: Ok(()) });
    }
    /// No request waits on the demo's stop. Keep it even when a start is held.
    pub(crate) fn stop(&mut self, forge: u16, out: &mut Queue<Out>) {
        if self.ended {
            return;
        }
        for _ in 0..self.held.len() {
            let ask = self.held.pop().expect("held ask");
            if ask != Ask::Start(forge) {
                self.held.push(ask);
            }
        }
        if self.asked.is_some() {
            self.stop = Some(forge);
        } else {
            self.asked = Some(Ask::Stop(forge));
            out.push(Out::Write(stop(forge)));
        }
    }
    /// Size held asks from the deployment's forge bound, preserving their order.
    pub(crate) fn limit_held(&mut self, forges: u32) {
        if self.held.capacity() == forges {
            return;
        }
        let mut held = Queue::with_capacity(forges);
        for _ in 0..self.held.len() {
            held.push(self.held.pop().expect("held ask"));
        }
        self.held = held;
    }
    /// The caller reserves two output slots. A failed removal stays kept.
    pub(crate) fn remove(&mut self, forge: u16, out: &mut Queue<Out>) {
        let mut held_remove = false;
        for ask in &self.held {
            if *ask == Ask::Remove(forge) {
                held_remove = true;
                break;
            }
        }
        if !self.piped || self.ended {
            out.push(Out::Removed { forge, removed: !self.piped });
        } else if self.asked == Some(Ask::Remove(forge)) || held_remove {
            // One kept removal per forge can be sent again before its answer.
        } else if self.asked.is_some() {
            if self.held.try_push(Ask::Remove(forge)).is_err() {
                out.push(Out::Named(Box::from(b"box link: remove queue reached forge.forges".as_slice())));
                out.push(Out::Removed { forge, removed: false });
            }
        } else {
            self.asked = Some(Ask::Remove(forge));
            out.push(Out::Write(remove_line(forge)));
        }
    }
    fn next(&mut self, out: &mut Queue<Out>) {
        if let Some(forge) = self.stop.take() {
            self.asked = Some(Ask::Stop(forge));
            out.push(Out::Write(stop(forge)));
        } else if let Some(ask) = self.held.pop() {
            match ask {
                Ask::Start(forge) => self.begin_start(forge, out),
                Ask::Remove(forge) => {
                    self.asked = Some(ask);
                    out.push(Out::Write(remove_line(forge)));
                }
                Ask::Writing(_) | Ask::Stop(_) => unreachable!("only starts and removes are held"),
            }
        }
    }
    /// Any chunking is accepted. The caller reserves two output slots per LF in
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
                    self.next(out);
                }
            } else if self.partial.room() == 0 {
                self.partial.clear();
                self.skipping = true;
            }
        }
    }
    fn begin_start(&mut self, forge: u16, out: &mut Queue<Out>) {
        if self.files.is_some() {
            self.asked = Some(Ask::Writing(forge));
            out.push(Out::GroupFile { forge });
        } else {
            self.asked = Some(Ask::Start(forge));
            out.push(Out::Write(start(forge)));
        }
    }
    pub(crate) fn file_written(&mut self, stored: bool, out: &mut Queue<Out>) {
        let forge = match self.asked {
            Some(Ask::Writing(forge)) => forge,
            Some(Ask::Start(_) | Ask::Stop(_) | Ask::Remove(_)) | None => return,
        };
        if stored {
            self.asked = Some(Ask::Start(forge));
            out.push(Out::Write(start(forge)));
        } else {
            self.asked = None;
            out.push(Out::Failed { forge, why: b"group file not written".as_slice().into() });
            self.next(out);
        }
    }
    /// fd 4 ended, or a read/write failed. The caller reserves three output slots.
    pub(crate) fn closed(&mut self, out: &mut Queue<Out>) {
        if self.ended {
            return;
        }
        self.ended = true;
        if let Some(files) = &mut self.files {
            files.close();
        }
        self.write = None;
        self.partial.clear();
        self.skipping = false;
        out.push(Out::Named(Box::from(b"box link: the box closed the pipe".as_slice())));
        match self.asked {
            Some(Ask::Remove(forge)) => out.push(Out::Removed { forge, removed: false }),
            Some(Ask::Start(_) | Ask::Writing(_) | Ask::Stop(_)) | None => {}
        }
        self.asked = None;
        self.held.clear();
        self.stop = None;
        out.push(Out::Closed);
    }
}

fn remove_line(forge: u16) -> Box<[u8]> {
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
            out.push(Out::Removed { forge, removed: true });
        }
        Reply::Failed(_, why) => {
            let (removing, stopping) = match *asked {
                Some(Ask::Remove(_)) => (true, false),
                Some(Ask::Stop(_)) => (false, true),
                Some(Ask::Start(_) | Ask::Writing(_)) | None => (false, false),
            };
            *asked = None;
            if removing || stopping {
                out.push(Out::Named(joined(&[
                    if removing { b"box link: remove failed for forge " } else { b"box link: stop failed for forge " },
                    Decimal::of(u64::from(forge)).as_bytes(),
                    b": ",
                    &why,
                ])));
                if removing {
                    out.push(Out::Removed { forge, removed: false });
                }
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
    if digits.is_empty() || digits.len() > 1 && digits.starts_with(b"0") {
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
            Out::GroupFile { forge } => {
                if let Some(bytes) = group_bytes(engine, forge) {
                    super::group_file::write(engine, forge, bytes);
                } else {
                    engine.box_link.asked = Some(Ask::Start(forge));
                    assert!(engine.box_link.write.is_none(), "one box line pending");
                    engine.box_link.write = Some(start(forge));
                }
            }
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
            Out::Removed { forge, removed } => engine.events.push(temper_engine_domain::Event::Forge(Box::new(
                temper_engine_domain_forge::Event::BoxRemoved { forge, removed },
            ))),
            Out::Closed => engine
                .events
                .push(temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::BoxClosed))),
            Out::Named(line) => engine.notes.push(crate::Note::BoxLink { line }),
        }
    }
}

fn group_bytes(engine: &crate::engine::Engine, forge: u16) -> Option<Box<[u8]>> {
    let record = crate::forge::table::record(engine, forge)?;
    let setup = record.setup.as_ref()?;
    Some(super::group_file::render(forge, setup, record.playground))
}

#[cfg(test)]
#[path = "../tests/box_remove.rs"]
mod tests;
