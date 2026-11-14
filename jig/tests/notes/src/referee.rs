//! The notes referee checks only callers' intents and durable store records.
//! It never inspects the child domain (domain/testing.md, section 7).

use std::collections::BTreeMap;

use jig_domain_notes::{Author, Change, Entry, Event, Key, Limits, Record, Scope};
use skein_lib::Token;

use crate::world::key_of;

/// One write staged by the parent before its atomic commit.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Write {
    /// Save this durable record.
    Save(Record),
    /// Erase this durable key.
    Erase(Key),
}

/// What the caller asked before the child loaded any records.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intent {
    /// Erase the lines and entries in this project.
    DropProject { owner: Token, project: u32 },
    /// An index or recall.
    Read { owner: Token },
    /// A task's write, with the revision it saw.
    Write { owner: Token, name: u64, recalled: Option<u32> },
    /// A party's correction or deletion, with the revision it saw.
    Edit { owner: Token, party: u64, name: u64, recalled: u32 },
}

impl Intent {
    /// Observe a caller event, before it is moved into the child.
    #[must_use]
    pub fn from_event(event: &Event) -> Intent {
        match event {
            Event::DropProject { owner, project } => Intent::DropProject { owner: *owner, project: *project },
            Event::Index { owner, scopes: _, most: _ } | Event::Recall { owner, by: _, page: _ } => {
                Intent::Read { owner: *owner }
            }
            Event::Write { owner, entry, recalled } => {
                Intent::Write { owner: *owner, name: entry.name, recalled: *recalled }
            }
            Event::WriteLines { owner, entry, recalled, .. } => {
                Intent::Write { owner: *owner, name: entry.name, recalled: Some(*recalled) }
            }
            Event::Edit { owner, party, name, change, .. } => {
                let recalled = match change {
                    Change::Correct { description: _, body: _, references: _, recalled }
                    | Change::Delete { recalled } => *recalled,
                };
                Intent::Edit { owner: *owner, party: *party, name: *name, recalled }
            }
            Event::Loaded { .. } | Event::LoadFailed { .. } => {
                unreachable!("a caller starts with a call")
            }
        }
    }

    /// The token the caller expects back.
    #[must_use]
    pub const fn owner(self) -> Token {
        match self {
            Intent::DropProject { owner, .. }
            | Intent::Read { owner }
            | Intent::Write { owner, .. }
            | Intent::Edit { owner, .. } => owner,
        }
    }
}

/// Checks commits and the resulting store from outside the notes child.
#[derive(Debug)]
pub struct Referee;

