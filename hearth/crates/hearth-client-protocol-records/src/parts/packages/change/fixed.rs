//! Seed changes by name; text prints the new row (hearth/docs/design/domain/render/README.md, 4.1).
use super::{Lines, SeedGroupKeyed, SeedIds, SeedUserKeyed, SeedsChange};
use skein_json::writer::Encoder;

pub(crate) fn user_sides(this: &SeedUserKeyed) -> (&[u8], Option<&SeedIds>, Option<&SeedIds>) {
    match this {
        SeedUserKeyed::Added { key, value } => (key, None, Some(value)),
        SeedUserKeyed::Removed { key, value } => (key, Some(value), None),
        SeedUserKeyed::Changed { key, before, after } => (key, Some(before), Some(after)),
    }
}
pub(crate) fn group_sides(this: &SeedGroupKeyed) -> (&[u8], Option<&u32>, Option<&u32>) {
    match this {
        SeedGroupKeyed::Added { key, value } => (key, None, Some(value)),
        SeedGroupKeyed::Removed { key, value } => (key, Some(value), None),
        SeedGroupKeyed::Changed { key, before, after } => (key, Some(before), Some(after)),
    }
}
pub(crate) fn text(this: &SeedsChange, lines: &mut Lines) {
    for entry in &this.users {
        let (key, before, after) = user_sides(entry);
        seed_header(lines, b"user", key, before.is_some(), after.is_some());
        let ids = after.or(before).expect("seed row");
        lines.decimal(u64::from(ids.uid));
        lines.put(b" ");
        match ids.gid {
            Some(gid) => lines.decimal(u64::from(gid)),
            None => lines.put(b"-"),
        }
        lines.put(b"\n");
    }
    for entry in &this.groups {
        let (key, before, after) = group_sides(entry);
        seed_header(lines, b"group", key, before.is_some(), after.is_some());
        lines.decimal(u64::from(*after.or(before).expect("seed row")));
        lines.put(b"\n");
    }
}

pub(crate) fn json(this: &SeedsChange, json: &mut Encoder) {
    json.object_start();
    json.key(b"groups");
    json.array_start();
    for entry in &this.groups {
        let (key, before, after) = group_sides(entry);
        crate::diff::placed::keyed(json, key, optional_gid(before), optional_gid(after));
    }
    json.array_end();
    json.key(b"users");
    json.array_start();
    for entry in &this.users {
        let (key, before, after) = user_sides(entry);
        crate::diff::placed::keyed(json, key, optional_ids(before), optional_ids(after));
    }
    json.array_end();
    json.object_end();
}
pub(crate) fn ids(ids: &SeedIds, json: &mut Encoder) {
    json.object_start();
    json.key(b"gid");
    match ids.gid {
        Some(gid) => json.unsigned(u64::from(gid)),
        None => json.null(),
    }
    json.key(b"uid");
    json.unsigned(u64::from(ids.uid));
    json.object_end();
}
fn seed_header(lines: &mut Lines, kind: &[u8], key: &[u8], before: bool, after: bool) {
    crate::diff::placed::header(lines, b"packages", b"seed ", before, after);
    lines.put(kind);
    lines.put(b" ");
    lines.put(key);
    lines.pad_to(52);
}

fn optional_ids(value: Option<&SeedIds>) -> Option<crate::diff::placed::Value<'_>> {
    Some(crate::diff::placed::Value::SeedIds(value?))
}

fn optional_gid(value: Option<&u32>) -> Option<crate::diff::placed::Value<'_>> {
    Some(crate::diff::placed::Value::Gid(*value?))
}
