//! Forge users in jig's parties (domain/forge/adoption.md; protocol/store.md, 2).

use jig_domain_people::IdentityKey;
use skein_lib::{Reader, Writer};

/// jig's provider whose subjects are forge users.
pub const FORGE_PROVIDER: u16 = 0;

/// The key of `user` on `forge`: `FORGE_PROVIDER`, with a subject of 10 bytes,
/// the forge's number then the user's, each big-endian.
#[must_use]
pub fn user_key(forge: u16, user: u64) -> IdentityKey {
    let mut subject = Writer::new(10);
    subject.put(&forge.to_be_bytes()).expect("forge fits the subject");
    subject.put(&user.to_be_bytes()).expect("user fits the subject");
    IdentityKey { provider: FORGE_PROVIDER, subject: subject.finish() }
}

/// The forge and user a key names; `None` for another provider's key or a
/// subject of another form.
#[must_use]
pub fn key_user(key: &IdentityKey) -> Option<(u16, u64)> {
    if key.provider != FORGE_PROVIDER || key.subject.len() != 10 {
        return None;
    }
    let mut subject = Reader::new(&key.subject);
    Some((subject.u16()?, subject.u64()?))
}
