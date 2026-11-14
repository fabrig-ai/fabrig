//! Both claimants and their namespace (hearth/docs/design/domain/render/README.md, 3.7, and hearth/docs/design/domain/render/errors.md, 3).
use crate::form::{json::field, text::Lines};
use hearth_domain_render_vocabulary::errors::{Collision, Id, Subject};
use skein_json::writer::Encoder;

pub(super) fn collision_text(item: &Collision, lines: &mut Lines) {
    match item {
        Collision::Key { key } => {
            lines.put(b"the key ");
            lines.quote(key);
            lines.put(b" is made twice");
        }
        Collision::Given { subject, name, first, second } => {
            subject_prefix(*subject, lines);
            lines.put(name);
            lines.put(b" is given by ");
            lines.put(first);
            lines.put(b" and by ");
            lines.put(second);
        }
        Collision::Differs { subject, name, first, second } => {
            subject_prefix(*subject, lines);
            if *subject == Subject::Tmpfiles {
                lines.put(name);
            } else {
                lines.quote(name);
            }
            lines.put(b" differs between ");
            lines.put(first);
            lines.put(b" and ");
            lines.put(second);
        }
        Collision::Shared { id, value, first, second } => {
            lines.put(id_name(*id));
            lines.put(b" ");
            lines.decimal(u64::from(*value));
            lines.put(b" is shared by ");
            lines.put(first);
            lines.put(b" and ");
            lines.put(second);
        }
        Collision::Hidden { path, data, .. } => {
            lines.put(path);
            lines.put(b" is under the data path ");
            lines.put(data);
            lines.put(b" (a mount would hide it)");
        }
        Collision::Overlap { subject: Subject::Range, first, second } => {
            lines.put(b"subordinate ids of ");
            lines.put(first);
            lines.put(b" overlap ");
            lines.put(second);
            lines.put(b"'s");
        }
        Collision::Overlap { subject, first, second } => {
            subject_prefix(*subject, lines);
            lines.put(first);
            lines.put(b" overlaps ");
            lines.put(second);
        }
    }
}
pub(super) fn collision_json(item: &Collision, json: &mut Encoder) {
    json.object_start();
    match item {
        Collision::Key { key } => {
            field(json, b"key", key);
            field(json, b"kind", b"key");
        }
        Collision::Given { subject, name, first, second } => {
            field(json, b"kind", b"given");
            field(json, b"subject", subject_name(*subject));
            field(json, b"name", name);
            field(json, b"first", first);
            field(json, b"second", second);
        }
        Collision::Differs { subject, name, first, second } => {
            field(json, b"kind", b"differs");
            field(json, b"subject", subject_name(*subject));
            field(json, b"name", name);
            field(json, b"first", first);
            field(json, b"second", second);
        }
        Collision::Shared { id, value, first, second } => {
            field(json, b"kind", b"shared");
            field(json, b"id", id_name(*id));
            json.key(b"value");
            json.unsigned(u64::from(*value));
            field(json, b"first", first);
            field(json, b"second", second);
        }
        Collision::Hidden { path, data, first, second } => {
            field(json, b"kind", b"hidden");
            field(json, b"path", path);
            field(json, b"data", data);
            field(json, b"first", first);
            field(json, b"second", second);
        }
        Collision::Overlap { subject, first, second } => {
            field(json, b"kind", b"overlap");
            field(json, b"subject", subject_name(*subject));
            field(json, b"first", first);
            field(json, b"second", second);
        }
    }
    json.object_end();
}

pub(super) fn subject_name(item: Subject) -> &'static [u8] {
    match item {
        Subject::Tree => b"tree",
        Subject::DataName => b"data_name",
        Subject::DataPath => b"data_path",
        Subject::Secret => b"secret",
        Subject::Identity => b"identity",
        Subject::Download => b"download",
        Subject::User => b"user",
        Subject::Group => b"group",
        Subject::Preset => b"preset",
        Subject::Tmpfiles => b"tmpfiles",
        Subject::Preseed => b"preseed",
        Subject::Parameter => b"parameter",
        Subject::Range => b"range",
    }
}
pub(super) fn subject_prefix(item: Subject, lines: &mut Lines) {
    lines.put(match item {
        Subject::Tree => b"",
        Subject::DataName => b"data ",
        Subject::DataPath => b"data path ",
        Subject::Secret => b"secret ",
        Subject::Identity => b"identity ",
        Subject::Download => b"download ",
        Subject::User => b"user ",
        Subject::Group => b"group ",
        Subject::Preset => b"preset ",
        Subject::Tmpfiles => b"tmpfiles ",
        Subject::Preseed => b"preseed ",
        Subject::Parameter => b"boot parameter ",
        Subject::Range => b"subordinate ids ",
    });
}

pub(super) const fn id_name(item: Id) -> &'static [u8] {
    match item {
        Id::Uid => b"uid",
        Id::Gid => b"gid",
    }
}
