//! Retained landing titles (temper/docs/design/domain/after-landing.md, 10).
use crate::{Domain, Key, LandingLog, LoggedChange, LoggedLanding, Request, Stored};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api::{Commit, Repository};
use skein_lib::{List, Queue, Wall};

/// A logged title's UTF-8 byte bound (temper/docs/design/domain/after-landing.md, 10).
pub const LOGGED_TITLE_BYTES: usize = 256;

impl Domain {
    /// A repository's ring, oldest first.
    #[must_use]
    pub fn landing_log(&self, repository: Repository) -> Option<&LandingLog> {
        self.landing_logs.get(&repository)
    }
}

/// Log `task`'s landing at `commit`, in the decision that saw it.
pub(crate) fn landed(
    d: &mut Domain,
    now: Wall,
    commit: Commit,
    task: u64,
    signed: Box<[crate::SignOff]>,
    out: &mut Queue<Request>,
) {
    let Some(change) = d.changes.get(&task) else { return };
    let repository = change.repository;
    if !d.repositories.contains_key(&repository) {
        return;
    }
    let logged = LoggedChange {
        task,
        pull: change.pull,
        title: crate::text::fit(&change.title, LOGGED_TITLE_BYTES, b"..."),
        signed,
    };
    let branch = change.base.clone();
    let limits = &d.landing.limits;
    let mut row = match d.landing_logs.remove(&repository) {
        Some(row) => row,
        None => LandingLog { repository, landings: Box::new([]) },
    };
    let mut found = false;
    let mut changed = false;
    for landing in &mut row.landings {
        if landing.commit != commit {
            continue;
        }
        found = true;
        let mut contains = false;
        for prior in &landing.changes {
            contains |= prior.task == task;
        }
        if !contains && landing.changes.len() < skein_lib::usize_of(limits.landing_members) {
            let mut members = List::with_capacity(limits.landing_members);
            for prior in &landing.changes {
                members.push(prior.clone()).expect("retained members");
            }
            members.push(logged.clone()).expect("member room checked");
            landing.changes = members.into_boxed();
            changed = true;
        }
        break;
    }
    if !found {
        let mut landings = List::with_capacity(limits.landings_kept);
        let mut oldest = true;
        for prior in &row.landings {
            if row.landings.len() == skein_lib::usize_of(limits.landings_kept) && oldest {
                oldest = false;
                // The oldest entry alone leaves a full ring.
                continue;
            }
            landings.push(prior.clone()).expect("retained ring");
        }
        landings.push(LoggedLanding { commit, branch, at: now, changes: Box::new([logged]) }).expect("ring room");
        row.landings = landings.into_boxed();
        changed = true;
    }
    if changed {
        crate::status::landed(d, now, repository, out);
        if let Some(subject) = crate::views::changes_mark(d, repository) {
            out.push(Request::Marked { tasks: Box::new(subject) });
        }
        out.push(Request::Save { read_afresh: None, release: None, record: Stored::LandingLog(row.clone()) });
    }
    d.landing_logs.insert(repository, row).expect("one log per adopted repository");
}

pub(crate) fn erased(d: &mut Domain, repository: Repository, out: &mut Queue<Request>) {
    if d.landing_logs.remove(&repository).is_some() {
        out.push(Request::Erase { key: Key::LandingLog { repository } });
    }
}

pub(crate) fn row_bytes(row: &LandingLog) -> Option<u64> {
    let mut held =
        skein_lib::len_u64(row.landings.len()).checked_mul(u64::try_from(size_of::<LoggedLanding>()).ok()?)?;
    for landing in &row.landings {
        held = held.checked_add(skein_lib::len_u64(landing.branch.len()))?;
        held = held.checked_add(
            skein_lib::len_u64(landing.changes.len()).checked_mul(u64::try_from(size_of::<LoggedChange>()).ok()?)?,
        )?;
        for change in &landing.changes {
            held = held.checked_add(skein_lib::len_u64(change.title.len()))?;
            held = held.checked_add(
                skein_lib::len_u64(change.signed.len())
                    .checked_mul(u64::try_from(size_of::<crate::SignOff>()).ok()?)?,
            )?;
        }
    }
    Some(held)
}
