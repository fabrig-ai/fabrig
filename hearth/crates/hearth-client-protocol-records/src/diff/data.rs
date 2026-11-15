use crate::form::{json::field, text::Lines};
use hearth_domain_render_vocabulary::parts::data::{DataChange, DataKeyed};

pub(crate) fn text(this: &DataChange, lines: &mut Lines) {
    for entry in &this.entries {
        let (mark, key, value) = match entry {
            DataKeyed::Added { key, value } => (b"+", key, value),
            DataKeyed::Removed { key, value } => (b"-", key, value),
            DataKeyed::Changed { key, after, .. } => (b"~", key, after),
        };
        lines.put(b"  data     ");
        lines.put(mark);
        lines.put(b" ");
        lines.put(key);
        lines.pad_to(52);
        lines.put(value.class.name());
        lines.put(b" ");
        lines.put(&value.path);
        lines.put(b"\n");
    }
}

pub(crate) fn json(this: &DataChange, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    field(json, b"kind", b"data");
    json.key(b"data");
    json.object_start();
    json.key(b"entries");
    json.array_start();
    for entry in &this.entries {
        json.object_start();
        match entry {
            DataKeyed::Added { key, value } => {
                field(json, b"key", key);
                field(json, b"kind", b"added");
                json.key(b"value");
                crate::parts::data::value_json(value, json);
            }
            DataKeyed::Removed { key, value } => {
                field(json, b"key", key);
                field(json, b"kind", b"removed");
                json.key(b"value");
                crate::parts::data::value_json(value, json);
            }
            DataKeyed::Changed { key, before, after } => {
                json.key(b"after");
                crate::parts::data::value_json(after, json);
                json.key(b"before");
                crate::parts::data::value_json(before, json);
                field(json, b"key", key);
                field(json, b"kind", b"changed");
            }
        }
        json.object_end();
    }
    json.array_end();
    json.object_end();
    json.object_end();
}
