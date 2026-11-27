//! Engine-side conversion between worker layouts (jig/docs/design/protocol/channel/layouts.md, 2–5).
use crate::{Carried, KINDS, Kind, v22};
use alloc::boxed::Box;
use skein_channel::{Direction, Kind as ChannelKind};
use skein_codec::{
    layout::{Misconverted, Mode, Plan, Refusal, Refused, Side, Workspace},
    schema::{Root, Schema},
};
use skein_lib::{Buffer, List};

mod gaps;
mod plans;
mod table;
mod words;
pub use gaps::{Gap, Site};
pub use plans::Plans;

/// Worker texts kept per wire.
pub const MAX_TEXTS: u32 = 8;

/// One wire as the worker's credential states it (jig/docs/design/protocol/channel/wires.md, 2).
#[derive(Clone, Copy, Debug)]
pub struct Theirs<'a> {
    pub name: &'a str,
    pub layout: [u8; 32],
    pub text: &'a [u8],
}

/// An independently converted part carried by a jig record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nested {
    Charter,
    Result,
    Turn,
    Carried(Carried),
}

impl Nested {
    fn crosses(self, mode: Mode) -> bool {
        let down = mode == Mode::Keep;
        match self {
            Self::Charter => down,
            Self::Result => !down,
            Self::Turn => true,
            Self::Carried(carried) => match carried {
                Carried::Items | Carried::JobSpec | Carried::Grant | Carried::Answer => down,
                Carried::Left | Carried::Saved | Carried::Preparation | Carried::JobResult | Carried::Ask => !down,
            },
        }
    }

    fn site(self) -> (&'static str, &'static str) {
        match self {
            Self::Charter => (smith_charter::wire::FAMILY, "Charter"),
            Self::Result => (smith_charter::wire::FAMILY, "RunResult"),
            Self::Turn => (smith_transcript::wire::FAMILY, "Turn"),
            Self::Carried(carried) => (
                "application",
                match carried {
                    Carried::Items => "Items",
                    Carried::Left => "Left",
                    Carried::Saved => "Saved",
                    Carried::Preparation => "Preparation",
                    Carried::JobSpec => "JobSpec",
                    Carried::JobResult => "JobResult",
                    Carried::Grant => "Grant",
                    Carried::Ask => "Ask",
                    Carried::Answer => "Answer",
                },
            ),
        }
    }
}

/// Why a value did not convert.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cause {
    Variant,
    Field,
    Narrowed,
    Absent,
    Large,
    Malformed,
    WrittenRequired,
    Written { value: Box<str>, why: Box<str> },
}

impl Cause {
    fn from_refused(why: Refused) -> Self {
        match why {
            Refused::Variant => Self::Variant,
            Refused::Field => Self::Field,
            Refused::Narrowed => Self::Narrowed,
            Refused::Absent => Self::Absent,
        }
    }

    fn words(&self) -> &[u8] {
        match self {
            Self::Variant => b"variant",
            Self::Field => b"field",
            Self::Narrowed => b"narrowed",
            Self::Absent => b"absent",
            Self::Large => b"too large",
            Self::Malformed => b"malformed",
            Self::WrittenRequired => b"written converter required",
            Self::Written { why, .. } => why.as_bytes(),
        }
    }
}

/// A value that did not convert, named by the source text's names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unconverted {
    pub wire: Box<str>,
    pub declaration: Box<str>,
    pub part: Box<str>,
    pub cause: Cause,
}

impl Unconverted {
    /// The text of a value the worker's layout cannot keep.
    #[must_use]
    pub fn kept(&self) -> Box<[u8]> {
        words::kept(&self.wire, &self.declaration, &self.part, &self.cause)
    }

    /// The tail of a diagnostic for a value the engine cannot read.
    #[must_use]
    pub fn read(&self) -> Box<[u8]> {
        words::value(b"", &self.wire, &self.declaration, &self.part, &self.cause)
    }
}

