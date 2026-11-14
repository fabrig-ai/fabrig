use crate::form::{
    json::{digest, field},
    text::{Lines, octal},
};
use hearth_domain_render_vocabulary::parts::config::{Action, Line, Script};
use skein_json::writer::Encoder;

pub(crate) fn action_text(this: Action, lines: &mut Lines) {
    lines.put(this.name());
}

pub(crate) fn action_json(this: Action, json: &mut Encoder) {
    json.object_start();
    field(json, b"kind", this.name());
    json.object_end();
}
pub(crate) fn line_text(this: &Line, lines: &mut Lines) {
    lines.put(this.kind.name());
    lines.put(b" ");
    match this.mode {
        Some(mode) => lines.mode(mode),
        None => lines.put(b"-"),
    }
    for value in [&this.owner, &this.group] {
        lines.put(b" ");
        lines.put(value.as_deref().unwrap_or(b"-"));
    }
    if this.age.is_some() || this.argument.is_some() {
        lines.put(b" ");
        lines.put(this.age.as_deref().unwrap_or(b"-"));
    }
    if let Some(argument) = &this.argument {
        lines.put(b" ");
        lines.put(argument);
    }
}

pub(crate) fn line_json(this: &Line, json: &mut Encoder) {
    json.object_start();
    optional(json, b"age", this.age.as_deref());
    optional(json, b"argument", this.argument.as_deref());
    optional(json, b"group", this.group.as_deref());
    if let Some(mode) = this.mode {
        json.key(b"mode");
        json.string(&octal(mode));
    }
    optional(json, b"owner", this.owner.as_deref());
    field(json, b"type", this.kind.name());
    json.object_end();
}
fn optional(json: &mut Encoder, key: &[u8], value: Option<&[u8]>) {
    if let Some(value) = value {
        field(json, key, value);
    }
}
pub(crate) fn script_text(this: &Script, lines: &mut Lines) {
    lines.put(&this.task);
    lines.put(b" ");
    lines.decimal(u64::from(this.order));
    lines.put(b" ");
    lines.hex12(&this.content_digest.0);
}

pub(crate) fn script_json(this: &Script, json: &mut Encoder) {
    json.object_start();
    json.key(b"content_digest");
    digest(json, &this.content_digest.0);
    json.key(b"order");
    json.unsigned(u64::from(this.order));
    field(json, b"task", &this.task);
    json.object_end();
}
pub(crate) fn directives_text(this: &super::Config, lines: &mut Lines) {
    for row in &this.presets {
        lines.put(b"  presets    ");
        lines.put(&row.unit);
        lines.put(b" ");
        action_text(row.action, lines);
        lines.put(b"\n");
    }
    for row in &this.tmpfiles {
        lines.put(b"  tmpfiles   ");
        lines.put(&row.path);
        lines.put(b" ");
        line_text(&row.line, lines);
        lines.put(b"\n");
    }
    for row in &this.scripts {
        lines.put(b"  scripts    ");
        script_text(row, lines);
        lines.put(b"\n");
    }
}

pub(crate) fn directives_json(this: &super::Config, json: &mut Encoder) {
    json.key(b"presets");
    json.object_start();
    for row in &this.presets {
        json.key(&row.unit);
        action_json(row.action, json);
    }
    json.object_end();
    json.key(b"scripts");
    scripts_json(&this.scripts, json);
    json.key(b"tmpfiles");
    json.object_start();
    for row in &this.tmpfiles {
        json.key(&row.path);
        line_json(&row.line, json);
    }
    json.object_end();
}
pub(crate) fn scripts_json(scripts: &[Script], json: &mut Encoder) {
    json.array_start();
    for script in scripts {
        script_json(script, json);
    }
    json.array_end();
}
