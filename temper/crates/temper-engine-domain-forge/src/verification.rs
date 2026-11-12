//! Retained landing verdicts (temper/docs/design/domain/after-landing.md, 5.3, 6 and 10).
use crate::{Domain, Verification};
use crate::{Key, Request, Stored, TierVerdict, TierVerification};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api::{Commit, Repository};
use skein_lib::{List, Queue};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_tiers::{self as policy, Blame};

/// What the forge knows of a landing a change names as the red it repairs (temper/docs/design/domain/agent/tools.md, 1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LandingRed {
    /// No live `Verification` row of temper's has this commit in the repository.
    Unknown,
    /// Its changes landed into another branch than the one asked.
    Elsewhere {
        branch: Box<[u8]>,
    },
    /// No tier holds it red: no verdict `Owns` or `Suspect` without its `repaired`.
    NotRed,
    Red,
}

impl Domain {
    /// The live landing's branch and whether any tier still holds it red.
    #[must_use]
    pub fn landing_red(&self, repository: Repository, commit: &Commit, branch: &[u8]) -> LandingRed {
        let Some(row) = self.verifications.get(&(repository, *commit)) else {
            return LandingRed::Unknown;
        };
        for task in &row.changes {
            if let Some(change) = self.changes.get(task) {
                if change.base.as_ref() != branch {
                    return LandingRed::Elsewhere { branch: change.base.clone() };
                }
                break;
            }
        }
        for tier in &row.tiers {
            if tier.repaired.is_none()
                && let Some(verdict) = &tier.verdict
            {
                match &verdict.blame {
                    Blame::Owns(_) | Blame::Suspect(_) => return LandingRed::Red,
                    Blame::Cleared => {}
                }
            }
        }
        LandingRed::NotRed
    }

    /// Borrow the admitted tiers' retained verdicts until the change releases its landing.
    #[must_use]
    pub fn change_verdicts(&self, task: u64) -> &[TierVerification] {
        for (_, row) in &self.verifications {
            if row.changes.contains(&task) {
                return &row.tiers;
            }
        }
        &[]
    }

    /// The landing's retained verdicts, including a merge observed before its change steps.
    #[must_use]
    pub fn change_verification(&self, task: u64) -> change::Verification {
        let tiers = self.change_verdicts(task);
        if tiers.is_empty() { change::Verification::Unfollowed } else { fact(tiers) }
    }

    /// The live row of this repository and commit (temper/docs/design/domain/after-landing.md, 6).
    #[must_use]
    pub fn landing_verification(&self, repository: Repository, commit: &Commit) -> Option<&Verification> {
        self.verifications.get(&(repository, *commit))
    }
}

/// Save with the merge's Landed row; a found-again merge preserves its verdicts.
pub(crate) fn landed(d: &mut Domain, commit: Commit, task: u64, out: &mut Queue<Request>) {
    let Some(change) = d.changes.get(&task) else { return };
    let repository = change.repository;
    let subject = crate::views::changes_mark(d, repository);
    if let Some(row) = d.verifications.get_mut(&(repository, commit)) {
        if !row.changes.contains(&task) {
            let mut changes = List::with_capacity(d.landing.limits.changes);
            for prior in &row.changes {
                changes.push(*prior).expect("live changes");
            }
            changes.push(task).expect("one landing per live change");
            row.changes = changes.into_boxed();
            save(row.clone(), subject, out);
        }
        return;
    }
    let Some(adopted) = d.repositories.get(&repository) else { return };
    let Some((_, configured, _, _, _, _)) = d.landing.projects.get(&adopted.project) else { return };
    let mut tiers = List::with_capacity(d.landing.limits.tiers.tiers);
    for entry in configured {
        if entry.repository != repository {
            continue;
        }
        for tier in &entry.tiers {
            let branch = match &tier.follows {
                crate::Follows::Landing => &adopted.settings.default_branch,
                crate::Follows::Branch(branch) => branch,
                crate::Follows::Goals => continue,
            };
            if branch == &change.base {
                tiers
                    .push(TierVerification { tier: tier.name.clone(), verdict: None, repaired: None })
                    .expect("admitted tiers");
            }
        }
    }
    if tiers.is_empty() {
        return;
    }
    let row = Verification { repository, commit, changes: Box::new([task]), tiers: tiers.into_boxed() };
    d.verifications.insert((repository, commit), row.clone()).expect("one landing per live change");
    save(row, subject, out);
}

