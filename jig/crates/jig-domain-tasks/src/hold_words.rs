//! The reason, start and way out of the current hold (domain/tasks/lifecycle-holds.md, 3).

use crate::{Bound, Class, Domain, Hold, HoldSource, Holding, Name, Phase, Status, TaskRecord};
use alloc::boxed::Box;
use skein_lib::{Decimal, Wall, bytes};

/// The most UTF-8 bytes in a hold explanation, including any cut mark.
pub const HOLD_WORDS: usize = 1_024;

/// One hold explanation. A cut mark is part of the words, before the start and way out.
#[derive(Debug)]
pub struct HoldWords {
    pub bytes: Box<[u8]>,
    pub cut: bool,
}

/// Read the same explanation for a chip, a run or an escalation. A row-only reader
/// may lack the related placed hold, dependency stub or current resource holders.
#[must_use]
pub fn hold_words(domain: Option<&Domain>, row: &TaskRecord) -> Option<HoldWords> {
    let Phase::Held { why, source, .. } = &row.phase else { return None };
    let since = row.held_since.unwrap_or(row.phase_since);
    let placed = match (domain, why.placed()) {
        (Some(domain), Some(hold)) => domain.placed_hold(hold),
        (None, _) | (Some(_), None) => None,
    };
    let owner = placed_owner(placed);
    let who = match owner {
        Some((label, number)) => (label, Decimal::of(number)),
        None => (b"".as_slice(), Decimal::of(0)),
    };
    let way = way_out(*why, owner.is_some());
    let number = if owner.is_some() { who.1.as_bytes() } else { b"" };
    let tail: [&[u8]; 6] = [b" Held since ", &minute(since), b". ", who.0, number, way];
    let mut room = HOLD_WORDS;
    for piece in tail {
        room = room.checked_sub(piece.len()).expect("room for hold start and way out");
    }
    let mut note = Note { bytes: [0; HOLD_WORDS], end: 0, room, cut: false };
    cause(&mut note, domain, row, *why, source.as_deref(), placed, since);
    if !note.cut && !note.ends_sentence() {
        if note.end < note.room {
            note.write(b".");
        } else {
            note.mark_cut();
        }
    }
    for piece in tail {
        note.write(piece);
    }
    Some(HoldWords { bytes: bytes::copy_of(note.bytes.get(..note.end).expect("written words")), cut: note.cut })
}

fn way_out(why: Hold, named_owner: bool) -> &'static [u8] {
    match why {
        Hold::Exceeded(_) => b"Lighten what was asked, or raise the allowance, then release it.",
        Hold::Failures(_) => b"Release it to try again; amend it first if its instructions failed it.",
        Hold::Stalled => b"Release it to run again.",
        Hold::Dependency(_) => b"Amend it to drop the dependency, then release it.",
        Hold::Stopped => b"Release it.",
        Hold::Placed { .. } if named_owner => b" lifts the hold or releases this task from it.",
        Hold::Placed { .. } => b"Its owner lifts the hold or releases this task from it.",
        Hold::Drift => b"Release it as the connector directs.",
        Hold::Effects | Hold::EffectFailed => b"Release it for its executor to decide afresh.",
        Hold::Uncertain { .. } => b"Look at the system, then release it, making the effect again or not.",
        Hold::Budget => b"Widen its budget, then release it.",
        Hold::Deadline => b"Move its deadline, then release it.",
        Hold::HoldsWaited => b"Release it to queue again, or amend it to drop the holdings.",
        Hold::ResourceUnavailable | Hold::ResourceAccess => {
            b"Release it once access is back, or amend it to drop the resource."
        }
        Hold::PoolLost { .. } => b"Release it to wait for a slot.",
        Hold::Procedure => b"Release it to reset its counts.",
        Hold::Circle => b"It lifts once the circle breaks; amend a task on it to drop a dependency.",
        Hold::Waited => b"Release it to wait again, or amend or cancel it.",
    }
}

