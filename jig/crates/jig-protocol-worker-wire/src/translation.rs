//! Nested bytes before outer records on send; outer before nested on receipt
//! (jig/docs/design/protocol/channel/bridges.md, 5, 6 and 8).
use crate::{Agreed, Application, Carried, Root, v13 as wire};
use alloc::boxed::Box;
use skein_channel::{Frame, frame_writer};
use skein_codec::{
    bridge::{Bridge, BridgeError},
    patch::{Capacity, Halt},
};
use skein_lib::{List, Reader, Secret, Writer, bytes};

/// A halted conversion, without any payload bytes.
#[derive(Debug)]
pub struct Untranslated {
    pub wire: Box<[u8]>,
    pub version: u16,
    pub halt: Halt,
}

/// Each component owns its bridges; none holds a payload between calls.
#[derive(Debug)]
pub struct Translation {
    jig: Lazy,
    charter: Lazy,
    transcript: Lazy,
    application: List<Bridge>,
    roots: List<Root>,
}

impl Translation {
    pub fn new(application: Application) -> Result<Self, BridgeError> {
        for root in &application.roots {
            let mut found = false;
            for bridge in &application.bridges {
                if bridge.family() == root.wire.as_bytes() {
                    found = bridge.total(bridge.current(), root.declaration.as_bytes()).is_some();
                }
            }
            if !found {
                return Err(BridgeError::Floor);
            }
        }
        Ok(Self {
            jig: Lazy::new(Family::Records),
            charter: Lazy::new(Family::Charter),
            transcript: Lazy::new(Family::Transcript),
            application: application.bridges,
            roots: application.roots,
        })
    }

    /// The outer record is the enumeration's value, not its body alone (5.2).
    pub fn send(
        &mut self,
        frame: Frame,
        version: u16,
        agreed: &[Agreed],
        limits: &wire::Limits,
    ) -> Result<Frame, Untranslated> {
        if version == crate::VERSION && self.current_payloads(frame.kind(), agreed) {
            return Ok(frame);
        }
        let (base, root) = direction(frame.kind()).ok_or(order())?;
        let tagged = tagged(base, frame.kind(), frame.bytes().get(8..).ok_or(order())?).ok_or(order())?;
        let nested = self.nested(root, tagged, agreed, true, limits)?;
        let current = self.jig.current();
        let translated = self.jig.translate(current, version, root, nested)?;
        untagged(base, translated).ok_or(order())
    }

    pub fn receive(
        &mut self,
        kind: u16,
        body: &[u8],
        version: u16,
        agreed: &[Agreed],
        limits: &wire::Limits,
    ) -> Result<Frame, Untranslated> {
        if version == crate::VERSION && self.current_payloads(kind, agreed) {
            return frame(kind, body).ok_or(order());
        }
        let (base, root) = direction(kind).ok_or(order())?;
        let tagged = tagged(base, kind, body).ok_or(order())?;
        let current = self.jig.current();
        let translated = self.jig.translate(version, current, root, tagged)?;
        let nested = self.nested(root, translated, agreed, false, limits)?;
        untagged(base, nested).ok_or(order())
    }

    /// Wires on which a range-stating worker cannot send whole to this engine (6.3).
    #[must_use]
    pub fn incomplete(&mut self, agreed: &[Agreed]) -> List<Agreed> {
        let mut refused = List::uncapped();
        for wire in agreed {
            let total = if wire.name.as_bytes() == self.jig.family() {
                // FromWorker includes each variant as well as each record.
                self.jig.total(wire.version, b"FromWorker") == Some(true)
            } else if wire.name.as_bytes() == self.charter.family() {
                self.charter.total(wire.version, b"RunResult") == Some(true)
            } else if wire.name.as_bytes() == self.transcript.family() {
                self.transcript.total(wire.version, b"Turn") == Some(true)
            } else {
                self.application_total(wire)
            };
            if !total {
                refused.push(*wire).expect("uncapped wire list");
            }
        }
        refused
    }

