//! CI workflow names at the landing branch's tip
//! (temper/docs/design/domain/forge/adoption.md, 7).
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Env, List, Map, Queue, Set, Token};
pub use temper_engine_domain_forge_vocabulary::workflows::*;

use crate::{Domain, Kind, Limits, Request, Stored};

const GITHUB: &[u8] = b".github/workflows";
const FORGEJO: &[u8] = b".forgejo/workflows";

#[derive(Hash, Debug)]
struct Pending {
    repository: api::Repository,
    tip: Option<api::Commit>,
    path: Box<[u8]>,
    again: bool,
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    reads: Map<Token, Pending>,
    marked: Set<api::Repository>,
}

impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { reads: Map::with_capacity(limits.repositories), marked: Set::with_capacity(limits.repositories) }
    }

    pub(crate) fn reading(&self, owner: Token) -> bool {
        self.reads.contains_key(&owner)
    }
}

fn mark(d: &mut Domain, repository: api::Repository) {
    for _ in 0..d.workflows.marked.capacity() {
        let mut gone = None;
        for marked in &d.workflows.marked {
            if !d.repositories.contains_key(marked) {
                gone = Some(*marked);
                break;
            }
        }
        let Some(gone) = gone else { break };
        d.workflows.marked.remove(&gone);
    }
    if d.repositories.contains_key(&repository) {
        d.workflows.marked.insert(repository).expect("one mark per adopted repository");
    }
}

/// Read at tip, else at the landing branch's tip read first.
pub(crate) fn read(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: api::Repository,
    tip: Option<api::Commit>,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.repositories.get(&repository) else { return };
    let Some(configured) = d.configured.get(&repository.forge) else { return };
    for (owner, pending) in &d.workflows.reads {
        if pending.repository == repository {
            let owner = *owner;
            d.workflows.reads.get_mut(&owner).expect("in-flight workflow read").again = true;
            return;
        }
    }
    let Some(sequence) = d.sequence.checked_add(1) else {
        mark(d, repository);
        return;
    };
    if d.workflows.reads.len() >= env.limits.repositories || sequence >= 1_u64 << 60_u32 {
        mark(d, repository);
        return;
    }
    let path: Box<[u8]> = Box::from(match configured.kind {
        Kind::GitHub => GITHUB,
        Kind::Forgejo => FORGEJO,
    });
    let read = match tip {
        Some(head) => api::Read::File { head, path: path.clone(), max_bytes: env.limits.client.answer_bytes },
        None => api::Read::Branch { branch: row.settings.default_branch.clone() },
    };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 60_u32));
    d.workflows.marked.remove(&repository);
    d.workflows
        .reads
        .insert(owner, Pending { repository, tip, path, again: false })
        .expect("read capacity preflighted");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}

