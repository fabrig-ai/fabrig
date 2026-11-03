//! Each goal branch keeps its own tip, failures and span (temper/docs/design/domain/after-landing.md, 3).
use super::{configured_all, empty, joined, name, p, red, save, settings};
use crate::{Class, Domain, Follows, Limits, Name, News, RedNews, RedOn, Request, Tier, TierGoal, TierRow};
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};
use skein_lib::{Duration, Env, List, Queue, Time};

fn holder(d: &Domain, repository: Repository, name: &Name) -> Option<u64> {
    if name.forge != repository.forge || name.repository != repository.repository {
        return None;
    }
    let adopted = d.repositories.get(&repository)?;
    let (_, goal) = crate::items::goal_branch(name, &adopted.prefix)?;
    if d.holds.get(name)?.task == goal { Some(goal) } else { None }
}

pub(super) fn watches(d: &Domain, repository: Repository, names: &mut List<Name>) {
    for (name, _) in &d.holds {
        if holder(d, repository, name).is_some() && !names.as_slice().contains(name) {
            names.push(name.clone()).expect("tier and held goal watches");
        }
    }
}

fn branch(name: &Name) -> Box<[u8]> {
    match &name.what {
        crate::What::Branch(parts) => {
            let mut bytes: Box<[u8]> = Box::new([]);
            for part in parts {
                bytes = if bytes.is_empty() { part.clone() } else { joined(&[&bytes, b"/", part]) };
            }
            bytes
        }
        crate::What::Repository | crate::What::Pull(_) | crate::What::Issue(_) => unreachable!("held goal branch"),
    }
}

fn observe(row: &mut TierRow, goal: u64, branch: &[u8], before: Option<Commit>, after: Commit, capacity: u32) {
    for entry in &mut row.goals {
        if entry.goal == goal {
            entry.tip = Some(after);
            return;
        }
    }
    let entry = TierGoal {
        goal,
        branch: Box::from(branch),
        tip: Some(after),
        ran: Some(before.unwrap_or(after)),
        failing: p::Failures::Tests(Box::new([])),
        started: None,
    };
    let mut entries = List::with_capacity(capacity);
    let mut incoming = Some(entry);
    for entry in &row.goals {
        if entry.goal > goal
            && let Some(entry) = incoming.take()
        {
            entries.push(entry).expect("one entry per held goal");
        }
        entries.push(entry.clone()).expect("held goals fit holds");
    }
    if let Some(entry) = incoming {
        entries.push(entry).expect("one entry per held goal");
    }
    row.goals = entries.into_boxed();
}

pub(super) fn observed(
    d: &mut Domain,
    repository: Repository,
    branch: &[u8],
    before: Option<Commit>,
    after: Commit,
    out: &mut Queue<Request>,
) {
    let Some(goal) = holder(d, repository, &name(repository, branch)) else { return };
    let mut tiers = List::with_capacity(d.tiers.capacity());
    for tier in configured_all(d, repository) {
        if tier.follows == Follows::Goals {
            tiers.push(tier.name.clone()).expect("configured tiers");
        }
    }
    if !tiers.is_empty() && before.is_none() {
        d.tier_ops.dirty = true;
    }
    for tier in tiers.into_boxed() {
        let key = (repository, tier.clone());
        let mut row = d.tiers.remove(&key).unwrap_or(empty(repository, tier));
        observe(&mut row, goal, branch, before, after, d.holds.capacity());
        save(&row, out);
        d.tiers.insert(key, row).expect("configured goal tier");
    }
}

pub(super) fn span_passed(started: Option<Time>, span: Option<Duration>, now: Time) -> bool {
    match (started, span) {
        (Some(started), Some(span)) => started.saturating_add(span) <= now,
        (Some(_), None) | (None, Some(_) | None) => true,
    }
}

pub(super) fn deadline(row: &TierRow, span: Option<Duration>) -> Option<Time> {
    let span = span?;
    let mut next: Option<Time> = None;
    for goal in &row.goals {
        if goal.tip.is_some()
            && goal.tip != goal.ran
            && let Some(started) = goal.started
        {
            let at = started.saturating_add(span);
            next = Some(match next {
                Some(old) => old.min(at),
                None => at,
            });
        }
    }
    next
}

