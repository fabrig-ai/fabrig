//! Pending landings and retained verdicts (temper/docs/design/domain/after-landing.md, 10).
use crate::{
    Domain, Follows, Key, LandingLog, LoggedChange, LoggedLanding, LoggedTier, LoggedVerdict, Request, Stored,
};
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};
use skein_lib::{List, Queue, Wall};
pub use temper_engine_domain_forge_vocabulary::landing_log::*;

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
    crate::milestone_rows::landed(d, now, task, out);
    let change = d.changes.get(&task).expect("landed change");
    let follows = change.follows.clone();
    if let Some(follows) = follows {
        let project = d.repositories.get(&repository).expect("adopted landing").project;
        crate::follows::followed(d, project, follows.follow, &follows.commits, out);
    }
    let change = d.changes.get(&task).expect("landed change");
    let logged = LoggedChange {
        task,
        fixes: fixes(&change.body),
        pull: change.pull,
        title: crate::text::fit(&change.title, LOGGED_TITLE_BYTES, b"..."),
        signed,
        goal: change.goal,
        milestone: change.milestone,
        person: change.person,
        goal_title: change.goal_title.clone(),
        draft: change.draft.clone(),
    };
    let branch = change.base.clone();
    let mut tiers = List::with_capacity(d.landing.limits.tiers.tiers);
    for tier in crate::tiers::configured_all(d, repository) {
        let follows = match &tier.follows {
            Follows::Landing => d.repositories.get(&repository).expect("adopted landing").landing == branch,
            Follows::Branch(followed) => *followed == branch,
            Follows::Goals => false,
        };
        if follows {
            tiers
                .push(LoggedTier { tier: tier.name.clone(), verdict: LoggedVerdict::Verifying })
                .expect("configured tiers");
        }
    }
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
        let count = limits.landings_kept.checked_add(limits.landings_pending).expect("checked landing count");
        let mut landings = List::with_capacity(count.checked_add(1).expect("one new landing"));
        for prior in &row.landings {
            landings.push(prior.clone()).expect("retained ring");
        }
        landings
            .push(LoggedLanding { commit, branch, at: now, changes: Box::new([logged]), tiers: tiers.into_boxed() })
            .expect("ring room");
        row.landings = landings.into_boxed();
        retain(&mut row, limits);
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

/// Keep the first eight distinct exact body lines, sorted by finding number (red-findings.md, 6).
fn fixes(body: &[u8]) -> Box<[u32]> {
    let mut fixes = List::with_capacity(8);
    let mut rest = body;
    for _byte in body {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"\n").unwrap_or(rest.len());
        let line = rest.get(..end).expect("line end within body");
        rest = rest.get(end..).expect("remaining body").strip_prefix(b"\n").unwrap_or(b"");
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let Some(number) = line.strip_prefix(b"Fixes-finding: ") else { continue };
        let Some(number) = skein_lib::bytes::decimal(number) else { continue };
        if let Ok(value) = u32::try_from(number)
            && !fixes.as_slice().contains(&value)
            && fixes.room() > 0
        {
            fixes.push(value).expect("finding room checked");
        }
    }
    let mut fixes = fixes.into_boxed();
    fixes.sort_unstable();
    fixes
}

pub(crate) fn erased(d: &mut Domain, repository: Repository, out: &mut Queue<Request>) {
    if d.landing_logs.remove(&repository).is_some() {
        out.push(Request::Erase { key: Key::LandingLog { repository } });
    }
}

fn pending(landing: &LoggedLanding) -> bool {
    for tier in &landing.tiers {
        if tier.verdict == LoggedVerdict::Verifying {
            return true;
        }
    }
    false
}

/// Pending and judged landings have independent bounds, preserving oldest-first order.
fn retain(row: &mut LandingLog, limits: &crate::Limits) {
    let mut awaiting = 0_u32;
    let mut judged = 0_u32;
    for landing in &row.landings {
        if pending(landing) {
            awaiting = awaiting.checked_add(1).expect("bounded log");
        } else {
            judged = judged.checked_add(1).expect("bounded log");
        }
    }
    let mut pending_drop = awaiting.saturating_sub(limits.landings_pending);
    let mut judged_drop = judged.saturating_sub(limits.landings_kept);
    if pending_drop == 0 && judged_drop == 0 {
        return;
    }
    let mut kept = List::with_capacity(limits.landings_kept.checked_add(limits.landings_pending).expect("checked log"));
    for landing in &row.landings {
        let drop = if pending(landing) { &mut pending_drop } else { &mut judged_drop };
        if *drop > 0 {
            *drop = drop.checked_sub(1).expect("positive drop count");
        } else {
            kept.push(landing.clone()).expect("bounded retained log");
        }
    }
    row.landings = kept.into_boxed();
}

