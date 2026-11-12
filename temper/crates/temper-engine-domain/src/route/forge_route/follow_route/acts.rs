//! One decision rewires the chain before it cancels (forge/follows.md, 5).
use super::{Box, Domain, Env, Limits, List, ProcedureStep, StepCancel, check, describe, forge, live, tasks};
use forge::follows::FollowAct;
use jig_domain_core::StepAmendment;
use skein_forge::api::Commit;

pub(super) fn apply(
    domain: &Domain,
    env: &Env<Limits>,
    row: &mut forge::FollowRow,
    decision: &mut ProcedureStep,
) -> Option<()> {
    // Allocate any missing changes first; their next step can amend the known task numbers.
    if let Some(FollowAct::Merge { afters }) = &row.pending {
        for entry in &row.queue {
            if afters.contains(&entry.after) && entry.change.is_none() {
                return Some(());
            }
        }
    }
    let Some(act) = row.pending.take() else { return Some(()) };
    match act {
        FollowAct::Start { after } => {
            let mut change = None;
            for entry in &row.queue {
                if entry.after == after {
                    change = entry.change;
                    break;
                }
            }
            for entry in &mut row.queue {
                if entry.after == after || (change.is_some() && entry.change == change) {
                    entry.word = true;
                    match &entry.state {
                        forge::FollowState::Ended(_) => entry.remake = true,
                        forge::FollowState::HeldByOwner(_) => entry.state = forge::FollowState::NotMade,
                        forge::FollowState::NotMade
                        | forge::FollowState::WaitsWord
                        | forge::FollowState::WaitsFor(_)
                        | forge::FollowState::Producing
                        | forge::FollowState::Held(_) => {}
                    }
                }
            }
        }
        FollowAct::Hold { after, reason } => {
            for entry in &mut row.queue {
                if entry.after == after {
                    entry.state = forge::FollowState::HeldByOwner(reason.clone());
                }
            }
        }
        FollowAct::Skip { after, reason } => skip(domain, env, row, decision, after, reason)?,

        FollowAct::Merge { afters } => merge(domain, env, row, decision, &afters)?,
    }
    Some(())
}
fn cancel(decision: &mut ProcedureStep, task: u64, reason: Box<[u8]>) {
    let mut cancels = List::uncapped();
    for item in core::mem::replace(&mut decision.cancels, Box::new([])) {
        cancels.push(item).expect("cancels");
    }
    cancels.push(StepCancel { task, reason, grace: None }).expect("act cancel");
    decision.cancels = cancels.into_boxed();
}
fn amend(
    domain: &Domain,
    decision: &mut ProcedureStep,
    task: u64,
    spec: Option<tasks::Spec>,
    dependencies: Option<Box<[u64]>>,
    reason: &[u8],
) -> Option<()> {
    let record = domain.core.tasks.task(task)?;
    let mut amendments = List::uncapped();
    for item in core::mem::replace(&mut decision.amendments, Box::new([])) {
        amendments.push(item).expect("amendments");
    }
    amendments
        .push(StepAmendment {
            task,
            revision: record.revisions.spec,
            amendment: tasks::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                release: None,
                spec,
                wake: None,
                dependencies,
                authority: None,
                llm: None,
                reason: reason.into(),
            },
        })
        .expect("act amendment");
    decision.amendments = amendments.into_boxed();
    Some(())
}
fn rewire(
    domain: &Domain,
    decision: &mut ProcedureStep,
    task: u64,
    previous: Option<u64>,
    reason: &[u8],
) -> Option<()> {
    let dependencies: Box<[u64]> = match previous {
        Some(task) => Box::new([task]),
        None => Box::new([]),
    };
    amend(domain, decision, task, None, Some(dependencies), reason)
}
fn amend_spec(
    domain: &Domain,
    env: &Env<Limits>,
    row: &forge::FollowRow,
    decision: &mut ProcedureStep,
    task: u64,
    entries: &[forge::FollowEntry],
) -> Option<()> {
    let spec = describe(domain, env, row, entries, domain.core.tasks.task(task)?.spec.clone())?;
    let mut commits = List::uncapped();
    for entry in entries {
        commits.push(entry.after).expect("merged commits");
    }
    let follows = forge::ChangeFollows { follow: row.follow, commits: commits.into_boxed() };
    let mut cancelled = List::uncapped();
    for entry in entries {
        if let Some(other) = entry.change
            && other != task
        {
            cancelled.push(other).expect("merged cancels");
        }
    }
    check(domain, row.project, tasks::Party::Task(row.task?), &follows, Some(task), cancelled.as_slice()).ok()?;
    amend(domain, decision, task, Some(spec), None, b"follow entries changed")
}

