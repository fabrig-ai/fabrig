use hearth_domain_render_vocabulary::{evaluate::FaultKind, errors::Required};
//! FaultKind messages and undeclared-setting diagnostics (hearth/docs/design/domain/render/errors.md, 3).
use super::{Encoder, Lines};
use alloc::boxed::Box;


pub(super) fn evaluation_message(item: FaultKind) -> &'static [u8] {
    match item {
        FaultKind::Unknown => b"an unknown name",
        FaultKind::NoKey => b"a missing key",
        FaultKind::NoIndex => b"an index outside the list",
        FaultKind::Undefined => b"an undefined value used",
        FaultKind::Type => b"a value of the wrong type",
        FaultKind::Overflow => b"an integer overflow",
        FaultKind::Empty => b"an empty list or pattern",
        FaultKind::Path => b"a path systemd_escape_path refuses",
        FaultKind::Output => b"a list or a map output",
        FaultKind::Limit => b"a limit exceeded",
    }
}
pub(super) fn evaluation_name(item: FaultKind) -> &'static [u8] {
    match item {
        FaultKind::Unknown => b"unknown",
        FaultKind::NoKey => b"no_key",
        FaultKind::NoIndex => b"no_index",
        FaultKind::Undefined => b"undefined",
        FaultKind::Type => b"type",
        FaultKind::Overflow => b"overflow",
        FaultKind::Empty => b"empty",
        FaultKind::Path => b"path",
        FaultKind::Output => b"output",
        FaultKind::Limit => b"limit",
    }
}

pub(super) fn undeclared(lines: &mut Lines, task: &[u8], setting: &[u8], declared: &[Box<[u8]>]) {
    lines.put(b"undeclared setting ");
    lines.quote(setting);
    lines.put(b"; ");
    lines.put(task);
    if declared.is_empty() {
        lines.put(b" declares no settings");
    } else {
        lines.put(b" declares ");
        names(lines, declared);
    }
}

pub(super) fn signed(lines: &mut Lines, value: i64) {
    if value < 0 {
        lines.put(b"-");
    }
    lines.decimal(value.unsigned_abs());
}
pub(super) fn names(lines: &mut Lines, names: &[Box<[u8]>]) {
    for (index, name) in names.iter().enumerate() {
        if index != 0 {
            lines.put(b", ");
        }
        lines.put(name);
    }
}
pub(super) fn json_names(json: &mut Encoder, names: &[Box<[u8]>]) {
    json.array_start();
    for name in names {
        json.string(name);
    }
    json.array_end();
}


pub(super) fn required_name(item: Required) -> &'static [u8] {
    match item {
        Required::If => b"if",
        Required::Message => b"message",
        Required::Type => b"type",
        Required::Doc => b"doc",
        Required::Value => b"value",
    }
}

