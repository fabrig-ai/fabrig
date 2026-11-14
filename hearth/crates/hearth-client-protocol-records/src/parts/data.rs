//! Persistent data records (hearth/docs/design/domain/render/spec.md, 1.3).
use crate::form::{json::field, text::Lines};
use hearth_domain_render_vocabulary::parts::data::{Data, DataValue};

pub(crate) fn value_json(this: &DataValue, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"class");
    json.object_start();
    field(json, b"kind", this.class.name());
    json.object_end();
    field(json, b"path", &this.path);
    json.object_end();
}
pub(crate) fn text(this: &Data, lines: &mut Lines) {
    for entry in &this.entries {
        lines.put(b"  ");
        lines.put(&entry.name);
        lines.pad_to(13);
        lines.put(&entry.value.path);
        lines.put(b"\n");
    }
}

pub(crate) fn json(this: &Data, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"entries");
    json.object_start();
    for entry in &this.entries {
        json.key(&entry.name);
        value_json(&entry.value, json);
    }
    json.object_end();
    json.object_end();
}
