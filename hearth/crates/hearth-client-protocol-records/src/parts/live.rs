//! Live consumers (hearth/docs/design/domain/render/spec.md, 1.4).
use crate::form::text::Lines;
use hearth_domain_render_vocabulary::parts::live::{Change, Consumer, ConsumersKeyed, Live};

pub(crate) fn text(this: &Live, lines: &mut Lines) {
    for covered in &this.paths {
        for consumer in &covered.consumers {
            lines.put(b"  path");
            lines.pad_to(13);
            lines.put(&covered.path);
            lines.put(b" ");
            consumer_text(consumer, lines);
            lines.put(b"\n");
        }
    }
}

pub(crate) fn json(this: &Live, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    for covered in &this.paths {
        json.key(&covered.path);
        json.array_start();
        for consumer in &covered.consumers {
            consumer_json(consumer, json);
        }
        json.array_end();
    }
    json.object_end();
}

pub(crate) fn consumer_text(this: &Consumer, lines: &mut Lines) {
    lines.put(&this.task);
    lines.put(b" ");
    lines.put(this.unit.as_deref().unwrap_or(b"-"));
    lines.put(b" ");
    lines.put(this.action.name());
}

pub(crate) fn consumer_json(this: &Consumer, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"action");
    json.string(this.action.name());
    json.key(b"task");
    json.string(&this.task);
    if let Some(unit) = &this.unit {
        json.key(b"unit");
        json.string(unit);
    }
    json.object_end();
}

type ConsumerSides<'a> = (&'a [u8], Option<&'a [Consumer]>, Option<&'a [Consumer]>);
pub(crate) fn consumers_sides(this: &ConsumersKeyed) -> ConsumerSides<'_> {
    match this {
        ConsumersKeyed::Added { key, value } => (key, None, Some(value.as_ref())),
        ConsumersKeyed::Removed { key, value } => (key, Some(value.as_ref()), None),
        ConsumersKeyed::Changed { key, before, after } => (key, Some(before.as_ref()), Some(after.as_ref())),
    }
}
pub(crate) fn change_text(this: &Change, lines: &mut Lines) {
    for entry in &this.paths {
        let (key, before, after) = consumers_sides(entry);
        for consumer in after.or(before).expect("changed path") {
            crate::diff::placed::header(lines, b"live", key, before.is_some(), after.is_some());
            lines.pad_to(52);
            consumer_text(consumer, lines);
            lines.put(b"\n");
        }
    }
}

pub(crate) fn change_json(this: &Change, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    crate::form::json::field(json, b"kind", b"live");
    json.key(b"live");
    json.object_start();
    json.key(b"paths");
    json.array_start();
    for entry in &this.paths {
        let (key, before, after) = consumers_sides(entry);
        crate::diff::placed::keyed(json, key, optional_consumers(before), optional_consumers(after));
    }
    json.array_end();
    json.object_end();
    json.object_end();
}
pub(crate) fn consumers(values: &[Consumer], json: &mut skein_json::writer::Encoder) {
    json.array_start();
    for value in values {
        consumer_json(value, json);
    }
    json.array_end();
}

fn optional_consumers(value: Option<&[Consumer]>) -> Option<crate::diff::placed::Value<'_>> {
    Some(crate::diff::placed::Value::Consumers(value?))
}