fn skip(
    domain: &Domain,
    env: &Env<Limits>,
    row: &mut forge::FollowRow,
    decision: &mut ProcedureStep,
    after: Commit,
    reason: Box<[u8]>,
) -> Option<()> {
    let mut found = None;
    for entry in &row.queue {
        if entry.after == after {
            found = Some(entry.clone());
            break;
        }
    }
    let entry = found?;
    let mut remaining = List::uncapped();
    for other in &row.queue {
        if other.after != after {
            remaining.push(other.clone()).expect("remaining queue");
        }
    }
    let mut next = None;
    let mut passed = false;
    for entry in &row.queue {
        if passed {
            next = entry.change;
            break;
        }
        passed = entry.after == after;
    }
    let mut previous = None;
    for other in &row.queue {
        if other.after == after {
            break;
        }
        if let Some(change) = other.change
            && live(domain, change)
        {
            previous = Some(change);
        }
    }
    row.queue = remaining.into_boxed();
    if let Some(change) = entry.change {
        let mut group = List::uncapped();
        for other in &row.queue {
            if other.change == Some(change) {
                group.push(other.clone()).expect("group");
            }
        }
        if group.is_empty() {
            if let Some(next) = next
                && next != change
                && live(domain, next)
            {
                rewire(domain, decision, next, previous, &reason)?;
            }
            if live(domain, change) {
                cancel(decision, change, reason.clone());
            }
        } else {
            amend_spec(domain, env, row, decision, change, group.as_slice())?;
        }
    }
    let mut skips = List::uncapped();
    let keep = skein_lib::usize_of(env.limits.forge.follow_skips).saturating_sub(1);
    for skip in row.skips.iter().skip(row.skips.len().saturating_sub(keep)) {
        skips.push(skip.clone()).expect("skips");
    }
    if env.limits.forge.follow_skips > 0 {
        skips.push(forge::FollowSkip { after, reason }).expect("skip");
    }
    row.skips = skips.into_boxed();

    Some(())
}

fn merge(
    domain: &Domain,
    env: &Env<Limits>,
    row: &mut forge::FollowRow,
    decision: &mut ProcedureStep,
    afters: &[Commit],
) -> Option<()> {
    let mut first = None;
    for entry in &row.queue {
        if Some(&entry.after) == afters.first() {
            first = entry.change;
            break;
        }
    }
    let first = first?;
    let mut group = List::uncapped();
    let mut removed = List::uncapped();
    for entry in &row.queue {
        if afters.contains(&entry.after) {
            group.push(entry.clone()).expect("merged entries");
            if let Some(change) = entry.change
                && change != first
                && !removed.as_slice().contains(&change)
            {
                removed.push(change).expect("merged changes");
            }
        }
    }
    // Include entries already carried by the first change.
    for entry in &row.queue {
        if entry.change == Some(first) && !afters.contains(&entry.after) {
            return None;
        }
    }
    let reason = skein_lib::bytes::concat(&[b"merged into change ", &super::super::decimal(first)]);
    amend_spec(domain, env, row, decision, first, group.as_slice())?;
    let mut passed = false;
    let mut next = None;
    for entry in &row.queue {
        if passed
            && let Some(change) = entry.change
            && !removed.as_slice().contains(&change)
            && change != first
        {
            next = Some(change);
            break;
        }
        if Some(&entry.after) == afters.last() {
            passed = true;
        }
    }
    if let Some(next) = next
        && live(domain, next)
    {
        rewire(domain, decision, next, Some(first), &reason)?;
    }
    for change in removed.into_boxed() {
        if live(domain, change) {
            cancel(decision, change, reason.clone());
        }
    }
    for entry in &mut row.queue {
        if afters.contains(&entry.after) {
            entry.change = Some(first);
        }
    }

    Some(())
}
