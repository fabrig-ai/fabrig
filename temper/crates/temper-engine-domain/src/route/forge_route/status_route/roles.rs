//! Standing roles' Nows in status words (domain/forge/attention.md, 3; domain/agent/standing.md, 8).
use super::{Domain, Env, Limits, Work, forge};
use jig_domain_notes as notes;
use skein_lib::{Decimal, List, Token};

const MOST: u32 = 8;
const NOW_BYTES: usize = 1024;

/// One status run reads its roles in order: the notes child serves one read at a time.
#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct Reading {
    project: u32,
    words: List<u8>,
    roles: List<u64>,
    at: u32,
    name: Option<u64>,
    heading: bool,
}

pub(super) fn start(domain: &mut Domain, env: &Env<Limits>, project: u32, words: List<u8>) {
    let mut candidates = List::uncapped();
    for candidate in domain.core.tasks.view_tasks() {
        let Some(row) = domain.core.tasks.task(candidate.number) else { continue };
        if row.project == project && row.label == 3 {
            candidates.push(row.number).expect("internal role list");
        }
    }
    let mut candidates = candidates.into_boxed();
    candidates.sort_unstable();
    let mut roles = List::with_capacity(MOST);
    for number in candidates.into_iter().take(skein_lib::usize_of(MOST)) {
        roles.push(number).expect("at most eight roles");
    }
    if roles.is_empty() {
        super::start_task(domain, env, project, words.as_slice());
        return;
    }
    let Some(serial) = domain.next_status_owner.checked_add(1) else {
        super::start_task(domain, env, project, words.as_slice());
        return;
    };
    domain.next_status_owner = serial;
    // Slot u32::MAX is outside any slab: a root owner cannot take a core note flight.
    let owner = Token::new(u64::MAX.saturating_sub(u64::from(serial)));
    let read = Reading { project, words, roles, at: 0, name: None, heading: false };
    next(domain, env, owner, read);
}

fn next(domain: &mut Domain, env: &Env<Limits>, owner: Token, mut read: Reading) {
    read.name = None;
    let Some(role) = read.roles.get(read.at).copied() else {
        super::start_task(domain, env, read.project, read.words.as_slice());
        return;
    };
    let mut scopes = List::with_capacity(1);
    scopes.push(notes::Scope::Goal { project: read.project, goal: role }).expect("one role scope");
    assert!(domain.status_reading.insert(owner, read) == Ok(None), "one pending status notes read");
    domain.work.push(Work::Core(jig_domain_core::Event::Notes(notes::Event::Index {
        owner,
        scopes,
        most: env.limits.notes.lines,
    })));
}

pub(crate) fn indexed(domain: &mut Domain, env: &Env<Limits>, owner: Token, lines: List<notes::Line>, more: u32) {
    let Some(mut read) = domain.status_reading.remove(&owner) else { return };
    let role = *read.roles.get(read.at).expect("pending role");
    let scope = notes::Scope::Goal { project: read.project, goal: role };
    for line in &lines {
        if line.scope == scope && line.description.as_ref() == b"now" {
            read.name = Some(match read.name {
                Some(prior) => prior.min(line.name),
                None => line.name,
            });
        }
    }
    if let Some(name) = read.name {
        assert!(domain.status_reading.insert(owner, read) == Ok(None), "role recall owns its flight");
        domain.work.push(Work::Core(jig_domain_core::Event::Notes(notes::Event::Recall {
            owner,
            by: notes::Recall::Name { name },
            page: 0,
        })));
    } else {
        // An incomplete index cannot establish that there is no Now.
        if more == 0 {
            append_role(&mut read, b"no Now yet");
        }
        advance(domain, env, owner, read);
    }
}

pub(crate) fn recalled(domain: &mut Domain, env: &Env<Limits>, owner: Token, entries: List<notes::Entry>) {
    let Some(mut read) = domain.status_reading.remove(&owner) else { return };
    let role = *read.roles.get(read.at).expect("pending role");
    let scope = notes::Scope::Goal { project: read.project, goal: role };
    for entry in &entries {
        if Some(entry.name) == read.name && entry.scope == scope && entry.description.as_ref() == b"now" {
            let body = forge::text::fit(&entry.body, NOW_BYTES, b"");
            append_role(&mut read, &body);
            break;
        }
    }
    advance(domain, env, owner, read);
}

pub(crate) fn refused(domain: &mut Domain, env: &Env<Limits>, owner: Token) {
    let Some(read) = domain.status_reading.remove(&owner) else { return };
    advance(domain, env, owner, read);
}

fn advance(domain: &mut Domain, env: &Env<Limits>, owner: Token, mut read: Reading) {
    read.at = read.at.checked_add(1).expect("at most eight roles");
    next(domain, env, owner, read);
}

fn append_role(read: &mut Reading, body: &[u8]) {
    if !read.heading {
        super::super::append(&mut read.words, b"Standing roles:\n");
        read.heading = true;
    }
    super::super::append(&mut read.words, b"Role ");
    let role = *read.roles.get(read.at).expect("pending role");
    super::super::append(&mut read.words, Decimal::of(role).as_bytes());
    super::super::append(&mut read.words, b":\n");
    super::super::append(&mut read.words, body);
    if !body.ends_with(b"\n") {
        super::super::append(&mut read.words, b"\n");
    }
}
