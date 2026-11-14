use crate::form::text::Lines;
use hearth_domain_render_vocabulary::parts::config::{GroupKind, Member, UserKind};
use skein_json::writer::Encoder;

pub(crate) fn group_text(this: &GroupKind, lines: &mut Lines) {
    match this {
        GroupKind::Package => lines.put(b"package"),
        GroupKind::Sysusers { gid } => id_text(*gid, lines),
    }
}

pub(crate) fn group_json(this: &GroupKind, json: &mut Encoder) {
    json.object_start();
    match this {
        GroupKind::Package => {
            json.key(b"kind");
            json.string(b"package");
        }
        GroupKind::Sysusers { gid } => {
            if let Some(gid) = gid {
                json.key(b"gid");
                json.number(skein_lib::Decimal::of(u64::from(*gid)).as_bytes());
            }
            json.key(b"kind");
            json.string(b"sysusers");
        }
    }
    json.object_end();
}
pub(crate) fn user_text(this: &UserKind, lines: &mut Lines) {
    match this {
        UserKind::Package => lines.put(b"package"),
        UserKind::Sysusers { uid, group, gecos, home, shell } => {
            id_text(*uid, lines);
            for value in [group, home, shell] {
                lines.put(b" ");
                lines.put(value);
            }
            if !gecos.is_empty() {
                lines.put(b" ");
                lines.put(gecos);
            }
        }
    }
}

pub(crate) fn user_json(this: &UserKind, json: &mut Encoder) {
    json.object_start();
    match this {
        UserKind::Package => {
            json.key(b"kind");
            json.string(b"package");
        }
        UserKind::Sysusers { uid, group, gecos, home, shell } => {
            json.key(b"gecos");
            json.string(gecos);
            json.key(b"group");
            json.string(group);
            json.key(b"home");
            json.string(home);
            json.key(b"kind");
            json.string(b"sysusers");
            json.key(b"shell");
            json.string(shell);
            if let Some(uid) = uid {
                json.key(b"uid");
                json.number(skein_lib::Decimal::of(u64::from(*uid)).as_bytes());
            }
        }
    }
    json.object_end();
}
fn id_text(id: Option<u32>, lines: &mut Lines) {
    match id {
        Some(id) => lines.decimal(u64::from(id)),
        None => lines.put(b"-"),
    }
}
pub(crate) fn accounts_text(this: &super::Config, lines: &mut Lines) {
    for group in &this.groups {
        lines.put(b"  groups     ");
        lines.put(&group.name);
        lines.put(b" ");
        group_text(&group.kind, lines);
        lines.put(b"\n");
    }
    for user in &this.users {
        lines.put(b"  users      ");
        lines.put(&user.name);
        lines.put(b" ");
        user_text(&user.kind, lines);
        lines.put(b"\n");
    }
    for member in &this.members {
        lines.put(b"  members    ");
        lines.put(&member.user);
        lines.put(b" ");
        lines.put(&member.group);
        lines.put(b"\n");
    }
}

pub(crate) fn accounts_json(this: &super::Config, json: &mut Encoder) {
    json.key(b"groups");
    json.object_start();
    for group in &this.groups {
        json.key(&group.name);
        group_json(&group.kind, json);
    }
    json.object_end();
    json.key(b"members");
    json.array_start();
    for member in &this.members {
        member_json(member, json);
    }
    json.array_end();
}

pub(crate) fn users_json(this: &super::Config, json: &mut Encoder) {
    json.key(b"users");
    json.object_start();
    for user in &this.users {
        json.key(&user.name);
        user_json(&user.kind, json);
    }
    json.object_end();
}
pub(crate) fn member_json(this: &Member, json: &mut Encoder) {
    json.object_start();
    json.key(b"group");
    json.string(&this.group);
    json.key(b"user");
    json.string(&this.user);
    json.object_end();
}
