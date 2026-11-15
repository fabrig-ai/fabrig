//! The config part's file tree (hearth/docs/design/domain/render/spec.md, 1.2).
use crate::form::text::{Lines, octal};
use hearth_domain_render_vocabulary::parts::config::{Config, Node};

pub(crate) fn text(this: &Config, lines: &mut Lines) {
    accounts::accounts_text(this, lines);
    directives::directives_text(this, lines);
    let mut column = 27;
    for entry in &*this.tree {
        column = column.max(entry.path.len().saturating_add(4));
    }
    for entry in &*this.tree {
        lines.put(b"  ");
        lines.put(&entry.path);
        lines.pad_to(column);
        match &entry.node {
            Node::Link { target, owner, group } => {
                lines.put(b"link ");
                lines.put(owner);
                lines.put(b":");
                lines.put(group);
                lines.put(b" ");
                lines.put(target);
            }
            Node::Directory { mode, owner, group } => {
                lines.put(b"directory ");
                lines.mode(*mode);
                lines.put(b" ");
                lines.put(owner);
                lines.put(b":");
                lines.put(group);
            }
            Node::File { content_digest, mode, owner, group } => {
                lines.put(b"file ");
                lines.mode(*mode);
                lines.put(b" ");
                lines.put(owner);
                lines.put(b":");
                lines.put(group);
                lines.put(b" ");
                lines.hex12(&content_digest.0);
            }
        }
        lines.put(b"\n");
    }
}

pub(crate) fn json(this: &Config, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    accounts::accounts_json(this, json);
    directives::directives_json(this, json);
    json.key(b"tree");
    json.object_start();
    for entry in &*this.tree {
        json.key(&entry.path);
        node_json(&entry.node, json);
    }
    json.object_end();
    accounts::users_json(this, json);
    json.object_end();
}

pub(crate) fn node_json(this: &Node, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    match this {
        Node::Link { target, owner, group } => {
            json.key(b"group");
            json.string(group);
            json.key(b"kind");
            json.string(b"link");
            json.key(b"owner");
            json.string(owner);
            json.key(b"target");
            json.string(target);
        }
        Node::Directory { mode, owner, group } => {
            json.key(b"group");
            json.string(group);
            json.key(b"kind");
            json.string(b"directory");
            json.key(b"mode");
            json.string(&octal(*mode));
            json.key(b"owner");
            json.string(owner);
        }
        Node::File { content_digest, mode, owner, group } => {
            json.key(b"content_digest");
            crate::form::json::digest(json, &content_digest.0);
            json.key(b"group");
            json.string(group);
            json.key(b"kind");
            json.string(b"file");
            json.key(b"mode");
            json.string(&octal(*mode));
            json.key(b"owner");
            json.string(owner);
        }
    }
    json.object_end();
}

mod accounts;
pub(crate) use accounts::{group_json, group_text, member_json, user_json, user_text};

mod directives;
pub(crate) use directives::scripts_json;
pub(crate) use directives::{action_json, action_text, line_json, line_text, script_json, script_text};
