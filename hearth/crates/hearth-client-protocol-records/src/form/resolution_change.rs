//! The packages closure's change in client forms (hearth/docs/design/domain/render/README.md, 4.1).
use crate::form::text::Lines;
use hearth_domain_render_vocabulary::diff::{Listed, Moved, ResolutionChange};
use skein_json::writer::Encoder;

pub(crate) fn json(this: &ResolutionChange, json: &mut Encoder) {
    json.object_start();
    match this {
        ResolutionChange::Same => super::json::field(json, b"kind", b"same"),
        ResolutionChange::NewKey => super::json::field(json, b"kind", b"new_key"),
        ResolutionChange::Resolved { added, removed, versions_changed } => {
            json.key(b"added");
            listed(json, added);
            super::json::field(json, b"kind", b"resolved");
            json.key(b"removed");
            listed(json, removed);
            json.key(b"versions_changed");
            json.array_start();
            for Moved { name, architecture, before, after } in versions_changed {
                json.object_start();
                super::json::field(json, b"after", after);
                super::json::field(json, b"architecture", architecture);
                super::json::field(json, b"before", before);
                super::json::field(json, b"name", name);
                json.object_end();
            }
            json.array_end();
        }
    }
    json.object_end();
}

pub(crate) fn text(this: &ResolutionChange, lines: &mut Lines) {
    lines.put(b"  packages   resolution ");
    match this {
        ResolutionChange::Same => lines.put(b"same\n"),
        ResolutionChange::NewKey => lines.put(b"new key\n"),
        ResolutionChange::Resolved { added, removed, versions_changed } => {
            lines.put(b"new key, resolved\n");
            for (sign, packages) in [(b"+".as_slice(), added), (b"-", removed)] {
                for Listed { name, architecture, version } in packages {
                    lines.put(b"    ");
                    lines.put(sign);
                    lines.put(b" ");
                    package_text(lines, name, architecture);
                    lines.put(b" ");
                    lines.put(version);
                    lines.put(b"\n");
                }
            }
            for Moved { name, architecture, before, after } in versions_changed {
                lines.put(b"    ~ ");
                package_text(lines, name, architecture);
                lines.put(b" ");
                lines.put(before);
                lines.put(" → ".as_bytes());
                lines.put(after);
                lines.put(b"\n");
            }
        }
    }
}
fn package_text(lines: &mut Lines, name: &[u8], architecture: &[u8]) {
    lines.put(name);
    lines.put(b":");
    lines.put(architecture);
}
fn listed(json: &mut Encoder, packages: &[Listed]) {
    json.array_start();
    for Listed { name, architecture, version } in packages {
        json.object_start();
        super::json::field(json, b"architecture", architecture);
        super::json::field(json, b"name", name);
        super::json::field(json, b"version", version);
        json.object_end();
    }
    json.array_end();
}
