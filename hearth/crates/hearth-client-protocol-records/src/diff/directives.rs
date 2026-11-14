use crate::form::{json::field, text::Lines};
use hearth_domain_render_vocabulary::parts::config::{
    Change as ConfigChange, LineKeyed, PresetKeyed, Script, ScriptsChange,
};
use skein_json::writer::Encoder;

fn header(lines: &mut Lines, mark: &[u8], list: &[u8], key: &[u8]) {
    lines.put(b"  config   ");
    lines.put(mark);
    lines.put(b" ");
    lines.put(list);
    if !key.is_empty() {
        lines.put(b" ");
        lines.put(key);
    }
    lines.pad_to(52);
}
pub(crate) fn preset_text(this: &PresetKeyed, lines: &mut Lines) {
    let (mark, key, value) = match this {
        PresetKeyed::Added { key, value } => (b"+", key, value),
        PresetKeyed::Removed { key, value } => (b"-", key, value),
        PresetKeyed::Changed { key, after, .. } => (b"~", key, after),
    };
    header(lines, mark, b"presets", key);
    crate::parts::config::action_text(*value, lines);
    lines.put(b"\n");
}

pub(crate) fn preset_json(this: &PresetKeyed, json: &mut Encoder) {
    json.object_start();
    match this {
        PresetKeyed::Added { key, value } | PresetKeyed::Removed { key, value } => {
            field(json, b"key", key);
            field(
                json,
                b"kind",
                match this {
                    PresetKeyed::Added { .. } => b"added",
                    PresetKeyed::Removed { .. } => b"removed",
                    PresetKeyed::Changed { .. } => unreachable!("added or removed"),
                },
            );
            json.key(b"value");
            crate::parts::config::action_json(*value, json);
        }
        PresetKeyed::Changed { key, before, after } => {
            json.key(b"after");
            crate::parts::config::action_json(*after, json);
            json.key(b"before");
            crate::parts::config::action_json(*before, json);
            field(json, b"key", key);
            field(json, b"kind", b"changed");
        }
    }
    json.object_end();
}
pub(crate) fn line_text(this: &LineKeyed, lines: &mut Lines) {
    let (mark, key, value) = match this {
        LineKeyed::Added { key, value } => (b"+", key, value),
        LineKeyed::Removed { key, value } => (b"-", key, value),
        LineKeyed::Changed { key, after, .. } => (b"~", key, after),
    };
    header(lines, mark, b"tmpfiles", key);
    crate::parts::config::line_text(value, lines);
    lines.put(b"\n");
}

pub(crate) fn line_json(this: &LineKeyed, json: &mut Encoder) {
    json.object_start();
    match this {
        LineKeyed::Added { key, value } | LineKeyed::Removed { key, value } => {
            field(json, b"key", key);
            field(
                json,
                b"kind",
                match this {
                    LineKeyed::Added { .. } => b"added",
                    LineKeyed::Removed { .. } => b"removed",
                    LineKeyed::Changed { .. } => unreachable!("added or removed"),
                },
            );
            json.key(b"value");
            crate::parts::config::line_json(value, json);
        }
        LineKeyed::Changed { key, before, after } => {
            json.key(b"after");
            crate::parts::config::line_json(after, json);
            json.key(b"before");
            crate::parts::config::line_json(before, json);
            field(json, b"key", key);
            field(json, b"kind", b"changed");
        }
    }
    json.object_end();
}
pub(crate) fn scripts_text(this: &ScriptsChange, lines: &mut Lines, part: &[u8]) {
    for row in &this.before {
        if script(&this.after, &row.task).is_none() {
            crate::diff::placed::header(lines, part, b"scripts", true, false);
            lines.pad_to(52);
            crate::parts::config::script_text(row, lines);
            lines.put(b"\n");
        }
    }
    for row in &this.after {
        let mark = match script(&this.before, &row.task) {
            Some(before) if before == row => continue,
            Some(_) => b"~",
            None => b"+",
        };
        crate::diff::placed::header(lines, part, b"scripts", mark != b"+", true);
        lines.pad_to(52);
        crate::parts::config::script_text(row, lines);
        lines.put(b"\n");
    }
}

pub(crate) fn scripts_json(this: &ScriptsChange, json: &mut Encoder) {
    json.object_start();
    json.key(b"after");
    script_list_json(&this.after, json);
    json.key(b"before");
    script_list_json(&this.before, json);
    json.object_end();
}
fn script<'a>(scripts: &'a [Script], task: &[u8]) -> Option<&'a Script> {
    let mut found = None;
    for script in scripts {
        if script.task.as_ref() == task {
            found = Some(script);
        }
    }
    found
}
fn script_list_json(scripts: &[Script], json: &mut Encoder) {
    json.array_start();
    for script in scripts {
        crate::parts::config::script_json(script, json);
    }
    json.array_end();
}
pub(crate) fn directives_text(this: &ConfigChange, lines: &mut Lines) {
    for row in &this.presets {
        preset_text(row, lines);
    }
    for row in &this.tmpfiles {
        line_text(row, lines);
    }
    if let Some(scripts) = &this.scripts {
        scripts_text(scripts, lines, b"config");
    }
}

pub(crate) fn directives_json(this: &ConfigChange, json: &mut Encoder) {
    json.key(b"presets");
    json.array_start();
    for row in &this.presets {
        preset_json(row, json);
    }
    json.array_end();
    if let Some(scripts) = &this.scripts {
        json.key(b"scripts");
        scripts_json(scripts, json);
    }
    json.key(b"tmpfiles");
    json.array_start();
    for row in &this.tmpfiles {
        line_json(row, json);
    }
    json.array_end();
}
