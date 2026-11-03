//! Source-text chunks and continuations (tools contract, 6.4).
use super::{Limits, Text};
use jig_client::contract as c;
use skein_json::writer::Encoder;

fn field(value: c::TaskTextField) -> &'static [u8] {
    match value {
        c::TaskTextField::Specification => b"specification",
        c::TaskTextField::Instructions => b"instructions",
    }
}

pub(super) fn call(task: u64, value: c::TaskTextField, from: u64, out: &mut Text) {
    out.put(b"task text ");
    out.number(task);
    out.put(b" --field ");
    out.put(field(value));
    out.put(b" --from ");
    out.number(from);
}

fn end(page: &c::TaskText, limits: &Limits) -> usize {
    // JSON can require six bytes for one source byte; reserve room for its envelope.
    let room = limits.result_bytes.saturating_sub(512).checked_div(6).expect("nonzero divisor");
    let room = u64::from(limits.part.min(limits.text.saturating_sub(512))).min(room);
    skein_lib::bytes::utf8_prefix(page.text.bytes(), usize::try_from(room).expect("text bound fits")).len()
}

pub(super) fn fits(page: &c::TaskText, limits: &Limits) -> bool {
    end(page, limits) > 0 || page.from == page.total
}

fn next(page: &c::TaskText, end: usize) -> Option<u64> {
    let offset = page.from.checked_add(u64::try_from(end).expect("chunk length fits")).expect("source bound");
    if offset < page.total { Some(offset) } else { None }
}

pub(super) fn text(page: &c::TaskText, limits: &Limits, out: &mut Text) {
    let end = end(page, limits);
    out.put(field(page.field));
    out.put(b" from ");
    out.number(page.from);
    out.put(b" of ");
    out.number(page.total);
    out.put(b" bytes:\n");
    out.words(page.text.bytes().get(..end).expect("chunk prefix"));
    out.put(b"\n");
    match next(page, end) {
        Some(offset) => {
            out.number(page.total.checked_sub(offset).expect("source bound"));
            out.put(b" bytes left; read on with: ");
            call(page.task, page.field, offset, out);
            out.put(b"\n");
        }
        None => out.put(b"no more\n"),
    }
}

pub(super) fn encode(page: &c::TaskText, limits: &Limits, out: &mut Encoder) {
    let end = end(page, limits);
    out.object_start();
    out.key(b"task");
    out.unsigned(page.task);
    out.key(b"field");
    out.string(field(page.field));
    out.key(b"from");
    out.unsigned(page.from);
    out.key(b"total");
    out.unsigned(page.total);
    out.key(b"text");
    out.string(page.text.bytes().get(..end).expect("chunk prefix"));
    out.key(b"next");
    match next(page, end) {
        Some(offset) => out.unsigned(offset),
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn summary(task: &c::Task, out: &mut Text) {
    if task.spec.specification.words.cut {
        out.put(b"\nread specification: ");
        call(task.chip.task, c::TaskTextField::Specification, 0, out);
    }
    if let Some(words) = &task.instructions
        && words.cut
    {
        out.put(b"\nread instructions: ");
        call(task.chip.task, c::TaskTextField::Instructions, 0, out);
    }
}

pub(super) fn reads(task: &c::Task, out: &mut Encoder) {
    out.key(b"text_reads");
    out.object_start();
    for value in [c::TaskTextField::Specification, c::TaskTextField::Instructions] {
        let mut measure = Text::measure();
        call(task.chip.task, value, 0, &mut measure);
        let mut text = Text::writing_exact(measure);
        call(task.chip.task, value, 0, &mut text);
        out.key(field(value));
        out.string(&text.finish());
    }
    out.object_end();
}

/// Print retrieved asked words and a continuation from the first undisplayed source byte.
pub(super) fn asked(task: &c::Task, source: Option<&crate::jig::runner::specification::Specification>, out: &mut Text) {
    let Some(source) = source else {
        out.words(task.spec.specification.words.bytes());
        return;
    };
    let end = skein_lib::bytes::utf8_prefix(source.bytes.as_slice(), out.room()).len();
    out.words(source.bytes.as_slice());
    if source.more || end < source.bytes.as_slice().len() {
        out.put(b"\n");
        if let Some(total) = source.total {
            out.number(total.saturating_sub(u64::try_from(end).expect("source bound")));
            out.put(b" bytes left; ");
        }
        out.put(b"read on with: ");
        call(task.chip.task, c::TaskTextField::Specification, u64::try_from(end).expect("source bound"), out);
    }
}