fn fact(tiers: &[TierVerification]) -> change::Verification {
    let mut cleared = true;
    for tier in tiers {
        if tier.repaired.is_some() {
            continue;
        }
        match &tier.verdict {
            Some(TierVerdict { blame: Blame::Owns(_) | Blame::Suspect(_), .. }) => return change::Verification::Red,
            Some(TierVerdict { blame: Blame::Cleared, .. }) => {}
            None => cleared = false,
        }
    }
    if cleared { change::Verification::Cleared } else { change::Verification::Verifying }
}

/// Judge the completed range even after the tier record has advanced to its next run.
pub(crate) fn judged(
    d: &mut Domain,
    repository: Repository,
    tier: &[u8],
    verdict: &policy::Verdict,
    landings: &[policy::Landing],
    out: &mut Queue<Request>,
) {
    assert_eq!(landings.len(), verdict.landings.len(), "one blame per landing in the judged range");
    let subject = crate::views::changes_mark(d, repository);
    let mut keys = List::with_capacity(d.verifications.len());
    for ((repo, commit), _) in &d.verifications {
        if *repo == repository {
            keys.push((repository, *commit)).expect("retained verifications");
        }
    }
    for key in keys.into_boxed() {
        let mut by = None;
        'landings: for landing in landings.iter().rev() {
            let Some(row) = d.verifications.get(&(repository, landing.commit)) else { continue };
            for task in &row.changes {
                if let Some(change) = d.changes.get(task)
                    && change.repairs == Some(key.1)
                {
                    by = Some(landing.commit);
                    break 'landings;
                }
            }
        }
        let row = d.verifications.get_mut(&key).expect("retained verification");
        let commit = row.commit;
        let mut changed = false;
        for verification in &mut row.tiers {
            if verification.tier.as_ref() != tier {
                continue;
            }
            for (landing, blame) in landings.iter().zip(&verdict.landings) {
                if commit == landing.commit && verification.verdict.is_none() {
                    verification.verdict = Some(TierVerdict { run: verdict.number, blame: blame.clone() });
                    changed = true;
                }
            }
            if verification.repaired.is_none()
                && let Some(prior) = &verification.verdict
                && prior.run < verdict.number
            {
                match &prior.blame {
                    Blame::Owns(_) | Blame::Suspect(_) if policy::repaired(&prior.blame, &verdict.failing) => {
                        verification.repaired = Some(crate::TierRepair { run: verdict.number, tip: verdict.tip, by });
                        changed = true;
                    }
                    Blame::Cleared | Blame::Owns(_) | Blame::Suspect(_) => {}
                }
            }
        }
        if changed {
            for task in &row.changes {
                d.procedure_wakes.insert(*task, true).expect("one pending wake per change");
            }
            save(row.clone(), subject, out);
        }
    }
}

pub(crate) fn released(d: &mut Domain, task: u64, out: &mut Queue<Request>) {
    let mut found = None;
    for (key, row) in &d.verifications {
        if row.changes.contains(&task) {
            found = Some(*key);
            break;
        }
    }
    let Some(key) = found else { return };
    let mut row = d.verifications.remove(&key).expect("found verification");
    let subject = crate::views::changes_mark(d, row.repository);
    if row.changes.len() == 1 {
        if let Some(subject) = subject {
            out.push(Request::Marked { tasks: Box::new(subject) });
        }
        out.push(Request::Erase { key: Key::Verification { repository: row.repository, commit: row.commit } });
    } else {
        let mut changes = List::with_capacity(u32::try_from(row.changes.len()).expect("admitted changes"));
        for change in &row.changes {
            if *change != task {
                changes.push(*change).expect("removes one change");
            }
        }
        row.changes = changes.into_boxed();
        d.verifications.insert(key, row.clone()).expect("replaces verification");
        save(row, subject, out);
    }
}

fn save(row: Verification, subject: Option<[u64; 3]>, out: &mut Queue<Request>) {
    if let Some(subject) = subject {
        out.push(Request::Marked { tasks: Box::new(subject) });
    }
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Verification(row) });
}

pub(crate) fn row_bytes(row: &Verification) -> Option<u64> {
    let mut held = skein_lib::len_u64(row.changes.len()).checked_mul(8)?.checked_add(
        skein_lib::len_u64(row.tiers.len()).checked_mul(u64::try_from(size_of::<TierVerification>()).ok()?)?,
    )?;
    for tier in &row.tiers {
        held = held.checked_add(skein_lib::len_u64(tier.tier.len()))?;
        if let Some(verdict) = &tier.verdict {
            held = held.checked_add(crate::tiers::blame_bytes(&verdict.blame)?)?;
        }
    }
    Some(held)
}
