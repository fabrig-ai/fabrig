//! Account changes in client forms (hearth/docs/design/domain/render/README.md, 4.3).
use crate::form::text::Lines;
use hearth_domain_render_vocabulary::parts::config::{Change as ConfigChange, GroupKeyed, MemberChange, UserKeyed};
use skein_json::writer::Encoder;

pub(crate) fn group_text(this: &GroupKeyed, lines: &mut Lines) {
    let (mark, key, value) = match this {
        GroupKeyed::Added { key, value } => (b"+", key, value),
        GroupKeyed::Removed { key, value } => (b"-", key, value),
        GroupKeyed::Changed { key, after, .. } => (b"~", key, after),
    };
    lines.put(b"  config   ");
    lines.put(mark);
    lines.put(b" groups ");
    lines.put(key);
    lines.pad_to(52);
    crate::parts::config::group_text(value, lines);
    lines.put(b"\n");
}

pub(crate) fn group_json(this: &GroupKeyed, json: &mut Encoder) {
    json.object_start();
    match this {
        GroupKeyed::Added { key, value } | GroupKeyed::Removed { key, value } => {
            json.key(b"key");
            json.string(key);
            json.key(b"kind");
            json.string(match this {
                GroupKeyed::Added { .. } => b"added",
                GroupKeyed::Removed { .. } => b"removed",
                GroupKeyed::Changed { .. } => unreachable!("added or removed"),
            });
            json.key(b"value");
            crate::parts::config::group_json(value, json);
        }
        GroupKeyed::Changed { key, before, after } => {
            json.key(b"after");
            crate::parts::config::group_json(after, json);
            json.key(b"before");
            crate::parts::config::group_json(before, json);
            json.key(b"key");
            json.string(key);
            json.key(b"kind");
            json.string(b"changed");
        }
    }
    json.object_end();
}
pub(crate) fn user_text(this: &UserKeyed, lines: &mut Lines) {
    let (mark, key, value) = match this {
        UserKeyed::Added { key, value } => (b"+", key, value),
        UserKeyed::Removed { key, value } => (b"-", key, value),
        UserKeyed::Changed { key, after, .. } => (b"~", key, after),
    };
    lines.put(b"  config   ");
    lines.put(mark);
    lines.put(b" users ");
    lines.put(key);
    lines.pad_to(52);
    crate::parts::config::user_text(value, lines);
    lines.put(b"\n");
}

pub(crate) fn user_json(this: &UserKeyed, json: &mut Encoder) {
    json.object_start();
    match this {
        UserKeyed::Added { key, value } | UserKeyed::Removed { key, value } => {
            json.key(b"key");
            json.string(key);
            json.key(b"kind");
            json.string(match this {
                UserKeyed::Added { .. } => b"added",
                UserKeyed::Removed { .. } => b"removed",
                UserKeyed::Changed { .. } => unreachable!("added or removed"),
            });
            json.key(b"value");
            crate::parts::config::user_json(value, json);
        }
        UserKeyed::Changed { key, before, after } => {
            json.key(b"after");
            crate::parts::config::user_json(after, json);
            json.key(b"before");
            crate::parts::config::user_json(before, json);
            json.key(b"key");
            json.string(key);
            json.key(b"kind");
            json.string(b"changed");
        }
    }
    json.object_end();
}
pub(crate) fn member_text(this: &MemberChange, lines: &mut Lines) {
    let (mark, member) = match this {
        MemberChange::Added(member) => (b"+", member),
        MemberChange::Removed(member) => (b"-", member),
    };
    lines.put(b"  config   ");
    lines.put(mark);
    lines.put(b" members ");
    lines.put(&member.user);
    lines.put(b" ");
    lines.put(&member.group);
    lines.put(
        b"
",
    );
}

pub(crate) fn member_json(this: &MemberChange, json: &mut Encoder) {
    json.object_start();
    json.key(b"kind");
    let member = match this {
        MemberChange::Added(member) => {
            json.string(b"added");
            member
        }
        MemberChange::Removed(member) => {
            json.string(b"removed");
            member
        }
    };
    json.key(b"value");
    crate::parts::config::member_json(member, json);
    json.object_end();
}
pub(crate) fn accounts_text(this: &ConfigChange, lines: &mut Lines) {
    for row in &this.groups {
        group_text(row, lines);
    }
    for row in &this.users {
        user_text(row, lines);
    }
    for row in &this.members {
        member_text(row, lines);
    }
}

pub(crate) fn accounts_json(this: &ConfigChange, json: &mut Encoder) {
    json.key(b"groups");
    json.array_start();
    for row in &this.groups {
        group_json(row, json);
    }
    json.array_end();
    json.key(b"users");
    json.array_start();
    for row in &this.users {
        user_json(row, json);
    }
    json.array_end();
    json.key(b"members");
    json.array_start();
    for row in &this.members {
        member_json(row, json);
    }
    json.array_end();
}