#[expect(clippy::too_many_lines, reason = "one cause for each held phase, before the common start and way out")]
fn cause(
    note: &mut Note,
    domain: Option<&Domain>,
    row: &TaskRecord,
    why: Hold,
    source: Option<&HoldSource>,
    placed: Option<&crate::PlacedHold>,
    since: Wall,
) {
    match why {
        Hold::Exceeded(value) => {
            let (bound, unit): (&[u8], &[u8]) = match value.bound {
                Bound::Reasoning => (b"model reasoning", b" bytes"),
                Bound::Metadata => (b"model metadata", b" bytes"),
                Bound::Items => (b"model answer items", b" items"),
                Bound::Answer => (b"model answer", b" bytes"),
                Bound::Tools => (b"offered tools", b" tools"),
                Bound::HistoryItems => (b"request history items", b" items"),
                Bound::History => (b"retained history", b""),
                Bound::Window => (b"model window", b" tokens"),
                Bound::Output => (b"model output", b" tokens"),
            };
            note.put(b"It reached its ");
            note.put(bound);
            note.put(b" bound of ");
            note.number(value.value);
            note.put(unit);
        }
        Hold::Failures(class) => {
            note.put(b"It ran out of tries for ");
            note.put(class_name(class));
            note.put(b" failures");
            if source.is_none()
                && let Some(crate::RetryNotice::Failed { class: last, reason: Some(words), .. }) = &row.retry_notice
                && *last == class
            {
                note.put(b": ");
                note.put(words);
            }
        }
        Hold::Stalled => note.put(b"Its executor made no progress within its stall bound"),
        Hold::Dependency(task) => {
            note.put(b"Dependency task ");
            note.number(task);
            let status = match domain {
                Some(domain) => match domain.stub(task) {
                    Some(stub) => Some(stub.phase),
                    None => None,
                },
                None => None,
            };
            note.put(match status {
                Some(Status::Failed) => b" failed",
                Some(Status::Cancelled) => b" was cancelled",
                Some(Status::Done) | None => b" failed or was cancelled",
            });
        }
        Hold::Stopped => note.put(b"It was stopped"),
        Hold::Placed { hold } => placed_cause(note, hold, placed),
        Hold::Drift => {
            note.put(b"A resource drifted");
            if let Some(HoldSource::Resource(resource)) = source {
                note.put(b": ");
                note.name(resource);
            }
        }
        Hold::Effects | Hold::EffectFailed => {
            note.put(b"An effect failed for good");
            if let Some(entry) = effect_entry(source) {
                note.put(b" at entry ");
                note.number(entry);
            }
        }
        Hold::Uncertain { entry } => {
            note.put(b"Effect entry ");
            note.number(entry);
            note.put(b" may or may not have happened on its system");
        }
        Hold::Budget => {
            note.put(b"It spent ");
            note.number(row.numbers.spent.saturating_add(row.numbers.spent_below));
            note.put(b"; its next run or effect cannot be funded");
        }
        Hold::Deadline => {
            note.put(b"Its deadline");
            if let Some(deadline) = crate::domain::deadline(row, since) {
                note.put(b" ");
                note.put(&minute(deadline));
            }
            note.put(b" passed");
        }
        Hold::HoldsWaited => queue_cause(note, domain, row),
        Hold::ResourceUnavailable | Hold::ResourceAccess => {
            note.put(if why == Hold::ResourceUnavailable {
                b"A resource is unavailable"
            } else {
                b"Access to a resource was refused"
            });
        }
        Hold::PoolLost { pool } => {
            note.put(b"It lost its allocation in pool ");
            note.number(pool);
        }
        Hold::Procedure => note.put(b"Its procedure asked for a decision"),
        Hold::Circle => {
            note.put(b"A circle of waits among tasks");
            if let Some(HoldSource::Circle(circle)) = source {
                let mut separator = b" ".as_slice();
                for &task in &circle.tasks {
                    if note.cut {
                        break;
                    }
                    note.put(separator);
                    note.number(task);
                    separator = b" -> ";
                }
                if circle.cut {
                    note.mark_cut();
                }
            }
        }
        Hold::Waited => {
            if let Some(HoldSource::Wait { words }) = source {
                note.put(words);
            } else {
                note.put(b"A wait reached its bound");
            }
        }
    }
    match source {
        Some(HoldSource::Preparation { words } | HoldSource::Account { words, .. }) => {
            note.put(b": ");
            note.put(words);
        }
        Some(HoldSource::Wait { words }) if why == Hold::Deadline => {
            note.put(b". ");
            note.put(words);
        }
        Some(HoldSource::Wait { .. } | HoldSource::Resource(_) | HoldSource::Effect { .. } | HoldSource::Circle(_))
        | None => {}
    }
}

fn class_name(class: Class) -> &'static [u8] {
    match class {
        Class::Transient => b"transient",
        Class::Permanent => b"permanent",
        Class::Run => b"run",
        Class::Agent => b"agent",
        Class::Lost => b"lost worker",
        Class::Invalid => b"invalid result",
        Class::Infrastructure => b"infrastructure",
    }
}

fn effect_entry(source: Option<&HoldSource>) -> Option<u64> {
    match source {
        Some(HoldSource::Effect { entry, .. }) => Some(*entry),
        Some(HoldSource::Account { entry, .. }) => *entry,
        Some(
            HoldSource::Preparation { .. } | HoldSource::Resource(_) | HoldSource::Circle(_) | HoldSource::Wait { .. },
        )
        | None => None,
    }
}

