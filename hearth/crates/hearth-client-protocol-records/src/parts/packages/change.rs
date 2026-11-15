//! Typed packages differences (hearth/docs/design/domain/render/README.md, 4.1).
use super::{Apt, Initramfs, Lines, Preseed, Sources, Variant, apt, initramfs, names, preseeds, sources, variant};
use hearth_domain_render_vocabulary::parts::packages::{Change, SeedGroupKeyed, SeedIds, SeedUserKeyed, SeedsChange};

pub(crate) fn json(this: &Change, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    crate::form::json::field(json, b"kind", b"packages");
    json.key(b"packages");
    json.object_start();
    if let Some(pair) = &this.apt {
        pair_json(json, b"apt", Value::Apt(&pair.before), Value::Apt(&pair.after));
    }
    if let Some(pair) = &this.initramfs {
        pair_json(json, b"initramfs", Value::Initramfs(pair.before.as_ref()), Value::Initramfs(pair.after.as_ref()));
    }
    if let Some(pair) = &this.preseeds {
        pair_json(json, b"preseeds", Value::Preseeds(pair.before.as_ref()), Value::Preseeds(pair.after.as_ref()));
    }
    if !this.request.added.is_empty() || !this.request.removed.is_empty() {
        json.key(b"request");
        json.object_start();
        json.key(b"added");
        names(json, &this.request.added);
        json.key(b"removed");
        names(json, &this.request.removed);
        json.object_end();
    }
    json.key(b"seeds");
    fixed::json(&this.seeds, json);
    json.key(b"resolution");
    crate::form::resolution_change::json(&this.resolution, json);
    if let Some(pair) = &this.sources {
        pair_json(json, b"sources", Value::Sources(&pair.before), Value::Sources(&pair.after));
    }
    if let Some(pair) = &this.variant {
        pair_json(json, b"variant", Value::Variant(pair.before), Value::Variant(pair.after));
    }
    json.object_end();
    json.object_end();
}

pub(crate) fn text(this: &Change, lines: &mut Lines) {
    crate::form::resolution_change::text(&this.resolution, lines);
    fixed::text(&this.seeds, lines);
    for (sign, names) in [(b"+".as_slice(), &this.request.added), (b"-", &this.request.removed)] {
        if !names.is_empty() {
            lines.put(b"  packages ");
            lines.put(sign);
            for name in names {
                lines.put(b" ");
                lines.put(name);
            }
            lines.put(b"\n");
        }
    }
    if let Some(pair) = &this.sources {
        pair_text(lines, b"sources", Value::Sources(&pair.before), Value::Sources(&pair.after));
    }
    if let Some(pair) = &this.variant {
        pair_text(lines, b"variant", Value::Variant(pair.before), Value::Variant(pair.after));
    }
    if let Some(pair) = &this.preseeds {
        pair_text(lines, b"preseeds", Value::Preseeds(pair.before.as_ref()), Value::Preseeds(pair.after.as_ref()));
    }
    if let Some(pair) = &this.initramfs {
        pair_text(lines, b"initramfs", Value::Initramfs(pair.before.as_ref()), Value::Initramfs(pair.after.as_ref()));
    }
    if let Some(pair) = &this.apt {
        pair_text(lines, b"apt", Value::Apt(&pair.before), Value::Apt(&pair.after));
    }
}
fn pair_json(json: &mut skein_json::writer::Encoder, name: &[u8], before: Value<'_>, after: Value<'_>) {
    json.key(name);
    json.object_start();
    json.key(b"after");
    encode(json, after);
    json.key(b"before");
    encode(json, before);
    json.object_end();
}
fn pair_text(lines: &mut Lines, name: &[u8], before: Value<'_>, after: Value<'_>) {
    lines.put(b"  packages ~ ");
    lines.put(name);
    lines.put(b" ");
    value_text(lines, before);
    lines.put(" → ".as_bytes());
    value_text(lines, after);
    lines.put(b"\n");
}
pub(crate) fn value_text(lines: &mut Lines, value: Value<'_>) {
    let limits = skein_json::writer::Limits { length: u32::MAX, depth: 16 };
    let mut measure = skein_json::writer::Encoder::measure(&limits);
    encode(&mut measure, value);
    let mut json = skein_json::writer::Encoder::write(measure.measured().expect("typed field"), &limits);
    encode(&mut json, value);
    lines.put(&json.finish());
}

pub(crate) mod fixed;

/// The fields printed compactly in a packages or boot change.
#[derive(Clone, Copy)]
pub(crate) enum Value<'a> {
    Apt(&'a Apt),
    Initramfs(Option<&'a Initramfs>),
    Preseeds(&'a [Preseed]),
    Sources(&'a Sources),
    Variant(Variant),
    Boot(&'a hearth_domain_render_vocabulary::parts::boot::Boot),
}
fn encode(json: &mut skein_json::writer::Encoder, value: Value<'_>) {
    match value {
        Value::Apt(value) => apt(json, value),
        Value::Initramfs(value) => initramfs(json, value),
        Value::Preseeds(value) => preseeds(json, value),
        Value::Sources(value) => sources(json, value),
        Value::Variant(value) => variant(json, value),
        Value::Boot(value) => crate::parts::boot::json(value, json),
    }
}
