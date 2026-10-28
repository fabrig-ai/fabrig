//! Directory retention shares a file read's byte budget.
use crate::api::{self, Entry, EntryKind};
use alloc::boxed::Box;
use skein_lib::List;

pub(crate) fn kind(text: &[u8]) -> Option<EntryKind> {
    match text {
        b"file" | b"symlink" => Some(EntryKind::File),
        b"dir" => Some(EntryKind::Directory),
        b"submodule" => Some(EntryKind::Submodule),
        _ => None,
    }
}
#[derive(Debug)]
pub(crate) struct Listing {
    entries: List<Entry>,
    room: u32,
    truncated: bool,
}
impl Listing {
    pub(crate) fn new(room: u32) -> Self {
        let width = u32::try_from(crate::measure::ENTRY_BYTES).expect("entry width fits");
        Self {
            entries: List::with_capacity(room.checked_div(width).expect("nonzero entry width")),
            room,
            truncated: false,
        }
    }
    pub(crate) fn keep(&mut self, name: Box<[u8]>, kind: Option<EntryKind>) {
        if self.truncated {
            return;
        }
        let Some(kind) = kind else { return };
        let Some(left) = remaining(self.room, &name) else {
            self.truncated = true;
            return;
        };
        self.entries.push(Entry { name, kind }).expect("capacity follows the byte budget");
        self.room = left;
    }
    pub(crate) fn answer(&mut self, head: api::Commit, path: Box<[u8]>) -> api::Answer {
        api::Answer::Directory {
            head,
            path,
            entries: core::mem::replace(&mut self.entries, List::with_capacity(0)).into_boxed(),
            truncated: self.truncated,
        }
    }
}

pub(crate) fn write(json: &mut skein_json::writer::Encoder, entries: &[Entry]) {
    json.array_start();
    for entry in entries {
        json.object_start();
        json.key(b"name");
        json.string(&entry.name);
        json.key(b"type");
        json.string(match entry.kind {
            EntryKind::File => b"file",
            EntryKind::Directory => b"dir",
            EntryKind::Submodule => b"submodule",
        });
        json.object_end();
    }
    json.array_end();
}

fn remaining(room: u32, name: &[u8]) -> Option<u32> {
    let charge = u32::try_from(name.len().checked_add(crate::measure::ENTRY_BYTES)?).ok()?;
    room.checked_sub(charge)
}
