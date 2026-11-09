//! The person's own preferences (temper/docs/design/client/README.md, 5.17).
use super::{Request, activity::Asking};
use crate::owner::Owner;
use alloc::boxed::Box;
use jig_client::{Answered, Client, Handle, Key, Outcome, Told, contract as c};
use skein_lib::{List, Queue, Time, Wall};

/// The person's own preferences (client/README.md, 5.17).
#[derive(Debug)]
pub struct Remembered {
    party: Option<u64>,
    notes: List<c::Note>,
    reading: List<c::Note>,
    pending_read: Option<Handle>,
    asking: Asking,
    wanted: bool,
    answered: bool,
    read_cut: bool,
    field: Option<(u64, Box<[u8]>)>,
    pending_act: Option<(Key, u64)>,
    refusal: Option<(u64, Refusal)>,
}

/// A borrowed row, in the scope's order; a task's first reference names its chat.
#[derive(Debug, PartialEq, Eq)]
pub enum Row<'a> {
    Task { place: u32, note: u64, line: &'a [u8], chat: Option<u64>, changed: Option<Wall> },
    Party { place: u32, note: u64, line: &'a [u8], party: u64, changed: Option<Wall> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Change { note: u64 },
    Words { words: Box<[u8]> },
    Save,
    Cancel,
    Delete { note: u64 },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Moved,
    TooLong { bound: u32 },
    Other(c::Refusal),
}
impl Remembered {
    pub(super) fn new() -> Self {
        Self {
            party: None,
            notes: List::uncapped(),
            reading: List::uncapped(),
            pending_read: None,
            asking: Asking::new(),
            wanted: true,
            answered: false,
            read_cut: false,
            field: None,
            pending_act: None,
            refusal: None,
        }
    }
    /// None until me supplies the person's number; no scope is read without it.
    #[must_use]
    pub const fn party(&self) -> Option<u64> {
        self.party
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "the contracted borrowed Row iterator projects notes without retaining borrowed state"
    )]
    pub fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        self.notes.iter().enumerate().map(|(index, note)| {
            let place = u32::try_from(index).expect("note rows fit").checked_add(1).expect("note places fit");
            match note.writer {
                c::NoteAuthor::Task { .. } => Row::Task {
                    place,
                    note: note.number,
                    line: note.description.bytes(),
                    chat: note.tasks.get(0).copied(),
                    changed: note.changed,
                },
                c::NoteAuthor::Party { party } => Row::Party {
                    place,
                    note: note.number,
                    line: note.description.bytes(),
                    party,
                    changed: note.changed,
                },
            }
        })
    }
    #[must_use]
    pub fn field(&self) -> Option<(u64, &[u8])> {
        let (note, words) = self.field.as_ref()?;
        Some((*note, words))
    }
    #[must_use]
    pub fn refusal(&self) -> Option<(u64, &Refusal)> {
        let (note, refusal) = self.refusal.as_ref()?;
        Some((*note, refusal))
    }
    #[must_use]
    pub const fn answered(&self) -> bool {
        self.answered
    }
    pub(super) fn configure(&mut self, me: Option<&c::Me>, next: &mut u64, out: &mut Queue<Request>) {
        let Some(me) = me else { return };
        if self.party != Some(me.party.number) {
            *self = Self::new();
            self.party = Some(me.party.number);
        }
        if !self.wanted || self.pending_read.is_some() {
            return;
        }
        let handle = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        self.pending_read = Some(handle);
        self.wanted = false;
        self.asking.ask(
            handle,
            c::Read::Notes { scope: c::NoteScope::Party { party: me.party.number }, description: None, after: None },
            out,
        );
    }
    pub(super) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if let Some(handle) = self.pending_read {
            self.asking.fire(now, handle, out);
        }
    }
    pub(super) const fn next_deadline(&self) -> Option<Time> {
        self.asking.next_deadline()
    }
    fn refresh(&mut self) {
        self.asking.answered();
        self.pending_read = None;
        self.reading.clear();
        self.answered = false;
        self.read_cut = false;
        self.wanted = true;
    }
    pub(super) fn told(&mut self, now: Time, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
        if let Told::Page { handle, page } = told
            && self.pending_read == Some(*handle)
        {
            if let jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) = page {
                self.read_cut |= page.cut;
                for note in &page.rows {
                    self.reading.push(note.clone()).expect("uncapped notes");
                }
                self.asking.answered();
                if let Some(after) = &page.after {
                    self.asking.ask(
                        *handle,
                        c::Read::Notes {
                            scope: c::NoteScope::Party { party: self.party.expect("reading own scope") },
                            description: None,
                            after: Some(after.clone()),
                        },
                        out,
                    );
                } else {
                    if !self.read_cut {
                        self.notes = core::mem::replace(&mut self.reading, List::uncapped());
                    }
                    self.pending_read = None;
                    self.answered = !self.read_cut;
                }
            } else if super::activity::read_end(client, page).transient() {
                self.asking.ended(now);
            } else {
                self.pending_read = None;
                self.asking.answered();
            }
            return true;
        }
        if let Told::Answered { key, outcome } = told
            && let Some((pending, note)) = self.pending_act
            && *key == pending
        {
            self.pending_act = None;
            self.field = None;
            if let Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal, .. })) = outcome {
                self.refusal = Some((note, refusal_for(refusal)));
            }
            self.refresh();
            return true;
        }
        false
    }
    #[expect(clippy::manual_find, reason = "step code searches notes without a closure")]
    fn note(&self, number: u64) -> Option<&c::Note> {
        for note in &self.notes {
            if note.number == number {
                return Some(note);
            }
        }
        None
    }
    pub(super) fn act(&mut self, act: Act, serial: &mut u64, out: &mut Queue<Request>) {
        if self.pending_act.is_some() {
            return;
        }
        self.refusal = None;
        let (note, ask) = match act {
            Act::Change { note } => {
                for row in &self.notes {
                    if row.number == note {
                        self.field = Some((note, row.description.bytes().into()));
                        return;
                    }
                }
                return;
            }
            Act::Words { words } => {
                if let Some((_, field)) = &mut self.field {
                    *field = words;
                }
                return;
            }
            Act::Cancel => {
                self.field = None;
                return;
            }
            Act::Save => {
                let Some((note, words)) = &self.field else { return };
                let words = words.trim_ascii();
                if words.is_empty() {
                    return;
                }
                let Some(row) = self.note(*note) else { return };
                let Ok(description) = c::Text::new(words.into(), u32::MAX) else { return };
                (
                    *note,
                    c::Ask::EditNote {
                        project: 0,
                        note: *note,
                        scope: c::NoteScope::Party { party: self.party.expect("a read row has its party") },
                        revision: row.revision,
                        description,
                        body: c::Text::new(Box::default(), 0).expect("empty body"),
                        references: row.tasks.clone(),
                    },
                )
            }
            Act::Delete { note } => {
                let Some(row) = self.note(note) else { return };
                (
                    note,
                    c::Ask::DeleteNote {
                        project: 0,
                        note,
                        scope: c::NoteScope::Party { party: self.party.expect("a read row has its party") },
                        revision: row.revision,
                    },
                )
            }
        };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending_act = Some((key, note));
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
    }
}
fn refusal_for(refusal: &c::Refusal) -> Refusal {
    if let c::Refusal::Moved { .. } = refusal {
        return Refusal::Moved;
    }
    if let c::Refusal::Limit { which: c::Limit::Words { limit } } = refusal {
        return Refusal::TooLong { bound: *limit };
    }
    if let c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::Words } } = refusal {
        // jig/docs/design/domain/engine.md, 10.1: admission names no bound; notes.party_bytes defaults to 120.
        return Refusal::TooLong { bound: 120 };
    }
    Refusal::Other(refusal.clone())
}