#[expect(clippy::too_many_lines, reason = "one read transition exhaustively names every answer and failure shape")]
pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.workflows.reads.remove(&owner) else { return };
    let repository = pending.repository;
    if !d.repositories.contains_key(&repository) || !d.configured.contains_key(&repository.forge) {
        return;
    }
    let mut next = None;
    let mut found = None;
    match result {
        Ok(api::Answer::Commit(tip)) => {
            if pending.tip.is_none() {
                pending.tip = Some(tip);
                next = Some(api::Read::File {
                    head: tip,
                    path: pending.path.clone(),
                    max_bytes: env.limits.client.answer_bytes,
                });
            }
        }
        Ok(api::Answer::Directory { head, path, entries, truncated: _ }) => {
            if pending.tip == Some(head) && pending.path == path {
                found = Some(names(&entries));
            }
        }
        Err(api::Error::Missing) | Ok(api::Answer::File { .. }) => {
            if let Some(head) = pending.tip {
                if pending.path.as_ref() == FORGEJO {
                    pending.path = Box::from(GITHUB);
                    next = Some(api::Read::File {
                        head,
                        path: pending.path.clone(),
                        max_bytes: env.limits.client.answer_bytes,
                    });
                } else {
                    found = Some(Box::default());
                }
            }
        }
        Ok(
            api::Answer::Located { .. }
            | api::Answer::Items { .. }
            | api::Answer::Item { .. }
            | api::Answer::Pull(_)
            | api::Answer::Reviews { .. }
            | api::Answer::Statuses { .. }
            | api::Answer::Remarks { .. }
            | api::Answer::Branches(_)
            | api::Answer::PullFiles { .. }
            | api::Answer::Compare { .. }
            | api::Answer::Checks(_)
            | api::Answer::Job { .. }
            | api::Answer::Protection(_)
            | api::Answer::Settings(_)
            | api::Answer::Collaborators { .. }
            | api::Answer::Permission(_)
            | api::Answer::Created(_)
            | api::Answer::Commented(_)
            | api::Answer::Reviewed(_)
            | api::Answer::Merged(_)
            | api::Answer::Queued
            | api::Answer::Branch(_)
            | api::Answer::Runs { .. }
            | api::Answer::Run(_)
            | api::Answer::Tree { .. }
            | api::Answer::Commits { .. }
            | api::Answer::Done,
        )
        | Err(
            api::Error::Forbidden
            | api::Error::Unavailable
            | api::Error::Busy
            | api::Error::Refused
            | api::Error::Conflict
            | api::Error::InvalidAnswer
            | api::Error::Timeout
            | api::Error::MissingJob
            | api::Error::TooLarge(_)
            | api::Error::Empty
            | api::Error::Full
            | api::Error::Exists
            | api::Error::NothingToMerge
            | api::Error::Closed
            | api::Error::Stale
            | api::Error::Protected
            | api::Error::NotConnected
            | api::Error::NotInstalled
            | api::Error::RateLimited { .. },
        ) => {}
    }
    if let Some(read) = next {
        d.workflows.reads.insert(owner, pending).expect("read advances in its reserved slot");
        crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
        return;
    }
    match found {
        Some(names) => {
            let row = d.repositories.get_mut(&repository).expect("answer's adopted repository");
            if row.workflows != names {
                row.workflows = names;
                out.push(Request::Save {
                    read_afresh: None,
                    release: None,
                    record: Stored::Repository(row.as_ref().clone()),
                });
            }
        }
        None => mark(d, repository),
    }
    if pending.again {
        read(d, env, repository, None, out);
    }
}

/// The landing branch's first observed tip.
pub(crate) fn first_tip(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: api::Repository,
    branch: &[u8],
    tip: api::Commit,
    out: &mut Queue<Request>,
) {
    if let Some(row) = d.repositories.get(&repository)
        && row.settings.default_branch.as_ref() == branch
    {
        read(d, env, repository, Some(tip), out);
    }
}

/// A landing published on branch; files none where unknown.
pub(crate) fn landed(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: api::Repository,
    branch: &[u8],
    after: api::Commit,
    files: Option<&[Box<[u8]>]>,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.repositories.get(&repository) else { return };
    if row.settings.default_branch.as_ref() != branch {
        return;
    }
    let mut changed = d.workflows.marked.contains(&repository);
    match files {
        None => changed = true,
        Some(files) => {
            for file in files {
                changed |= file.starts_with(b".github/workflows/") || file.starts_with(b".forgejo/workflows/");
            }
        }
    }
    if changed {
        read(d, env, repository, Some(after), out);
    }
}

fn names(entries: &[api::Entry]) -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(u32::try_from(WORKFLOWS).expect("workflow count fits"));
    for entry in entries {
        if names.len() == names.capacity() {
            break;
        }
        if entry.kind == api::EntryKind::File
            && (entry.name.ends_with(b".yml") || entry.name.ends_with(b".yaml"))
            && entry.name.len() <= WORKFLOW_BYTES
            && skein_lib::bytes::is_utf8(&entry.name)
        {
            names.push(entry.name.clone()).expect("workflow count checked");
        }
    }
    names.into_boxed()
}
