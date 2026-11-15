//! Secret names and places (hearth/docs/design/protocol/format/README.md, 7.4).
use crate::form::text::{Lines, octal};
use hearth_domain_render_vocabulary::parts::secrets::{Algorithm, Change, IdentityKind, Named, Secrets};

pub(crate) fn text(this: &Secrets, lines: &mut Lines) {
    for file in &this.files {
        lines.put(b"  secret");
        lines.pad_to(13);
        lines.put(&file.name);
        lines.put(b" ");
        place(lines, &file.path, file.mode, &file.owner, &file.group);
    }
    for password in &this.passwords {
        lines.put(b"  password");
        lines.pad_to(13);
        lines.put(&password.user);
        lines.put(b" ");
        lines.put(&password.secret);
        lines.put(b"\n");
    }
    for item in &this.identity {
        lines.put(b"  identity");
        lines.pad_to(13);
        lines.put(&item.name);
        lines.put(b" ssh-host-key ");
        lines.put(algorithm(item.kind));
        lines.put(b" ");
        lines.put(&item.path);
        lines.put(b" ");
        lines.put(item.public_path.as_deref().unwrap_or(b"-"));
        lines.put(b" ");
        owned(lines, item.mode, &item.owner, &item.group);
    }
}

pub(crate) fn json(this: &Secrets, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    json.key(b"files");
    json.object_start();
    for file in &this.files {
        json.key(&file.name);
        json.object_start();
        json.key(b"group");
        json.string(&file.group);
        json_place(json, &file.path, file.mode, &file.owner);
        json.object_end();
    }
    json.object_end();
    json.key(b"identity");
    json.object_start();
    for item in &this.identity {
        json.key(&item.name);
        json.object_start();
        json.key(b"algorithm");
        json.string(algorithm(item.kind));
        json.key(b"group");
        json.string(&item.group);
        json.key(b"kind");
        json.string(b"ssh-host-key");
        json_place(json, &item.path, item.mode, &item.owner);
        if let Some(path) = &item.public_path {
            json.key(b"public_path");
            json.string(path);
        }
        json.object_end();
    }
    json.object_end();
    json.key(b"passwords");
    json.object_start();
    for password in &this.passwords {
        json.key(&password.user);
        json.string(&password.secret);
    }
    json.object_end();
    json.object_end();
}
fn algorithm(kind: IdentityKind) -> &'static [u8] {
    let IdentityKind::SshHostKey { algorithm } = kind;
    match algorithm {
        Algorithm::Ed25519 => b"ed25519",
        Algorithm::Ecdsa => b"ecdsa",
        Algorithm::Rsa => b"rsa",
    }
}
fn place(lines: &mut Lines, path: &[u8], mode: u16, owner: &[u8], group: &[u8]) {
    lines.put(path);
    lines.put(b" ");
    owned(lines, mode, owner, group);
}
fn owned(lines: &mut Lines, mode: u16, owner: &[u8], group: &[u8]) {
    lines.mode(mode);
    lines.put(b" ");
    lines.put(owner);
    lines.put(b":");
    lines.put(group);
    lines.put(b"\n");
}
fn json_place(json: &mut skein_json::writer::Encoder, path: &[u8], mode: u16, owner: &[u8]) {
    json.key(b"mode");
    json.string(&octal(mode));
    json.key(b"owner");
    json.string(owner);
    json.key(b"path");
    json.string(path);
}

pub(crate) fn change_text(this: &Change, lines: &mut Lines) {
    for (list, entries) in
        [(b"secret".as_slice(), &this.files), (b"password", &this.passwords), (b"identity", &this.identity)]
    {
        for entry in entries {
            let (key, before, after) = named_sides(entry);
            crate::diff::placed::header(lines, b"secrets", list, before, after);
            lines.put(b" ");
            lines.put(key);
            lines.put(b"\n");
        }
    }
}

pub(crate) fn change_json(this: &Change, json: &mut skein_json::writer::Encoder) {
    json.object_start();
    crate::form::json::field(json, b"kind", b"secrets");
    json.key(b"secrets");
    json.object_start();
    for (list, entries) in
        [(b"files".as_slice(), &this.files), (b"passwords", &this.passwords), (b"identity", &this.identity)]
    {
        json.key(list);
        json.array_start();
        for entry in entries {
            let (key, before, after) = named_sides(entry);
            json.object_start();
            crate::form::json::field(json, b"key", key);
            crate::form::json::field(
                json,
                b"kind",
                match (before, after) {
                    (false, true) => b"added",
                    (true, false) => b"removed",
                    (true, true) => b"changed",
                    (false, false) => unreachable!("change has a side"),
                },
            );
            json.object_end();
        }
        json.array_end();
    }
    json.object_end();
    json.object_end();
}
pub(crate) fn named_sides(this: &Named) -> (&[u8], bool, bool) {
    match this {
        Named::Added(key) => (key, false, true),
        Named::Removed(key) => (key, true, false),
        Named::Changed(key) => (key, true, true),
    }
}