pub(super) fn start_due(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: Repository,
    tier: &Tier,
    out: &mut Queue<Request>,
) {
    let key = (repository, tier.name.clone());
    let existing = d.tiers.contains_key(&key);
    let mut row = d.tiers.remove(&key).unwrap_or(empty(repository, tier.name.clone()));
    // A hold's making can have recorded its head before the first watched move.
    for (name, _) in &d.holds {
        if let Some(goal) = holder(d, repository, name) {
            let mut known = false;
            for entry in &row.goals {
                known |= entry.goal == goal;
            }
            if !known && let Some(head) = d.heads.get(name) {
                observe(&mut row, goal, &branch(name), None, head.commit, env.limits.holds);
            }
        }
    }
    if !existing && row.goals.is_empty() && row.record.run.is_none() {
        return;
    }
    if row.record.run.is_none() {
        let mut selected: Option<(Option<Time>, u64)> = None;
        for goal in &row.goals {
            if goal.tip.is_none() || goal.tip == goal.ran || !span_passed(goal.started, tier.span, env.now) {
                continue;
            }
            let order = (goal.started, goal.goal);
            if selected.is_none() || Some(order) < selected {
                selected = Some(order);
            }
        }
        if let Some((_, goal)) = selected {
            for entry in &mut row.goals {
                if entry.goal != goal {
                    continue;
                }
                let tip = entry.tip.expect("due goal tip");
                // Goal runs never reuse content receipts; their immutable tip identifies the dispatch.
                let next = p::start(&row.record, tip, tip, Box::new([]), None, &settings(d, repository));
                let p::Next::Dispatch(run) = next else { unreachable!("a goal run takes no receipt") };
                row.record.run = Some(run);
                row.goal = Some(goal);
                row.requested = false;
                row.cancelled = false;
                row.purpose = None;
                row.task = None;
                row.branch = crate::TierBranch::Absent;
                row.probe = None;
                row.flight = None;
                row.held = None;
                row.result = None;
                row.received = false;
                entry.started = Some(env.now);
            }
        }
    }
    if !row.requested
        && let Some(run) = &row.record.run
    {
        row.requested = true;
        save(&row, out);
        out.push(Request::TierRun { repository, tier: row.tier.clone(), number: run.number, tip: run.tip });
    } else {
        save(&row, out);
    }
    d.tiers.insert(key, row).expect("configured goal tier");
}

pub(super) fn released(d: &mut Domain, name: &Name, out: &mut Queue<Request>) {
    let repository = Repository { forge: name.forge, repository: name.repository };
    let Some(adopted) = d.repositories.get(&repository) else { return };
    let Some((_, goal)) = crate::items::goal_branch(name, &adopted.prefix) else { return };
    let mut follows = false;
    for tier in configured_all(d, repository) {
        follows |= tier.follows == Follows::Goals;
    }
    if !follows {
        return;
    }
    let mut keys = List::with_capacity(d.tiers.capacity());
    for (key, row) in &d.tiers {
        if row.repository == repository {
            keys.push(key.clone()).expect("repository tiers");
        }
    }
    for key in keys.into_boxed() {
        let row = d.tiers.get_mut(&key).expect("repository tier");
        let mut entries = List::with_capacity(u32::try_from(row.goals.len()).expect("bounded goal entries"));
        for entry in &row.goals {
            if entry.goal != goal {
                entries.push(entry.clone()).expect("remaining entries");
            }
        }
        if entries.len() != u32::try_from(row.goals.len()).expect("bounded goal entries") {
            row.goals = entries.into_boxed();
            save(row, out);
        }
    }
    d.tier_ops.dirty = true;
}

fn split(failing: &p::Failures, previous: &p::Failures) -> (p::Failures, p::Failures) {
    match (failing, previous) {
        (p::Failures::Many, p::Failures::Many) => (p::Failures::Tests(Box::new([])), p::Failures::Many),
        (p::Failures::Many, p::Failures::Tests(_)) => (p::Failures::Many, p::Failures::Tests(Box::new([]))),
        (p::Failures::Tests(tests), previous) => {
            let capacity = u32::try_from(tests.len()).expect("bounded failing tests");
            let mut new = List::with_capacity(capacity);
            let mut still = List::with_capacity(capacity);
            for test in tests {
                let known = match previous {
                    p::Failures::Tests(previous) => previous.contains(test),
                    p::Failures::Many => false,
                };
                if known {
                    still.push(test.clone()).expect("failing subset");
                } else {
                    new.push(test.clone()).expect("failing subset");
                }
            }
            (p::Failures::Tests(new.into_boxed()), p::Failures::Tests(still.into_boxed()))
        }
    }
}

pub(super) fn finish(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut TierRow,
    verdict: &p::Verdict,
    out: &mut Queue<Request>,
) -> bool {
    let goal = row.goal.expect("goal run");
    for index in 0..row.goals.len() {
        let entry = row.goals.get(index).expect("goal entry index");
        if entry.goal != goal {
            continue;
        }
        let red = match &verdict.failing {
            p::Failures::Many => true,
            p::Failures::Tests(tests) => !tests.is_empty(),
        };
        if red {
            let Some(logs) = red::logs(d, env, row, out) else { return false };
            let (new, still) = split(&verdict.failing, &entry.failing);
            let class = match &new {
                p::Failures::Many => Class::Wakes,
                p::Failures::Tests(tests) => {
                    if tests.is_empty() {
                        Class::Kept
                    } else {
                        Class::Wakes
                    }
                }
            };
            let result = row.result.as_ref().expect("goal run result");
            let news = News::Red(RedNews {
                tier: row.tier.clone(),
                run: verdict.number,
                url: if result.run.url.is_empty() { None } else { Some(result.run.url.clone()) },
                on: RedOn::Goal { branch: entry.branch.clone(), tip: verdict.tip, new, still },
                logs,
            });
            red::tell(d, row.repository, &entry.branch, &[goal], news, class, out);
        }
        let entry = row.goals.get_mut(index).expect("finished goal entry");
        entry.ran = Some(verdict.tip);
        entry.failing = verdict.failing.clone();
        break;
    }
    red::forget(d, row);
    row.goal = None;
    true
}
