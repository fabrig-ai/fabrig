//! Routed reds are judged at their goal's end (domain/red-repairs.md, 4.3, 5–6).
use super::{Finding, FindingCleared, FindingRepair, FindingState, FindingWrite, RepairJudging};
use crate::{Domain, Follows, Limits, LoggedVerdict, ReleaseEnding, Request, TierRow};
use alloc::boxed::Box;
use skein_lib::{Decimal, Env, List, Queue, Wall};
use temper_engine_domain_forge_tiers as p;

impl Domain {
    /// The root calls this in the decision that hears jig's release.
    pub fn judge_goal_findings(
        &mut self,
        env: &Env<Limits>,
        project: u32,
        goal: u64,
        ending: ReleaseEnding,
        out: &mut Queue<Request>,
    ) {
        let mut rows = List::uncapped();
        for (_, finding) in self.findings(project) {
            let routed = finding.uncleared_red() && finding.state == FindingState::Routed && finding.goal == Some(goal);
            let capped = match &finding.repair {
                Some(repair) => repair.goal == goal && repair.until.is_some(),
                None => false,
            };
            if routed || capped {
                rows.push(finding.clone()).expect("project findings bounded at admission");
            }
        }
        for mut finding in rows.into_boxed() {
            let routed = finding.uncleared_red() && finding.state == FindingState::Routed && finding.goal == Some(goal);
            if routed {
                let (attempts, reset) = match &finding.repair {
                    Some(repair) => (repair.attempts, repair.goal != goal),
                    None => (0, true),
                };
                if reset {
                    finding.repair = Some(FindingRepair {
                        goal,
                        made: false,
                        attempts,
                        cancelled: false,
                        failing: Box::new([]),
                        until: None,
                        judging: None,
                    });
                }
            }
            let repair = finding.repair.as_mut().expect("routed or capped repair");
            repair.until = None;
            if !routed {
                self.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
                continue;
            }
            match ending {
                ReleaseEnding::Cancelled => {
                    repair.cancelled = true;
                    let why = joined(&[&goal_words(repair), b" was cancelled"]);
                    back(finding, why, out);
                }
                ReleaseEnding::Done | ReleaseEnding::Failed => {
                    if repair.judging.is_none() {
                        let signature = finding.signature.as_ref().expect("red signature");
                        let tier = self.tiers.get(&(signature.repository, signature.source.clone())).cloned();
                        let waiting = wait_for(self, &finding, tier.as_ref());
                        match waiting {
                            Some(waiting) => wait(self, env, finding, waiting, out),
                            None => assess(self, env, finding, tier.as_ref(), out),
                        }
                    } else {
                        self.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
                    }
                }
            }
        }
    }
}

fn wait_for(d: &Domain, finding: &Finding, row: Option<&TierRow>) -> Option<RepairJudging> {
    let Some(row) = row else { return Some(RepairJudging::After(0)) };
    if row.hold.is_some() {
        return Some(RepairJudging::Release);
    }
    let signature = finding.signature.as_ref().expect("red signature");
    let goal = finding.repair.as_ref().expect("judging repair").goal;
    if let Some(log) = d.landing_log(signature.repository) {
        for landing in log.landings.iter().rev() {
            let mut member = false;
            for change in &landing.changes {
                member |= change.goal == Some(goal);
            }
            if !member {
                continue;
            }
            let mut follows = false;
            for tier in crate::tiers::configured_all(d, signature.repository) {
                if tier.name == row.tier {
                    follows = match &tier.follows {
                        Follows::Landing => {
                            d.repositories.get(&signature.repository).expect("adopted tier").landing == landing.branch
                        }
                        Follows::Branch(branch) => branch == &landing.branch,
                        Follows::Goals => false,
                    };
                }
            }
            if !follows {
                return Some(RepairJudging::After(started(row)));
            }
            for tier in &landing.tiers {
                if tier.tier == row.tier {
                    match tier.verdict {
                        LoggedVerdict::Verifying => return Some(RepairJudging::Landing(landing.commit)),
                        LoggedVerdict::Cleared { .. } | LoggedVerdict::Red { .. } => return None,
                    }
                }
            }
            return Some(RepairJudging::Landing(landing.commit));
        }
    }
    // A retained change can still witness a landing that has left the bounded log.
    for (_, change) in &d.changes {
        if change.repository == signature.repository
            && change.goal == Some(goal)
            && crate::domain::landed_commit(&change.change.state).is_some()
        {
            return Some(RepairJudging::After(started(row)));
        }
    }
    None
}
fn started(row: &TierRow) -> u64 {
    match &row.record.run {
        Some(run) => row.record.sequence.max(run.number),
        None => row.record.sequence,
    }
}
fn wait(d: &mut Domain, env: &Env<Limits>, mut finding: Finding, judging: RepairJudging, out: &mut Queue<Request>) {
    finding.repair.as_mut().expect("judging repair").judging = Some(judging);
    d.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
}
fn assess(d: &mut Domain, env: &Env<Limits>, mut finding: Finding, row: Option<&TierRow>, out: &mut Queue<Request>) {
    let Some(row) = row else {
        wait(d, env, finding, RepairJudging::After(0), out);
        return;
    };
    if row.record.judged.is_none() {
        wait(d, env, finding, RepairJudging::After(0), out);
        return;
    }
    let failed = match &row.record.failing {
        p::Failures::Tests(failed) => failed,
        p::Failures::Many => {
            wait(d, env, finding, RepairJudging::After(row.record.last_run.unwrap_or(row.record.sequence)), out);
            return;
        }
    };
    let mut failing = List::with_capacity(64);
    for id in &finding.signature.as_ref().expect("red signature").ids {
        if failed.contains(id) {
            failing.push(id.clone()).expect("signature holds at most 64 ids");
        }
    }
    let repair = finding.repair.as_mut().expect("judging repair");
    repair.failing = failing.into_boxed();
    repair.judging = None;
    if repair.failing.is_empty() && !finding.signature.as_ref().expect("red signature").many {
        finding.cleared =
            Some(FindingCleared { run: row.record.last_run.unwrap_or(row.record.sequence), at: env.wall });
        d.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
    } else {
        let run = match row.record.last_run {
            Some(number) => joined(&[&row.tier, b" run ", Decimal::of(number).as_bytes()]),
            None => joined(&[&row.tier, b"'s last run"]),
        };
        let why = joined(&[
            &goal_words(repair),
            b" ended and ",
            &run,
            b" still fails ",
            Decimal::of(skein_lib::len_u64(repair.failing.len())).as_bytes(),
            b" of its tests",
        ]);
        back(finding, why, out);
    }
}
fn back(mut finding: Finding, why: Box<[u8]>, out: &mut Queue<Request>) {
    finding.state = FindingState::New;
    finding.goal = None;
    finding.why = Some(why);
    finding.hander = None;
    finding.repair.as_mut().expect("judging repair").judging = None;
    out.push(Request::ReturnFinding(finding));
}
fn goal_words(repair: &FindingRepair) -> Box<[u8]> {
    joined(&[if repair.made { b"repair goal " } else { b"goal " }, Decimal::of(repair.goal).as_bytes()])
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut out = List::uncapped();
    for part in parts {
        for &byte in *part {
            out.push(byte).expect("bounded judging words");
        }
    }
    out.into_boxed()
}

