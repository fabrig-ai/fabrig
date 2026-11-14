//! Fixed ids in the packages record (hearth/docs/design/domain/render/spec.md, 1.2).
use super::Seeds;
use crate::form::text::Lines;
use skein_json::writer::Encoder;

pub(crate) fn text(this: &Seeds, lines: &mut Lines) {
    for user in &this.users {
        lines.put(b"  seed       user ");
        lines.put(&user.name);
        lines.put(b" ");
        lines.decimal(u64::from(user.uid));
        lines.put(b" ");
        match user.gid {
            Some(gid) => lines.decimal(u64::from(gid)),
            None => lines.put(b"-"),
        }
        lines.put(b"\n");
    }
    for group in &this.groups {
        lines.put(b"  seed       group ");
        lines.put(&group.name);
        lines.put(b" ");
        lines.decimal(u64::from(group.gid));
        lines.put(b"\n");
    }
}

pub(crate) fn json(this: &Seeds, json: &mut Encoder) {
    json.object_start();
    json.key(b"groups");
    json.object_start();
    for group in &this.groups {
        json.key(&group.name);
        json.object_start();
        json.key(b"gid");
        json.unsigned(u64::from(group.gid));
        json.object_end();
    }
    json.object_end();
    json.key(b"users");
    json.object_start();
    for user in &this.users {
        json.key(&user.name);
        json.object_start();
        json.key(b"gid");
        match user.gid {
            Some(gid) => json.unsigned(u64::from(gid)),
            None => json.null(),
        }
        json.key(b"uid");
        json.unsigned(u64::from(user.uid));
        json.object_end();
    }
    json.object_end();
    json.object_end();
}
