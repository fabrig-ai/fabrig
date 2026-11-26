//! Stored forming refusals (temper/docs/design/domain/landing-forming.md, 3).
use crate::{Domain, FormingRefusal, Request, Stored, TrainRefusal};
use alloc::boxed::Box;
use skein_forge::api::Repository;
use skein_lib::{List, Queue, Wall};

impl Domain {
    /// The train's episode and, for a table refusal, its current holders in task order.
    #[must_use]
    pub fn forming_refusal(&self, repository: Repository, branch: &[u8]) -> Option<(&TrainRefusal, Box<[u64]>)> {
        let row = self.repository(repository)?;
        for refusal in &row.refusals {
            if refusal.branch.as_ref() == branch {
                let holders = match refusal.cause {
                    FormingRefusal::Landings => self.landing_room_holders(),
                    FormingRefusal::Jig { .. } => Box::new([]),
                };
                return Some((refusal, holders));
            }
        }
        None
    }

    /// The first jig retry not already judged at the root's quiet point.
    #[must_use]
    pub fn forming_retry_after(&self, judged: Option<Wall>) -> Option<Wall> {
        let mut next: Option<Wall> = None;
        for (_, row) in &self.repositories {
            for refusal in &row.refusals {
                let retry = match refusal.cause {
                    FormingRefusal::Jig { retry, .. } => retry,
                    FormingRefusal::Landings => continue,
                };
                if let Some(judged) = judged
                    && retry <= judged
                {
                    continue;
                }
                next = Some(match next {
                    Some(prior) => prior.min(retry),
                    None => retry,
                });
            }
        }
        next
    }

    /// The next stored episode whose train has no waiting changes.
    #[must_use]
    pub fn empty_refused_train(&self, waiting: &[(Repository, Box<[u8]>)]) -> Option<(Repository, Box<[u8]>)> {
        for (provider, row) in &self.repositories {
            for refusal in &row.refusals {
                let mut found = false;
                for (repository, branch) in waiting {
                    if provider == repository && refusal.branch == *branch {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Some((*provider, refusal.branch.clone()));
                }
            }
        }
        None
    }
}

pub(crate) fn replace(
    d: &mut Domain,
    out: &mut Queue<Request>,
    repository: Repository,
    branch: Box<[u8]>,
    cause: Option<FormingRefusal>,
    now: Wall,
) {
    let Some(row) = d.repositories.get_mut(&repository) else { return };
    let mut since = now;
    let mut found = false;
    for refusal in &row.refusals {
        if refusal.branch == branch {
            found = true;
            since = refusal.since;
            break;
        }
    }
    if cause.is_none() && !found {
        return;
    }
    let mut next = match cause {
        Some(cause) => Some(TrainRefusal { branch: branch.clone(), since, cause }),
        None => None,
    };
    let mut updated = List::with_capacity(d.landing.limits.changes);
    for refusal in &row.refusals {
        if refusal.branch >= branch
            && let Some(next) = next.take()
        {
            updated.push(next).expect("bounded by waiting changes");
        }
        if refusal.branch != branch {
            updated.push(refusal.clone()).expect("bounded by waiting changes");
        }
    }
    if let Some(next) = next {
        updated.push(next).expect("bounded by waiting changes");
    }
    row.refusals = updated.into_boxed();
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Repository(row.as_ref().clone()) });
    for (task, change) in &d.changes {
        if change.repository == repository && change.base == branch {
            d.wait_reports.changed(*task);
        }
    }
}
