//! A landing verdict files only failures not held by its tier (domain/red-findings.md, 2–5 and 9).
use super::{Domain, Env, Limits, List, Request, TierRow, p, red};
use crate::findings::{
    BlameStrength, BlamedGoal, BlamedLanding, FindingBlame, FindingKind, FindingState, FindingWrite, VerdictFiling,
};
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::Queue;
mod clearing;
mod words;

struct Filings {
    rows: List<VerdictFiling>,
    counted: List<u32>,
}

pub(super) fn finish(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut TierRow,
    verdict: &p::Verdict,
    out: &mut Queue<Request>,
) -> bool {
    clearing::judged(d, env, row, verdict, out);
    let mut filings = prepare(d, row, verdict, &env.limits, &[]);
    let mut needs_logs = false;
    for filing in &filings.rows {
        needs_logs |= filing.kind == FindingKind::Red;
    }
    if needs_logs {
        let Some(logs) = red::logs(d, env, row, out) else { return false };
        filings = prepare(d, row, verdict, &env.limits, &logs);
    }
    let project = d.repositories.get(&row.repository).expect("adopted tier").project;
    for number in filings.counted.into_boxed() {
        let mut finding = d.finding(project, number).expect("matched flake").clone();
        finding.seen = Some(finding.seen.unwrap_or(0).saturating_add(1));
        d.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
    }
    let mut room = d.verdict_room(&env.limits, project);
    let mut waiting = List::with_capacity(8);
    for filing in core::mem::replace(&mut row.waiting, Box::new([])) {
        if room > 0 {
            room = room.checked_sub(1).expect("filing room");
            out.push(Request::FileVerdict { task: row.task.expect("judged task"), filing });
        } else {
            waiting.push(filing).expect("bounded waiting filings");
        }
    }
    for filing in filings.rows.into_boxed() {
        if room > 0 {
            room = room.checked_sub(1).expect("filing room");
            out.push(Request::FileVerdict { task: row.task.expect("judged task"), filing });
        } else if filing.kind == FindingKind::Red {
            if waiting.room() > 0 {
                waiting.push(filing).expect("waiting room");
            } else {
                row.unkept = row.unkept.saturating_add(1);
            }
        }
    }
    row.waiting = waiting.into_boxed();
    true
}
/// Preserve each unrepaired red before its verification leaves (domain/red-findings.md, 8).
pub(crate) fn switched(d: &mut Domain, task: u64, verification: &crate::Verification, out: &mut Queue<Request>) {
    let Some(change) = d.change(task) else { return };
    let branch = change.base.clone();
    let project = d.repositories.get(&verification.repository).expect("adopted landing").project;
    let mut room = d.verdict_room(&d.landing.limits, project);
    for tier in &verification.tiers {
        if tier.repaired.is_some() {
            continue;
        }
        let Some(verdict) = &tier.verdict else { continue };
        let (strength, failures) = match &verdict.blame {
            p::Blame::Cleared => continue,
            p::Blame::Owns(failing) => (BlameStrength::Owned, failing),
            p::Blame::Suspect(failing) => (BlameStrength::Suspected, failing),
        };
        let key = (verification.repository, tier.tier.clone());
        let mut row = match d.tiers.remove(&key) {
            Some(row) => row,
            None => super::empty(key.0, key.1.clone()),
        };
        let (held, many_held) = held(d, &row, project);
        if let Some(failures) = remaining(failures, held.as_slice(), many_held) {
            let blame = FindingBlame {
                moved: None,
                strength,
                landings: Box::new([landing(d, &row, verification.commit, &d.landing.limits)]),
            };
            let filing = words::switched(d, &row, verdict, failures, blame, &branch);
            if room > 0 {
                room = room.checked_sub(1).expect("filing room");
                out.push(Request::FileVerdict { task, filing });
            } else {
                let mut waiting = List::with_capacity(8);
                for prior in row.waiting {
                    waiting.push(prior).expect("bounded waiting filings");
                }
                if waiting.room() > 0 {
                    waiting.push(filing).expect("waiting room");
                } else {
                    row.unkept = row.unkept.saturating_add(1);
                }
                row.waiting = waiting.into_boxed();
                super::save(&row, out);
            }
        }
        d.tiers.insert(key, row).expect("replaces tier");
    }
}

