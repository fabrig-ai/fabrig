use hearth_domain_render_vocabulary::errors::{Resolution};
//! Apt's configuration verdict and the bounded closure (hearth/docs/design/domain/render/spec.md, 2.5, hearth/docs/design/domain/render/errors.md, 2).
use super::{Box, Encoder, Lines, field};


pub(super) fn resolution_text(item: &Resolution, lines: &mut Lines) {
    match item {
        Resolution::Unknown { package } => {
            lines.put(b"apt cannot locate package ");
            lines.quote(package);
        }
        Resolution::Unsatisfiable { package } => {
            lines.put(b"unsatisfiable dependencies");
            if let Some(package) = package {
                lines.put(b" of ");
                lines.quote(package);
            }
        }
        Resolution::Absent { uri, suite } => {
            lines.put(b"the archive ");
            lines.put(uri);
            lines.put(b" does not have ");
            lines.put(suite);
        }
        Resolution::Missing { package } => {
            lines.quote(package);
            lines.put(b" was requested, and apt installs no package of that name");
        }
        Resolution::Unmatched { name, version, architecture } => {
            lines.put(b"apt would install ");
            triple(lines, name, version, architecture);
            lines.put(b", which no Packages index lists");
        }
        Resolution::Disagree { name, version, architecture } => {
            lines.put(b"the Packages indices disagree on ");
            triple(lines, name, version, architecture);
        }
        Resolution::TooLarge { max } => {
            lines.put(b"a resolution record past ");
            lines.decimal(*max);
            lines.put(b" bytes");
        }
        Resolution::TooMany { max } => {
            lines.put(b"a closure past ");
            lines.decimal(u64::from(*max));
            lines.put(b" packages");
        }
    }
}
pub(super) fn resolution_json(item: &Resolution, json: &mut Encoder) {
    json.object_start();
    match item {
        Resolution::Unknown { package } => {
            field(json, b"kind", b"unknown");
            field(json, b"package", package);
        }
        Resolution::Unsatisfiable { package } => {
            field(json, b"kind", b"unsatisfiable");
            if let Some(package) = package {
                field(json, b"package", package);
            }
        }
        Resolution::Absent { uri, suite } => {
            field(json, b"kind", b"absent");
            field(json, b"uri", uri);
            field(json, b"suite", suite);
        }
        Resolution::Missing { package } => {
            field(json, b"kind", b"missing");
            field(json, b"package", package);
        }
        Resolution::Unmatched { name, version, architecture } => {
            field(json, b"kind", b"unmatched");
            json_triple(json, name, version, architecture);
        }
        Resolution::Disagree { name, version, architecture } => {
            field(json, b"kind", b"disagree");
            json_triple(json, name, version, architecture);
        }
        Resolution::TooLarge { max } => {
            field(json, b"kind", b"too_large");
            json.key(b"max");
            json.unsigned(*max);
        }
        Resolution::TooMany { max } => {
            field(json, b"kind", b"too_many");
            json.key(b"max");
            json.unsigned(u64::from(*max));
        }
    }
    json.object_end();
}


fn triple(lines: &mut Lines, name: &[u8], version: &[u8], architecture: &[u8]) {
    lines.put(name);
    lines.put(b" ");
    lines.put(version);
    lines.put(b" ");
    lines.put(architecture);
}
fn json_triple(json: &mut Encoder, name: &[u8], version: &[u8], architecture: &[u8]) {
    field(json, b"name", name);
    field(json, b"version", version);
    field(json, b"architecture", architecture);
}