    #[cfg(test)]
    pub(crate) fn planned(&self) -> u32 {
        u32::from(self.jig.machine.is_some())
            .checked_add(u32::from(self.charter.machine.is_some()))
            .expect("three families")
            .checked_add(u32::from(self.transcript.machine.is_some()))
            .expect("three families")
    }

    fn current_payloads(&self, kind: u16, agreed: &[Agreed]) -> bool {
        for wire in agreed {
            if wire.name.as_bytes() == self.charter.family() && wire.version != self.charter.current() {
                return false;
            }
            if wire.name.as_bytes() == self.transcript.family() && wire.version != self.transcript.current() {
                return false;
            }
            for bridge in &self.application {
                if wire.name.as_bytes() == bridge.family() && wire.version != bridge.current() {
                    return false;
                }
            }
        }
        // A kept window's turns carry their own versions, even on a current link.
        !(self.transcript.first() < self.transcript.current()
            && (kind == crate::Kind::Turn.number() || kind == crate::Kind::Assignment.number()))
    }

    fn application_total(&self, wire: &Agreed) -> bool {
        for root in &self.roots {
            let sent = match root.carried {
                Carried::Left | Carried::Saved | Carried::Preparation | Carried::JobResult | Carried::Items => true,
                Carried::JobSpec | Carried::Grant => false,
            };
            if sent && root.wire == wire.name {
                for bridge in &self.application {
                    if bridge.family() == wire.name.as_bytes()
                        && bridge.total(wire.version, root.declaration.as_bytes()) != Some(true)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn carried(
        &mut self,
        carried: Carried,
        value: Box<[u8]>,
        agreed: &[Agreed],
        down: bool,
    ) -> Result<Box<[u8]>, Untranslated> {
        for root in &self.roots {
            if root.carried == carried {
                for bridge in self.application.iter_mut() {
                    if bridge.family() == root.wire.as_bytes() {
                        let version = version_of(agreed, bridge.family()).unwrap_or(bridge.current());
                        let (from, to) = versions(bridge.current(), version, down);
                        return convert(bridge, from, to, root.declaration.as_bytes(), value);
                    }
                }
            }
        }
        Ok(value)
    }

    fn optional(
        &mut self,
        carried: Carried,
        value: Option<Box<[u8]>>,
        agreed: &[Agreed],
        down: bool,
    ) -> Result<Option<Box<[u8]>>, Untranslated> {
        match value {
            Some(value) => Ok(Some(self.carried(carried, value, agreed, down)?)),
            None => Ok(None),
        }
    }

    fn smith(
        &mut self,
        charter: bool,
        root: &[u8],
        value: Box<[u8]>,
        agreed: &[Agreed],
        down: bool,
        window: bool,
    ) -> Result<Box<[u8]>, Untranslated> {
        let bridge = if charter { &mut self.charter } else { &mut self.transcript };
        let version = version_of(agreed, bridge.family()).unwrap_or(bridge.current());
        let (mut from, to) = versions(bridge.current(), version, down);
        if !charter {
            // Turns have their own version, including turns kept in an older window.
            let mut reader = Reader::new(&value);
            if let Some(written) = reader.u16() {
                if version == bridge.current() && (written < bridge.first() || written > bridge.current()) {
                    return Ok(value);
                }
                from = written;
                if down && window && written <= to {
                    return Ok(value);
                }
            } else if to == from {
                return Ok(value);
            }
        }
        bridge.translate(from, to, root, value)
    }

    fn window(&mut self, value: Box<[u8]>, agreed: &[Agreed], down: bool) -> Result<Box<[u8]>, Untranslated> {
        let mut reader = Reader::new(&value);
        let mut turns = List::uncapped();
        let mut length = 0_usize;
        for _ in 0..=value.len().saturating_div(4) {
            let turn = match skein_codec::piece::split(&mut reader) {
                Ok(Some(turn)) => turn,
                Ok(None) => break,
                // Preserve malformed current payloads for the hand-out's invalid refusal.
                Err(_) => return Ok(value),
            };
            let turn = self.smith(false, b"Turn", turn, agreed, down, true)?;
            length = length.checked_add(turn.len()).ok_or(order())?.checked_add(4).ok_or(order())?;
            turns.push(turn).expect("uncapped turn list");
        }
        let mut writer = Writer::new(length);
        for turn in turns.into_boxed() {
            skein_codec::piece::write(&mut writer, &turn).or(Err(order()))?;
        }
        Ok(writer.finish())
    }

    fn nested(
        &mut self,
        root: &[u8],
        value: Box<[u8]>,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<Box<[u8]>, Untranslated> {
        let mut reader = Reader::new(&value);
        if root == b"FromEngine" {
            let record = wire::FromEngine::decode(limits, &mut reader).or(Err(order()))?;
            let record = self.engine(record, agreed, down, limits)?;
            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
            record.encode(&mut writer).or(Err(order()))?;
            Ok(writer.finish())
        } else {
            let record = wire::FromWorker::decode(limits, &mut reader).or(Err(order()))?;
            let record = self.worker(record, agreed, down, limits)?;
            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
            record.encode(&mut writer).or(Err(order()))?;
            Ok(writer.finish())
        }
    }
}

/// Generated static texts stay with their owner; parsed schemas and plans exist only
/// after a nonidentity conversion or an older-version totality check needs them.
#[derive(Clone, Copy, Debug)]
enum Family {
    Records,
    Charter,
    Transcript,
}
#[derive(Debug)]
struct Lazy {
    family: Family,
    machine: Option<Bridge>,
    first: u16,
}
impl Lazy {
    fn new(family: Family) -> Self {
        let texts = match family {
            Family::Records => crate::patches::kept::jig_worker_wire::TEXTS,
            Family::Charter => smith_charter::patches::kept::smith_charter::TEXTS,
            Family::Transcript => smith_transcript::patches::kept::smith_transcript::TEXTS,
        };
        Self { family, machine: None, first: header_version(texts.first().expect("kept family")) }
    }
    fn family(&self) -> &'static [u8] {
        match self.family {
            Family::Records => crate::wire::FAMILY.as_bytes(),
            Family::Charter => smith_charter::wire::FAMILY.as_bytes(),
            Family::Transcript => smith_transcript::wire::FAMILY.as_bytes(),
        }
    }
    const fn current(&self) -> u16 {
        match self.family {
            Family::Records => crate::VERSION,
            Family::Charter => smith_charter::wire::VERSION,
            Family::Transcript => smith_transcript::wire::VERSION,
        }
    }
    const fn first(&self) -> u16 {
        self.first
    }
    fn materialize(&mut self) -> Result<&mut Bridge, BridgeError> {
        if self.machine.is_none() {
            self.machine = Some(match self.family {
                Family::Records => crate::patches::kept::jig_worker_wire::bridge(capacity(
                    crate::patches::kept::jig_worker_wire::TEXTS,
                )?)?,
                Family::Charter => smith_charter::patches::kept::smith_charter::bridge(capacity(
                    smith_charter::patches::kept::smith_charter::TEXTS,
                )?)?,
                Family::Transcript => smith_transcript::patches::kept::smith_transcript::bridge(capacity(
                    smith_transcript::patches::kept::smith_transcript::TEXTS,
                )?)?,
            });
        }
        Ok(self.machine.as_mut().expect("materialized bridge"))
    }
    fn translate(&mut self, from: u16, to: u16, root: &[u8], value: Box<[u8]>) -> Result<Box<[u8]>, Untranslated> {
        if from == to {
            return Ok(value);
        }
        let family = self.family();
        let current = self.current();
        let bridge = self.materialize().or(Err(Untranslated {
            wire: bytes::copy_of(family),
            version: current,
            halt: Halt::Room,
        }))?;
        convert(bridge, from, to, root, value)
    }
    fn total(&mut self, to: u16, root: &[u8]) -> Option<bool> {
        // These roots are the built-in families' fixed sending roots.
        if to == self.current() {
            return Some(true);
        }
        self.materialize().ok()?.total(to, root)
    }
}
fn header_version(text: &str) -> u16 {
    let mut version = 0_u16;
    for byte in text.as_bytes() {
        if *byte == b'\n' {
            break;
        }
        if *byte == b' ' {
            version = 0;
        } else if byte.is_ascii_digit() {
            version = version
                .checked_mul(10)
                .expect("kept version fits")
                .checked_add(u16::from(byte.checked_sub(b'0').expect("digit")))
                .expect("kept version fits");
        }
    }
    version
}
fn frame(kind: u16, body: &[u8]) -> Option<Frame> {
    let mut frame = frame_writer(kind, u32::try_from(body.len()).ok()?).ok()?;
    frame.put(body).ok()?;
    frame.finish().ok()
}

fn versions(current: u16, agreed: u16, down: bool) -> (u16, u16) {
    if down { (current, agreed) } else { (agreed, current) }
}
fn version_of(agreed: &[Agreed], family: &[u8]) -> Option<u16> {
    for wire in agreed {
        if wire.name.as_bytes() == family {
            return Some(wire.version);
        }
    }
    None
}
fn convert(bridge: &mut Bridge, from: u16, to: u16, root: &[u8], value: Box<[u8]>) -> Result<Box<[u8]>, Untranslated> {
    match bridge.translate(from, to, root, value) {
        Ok(value) => Ok(value),
        Err(halt) => Err(Untranslated { wire: bytes::copy_of(bridge.family()), version: bridge.current(), halt }),
    }
}
fn direction(kind: u16) -> Option<(u16, &'static [u8])> {
    if (0x100..0x180).contains(&kind) {
        Some((0x100, b"FromWorker"))
    } else if (0x180..0x200).contains(&kind) {
        Some((0x180, b"FromEngine"))
    } else {
        None
    }
}
fn tagged(base: u16, kind: u16, body: &[u8]) -> Option<Box<[u8]>> {
    let mut writer = Writer::new(body.len().checked_add(1)?);
    writer.put(&[u8::try_from(kind.checked_sub(base)?).ok()?]).ok()?;
    writer.put(body).ok()?;
    Some(writer.finish())
}
fn untagged(base: u16, value: Box<[u8]>) -> Option<Frame> {
    let mut reader = Reader::new(&value);
    let kind = base.checked_add(u16::from(reader.u8()?))?;
    let body = value.get(1..)?;
    let mut frame = frame_writer(kind, u32::try_from(body.len()).ok()?).ok()?;
    frame.put(body).ok()?;
    frame.finish().ok()
}
/// Largest kept record bounds; the patcher's buffers grow only as used.
fn capacity(texts: &[&str]) -> Result<Capacity, BridgeError> {
    let mut bytes = 0_u32;
    for text in texts {
        let schema = skein_codec::schema::parse(text.as_bytes()).or(Err(BridgeError::Floor))?;
        for declaration in 0..schema.declarations() {
            bytes = bytes.max(schema.most_bytes(skein_codec::schema::DeclIx::new(declaration)));
        }
    }
    Ok(Capacity {
        versions: u32::try_from(texts.len()).or(Err(BridgeError::Room))?,
        patches: u32::try_from(texts.len().saturating_mul(2)).or(Err(BridgeError::Room))?,
        text: skein_codec::schema::MAX_TEXT,
        bytes,
        nodes: bytes.saturating_mul(8),
    })
}

fn order() -> Untranslated {
    Untranslated { wire: bytes::copy_of(crate::wire::FAMILY.as_bytes()), version: crate::VERSION, halt: Halt::Order }
}

impl Translation {
    fn assignment(
        &mut self,
        record: wire::Assignment,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Assignment, Untranslated> {
        let record = record.into_parts();
        let workspace = self.workspace(record.workspace, agreed, down, limits)?;
        let charter = self.smith(true, b"Charter", record.charter, agreed, down, false)?;
        let transcript = self.window(record.transcript, agreed, down)?;
        let grants = self.grants(record.grants, agreed, down, limits)?;
        wire::Assignment::new(
            limits,
            wire::AssignmentParts {
                task: record.task,
                attempt: record.attempt,
                identity: record.identity,
                workspace,
                save: record.save,
                live: record.live,
                charter,
                transcript,
                answered: record.answered,
                messages: record.messages,
                grants,
            },
        )
        .or(Err(order()))
    }
    fn job(
        &mut self,
        record: wire::Job,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Job, Untranslated> {
        let record = record.into_parts();
        let workspace = self.workspace(record.workspace, agreed, down, limits)?;
        let spec = self.carried(Carried::JobSpec, record.spec, agreed, down)?;
        let grants = self.grants(record.grants, agreed, down, limits)?;
        wire::Job::new(
            limits,
            wire::JobParts {
                task: record.task,
                attempt: record.attempt,
                kind: record.kind,
                spec,
                deadline: record.deadline,
                workspace,
                grants,
            },
        )
        .or(Err(order()))
    }
    fn granted(
        &mut self,
        record: wire::Granted,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Granted, Untranslated> {
        let record = record.into_parts();
        let grant = self.grant(record.grant, agreed, down, limits)?;
        wire::Granted::new(limits, wire::GrantedParts { task: record.task, attempt: record.attempt, grant })
            .or(Err(order()))
    }
    fn read_list(
        &mut self,
        record: wire::ReadList,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::ReadList, Untranslated> {
        let record = record.into_parts();
        let grant = self.grant(record.grant, agreed, down, limits)?;
        wire::ReadList::new(limits, wire::ReadListParts { read: record.read, endpoint: record.endpoint, grant })
            .or(Err(order()))
    }
    fn grant(
        &mut self,
        record: wire::Grant,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Grant, Untranslated> {
        let record = record.into_parts();
        let value = match record.value {
            wire::GrantValue::Application(value) => {
                let value = self.carried(Carried::Grant, bytes::copy_of(value.secret().expose()), agreed, down)?;
                wire::GrantValue::Application(
                    wire::ApplicationValue::new(limits, wire::ApplicationValueParts { secret: Secret::new(value) })
                        .or(Err(order()))?,
                )
            }
            value @ (wire::GrantValue::Credential(_) | wire::GrantValue::Program(_)) => value,
        };
        wire::Grant::new(
            limits,
            wire::GrantParts { number: record.number, generation: record.generation, valid: record.valid, value },
        )
        .or(Err(order()))
    }
    fn workspace_value(
        &mut self,
        record: wire::Workspace,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Workspace, Untranslated> {
        let record = record.into_parts();
        let items = self.carried(Carried::Items, record.items, agreed, down)?;
        wire::Workspace::new(limits, wire::WorkspaceParts { workstream: record.workstream, items }).or(Err(order()))
    }
    fn turn(
        &mut self,
        record: wire::Turn,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Turn, Untranslated> {
        let record = record.into_parts();
        let body = self.smith(false, b"Turn", record.body, agreed, down, false)?;
        wire::Turn::new(
            limits,
            wire::TurnParts {
                task: record.task,
                attempt: record.attempt,
                number: record.number,
                spent: record.spent,
                read: record.read,
                opens_window: record.opens_window,
                refused: record.refused,
                body,
            },
        )
        .or(Err(order()))
    }
    fn answer(
        &mut self,
        record: wire::Answer,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Answer, Untranslated> {
        let record = record.into_parts();
        let ending = self.ending(record.ending, agreed, down, limits)?;
        wire::Answer::new(
            limits,
            wire::AnswerParts {
                task: record.task,
                attempt: record.attempt,
                turns: record.turns,
                spent: record.spent,
                read: record.read,
                refused: record.refused,
                ending,
            },
        )
        .or(Err(order()))
    }
    fn job_answer(
        &mut self,
        record: wire::JobAnswer,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::JobAnswer, Untranslated> {
        let record = record.into_parts();
        let ending = self.job_ending(record.ending, agreed, down, limits)?;
        wire::JobAnswer::new(limits, wire::JobAnswerParts { task: record.task, attempt: record.attempt, ending })
            .or(Err(order()))
    }
    fn ended(
        &mut self,
        record: wire::Ended,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Ended, Untranslated> {
        let record = record.into_parts();
        let outcome = self.smith(true, b"RunResult", record.outcome, agreed, down, false)?;
        let work = self.work(record.work, agreed, down, limits)?;
        wire::Ended::new(limits, wire::EndedParts { outcome, work }).or(Err(order()))
    }
    fn parked(
        &mut self,
        record: wire::Parked,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Parked, Untranslated> {
        let record = record.into_parts();
        let work = self.work(record.work, agreed, down, limits)?;
        wire::Parked::new(limits, wire::ParkedParts { work }).or(Err(order()))
    }
    fn failed(
        &mut self,
        record: wire::Failed,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Failed, Untranslated> {
        let record = record.into_parts();
        let work = self.work(record.work, agreed, down, limits)?;
        let failure = self.failure(record.failure, agreed, down, limits)?;
        wire::Failed::new(limits, wire::FailedParts { failure, detail: record.detail, work, model: record.model })
            .or(Err(order()))
    }
    fn job_finished(
        &mut self,
        record: wire::JobFinished,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::JobFinished, Untranslated> {
        let record = record.into_parts();
        let work = self.work(record.work, agreed, down, limits)?;
        let result = self.carried(Carried::JobResult, record.result, agreed, down)?;
        wire::JobFinished::new(limits, wire::JobFinishedParts { result, work }).or(Err(order()))
    }
    fn job_failed(
        &mut self,
        record: wire::JobFailed,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::JobFailed, Untranslated> {
        let record = record.into_parts();
        let work = self.work(record.work, agreed, down, limits)?;
        let failure = self.job_failure(record.failure, agreed, down, limits)?;
        wire::JobFailed::new(limits, wire::JobFailedParts { failure, detail: record.detail, work }).or(Err(order()))
    }
    fn work(
        &mut self,
        record: wire::Work,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Work, Untranslated> {
        let record = record.into_parts();
        let left = self.optional(Carried::Left, record.left, agreed, down)?;
        let saved = self.optional(Carried::Saved, record.saved, agreed, down)?;
        wire::Work::new(limits, wire::WorkParts { left, saved }).or(Err(order()))
    }
    fn resource(
        &mut self,
        record: wire::Resource,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Resource, Untranslated> {
        let record = record.into_parts();
        let resource = self.optional(Carried::Preparation, record.resource, agreed, down)?;
        wire::Resource::new(limits, wire::ResourceParts { resource }).or(Err(order()))
    }
    fn preparation_value(
        &mut self,
        record: wire::PreparationValue,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::PreparationValue, Untranslated> {
        let record = record.into_parts();
        let value = self.preparation(record.value, agreed, down, limits)?;
        wire::PreparationValue::new(limits, wire::PreparationValueParts { value }).or(Err(order()))
    }
    fn engine(
        &mut self,
        record: wire::FromEngine,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::FromEngine, Untranslated> {
        Ok(match record {
            wire::FromEngine::Assignment(record) => {
                wire::FromEngine::Assignment(self.assignment(record, agreed, down, limits)?)
            }
            wire::FromEngine::Job(record) => wire::FromEngine::Job(self.job(record, agreed, down, limits)?),
            wire::FromEngine::Grant(record) => wire::FromEngine::Grant(self.granted(record, agreed, down, limits)?),
            wire::FromEngine::ReadList(record) => {
                wire::FromEngine::ReadList(self.read_list(record, agreed, down, limits)?)
            }
            record @ (wire::FromEngine::Message(_)
            | wire::FromEngine::Relayed(_)
            | wire::FromEngine::TurnKept(_)
            | wire::FromEngine::TurnBusy(_)
            | wire::FromEngine::Acknowledge(_)
            | wire::FromEngine::Cancel(_)
            | wire::FromEngine::Sources(_)
            | wire::FromEngine::Check(_)
            | wire::FromEngine::SignOut(_)
            | wire::FromEngine::Wires(_)) => record,
        })
    }
    fn worker(
        &mut self,
        record: wire::FromWorker,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::FromWorker, Untranslated> {
        Ok(match record {
            wire::FromWorker::Turn(record) => wire::FromWorker::Turn(self.turn(record, agreed, down, limits)?),
            wire::FromWorker::Answer(record) => wire::FromWorker::Answer(self.answer(record, agreed, down, limits)?),
            wire::FromWorker::JobAnswer(record) => {
                wire::FromWorker::JobAnswer(self.job_answer(record, agreed, down, limits)?)
            }
            record @ (wire::FromWorker::Hello(_)
            | wire::FromWorker::Call(_)
            | wire::FromWorker::Fact(_)
            | wire::FromWorker::Unauthorized(_)
            | wire::FromWorker::Limited(_)
            | wire::FromWorker::AccountRefused(_)
            | wire::FromWorker::Logins(_)
            | wire::FromWorker::SignedOut(_)
            | wire::FromWorker::ListRead(_)
            | wire::FromWorker::Draining(_)) => record,
        })
    }
    fn ending(
        &mut self,
        record: wire::Ending,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Ending, Untranslated> {
        Ok(match record {
            wire::Ending::Ended(record) => wire::Ending::Ended(self.ended(record, agreed, down, limits)?),
            wire::Ending::Parked(record) => wire::Ending::Parked(self.parked(record, agreed, down, limits)?),
            wire::Ending::Failed(record) => wire::Ending::Failed(self.failed(record, agreed, down, limits)?),
            record @ wire::Ending::Refused(_) => record,
        })
    }
    fn job_ending(
        &mut self,
        record: wire::JobEnding,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::JobEnding, Untranslated> {
        Ok(match record {
            wire::JobEnding::Finished(record) => {
                wire::JobEnding::Finished(self.job_finished(record, agreed, down, limits)?)
            }
            wire::JobEnding::Failed(record) => wire::JobEnding::Failed(self.job_failed(record, agreed, down, limits)?),
            record @ wire::JobEnding::Refused(_) => record,
        })
    }
    fn failure(
        &mut self,
        record: wire::Failure,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Failure, Untranslated> {
        Ok(match record {
            wire::Failure::Unprepared(record) => {
                wire::Failure::Unprepared(self.preparation_value(record, agreed, down, limits)?)
            }
            record @ (wire::Failure::Run(_) | wire::Failure::Agent(_) | wire::Failure::Cancelled(_)) => record,
        })
    }
    fn job_failure(
        &mut self,
        record: wire::JobFailure,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::JobFailure, Untranslated> {
        Ok(match record {
            wire::JobFailure::Unprepared(record) => {
                wire::JobFailure::Unprepared(self.preparation_value(record, agreed, down, limits)?)
            }
            record @ (wire::JobFailure::Failed
            | wire::JobFailure::Deadline
            | wire::JobFailure::Result
            | wire::JobFailure::Unstarted
            | wire::JobFailure::Undelivered(_)
            | wire::JobFailure::Cancelled(_)) => record,
        })
    }
    fn preparation(
        &mut self,
        record: wire::Preparation,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<wire::Preparation, Untranslated> {
        Ok(match record {
            wire::Preparation::Permanent(record) => {
                wire::Preparation::Permanent(self.resource(record, agreed, down, limits)?)
            }
            record @ wire::Preparation::Transient => record,
        })
    }
    fn workspace(
        &mut self,
        record: Option<wire::Workspace>,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<Option<wire::Workspace>, Untranslated> {
        match record {
            Some(record) => Ok(Some(self.workspace_value(record, agreed, down, limits)?)),
            None => Ok(None),
        }
    }
    fn grants(
        &mut self,
        records: List<wire::Grant>,
        agreed: &[Agreed],
        down: bool,
        limits: &wire::Limits,
    ) -> Result<List<wire::Grant>, Untranslated> {
        let mut result = List::with_capacity(records.len());
        for record in records.into_boxed() {
            result.push(self.grant(record, agreed, down, limits)?).expect("same grant count");
        }
        Ok(result)
    }
}