fn held(d: &Domain, row: &TierRow, project: u32) -> (List<Box<[u8]>>, bool) {
    let mut ids = List::uncapped();
    let mut many = false;
    for (_, finding) in d.findings(project) {
        if !finding.uncleared_red() {
            continue;
        }
        let Some(signature) = &finding.signature else { continue };
        if signature.repository == row.repository && signature.source == row.tier {
            many |= signature.many;
            for id in &signature.ids {
                if !ids.as_slice().contains(id) {
                    ids.push(id.clone()).expect("held ids");
                }
            }
        }
    }
    for filing in &row.waiting {
        many |= filing.signature.many;
        for id in &filing.signature.ids {
            if !ids.as_slice().contains(id) {
                ids.push(id.clone()).expect("waiting ids");
            }
        }
    }
    (ids, many)
}
fn remaining(failing: &p::Failures, held: &[Box<[u8]>], many_held: bool) -> Option<p::Failures> {
    match failing {
        p::Failures::Many => {
            if many_held {
                None
            } else {
                Some(p::Failures::Many)
            }
        }
        p::Failures::Tests(tests) => {
            let mut ids = List::uncapped();
            for test in tests {
                if !held.contains(test) {
                    ids.push(test.clone()).expect("bounded failures");
                }
            }
            if ids.is_empty() { None } else { Some(p::Failures::Tests(ids.into_boxed())) }
        }
    }
}
fn landing(d: &Domain, row: &TierRow, commit: Commit, limits: &Limits) -> BlamedLanding {
    let mut goals = List::with_capacity(limits.landing_members);
    for member in words::members(d, row, commit) {
        let Some(goal) = member.goal else { continue };
        let mut seen = false;
        for kept in &goals {
            let kept: &BlamedGoal = kept;
            seen |= kept.goal == goal;
        }
        if !seen && goals.room() > 0 {
            goals.push(BlamedGoal { goal, milestone: member.milestone, person: member.person }).expect("goal room");
        }
    }
    BlamedLanding { commit, goals: goals.into_boxed() }
}

fn prepare(d: &Domain, row: &TierRow, verdict: &p::Verdict, limits: &Limits, logs: &[crate::JobLog]) -> Filings {
    let project = d.repositories.get(&row.repository).expect("adopted tier").project;
    let (held, many_held) = held(d, row, project);
    let mut filings = Filings { rows: List::uncapped(), counted: List::uncapped() };
    let mut blamed = List::uncapped();
    let mut blamed_many = false;
    let mut suspects = List::with_capacity(limits.landings_pending);
    let mut suspected = List::uncapped();
    let mut suspect_many = false;
    let range = &row.record.run.as_ref().expect("judged run").landings;
    for (candidate, blame) in range.iter().zip(&verdict.landings) {
        let failing = match blame {
            p::Blame::Cleared => continue,
            p::Blame::Owns(failing) | p::Blame::Suspect(failing) => failing,
        };
        match failing {
            p::Failures::Many => blamed_many = true,
            p::Failures::Tests(ids) => {
                for id in ids {
                    if !blamed.as_slice().contains(id) {
                        blamed.push(id.clone()).expect("blamed failures");
                    }
                }
            }
        }
        match blame {
            p::Blame::Owns(_) => {
                if let Some(failing) = remaining(failing, held.as_slice(), many_held) {
                    let blame = FindingBlame {
                        moved: None,
                        strength: BlameStrength::Owned,
                        landings: Box::new([landing(d, row, candidate.commit, limits)]),
                    };
                    filings.rows.push(words::red(d, row, verdict, failing, blame, logs)).expect("one filing per owner");
                }
            }
            p::Blame::Suspect(_) => {
                if suspects.room() > 0 {
                    suspects.push(landing(d, row, candidate.commit, limits)).expect("suspect room");
                }
                match failing {
                    p::Failures::Many => suspect_many = true,
                    p::Failures::Tests(ids) => {
                        for id in ids {
                            if !suspected.as_slice().contains(id) {
                                suspected.push(id.clone()).expect("suspected ids");
                            }
                        }
                    }
                }
            }
            p::Blame::Cleared => unreachable!("cleared skipped"),
        }
    }
    let mut suspected = suspected.into_boxed();
    suspected.sort();
    let suspected = if suspect_many { p::Failures::Many } else { p::Failures::Tests(suspected) };
    if let Some(failing) = remaining(&suspected, held.as_slice(), many_held) {
        let blame = FindingBlame { moved: None, strength: BlameStrength::Suspected, landings: suspects.into_boxed() };
        filings.rows.push(words::red(d, row, verdict, failing, blame, logs)).expect("one suspected filing");
    }
    let mut excluded = held;
    for id in blamed.into_boxed() {
        excluded.push(id).expect("held and blamed ids");
    }
    if let Some(failing) = remaining(&verdict.failing, excluded.as_slice(), many_held || blamed_many) {
        let blame = FindingBlame { moved: None, strength: BlameStrength::Unblamed, landings: Box::new([]) };
        filings.rows.push(words::red(d, row, verdict, failing, blame, logs)).expect("one unblamed filing");
    }
    flakes(d, row, verdict, project, &mut filings);
    filings
}

fn flakes(d: &Domain, row: &TierRow, verdict: &p::Verdict, project: u32, filings: &mut Filings) {
    let mut fresh = List::uncapped();
    for id in &verdict.flakes {
        let mut matched = None;
        for (_, finding) in d.findings(project) {
            if finding.kind != FindingKind::Flake || finding.state == FindingState::Dropped {
                continue;
            }
            let Some(signature) = &finding.signature else { continue };
            if signature.repository == row.repository && signature.source == row.tier && signature.ids.contains(id) {
                matched = Some(finding.number);
                break;
            }
        }
        if let Some(number) = matched {
            if !filings.counted.as_slice().contains(&number) {
                filings.counted.push(number).expect("one count per matched finding");
            }
        } else {
            fresh.push(id.clone()).expect("new flakes");
        }
    }
    if !fresh.is_empty() {
        filings.rows.push(words::flake(d, row, verdict, fresh.into_boxed())).expect("one flake filing");
    }
}
