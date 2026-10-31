//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{Limits, List, Reader, Wall, c, cursor_fields, offered_piece, piece_problem, too_large, w};

pub(crate) fn note_author_fields(value: &w::NoteAuthor) -> c::NoteAuthor {
    match value {
        w::NoteAuthor::Party(row) => c::NoteAuthor::Party { party: row.party() },
        w::NoteAuthor::Task(row) => c::NoteAuthor::Task { task: row.task(), attempt: row.attempt() },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn note_fields(value: &w::Note, limits: &Limits) -> Result<c::Note, w::Problem> {
    if value.tasks().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.tasks().len());
    for item in value.tasks() {
        list0.push(*item).expect("reserved supplied list");
    }
    let opt1 = match value.changed() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::Note {
        number: value.number(),
        revision: crate::translate::answers::revision(value.revision()),
        scope: crate::translate::answers::note_scope(value.scope()),
        description: crate::translate::facts::document_text(value.description(), limits)?,
        body: crate::translate::facts::document_text(value.body(), limits)?,
        writer: note_author_fields(value.writer()),
        tasks: list0,
        changed: opt1,
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        cut: value.cut(),
    })
}

pub(crate) fn note_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Note>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::Note::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = note_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn note_page_fields(value: &w::NotePage, limits: &Limits) -> Result<c::NotePage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::NotePage {
        cut: value.cut(),
        rows: note_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}