fn placed_owner(placed: Option<&crate::PlacedHold>) -> Option<(&'static [u8], u64)> {
    match placed?.owner {
        crate::Owner::Party { person } => Some((b"Person ", person)),
        crate::Owner::Task { task } => Some((b"Task ", task)),
    }
}

fn placed_cause(note: &mut Note, hold: u64, placed: Option<&crate::PlacedHold>) {
    note.put(b"Hold ");
    note.number(hold);
    note.put(b" was placed");
    if let Some(placed) = placed {
        if placed.reason.is_empty() {
            note.put(b" with no reason");
        }
        match placed.owner {
            crate::Owner::Party { person } => {
                note.put(b" by person ");
                note.number(person);
            }
            crate::Owner::Task { task } => {
                note.put(b" by task ");
                note.number(task);
            }
        }
        if !placed.reason.is_empty() {
            note.put(b": ");
            note.put(&placed.reason);
        }
    }
}

fn queue_cause(note: &mut Note, domain: Option<&Domain>, row: &TaskRecord) {
    note.put(b"Its resource queue reached its bound");
    if let Some(since) = row.hold_wait_since {
        note.put(b" after queuing since ");
        note.put(&minute(since));
    }
    let mut separator = b": ".as_slice();
    for holding in &row.holdings {
        if note.cut {
            break;
        }
        note.put(separator);
        match holding {
            Holding::Write { resource, .. } => {
                note.put(b"resource ");
                note.name(resource);
            }
            Holding::Slot { pool, .. } => {
                note.put(b"pool ");
                note.name(pool);
            }
        }
        if let Some(domain) = domain
            && let Some(holder) = crate::holds::holder(domain, holding)
            && holder != row.number
        {
            note.put(b" (held by task ");
            note.number(holder);
            note.put(b")");
        }
        separator = b", ";
    }
}

fn minute(wall: Wall) -> [u8; 17] {
    let second = wall.utc_second();
    let mut minute = *b"0000-00-00T00:00Z";
    for (slot, byte) in minute.get_mut(..16).expect("minute fields").iter_mut().zip(second) {
        *slot = byte;
    }
    minute
}

// Opaque values and values stored without validation may contain invalid UTF-8.
// Check only the part that can fit; on a valid input the most a character can
// overhang its room is three bytes.
fn text_prefix(text: &[u8], room: usize) -> (&[u8], bool) {
    let full = room.min(text.len());
    for back in 0..=3 {
        let length = full.saturating_sub(back);
        let prefix = text.get(..length).expect("within input");
        if bytes::is_utf8(prefix) {
            return (prefix, true);
        }
    }
    (text.get(..full).expect("within input"), false)
}

struct Note {
    bytes: [u8; HOLD_WORDS],
    end: usize,
    room: usize,
    cut: bool,
}

const CUT: &[u8] = "…".as_bytes();

impl Note {
    fn write(&mut self, bytes: &[u8]) {
        let end = self.end.checked_add(bytes.len()).expect("hold text length");
        for (slot, &byte) in self.bytes.get_mut(self.end..end).expect("hold text fits").iter_mut().zip(bytes) {
            *slot = byte;
        }
        self.end = end;
    }

    fn put(&mut self, text: &[u8]) {
        if self.cut {
            return;
        }
        let available = self.room.saturating_sub(self.end);
        let mut room = available;
        if text.len() > available {
            let before_mark = self.room.saturating_sub(CUT.len());
            self.trim(before_mark);
            room = before_mark.saturating_sub(self.end);
        }
        let (prefix, valid) = text_prefix(text, room);
        if valid {
            self.write(prefix);
        } else {
            self.write(&bytes::utf8_replaced(prefix));
        }
        if prefix.len() < text.len() {
            self.mark_cut();
        }
    }

    fn trim(&mut self, maximum: usize) {
        let written = self.bytes.get(..self.end).expect("written words");
        self.end = bytes::utf8_prefix(written, maximum).len();
    }

    fn mark_cut(&mut self) {
        if !self.cut {
            self.trim(self.room.saturating_sub(CUT.len()));
            self.write(CUT);
            self.cut = true;
        }
    }

    fn ends_sentence(&self) -> bool {
        let text = self.bytes.get(..self.end).expect("written words");
        if let Some(&last) = text.last() { last == b'.' || last == b'!' || last == b'?' } else { false }
    }

    fn number(&mut self, number: u64) {
        self.put(Decimal::of(number).as_bytes());
    }

    fn name(&mut self, name: &Name) {
        self.put(b"connector ");
        self.number(u64::from(name.connector));
        for segment in &name.path {
            if self.cut {
                break;
            }
            self.put(b"/");
            self.put(segment);
        }
    }
}