impl Referee {
    /// A commit saves or erases both records and never advances over an unseen revision.
    #[expect(
        clippy::too_many_lines,
        reason = "the referee checks each atomic commit shape and its caller's intent together"
    )]
    pub fn before_commit(
        &self,
        store: &BTreeMap<Key, Record>,
        writes: &[Write],
        intent: &Intent,
    ) -> Result<(), String> {
        if let Intent::DropProject { project, .. } = intent {
            for write in writes {
                let Write::Erase(key) = write else {
                    return Err("a project drop only erases index lines and their entries".into());
                };
                let scope = match key {
                    Key::Line { scope, .. } => scope,
                    Key::Entry { .. } => {
                        let Some(Record::Entry(entry)) = store.get(key) else {
                            return Err("drop erases an existing entry".into());
                        };
                        &entry.scope
                    }
                };
                let in_project = match scope {
                    Scope::Deployment | Scope::Party { .. } => false,
                    Scope::Project { project: found }
                    | Scope::Goal { project: found, .. }
                    | Scope::Resources { project: found, .. } => project == found,
                };
                if !in_project {
                    return Err("drop erases only its project's scopes".into());
                }
                if !store.contains_key(key) {
                    return Err("drop erases an existing line or entry".into());
                }
            }
            return Ok(());
        }
        match writes {
            [Write::Save(Record::Entry(entry)), Write::Save(Record::Line(line))] => {
                if entry.name != line.name {
                    return Err("entry and index line name the same note".into());
                }
                if entry.scope != line.scope {
                    return Err("entry and index line have one scope".into());
                }
                if entry.description != line.description {
                    return Err("index line has the committed description".into());
                }
                if entry.revision != line.revision {
                    return Err("index line has the committed revision".into());
                }
                let recalled = match intent {
                    Intent::Write { name, recalled, .. } => {
                        if *name != entry.name {
                            return Err("write names its entry".into());
                        }
                        *recalled
                    }
                    Intent::Edit { party, name, recalled, .. } => {
                        if *name != entry.name {
                            return Err("edit names its entry".into());
                        }
                        if entry.author != (Author::Party { party: *party }) {
                            return Err("party authored the correction".into());
                        }
                        Some(*recalled)
                    }
                    Intent::Read { .. } | Intent::DropProject { .. } => return Err("a read makes no write".into()),
                };
                match store.get(&Key::Entry { name: entry.name }) {
                    None => {
                        if recalled.is_some() {
                            return Err("a new entry was not recalled".into());
                        }
                        if entry.revision != 1 {
                            return Err("a new entry starts at revision one".into());
                        }
                    }
                    Some(Record::Entry(old)) => {
                        if recalled != Some(old.revision) {
                            return Err("a revision names the one the writer saw".into());
                        }
                        if entry.revision != old.revision.checked_add(1).ok_or("revision fits")? {
                            return Err("revision advances once".into());
                        }
                        if entry.scope != old.scope {
                            return Err("an entry does not move between scopes".into());
                        }
                    }
                    Some(Record::Line(_)) => return Err("an entry key holds an entry".into()),
                }
            }
            [Write::Erase(Key::Entry { name }), Write::Erase(Key::Line { scope, name: line_name })] => {
                if name != line_name {
                    return Err("deletion erases entry and line together".into());
                }
                let Intent::Edit { name: asked, recalled, .. } = intent else {
                    return Err("a party deletes an entry".into());
                };
                if name != asked {
                    return Err("deletion names the asked entry".into());
                }
                let Some(Record::Entry(old)) = store.get(&Key::Entry { name: *name }) else {
                    return Err("a deletion has an entry".into());
                };
                if old.scope != *scope {
                    return Err("deletion erases the right scope line".into());
                }
                if old.revision != *recalled {
                    return Err("deletion names the revision the party saw".into());
                }
            }
            _ => return Err(format!("one commit saves or erases an entry and its line together: {writes:?}")),
        }
        Ok(())
    }

    /// Durable index lines agree with their entries, and every scope stays bounded.
    pub fn after_commit(&self, store: &BTreeMap<Key, Record>, limits: &Limits) -> Result<(), String> {
        let mut counts = BTreeMap::<Scope, u32>::new();
        for (key, record) in store {
            if key != &key_of(record) {
                return Err("record is stored under its own key".into());
            }
            match record {
                Record::Entry(entry) => {
                    let line_key = Key::Line { scope: entry.scope.clone(), name: entry.name };
                    if let Some(Record::Line(line)) = store.get(&line_key) {
                        if line.description != entry.description {
                            return Err("index describes the committed entry".into());
                        }
                        if line.revision != entry.revision {
                            return Err("index gives the committed revision".into());
                        }
                    }
                    if entry.description.len() > skein_lib::length::usize_of(limits.description_bytes) {
                        return Err(
                            "entry.description.len() <= skein_lib::length::usize_of(limits.description_bytes)".into()
                        );
                    }
                    if entry.body.len() > skein_lib::length::usize_of(limits.body_bytes) {
                        return Err("entry.body.len() <= skein_lib::length::usize_of(limits.body_bytes)".into());
                    }
                    if entry.references.len() > limits.references {
                        return Err("entry.references.len() <= limits.references".into());
                    }
                }
                Record::Line(line) => {
                    let Some(Record::Entry(entry)) = store.get(&Key::Entry { name: line.name }) else {
                        return Err("index line has its entry".into());
                    };
                    if entry.description != line.description {
                        return Err("index describes the committed entry".into());
                    }
                    if entry.revision != line.revision {
                        return Err("index gives the committed revision".into());
                    }
                    let count = counts.entry(line.scope.clone()).or_insert(0);
                    *count = count.checked_add(1).ok_or("scope count fits")?;
                    if *count > limits.entries_per_scope {
                        return Err("nothing held beyond entries per scope".into());
                    }
                }
            }
        }
        Ok(())
    }
}

/// A durable entry fixture for the referee's negative cases.
#[must_use]
pub fn saved_entry(name: u64, scope: Scope, revision: u32) -> Entry {
    Entry {
        name,
        scope,
        description: b"entry".as_slice().into(),
        body: b"body".as_slice().into(),
        references: skein_lib::List::with_capacity(0),
        author: Author::Task { task: 1, attempt: 1 },
        revision,
        changed: skein_lib::Wall::EPOCH,
    }
}