/// Called after ordinary clearing and many enumeration, before the verdict is committed.
pub(crate) fn judged(d: &mut Domain, env: &Env<Limits>, row: &TierRow, verdict: &p::Verdict, out: &mut Queue<Request>) {
    let mut judged = row.clone();
    judged.record = p::judged(
        &row.record,
        verdict,
        &p::Limits { failing_tests: env.limits.tiers.failing_tests, test_id_bytes: env.limits.tiers.test_id_bytes },
    );
    let mut rows = List::uncapped();
    let project = d.repositories.get(&row.repository).expect("adopted tier").project;
    for (_, finding) in d.findings(project) {
        if !of_tier(finding, row) {
            continue;
        }
        let Some(waiting) = &finding.repair.as_ref().expect("waiting repair").judging else { continue };
        let ready = match waiting {
            RepairJudging::Landing(commit) => {
                let mut covered = false;
                for landing in &row.record.run.as_ref().expect("judged range").landings {
                    covered |= landing.commit == *commit;
                }
                covered
            }
            RepairJudging::After(number) => verdict.number > *number,
            RepairJudging::Release => false,
        };
        if ready {
            rows.push(finding.clone()).expect("bounded findings");
        }
    }
    for finding in rows.into_boxed() {
        if row.hold.is_some() {
            wait(d, env, finding, RepairJudging::Release, out);
        } else {
            assess(d, env, finding, Some(&judged), out);
        }
    }
}
fn of_tier(finding: &Finding, row: &TierRow) -> bool {
    finding.uncleared_red()
        && finding.repair.is_some()
        && match &finding.signature {
            Some(signature) => signature.repository == row.repository && signature.source == row.tier,
            None => false,
        }
}
pub(crate) fn released(d: &mut Domain, env: &Env<Limits>, row: &TierRow, out: &mut Queue<Request>) {
    let project = d.repositories.get(&row.repository).expect("adopted tier").project;
    let mut rows = List::uncapped();
    for (_, finding) in d.findings(project) {
        if of_tier(finding, row)
            && finding.repair.as_ref().expect("waiting repair").judging == Some(RepairJudging::Release)
        {
            rows.push(finding.clone()).expect("bounded findings");
        }
    }
    for finding in rows.into_boxed() {
        assess(d, env, finding, Some(row), out);
    }
}
pub(crate) fn deadline(d: &Domain) -> Option<Wall> {
    let mut next = None;
    for (_, finding) in d.finding_rows() {
        if let Some(repair) = &finding.repair
            && !d.repairs_asked.contains(&repair.goal)
            && let Some(until) = repair.until
        {
            next = Some(match next {
                Some(previous) => Wall::min(previous, until),
                None => until,
            });
        }
    }
    next
}
pub(crate) fn fire(d: &mut Domain, wall: Wall, out: &mut Queue<Request>) {
    let capacity = d.repairs_asked.capacity();
    let asked = core::mem::replace(&mut d.repairs_asked, skein_lib::Set::with_capacity(capacity));
    for (_, finding) in d.findings.as_deref().unwrap_or(&super::NO_FINDINGS) {
        if let Some(repair) = &finding.repair
            && let Some(until) = repair.until
            && !d.repairs_asked.contains(&repair.goal)
        {
            let was_asked = asked.contains(&repair.goal);
            if was_asked || until <= wall {
                d.repairs_asked.insert(repair.goal).expect("one request per live repair goal");
                if !was_asked {
                    out.push(Request::FailRepair { project: finding.project, task: repair.goal, until });
                }
            }
        }
    }
}
