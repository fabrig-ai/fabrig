//! Engine-side conversion between worker layouts (jig/docs/design/protocol/channel/layouts.md, 2–5).
use crate::{Carried, KINDS, Kind, v22};
use alloc::boxed::Box;
use skein_channel::{Direction, Version};
use skein_codec::layout::{Misconverted, Mode, Plan, Refused, Workspace};
use skein_lib::{Buffer, List};

mod plans;
mod table;
mod words;
pub use plans::Plans;

/// Worker texts kept per wire.
pub const MAX_TEXTS: u32 = 8;

/// One wire as the worker's credential states it.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Variant,
    Field,
    Narrowed,
    Absent,
    Large,
    Malformed,
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
        self.words(b"the worker's layout cannot keep ")
    }

    /// The tail of a diagnostic for a value the engine cannot read.
    #[must_use]
    pub fn read(&self) -> Box<[u8]> {
        self.words(b"")
    }

    fn words(&self, prefix: &[u8]) -> Box<[u8]> {
        let why = match self.cause {
            Cause::Variant => b"variant".as_slice(),
            Cause::Field => b"field",
            Cause::Narrowed => b"narrowed",
            Cause::Absent => b"absent",
            Cause::Large => b"too large",
            Cause::Malformed => b"malformed",
        };
        let dot = if self.part.is_empty() { b"".as_slice() } else { b"." };
        words::join(
            &[prefix, self.wire.as_bytes(), b" ", self.declaration.as_bytes(), dot, self.part.as_bytes(), b": ", why],
            usize::MAX,
        )
    }
}

/// One channel's conversions and their fitted workspace.
#[derive(Clone, Debug)]
pub struct Peer {
    same: bool,
    frames: Option<table::Frames>,
    send_bounds: Option<(v22::Limits, Version)>,
    plans: List<Pass>,
    work: Workspace,
}

#[derive(Clone, Debug)]
struct Pass {
    value: Value,
    mode: Mode,
    wire: Box<str>,
    source: Box<str>,
    plan: Option<Plan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Value {
    Frame,
    Part(Nested),
}

impl Peer {
    /// Whether every wire is at the engine's layout.
    #[must_use]
    pub fn same(&self) -> bool {
        self.same
    }

    /// The worker's kinds and their bounds, at the channel's version.
    pub fn table(&self, limits: &v22::Limits) -> Result<Version, Box<[u8]>> {
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
        for rule in &table.kinds {
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

    fn frame_kind(&self, mode: Mode, kind: u16) -> Result<u16, Misconverted> {
        let (base, _) = table::direction(mode);
        let tag = table::tag(base, kind).ok_or(Misconverted::Tag)?;
        let (plan, _) = Self::frame_pass(&self.plans, mode);
        let tag = match plan {
            Some(plan) => plan.variant_tag(tag)?,
            None => tag,
        };
        Ok(table::number(base, tag))
    }

    pub(crate) fn converts_part(&self, mode: Mode, nested: Nested) -> bool {
        match Self::find(&self.plans, Value::Part(nested), mode) {
            Some(pass) => pass.plan.is_some(),
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
            let result = match &pass.plan {
                Some(plan) => plan.convert(bytes, out, &mut self.work),
                None => copy(bytes, out),
            };
            if let Err(error) = result {
                out.clear();
                return Err(problem(&pass.wire, &pass.source, "", pass.plan.as_ref(), error));
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
        let (plan, _) = Self::frame_pass(&self.plans, mode);
        let result = match plan {
            Some(plan) => plan.convert_variant(tag, body, out, &mut self.work),
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
            Some(pass) => (pass.plan.as_ref(), &pass.source),
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
            let cause = match why {
                Refused::Variant => Cause::Variant,
                Refused::Field => Cause::Field,
                Refused::Narrowed => Cause::Narrowed,
                Refused::Absent => Cause::Absent,
            };
            return unconverted(wire, declaration, part, cause);
        }
        Misconverted::Room => Cause::Large,
        Misconverted::Short | Misconverted::Tag | Misconverted::Bool | Misconverted::Bound | Misconverted::Trailing => {
            Cause::Malformed
        }
    };
    unconverted(wire, declaration, part, cause)
}
