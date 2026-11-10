//! Whole scoped notes (jig/docs/design/client/domain/contract/README.md, 4).
use super::{Limits, Printed, Text};
use crate::jig::call::{Call, Project, Value, number, value};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::Map;

pub(super) fn print(page: &c::NotePage, call: &Call, json: bool, limits: &Limits) -> Printed {
    let mut rows = Map::with_capacity(page.rows.len());
    for note in &page.rows {
        rows.insert(note.number, note).expect("one entry per supplied note");
    }
    let stdout = if json {
        let bounds = JsonLimits { depth: 4, length: u32::try_from(limits.result_bytes).expect("output bound fits") };
        let mut measure = Encoder::measure(&bounds);
        encode(&rows, page.after.as_ref(), &mut measure);
        let Ok(size) = measure.measured() else { return super::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        encode(&rows, page.after.as_ref(), &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(&rows, page.after.as_ref(), call, &mut measure);
        if skein_lib::len_u64(measure.structural()) > limits.result_bytes {
            return super::failure(b"result limit", false, 1);
        }
        let mut out = Text::writing_exact(measure);
        text(&rows, page.after.as_ref(), call, &mut out);
        out.finish()
    };
    Printed { file: crate::jig::FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
}

fn text(rows: &Map<u64, &c::Note>, after: Option<&c::Cursor>, call: &Call, out: &mut Text) {
    if rows.is_empty() {
        out.put(b"no notes\n");
    }
    for (_, note) in rows {
        out.put(b"note ");
        out.number(note.number);
        out.put(b" revision ");
        out.number(note.revision.value);
        out.put(b": ");
        out.put(note.description.bytes());
        out.put(b"\n");
        out.put(note.body.bytes());
        if !note.body.bytes().ends_with(b"\n") {
            out.put(b"\n");
        }
        if note.body.cut {
            out.put(b"body cut; report it\n");
        }
    }
    if let Some(after) = after {
        out.put(b"read on with: task notes ");
        out.number(number(call, 0).expect("task argument"));
        out.put(b" --project ");
        let Some(Value::Project(project)) = value(call, 1) else { unreachable!("project argument") };
        match project {
            Project::Number(project) => out.number(u64::from(*project)),
            Project::Name(project) => super::lists::argument(project, out),
        }
        out.put(b" --after ");
        let cursor = crate::jig::argv::value::encode(
            &[],
            crate::jig::table::Kind::Cursor,
            &Value::Cursor(after.value.bytes().into()),
        )
        .expect("bounded cursor hexadecimal");
        out.put(&cursor);
        out.put(b"\n");
    }
}

fn encode(rows: &Map<u64, &c::Note>, after: Option<&c::Cursor>, out: &mut Encoder) {
    out.object_start();
    out.key(b"notes");
    out.array_start();
    for (_, note) in rows {
        out.object_start();
        out.key(b"number");
        out.unsigned(note.number);
        out.key(b"revision");
        out.unsigned(note.revision.value);
        out.key(b"description");
        out.string(note.description.bytes());
        out.key(b"body");
        out.string(note.body.bytes());
        out.key(b"cut");
        out.boolean(note.body.cut);
        out.object_end();
    }
    out.array_end();
    out.key(b"after");
    match after {
        Some(cursor) => super::document::hex(cursor.value.bytes(), out),
        None => out.null(),
    }
    out.object_end();
}
