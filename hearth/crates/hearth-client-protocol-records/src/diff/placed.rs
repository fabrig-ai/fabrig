//! Forms shared by path and seed changes (hearth/docs/design/domain/render/README.md, 4.1).
use crate::form::{json::field, text::Lines};
use skein_json::writer::Encoder;

pub(crate) fn header(lines: &mut Lines, part: &[u8], key: &[u8], before: bool, after: bool) {
    lines.put(b"  ");
    lines.put(part);
    lines.pad_to(11);
    lines.put(match (before, after) {
        (false, true) => b"+ ",
        (true, false) => b"- ",
        (true, true) => b"~ ",
        (false, false) => unreachable!("change has a side"),
    });
    lines.put(key);
}
pub(crate) fn keyed(json: &mut Encoder, key: &[u8], before: Option<Value<'_>>, after: Option<Value<'_>>) {
    json.object_start();
    match (before, after) {
        (Some(before), Some(after)) => {
            json.key(b"after");
            encode(after, json);
            json.key(b"before");
            encode(before, json);
            field(json, b"kind", b"changed");
        }
        (None, Some(value)) => {
            field(json, b"kind", b"added");
            json.key(b"value");
            encode(value, json);
        }
        (Some(value), None) => {
            field(json, b"kind", b"removed");
            json.key(b"value");
            encode(value, json);
        }
        (None, None) => unreachable!("change has a side"),
    }
    field(json, b"key", key);
    json.object_end();
}

/// A keyed row's payload, borrowed only during printing.
#[derive(Clone, Copy)]
pub(crate) enum Value<'a> {
    Download(&'a hearth_domain_render_vocabulary::parts::software::Download),
    Consumers(&'a [hearth_domain_render_vocabulary::parts::live::Consumer]),
    SeedIds(&'a hearth_domain_render_vocabulary::parts::packages::SeedIds),
    Gid(u32),
}
fn encode(value: Value<'_>, json: &mut Encoder) {
    match value {
        Value::Download(value) => crate::parts::software::download_json(value, json),
        Value::Consumers(value) => crate::parts::live::consumers(value, json),
        Value::SeedIds(value) => crate::parts::packages::change::fixed::ids(value, json),
        Value::Gid(value) => json.unsigned(u64::from(value)),
    }
}