/// One channel's conversions and their fitted workspace
/// (jig/docs/design/protocol/channel/layouts.md, 3 and 4).
#[derive(Clone, Debug)]
pub struct Peer {
    same: bool,
    frames: Option<table::Frames>,
    send_bounds: Option<(v22::Limits, List<ChannelKind>)>,
    plans: List<Pass>,
    written: List<Texts>,
    work: Workspace,
}

#[derive(Clone, Debug)]
struct Pass {
    value: Value,
    mode: Mode,
    wire: Box<str>,
    source: Box<str>,
    passing: Passing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Frame,
    Part(Nested),
}

#[derive(Clone, Debug)]
enum Passing {
    Same,
    Plan(Plan),
    Written { from: Root, to: Root },
}

impl Passing {
    fn plan(&self) -> Option<&Plan> {
        match self {
            Self::Plan(plan) => Some(plan),
            Self::Same | Self::Written { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
struct Texts {
    wire: Box<str>,
    engine: Schema,
    worker: Schema,
}

/// Both parsed sides of a pass the wire owner writes.
#[derive(Debug)]
pub struct Sides<'a> {
    pub wire: &'a str,
    pub from: Side<'a>,
    pub to: Side<'a>,
}

impl Peer {
    /// Whether every wire is at the engine's layout.
    #[must_use]
    pub fn same(&self) -> bool {
        self.same
    }

    /// The worker's kinds and their bounds.
    pub fn table(&self, limits: &v22::Limits) -> Result<List<ChannelKind>, Box<[u8]>> {
        if let Some(frames) = &self.frames {
            return Ok(frames.table(limits));
        }
        table::current(limits)
    }

    /// The number of an engine kind at the worker's layout, if it has one.
    #[must_use]
    pub fn sends(&self, kind: Kind) -> Option<u16> {
        self.frame_kind(Mode::Keep, kind.number()).ok()
    }

    /// The parsed sides for a part with a written pass, if it has one.
    #[must_use]
    pub fn written_part(&self, mode: Mode, nested: Nested) -> Option<Sides<'_>> {
        self.written_at(mode, Value::Part(nested))
    }

    pub(crate) fn written_at(&self, mode: Mode, value: Value) -> Option<Sides<'_>> {
        let pass = Self::find(&self.plans, value, mode)?;
        let Passing::Written { from, to } = pass.passing else { return None };
        for texts in &self.written {
            if texts.wire == pass.wire {
                let (source, target) = match mode {
                    Mode::Read => (&texts.worker, &texts.engine),
                    Mode::Keep => (&texts.engine, &texts.worker),
                };
                return Some(Sides {
                    wire: &pass.wire,
                    from: Side { text: source, root: from, held: &[] },
                    to: Side { text: target, root: to, held: &[] },
                });
            }
        }
        unreachable!("written passes retain both texts");
    }

    pub(crate) fn written_problem(&self, mode: Mode, value: Value, cause: Cause) -> Unconverted {
        let pass = Self::find(&self.plans, value, mode).expect("a written pass waits");
        unconverted(&pass.wire, &pass.source, "", cause)
    }

    pub(crate) fn written_refusal(&self, mode: Mode, value: Value, refusal: Refusal) -> Unconverted {
        let pass = Self::find(&self.plans, value, mode).expect("a written pass waits");
        Unconverted {
            wire: pass.wire.clone(),
            declaration: refusal.declaration,
            part: refusal.part,
            cause: Cause::Written { value: pass.source.clone(), why: refusal.why },
        }
    }

    // A channel normally uses one set of limits. Reuse the worker's table
    // instead of recomputing every declaration's bound for each outgoing frame.
    pub(crate) fn send_room(&mut self, kind: Kind, limits: &v22::Limits) -> Result<u32, Unconverted> {
        let source = kind.number();
        let Some(frames) = &self.frames else {
            let Some(room) = crate::sizes::body(limits, kind) else {
                return Err(self.frame_problem(Mode::Keep, source, Misconverted::Room));
            };
            return Ok(room);
        };
        let number = match self.frame_kind(Mode::Keep, source) {
            Ok(number) => number,
            Err(error) => return Err(self.frame_problem(Mode::Keep, source, error)),
        };
        let current = match &self.send_bounds {
            Some((previous, _)) => previous == limits,
            None => false,
        };
        if !current {
            self.send_bounds = Some((*limits, frames.table(limits)));
        }
        let (_, table) = self.send_bounds.as_ref().expect("bounds fitted to these limits");
        for rule in table {
            if rule.kind == number && rule.direction == Direction::FromResponder {
                return Ok(rule.largest);
            }
        }
        Err(self.frame_problem(Mode::Keep, source, Misconverted::Tag))
    }

    // Resolve the destination before allocating a buffer bounded by its kind.
    pub(crate) fn reads(&self, kind: u16) -> Result<Kind, Unconverted> {
        let number = match self.frame_kind(Mode::Read, kind) {
            Ok(number) => number,
            Err(error) => return Err(self.frame_problem(Mode::Read, kind, error)),
        };
        let Some(body) = worker_kind(number) else {
            return Err(self.frame_problem(Mode::Read, kind, Misconverted::Tag));
        };
        Ok(body)
    }

    pub(crate) fn frame_kind(&self, mode: Mode, kind: u16) -> Result<u16, Misconverted> {
        let (base, _) = table::direction(mode);
        let tag = table::tag(base, kind).ok_or(Misconverted::Tag)?;
        let tag = match Self::find(&self.plans, Value::Frame, mode) {
            Some(pass) => match &pass.passing {
                Passing::Plan(plan) => plan.variant_tag(tag)?,
                Passing::Written { .. } => {
                    self.frames.as_ref().expect("written frame table").mapped(mode, tag).ok_or(Misconverted::Tag)?
                }
                Passing::Same => tag,
            },
            None => tag,
        };
        Ok(table::number(base, tag))
    }

    pub(crate) fn converts_part(&self, mode: Mode, nested: Nested) -> bool {
        match Self::find(&self.plans, Value::Part(nested), mode) {
            Some(pass) => match &pass.passing {
                Passing::Same => false,
                Passing::Plan(_) | Passing::Written { .. } => true,
            },
            None => false,
        }
    }

    /// Converts an engine body to the worker's kind and body.
    pub fn send(&mut self, kind: Kind, body: &[u8], out: &mut Buffer) -> Result<u16, Unconverted> {
        self.frame(Mode::Keep, kind.number(), body, out)
    }

    /// Converts a worker frame to the engine's kind and body.
    pub fn receive(&mut self, kind: u16, body: &[u8], out: &mut Buffer) -> Result<Kind, Unconverted> {
        let number = self.frame(Mode::Read, kind, body, out)?;
        if let Some(kind) = worker_kind(number) {
            return Ok(kind);
        }
        out.clear();
        Err(unconverted(crate::wire::FAMILY, "FromWorker", "", Cause::Malformed))
    }

    /// Keeps a part at the worker's layout.
    pub fn down(&mut self, nested: Nested, bytes: &[u8], out: &mut Buffer) -> Result<(), Unconverted> {
        self.part(Mode::Keep, nested, bytes, out)
    }

    /// Reads a part at the engine's layout.
    pub fn up(&mut self, nested: Nested, bytes: &[u8], out: &mut Buffer) -> Result<(), Unconverted> {
        self.part(Mode::Read, nested, bytes, out)
    }

    #[expect(clippy::manual_find, reason = "step code uses no closures")]
    fn find(plans: &List<Pass>, value: Value, mode: Mode) -> Option<&Pass> {
        for pass in plans {
            if pass.value == value && pass.mode == mode {
                return Some(pass);
            }
        }
        None
    }

    fn part(&mut self, mode: Mode, nested: Nested, bytes: &[u8], out: &mut Buffer) -> Result<(), Unconverted> {
        if let Some(pass) = Self::find(&self.plans, Value::Part(nested), mode) {
            let result = match &pass.passing {
                Passing::Plan(plan) => plan.convert(bytes, out, &mut self.work),
                Passing::Same => copy(bytes, out),
                Passing::Written { .. } => {
                    out.clear();
                    return Err(unconverted(&pass.wire, &pass.source, "", Cause::WrittenRequired));
                }
            };
            if let Err(error) = result {
                out.clear();
                return Err(problem(&pass.wire, &pass.source, "", pass.passing.plan(), error));
            }
            return Ok(());
        }
        let (wire, declaration) = nested.site();
        if !nested.crosses(mode) {
            out.clear();
            return Err(unconverted(wire, declaration, "", Cause::Malformed));
        }
        if let Err(error) = copy(bytes, out) {
            return Err(problem(wire, declaration, "", None, error));
        }
        Ok(())
    }

    fn frame(&mut self, mode: Mode, kind: u16, body: &[u8], out: &mut Buffer) -> Result<u16, Unconverted> {
        let (base, _) = table::direction(mode);
        let Some(tag) = table::tag(base, kind) else {
            out.clear();
            return Err(self.frame_problem(mode, kind, Misconverted::Tag));
        };
        let result = match Self::find(&self.plans, Value::Frame, mode) {
            Some(pass) => match &pass.passing {
                Passing::Plan(plan) => plan.convert_variant(tag, body, out, &mut self.work),
                Passing::Same => copy(body, out).and(Ok(tag)),
                Passing::Written { .. } => {
                    out.clear();
                    return Err(unconverted(&pass.wire, &pass.source, "", Cause::WrittenRequired));
                }
            },
            None => copy(body, out).and(Ok(tag)),
        };
        match result {
            Ok(tag) => Ok(table::number(base, tag)),
            Err(error) => {
                out.clear();
                Err(self.frame_problem(mode, kind, error))
            }
        }
    }

    fn frame_pass(plans: &List<Pass>, mode: Mode) -> (Option<&Plan>, &str) {
        match Self::find(plans, Value::Frame, mode) {
            Some(pass) => (pass.passing.plan(), &pass.source),
            None => (None, table::direction(mode).1),
        }
    }

    pub(crate) fn frame_problem(&self, mode: Mode, kind: u16, error: Misconverted) -> Unconverted {
        let (base, _) = table::direction(mode);
        let (plan, declaration) = Self::frame_pass(&self.plans, mode);
        let part = match &self.frames {
            Some(frames) => match table::tag(base, kind) {
                Some(tag) => frames.source(mode, tag),
                None => "",
            },
            None => "",
        };
        problem(crate::wire::FAMILY, declaration, part, plan, error)
    }
}

fn worker_kind(number: u16) -> Option<Kind> {
    for rule in KINDS {
        if rule.kind == number && rule.direction == Direction::FromInitiator {
            return Some(rule.body);
        }
    }
    None
}

fn copy(bytes: &[u8], out: &mut Buffer) -> Result<(), Misconverted> {
    out.clear();
    out.put(bytes).or(Err(Misconverted::Room))
}

fn unconverted(wire: &str, declaration: &str, part: &str, cause: Cause) -> Unconverted {
    Unconverted { wire: Box::from(wire), declaration: Box::from(declaration), part: Box::from(part), cause }
}

fn problem(wire: &str, declaration: &str, part: &str, plan: Option<&Plan>, error: Misconverted) -> Unconverted {
    let cause = match error {
        Misconverted::Refused(index) => {
            let (why, declaration, part) =
                plan.expect("a plan refused the value").refusal(index).expect("an indexed plan refusal");
            return unconverted(wire, declaration, part, Cause::from_refused(why));
        }
        Misconverted::Room => Cause::Large,
        Misconverted::Short | Misconverted::Tag | Misconverted::Bool | Misconverted::Bound | Misconverted::Trailing => {
            Cause::Malformed
        }
    };
    unconverted(wire, declaration, part, cause)
}
