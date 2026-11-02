//! Retained landing verdicts (temper/docs/design/domain/after-landing.md, 5.3, 6 and 10).
use crate::{Domain, Verification};
use crate::{Key, Request, Stored, TierVerdict, TierVerification};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api::{Commit, Repository};
use skein_lib::{List, Queue};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_tiers::{self as policy, Blame};

impl Domain {
    /// The landing's retained verdicts, including a merge observed before its change steps.
    #[must_use]
    pub fn change_verification(&self, task: u64) -> change::Verification {
        for (_, row) in &self.verifications {
            if row.changes.contains(&task) {
                return fact(row);
            }
        }
        change::Verification::Unfollowed
    }
}

/// Save with the merge's Landed row; a found-again merge preserves its verdicts.
pub(crate) fn landed(d: &mut Domain, commit: Commit, task: u64, out: &mut Queue<Request>) {
    let Some(change) = d.changes.get(&task) else { return };
    let repository = change.repository;
    if let Some(row) = d.verifications.get_mut(&(repository, commit)) {
        if !row.changes.contains(&task) {
            let mut changes = List::with_capacity(d.landing.limits.changes);
            for prior in &row.changes {
                changes.push(*prior).expect("live changes");
            }
            changes.push(task).expect("one landing per live change");
            row.changes = changes.into_boxed();
            save(row.clone(), out);
        }
        return;
    }
    let Some(adopted) = d.repositories.get(&repository) else { return };
    let Some((_, configured, _)) = d.landing.projects.get(&adopted.project) else { return };
    let mut tiers = List::with_capacity(d.landing.limits.tiers.tiers);
    for entry in configured {
        if entry.repository != repository {
            continue;
        }
        for tier in &entry.tiers {
            let branch = tier.branch.as_deref().unwrap_or(&adopted.settings.default_branch);
            if branch == change.base.as_ref() {
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
    save(row, out);
}

fn fact(row: &Verification) -> change::Verification {
    let mut cleared = true;
    for tier in &row.tiers {
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
    let mut keys = List::with_capacity(d.verifications.len());
    for ((repo, commit), _) in &d.verifications {
        if *repo == repository {
            keys.push((repository, *commit)).expect("retained verifications");
        }
    }
    for key in keys.into_boxed() {
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
                        verification.repaired = Some(verdict.tip);
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
            save(row.clone(), out);
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
    if row.changes.len() == 1 {
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
        save(row, out);
    }
}

fn save(row: Verification, out: &mut Queue<Request>) {
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Verification(row) });
}

pub(crate) fn row_bytes(row: &Verification) -> Option<u64> {
    let mut held = u64::try_from(row.changes.len()).ok()?.checked_mul(8)?.checked_add(
        u64::try_from(row.tiers.len()).ok()?.checked_mul(u64::try_from(size_of::<TierVerification>()).ok()?)?,
    )?;
    for tier in &row.tiers {
        held = held.checked_add(u64::try_from(tier.tier.len()).ok()?)?;
        if let Some(verdict) = &tier.verdict {
            held = held.checked_add(crate::tiers::blame_bytes(&verdict.blame)?)?;
        }
    }
    Some(held)
}

/// The verification task fills this verdict join (temper/docs/design/domain/after-landing.md, 6).
pub(crate) fn judged(
    _d: &mut Domain,
    _repository: skein_forge::api::Repository,
    _tier: &[u8],
    _verdict: &temper_engine_domain_forge_tiers::Verdict,
) {
}
