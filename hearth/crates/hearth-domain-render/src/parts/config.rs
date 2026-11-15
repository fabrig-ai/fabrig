//! The static file tree's canonical record (hearth/docs/design/domain/render/spec.md, 1.2).
use crate::revision::Mode;
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::parts::config::{
    Change, Config, EntryChange, GroupKeyed, LineKeyed, MemberChange, Node, PresetKeyed, ScriptsChange, Sizes,
    TreeEntry, UserKeyed,
};
use hearth_lib::{
    encode::{Digest, Encoder},
    hash::H,
};
use skein_lib::{Joined, List, Map, bytes};

#[must_use]
pub fn file(path: &[u8], mode: Mode, content_digest: Digest) -> TreeEntry {
    TreeEntry {
        path: absolute(path),
        node: Node::File {
            content_digest,
            mode: if mode == Mode::Executable { 0o755 } else { 0o644 },
            owner: bytes::copy_of(b"root"),
            group: bytes::copy_of(b"root"),
        },
    }
}
#[must_use]
pub(crate) fn digest(config: &Config) -> Digest {
    let mut hash = H::new(b"part:config", 1);
    encode(config, hash.values());
    hash.finish()
}
pub(crate) fn encode(config: &Config, record: &mut Encoder) {
    record.map(7);
    accounts::encode(config, record);
    directives::encode(config, record);
    record.key(b"tree");
    record.map(u64::try_from(config.tree.len()).expect("bounded tree"));
    for entry in &config.tree {
        record.key(&entry.path);
        match &entry.node {
            Node::Link { target, owner, group } => {
                record.map(4);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"link");
                record.key(b"owner");
                record.bytes(owner);
                record.key(b"target");
                record.bytes(target);
            }
            Node::Directory { mode, owner, group } => {
                record.map(4);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"directory");
                record.key(b"mode");
                record.uint(u64::from(*mode));
                record.key(b"owner");
                record.bytes(owner);
            }
            Node::File { content_digest, mode, owner, group } => {
                record.map(5);
                record.key(b"content_digest");
                record.digest(content_digest);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"file");
                record.key(b"mode");
                record.uint(u64::from(*mode));
                record.key(b"owner");
                record.bytes(owner);
            }
        }
    }
    accounts::encode_users(config, record);
}

fn absolute(path: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(path.len().saturating_add(1));
    writer.put(b"/").expect("prefix room");
    writer.put(path).expect("path room");
    writer.finish()
}

#[must_use]
pub(crate) fn change(before: Option<&Config>, after: Option<&Config>) -> Change {
    let scripts = directives_changes::scripts(before, after);
    let members = changes::members(before, after);
    let empty = Config {
        presets: Box::new([]),
        tmpfiles: Box::new([]),
        scripts: Box::new([]),
        groups: Box::new([]),
        users: Box::new([]),
        members: Box::new([]),
        tree: Box::new([]),
    };
    let before = before.unwrap_or(&empty);
    let after = after.unwrap_or(&empty);
    let presets = directives_changes::presets(before, after);
    let tmpfiles = directives_changes::tmpfiles(before, after);
    let groups = changes::groups(before, after);
    let users = changes::users(before, after);
    let mut old = Map::uncapped();
    for row in &before.tree {
        old.insert(row.path.as_ref(), row).expect("an uncapped map");
    }
    let mut new = Map::uncapped();
    for row in &after.tree {
        new.insert(row.path.as_ref(), row).expect("an uncapped map");
    }
    let mut tree = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => EntryChange::Removed { path: bytes::copy_of(key), node: row.node.clone() },
            Joined::Second(row) => EntryChange::Added { path: bytes::copy_of(key), node: row.node.clone() },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => EntryChange::Changed {
                path: bytes::copy_of(key),
                before: was.node.clone(),
                after: now.node.clone(),
                sizes: None,
                hunks: None,
            },
        };
        tree.push(change).expect("an uncapped list");
    }
    let tree = tree.into_boxed();
    Change { presets, tmpfiles, scripts, groups, users, members, tree }
}

/// Retain cached file sizes and select differing files below the hunk limit.
pub fn pair(
    change: &mut EntryChange,
    before_sizes: &crate::call::Sizes,
    after_sizes: &crate::call::Sizes,
    max: u32,
) -> Option<(Digest, Digest)> {
    match change {
        EntryChange::Changed {
            before: Node::File { content_digest: before, .. },
            after: Node::File { content_digest: after, .. },
            sizes,
            ..
        } => {
            let pair = Sizes { before: *before_sizes.get(before)?, after: *after_sizes.get(after)? };
            *sizes = Some(pair);
            if before != after && pair.before < u64::from(max) && pair.after < u64::from(max) {
                Some((*before, *after))
            } else {
                None
            }
        }
        EntryChange::Added { .. } | EntryChange::Removed { .. } | EntryChange::Changed { .. } => None,
    }
}

/// A rendered template installs with exactly its last suffix removed.
#[cfg(test)]
#[must_use]
pub(crate) fn rendered(path: &[u8], mode: Mode, content_digest: Digest) -> TreeEntry {
    file(path.strip_suffix(b".j2").expect("template suffix"), mode, content_digest)
}

mod accounts;
pub use accounts::{Group, GroupKind, Member, User, UserKind};

pub(crate) use accounts::build as accounts;
pub(crate) use accounts::rows as account_rows;

mod changes;

mod directives;
pub(crate) use directives::encode_scripts;
pub use directives::{Action, Line, LineKind, Preset, Script, Tmpfile};

mod directives_changes;