pub(crate) fn judged(
    d: &mut Domain,
    repository: Repository,
    tier: &[u8],
    verdict: &temper_engine_domain_forge_tiers::Verdict,
    landings: &[temper_engine_domain_forge_tiers::Landing],
    out: &mut Queue<Request>,
) {
    use temper_engine_domain_forge_tiers::{Blame, Failures};
    let Some(row) = d.landing_logs.get_mut(&repository) else { return };
    let mut changed = false;
    for (landing, blame) in landings.iter().zip(&verdict.landings) {
        for logged in &mut row.landings {
            if logged.commit != landing.commit {
                continue;
            }
            let verdict = match blame {
                Blame::Cleared => LoggedVerdict::Cleared { run: verdict.number },
                Blame::Owns(failures) | Blame::Suspect(failures) => {
                    let failing = match failures {
                        Failures::Many => None,
                        Failures::Tests(tests) => {
                            let mut distinct = List::with_capacity(u32::try_from(tests.len()).expect("bounded tests"));
                            for test in tests {
                                if !distinct.as_slice().contains(&test.as_ref()) {
                                    distinct.push(test.as_ref()).expect("distinct subset");
                                }
                            }
                            Some(distinct.len())
                        }
                    };
                    LoggedVerdict::Red { run: verdict.number, failing }
                }
            };
            let mut entries = List::with_capacity(32);
            let mut found = false;
            for entry in &logged.tiers {
                let mut entry = entry.clone();
                if entry.tier.as_ref() == tier {
                    entry.verdict = verdict.clone();
                    found = true;
                }
                entries.push(entry).expect("logged tier bound");
            }
            if !found {
                entries.push(LoggedTier { tier: tier.into(), verdict }).expect("logged tier bound");
            }
            logged.tiers = entries.into_boxed();
            changed = true;
        }
    }
    if changed {
        retain(row, &d.landing.limits);
        let row = row.clone();
        if let Some(subject) = crate::views::changes_mark(d, repository) {
            out.push(Request::Marked { tasks: Box::new(subject) });
        }
        out.push(Request::Save { read_afresh: None, release: None, record: Stored::LandingLog(row) });
    }
}

/// Earlier logs infer verifying entries from the tier records; removed tiers no longer keep them pending.
pub(crate) fn restored(d: &mut Domain, out: &mut Queue<Request>) {
    let mut repositories = List::with_capacity(d.landing_logs.len());
    for (repository, _) in &d.landing_logs {
        repositories.push(*repository).expect("retained repositories");
    }
    for repository in repositories.into_boxed() {
        let mut row = d.landing_logs.remove(&repository).expect("retained log");
        let before = row.clone();
        for landing in &mut row.landings {
            let mut entries = List::with_capacity(32);
            if landing.tiers.is_empty() {
                for ((repo, _), tier) in &d.tiers {
                    if *repo != repository {
                        continue;
                    }
                    let mut held = false;
                    for pending in &tier.pending {
                        held |= pending.commit == landing.commit;
                    }
                    let in_flight = match &tier.posting {
                        Some(posting) => posting.part != 0,
                        None => true,
                    };
                    if in_flight && let Some(run) = &tier.record.run {
                        for pending in &run.landings {
                            held |= pending.commit == landing.commit;
                        }
                    }
                    if held {
                        entries
                            .push(LoggedTier { tier: tier.tier.clone(), verdict: LoggedVerdict::Verifying })
                            .expect("restored tier bound");
                    }
                }
            } else {
                for entry in &landing.tiers {
                    entries.push(entry.clone()).expect("logged tier bound");
                }
            }
            let mut kept = List::with_capacity(32);
            for entry in entries.into_boxed() {
                let mut configured = false;
                for tier in crate::tiers::configured_all(d, repository) {
                    configured |= tier.name == entry.tier;
                }
                if entry.verdict != LoggedVerdict::Verifying || configured {
                    kept.push(entry).expect("retained tier entries");
                }
            }
            landing.tiers = kept.into_boxed();
        }
        retain(&mut row, &d.landing.limits);
        if row != before {
            out.push(Request::Save { read_afresh: None, release: None, record: Stored::LandingLog(row.clone()) });
        }
        d.landing_logs.insert(repository, row).expect("replaces restored log");
    }
}
