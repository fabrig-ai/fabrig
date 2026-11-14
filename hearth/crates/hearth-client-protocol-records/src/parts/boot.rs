//! Boot record forms (hearth/docs/design/domain/render/spec.md, 1.3).
use hearth_domain_render_vocabulary::parts::boot::{Boot, Change};

pub(crate) fn text(this: &Boot, lines: &mut crate::form::text::Lines) {
    if !this.params.is_empty() {
        lines.put(b"  params    ");
        for param in &this.params {
            lines.put(b" ");
            lines.put(param);
        }
        lines.put(b"\n");
    }
    lines.put(b"  mutable    ");
    lines.put(if this.mutable { b"true" } else { b"false" });
    lines.put(b"\n");
}

pub(crate) fn json(this: &Boot, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"mutable");
    json.boolean(this.mutable);
    json.key(b"params");
    json.array_start();
    for param in &this.params {
        json.string(param);
    }
    json.array_end();
    json.object_end();
}

pub(crate) fn change_json(this: &Change, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    crate::form::json::field(json, b"kind", b"boot");
    json.key(b"boot");
    json.object_start();
    json.key(b"after");
    self::json(&this.after, json);
    json.key(b"before");
    self::json(&this.before, json);
    json.object_end();
    json.object_end();
}

pub(crate) fn change_text(this: &Change, lines: &mut crate::form::text::Lines) {
    lines.put(b"  boot     ~ ");
    super::packages::value_text(lines, super::packages::change::Value::Boot(&this.before));
    lines.put(" → ".as_bytes());
    super::packages::value_text(lines, super::packages::change::Value::Boot(&this.after));
    lines.put(b"\n");
}
