//! Bounded in-memory notes, separate from the server's routes (examples.md, 9.3).
#![forbid(unsafe_code)]
use skein_lib::{List, Reader};
use skein_ui_example_view::records::{Add, CEILINGS, Note, Notes, Record};
use std::collections::BTreeMap;
pub mod bundle;
pub mod routes;

#[derive(Debug)]
pub struct State {
    notes: Notes,
    adds: BTreeMap<u64, Note>,
    made: Option<Box<[u8]>>,
}
impl State {
    #[must_use]
    pub fn new() -> State {
        State { adds: BTreeMap::new(), made: None, notes: Notes { notes: List::with_capacity(CEILINGS.notes_notes) } }
    }
    pub fn add(&mut self, body: &[u8]) -> Result<Box<[u8]>, u16> {
        let add = Add::decode(&CEILINGS, &mut Reader::new(body)).map_err(|_| 400_u16)?;
        let mut made = false;
        let note = if let Some(note) = self.adds.get(&add.key) {
            note.clone()
        } else {
            if self.notes.notes.room() == 0 {
                return Err(409);
            }
            let number = u64::from(self.notes.notes.len()).checked_add(1).expect("64 notes fit");
            let note =
                Note { number, title: add.title, body: add.body, due: add.due, minutes: add.minutes, label: add.label };
            self.notes.notes.push(note.clone()).expect("note capacity checked");
            self.adds.insert(add.key, note.clone());
            made = true;
            note
        };
        let bytes = note.encode(&CEILINGS).expect("add and note have the same bounds");
        if made {
            self.made = Some(bytes.clone());
        }
        Ok(bytes)
    }
    pub fn take_made(&mut self) -> Option<Box<[u8]>> {
        self.made.take()
    }
    #[must_use]
    pub fn list(&self) -> Box<[u8]> {
        self.notes.encode(&CEILINGS).expect("bounded notes")
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests;
