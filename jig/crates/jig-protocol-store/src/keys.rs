//! Ordered keys and owner tags (protocol/store/README.md, sections 4.1 and 4.2).
use alloc::boxed::Box;
use jig_domain_core_vocabulary as domain;
use skein_kv::{KeyReader, KeyWriter};
mod accounts;
mod core;
mod notes;
mod own;
mod people;
mod settings;
pub(crate) use settings::{changed_model, model_changes};
mod tasks;
pub use own::Own;
pub const COMPONENT: u8 = 0;
pub const JIG: u8 = 1;
pub const APPLICATION: u8 = 2;
/// Owner tags and their census prefix lengths (protocol/store/README.md, 4.2 and 6.3).
pub(crate) const TAGS: &[(u8, u8)] = &[(COMPONENT, 2), (JIG, 3), (APPLICATION, 2)];

/// Names beside the key writers, one per family prefix (6.3).
pub const KEYSPACES: &[(&[u8], &str)] = &[
    (&[0, 0], "component.format"),
    (&[0, 1], "component.transcripts"),
    (&[0, 2], "component.attempts"),
    (&[0, 3], "component.layouts"),
    (&[0, 4], "component.wires"),
    (&[0, 5], "component.openings"),
    (&[1, 0, 0], "core.deployment"),
    (&[1, 0, 1], "core.projections"),
    (&[1, 0, 2], "core.run-proofs"),
    (&[1, 0, 3], "core.calls"),
    (&[1, 0, 4], "core.turns"),
    (&[1, 0, 5], "core.terminals"),
    (&[1, 0, 6], "core.proposal-decisions"),
    (&[1, 0, 7], "core.escalation-decisions"),
    (&[1, 0, 8], "core.settings"),
    (&[1, 0, 9], "core.settings-history"),
    (&[1, 1, 0], "tasks.live"),
    (&[1, 1, 1], "tasks.stubs"),
    (&[1, 1, 2], "tasks.ledgers"),
    (&[1, 1, 3], "tasks.writers"),
    (&[1, 1, 4], "tasks.pools"),
    (&[1, 1, 5], "tasks.person-proposals"),
    (&[1, 1, 6], "tasks.ended"),
    (&[1, 1, 7], "tasks.closed-ledgers"),
    (&[1, 1, 8], "tasks.decided-proposals"),
    (&[1, 1, 9], "tasks.history"),
    (&[1, 1, 10], "tasks.milestones"),
    (&[1, 1, 11], "tasks.activity"),
    (&[1, 1, 12], "tasks.updates"),
    (&[1, 1, 13], "tasks.results"),
    (&[1, 1, 17], "tasks.ended-delegates"),
    (&[1, 1, 18], "tasks.delegates"),
    (&[1, 1, 19], "tasks.ended-by-party"),
    (&[1, 1, 20], "tasks.ended-by-project"),
    (&[1, 1, 21], "tasks.placed-holds"),
    (&[1, 2, 0], "people.people"),
    (&[1, 2, 1], "people.read-results"),
    (&[1, 2, 2], "people.read-conversations"),
    (&[1, 2, 3], "people.sign-ins"),
    (&[1, 2, 4], "people.roles"),
    (&[1, 2, 5], "people.policies"),
    (&[1, 2, 6], "people.answers"),
    (&[1, 2, 7], "people.ended-reads"),
    (&[1, 2, 8], "people.acted"),
    (&[1, 2, 9], "people.deletions"),
    (&[1, 2, 10], "people.public"),
    (&[1, 2, 11], "people.deleted"),
    (&[1, 3, 0], "notes.entries"),
    (&[1, 3, 1], "notes.lines"),
    (&[1, 4, 0], "accounts.next"),
    (&[1, 4, 1], "accounts.accounts"),
    (&[1, 4, 2], "accounts.sources"),
    (&[1, 4, 3], "component.installation"),
    (&[1, 4, 4], "accounts.issued"),
    (&[1, 4, 5], "accounts.facts"),
];

/// The accounts child's tag under jig's, after the core's own and the tasks',
/// people's and notes' children (4.2).
pub const ACCOUNTS: u8 = 4;
/// The installation's identifier's key (4.4): the accounts' one row without a
/// key of its own, after the tags of the domain's accounts keys (the counter,
/// an account, a source: 0 to 2).
#[must_use]
pub fn installation(limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(JIG);
    writer.tag(ACCOUNTS);
    writer.tag(3);
    writer.finish()
}
#[must_use]
pub fn encode_own(key: &Own, limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(COMPONENT);
    own::write(&mut writer, key);
    writer.finish()
}
#[must_use]
pub fn decode_own(bytes: &[u8]) -> Option<Own> {
    let mut reader = KeyReader::new(bytes);
    if reader.tag()? != COMPONENT {
        return None;
    }
    let key = own::read(&mut reader)?;
    if !reader.is_empty() {
        return None;
    }
    Some(key)
}
#[must_use]
pub fn encode(key: &domain::Key, limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(JIG);
    match key {
        domain::Key::Core(key) => {
            writer.tag(0);
            core::write(&mut writer, key);
        }
        domain::Key::People(key) => {
            writer.tag(2);
            people::write(&mut writer, key);
        }
        domain::Key::Tasks(key) => {
            writer.tag(1);
            tasks::write(&mut writer, key);
        }
        domain::Key::Notes(key) => {
            writer.tag(3);
            notes::write(&mut writer, key);
        }
        domain::Key::Accounts(key) => {
            writer.tag(ACCOUNTS);
            accounts::write(&mut writer, key);
        }
    }
    writer.finish()
}
#[must_use]
pub fn decode(bytes: &[u8]) -> Option<domain::Key> {
    let mut reader = KeyReader::new(bytes);
    if reader.tag()? != JIG {
        return None;
    }
    let key = match reader.tag()? {
        0 => domain::Key::Core(core::read(&mut reader)?),
        1 => domain::Key::Tasks(tasks::read(&mut reader)?),
        2 => domain::Key::People(people::read(&mut reader)?),
        3 => domain::Key::Notes(notes::read(&mut reader, bytes.len())?),
        ACCOUNTS => domain::Key::Accounts(accounts::read(&mut reader)?),
        _ => return None,
    };
    if !reader.is_empty() {
        return None;
    }
    Some(key)
}
#[must_use]
pub fn application(key: &[u8], limit: u32) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(limit);
    writer.tag(APPLICATION);
    for &byte in key {
        writer.tag(byte);
    }
    writer.finish()
}
#[must_use]
pub fn of_application(bytes: &[u8]) -> Option<&[u8]> {
    let (&tag, key) = bytes.split_first()?;
    if tag != APPLICATION {
        return None;
    }
    Some(key)
}
