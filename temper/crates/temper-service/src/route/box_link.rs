//! Serve's box pipe (temper/docs/design/draft/playground-reset.md, 3 and 4).

use alloc::boxed::Box;
use skein_lib::{Decimal, List, Queue, Token, Writer, bytes};
use temper_engine_domain_forge::BoxKind;

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
    /// `failed FORGE WHY`.
    Failed { forge: u16, why: Box<[u8]> },
    /// `removed FORGE`: the group and its directory are gone.
    Removed { forge: u16 },
    /// One log line naming what the link dropped, or the pipe's end.
    Named(Box<[u8]>),
    /// The pipe ended; the root decides what it means for the forges.
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    Start(u16),
    End(End),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum End {
    Remove(u16),
}
impl End {
    fn forge(self) -> u16 {
        match self {
            Self::Remove(forge) => forge,
        }
    }
    fn word(self) -> &'static [u8] {
        match self {
            Self::Remove(_) => b"remove",
        }
    }
    fn line(self) -> Box<[u8]> {
        joined(&[self.word(), b" ", Decimal::of(u64::from(self.forge())).as_bytes(), b"\n"])
    }
}

struct Start {
    forge: u16,
    kind: Option<BoxKind>,
}
pub(crate) struct Link {
    pub(crate) write: Option<Box<[u8]>>,
    ended: bool,
    asked: Option<Ask>,
    // A start retains its kind until the active ask ends (temper/docs/design/draft/playground-reset.md, 3).
    held: Option<Start>,
    pending_end: Option<End>,
    line_bytes: u32,
    partial: List<u8>,
    skipping: bool,
}
impl Link {
    /// The routing only sends asks admitted by the domain.
    pub(crate) fn new(line_bytes: u32, _piped: bool) -> Link {
        Link {
            write: None,
            ended: false,
            asked: None,
            held: None,
            pending_end: None,
            line_bytes,
            partial: List::with_capacity(line_bytes),
            skipping: false,
        }
    }
    /// The caller reserves three output slots before asking.
    #[cfg(test)]
    pub(crate) fn ask(&mut self, forge: u16, request: Token, out: &mut Queue<Out>) {
        self.start(forge, None, request, out);
    }
    /// A kind is retained with a held start until the current ask ends.
    pub(crate) fn group(&mut self, forge: u16, kind: BoxKind, request: Token, out: &mut Queue<Out>) {
        self.start(forge, Some(kind), request, out);
    }
    fn start(&mut self, forge: u16, kind: Option<BoxKind>, request: Token, out: &mut Queue<Out>) {
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
            self.held = Some(Start { forge, kind });
        } else {
            self.asked = Some(Ask::Start(forge));
            out.push(Out::Write(start(forge, kind)));
        }
        out.push(Out::SettingUp { forge, step: None });
        out.push(Out::Answered { forge, request, answer: Ok(()) });
    }
    /// Removal has a terminal only after the box reports the group gone.
    pub(crate) fn remove(&mut self, forge: u16, out: &mut Queue<Out>) {
        self.end(End::Remove(forge), out);
    }
    fn end(&mut self, end: End, out: &mut Queue<Out>) {
        if self.ended {
            return;
        }
        if let Some(held) = &self.held
            && held.forge == end.forge()
        {
            self.held = None;
        }
        if self.asked.is_some() {
            self.pending_end = Some(end);
        } else {
            self.asked = Some(Ask::End(end));
            out.push(Out::Write(end.line()));
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
                    if let Some(end) = self.pending_end.take() {
                        self.asked = Some(Ask::End(end));
                        out.push(Out::Write(end.line()));
                    } else if let Some(held) = self.held.take() {
                        self.asked = Some(Ask::Start(held.forge));
                        out.push(Out::Write(start(held.forge, held.kind)));
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
        self.pending_end = None;
        out.push(Out::Closed);
    }
}

fn start(forge: u16, kind: Option<BoxKind>) -> Box<[u8]> {
    let suffix = match kind {
        None => b"".as_slice(),
        Some(BoxKind::Owned) => b" owned",
        Some(BoxKind::Fake) => b" fake",
    };
    joined(&[b"start ", Decimal::of(u64::from(forge)).as_bytes(), suffix, b"\n"])
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
        | (Some(Ask::End(End::Remove(active))), Reply::Removed(_) | Reply::Failed(..)) => active == forge,
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
            let ending = match *asked {
                Some(Ask::End(end)) => Some(end),
                Some(Ask::Start(_)) | None => None,
            };
            *asked = None;
            if let Some(end) = ending {
                out.push(Out::Named(joined(&[
                    b"box link: ",
                    end.word(),
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
            Out::Started { forge } => {
                crate::setup::lifecycle::started(engine, forge);
            }
            Out::Removed { forge } => engine.events.push(temper_engine_domain::Event::Forge(Box::new(
                temper_engine_domain_forge::Event::BoxRemoved { forge },
            ))),
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
