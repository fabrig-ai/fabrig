//! The last turn's changed sections (temper/docs/design/client/tui/pages/design-session.md, 1, The draft).
use alloc::boxed::Box;
use skein_lib::List;
use temper_client_domain::pages::chat::document::{self, NoteChange};

fn moved(line: u32, first: u32, added: u32, removed: u32) -> u32 {
    if line < first {
        line
    } else if line < first.saturating_add(removed) {
        first
    } else {
        line.saturating_sub(removed).saturating_add(added)
    }
}

/// The sections the writes touched, in document order, each once.
#[must_use]
pub fn marked(body: &[u8], writes: &[NoteChange]) -> Box<[usize]> {
    let mut ranges = List::<(u32, u32)>::uncapped();
    let mut all = false;
    for write in writes {
        match *write {
            NoteChange::First { .. } => {
                ranges.clear();
                all = true;
            }
            NoteChange::Later { added, removed, first } => {
                for (start, end) in ranges.iter_mut() {
                    *start = moved(*start, first, added, removed);
                    *end = moved(*end, first, added, removed);
                }
                ranges.push((first, first.saturating_add(added.saturating_sub(1)))).expect("write ranges fit memory");
            }
        }
    }
    let mut sections = List::uncapped();
    for (first, last) in &ranges {
        sections
            .push((document::section_holding(body, *first), document::section_holding(body, *last)))
            .expect("write sections fit memory");
    }
    let mut result = List::uncapped();
    for (index, _section) in document::sections(body).enumerate() {
        let mut touched = all;
        for (first, last) in &sections {
            if index >= *first && index <= *last {
                touched = true;
            }
        }
        if touched {
            result.push(index).expect("section indexes fit memory");
        }
    }
    result.into_boxed()
}
