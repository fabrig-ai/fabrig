//! The core's structured task waits, checked against the adapter's waits
//! (jig/docs/design/domain/testing-fakes.md, 1.1).

use jig_domain_core::{Bound, Core, Wait as Record, WaitKind, WaitObject};
use jig_domain_people::EntryKind;
use jig_domain_tasks::{Escalation, Phase};
use skein_lib::Wall;
use skein_world::inspect::{Inspection, Moment, Wait, Wake};

/// Check the live tasks' records at a quiet point. `wall` is the greatest wall
/// reading the world has given the core; a record's bound arms nothing.
pub fn records(core: &Core, moment: Moment, wall: Wall) -> Result<(), String> {
    let mut after = None;
    while let Some(row) = core.tasks.inspection_task_after(after) {
        after = Some(row.number);
        let mut adapter = Inspection::new("jig", crate::KINDS);
        if !matches!(row.phase, Phase::Closing(_)) {
            crate::waits::task(&core.tasks, Some(core), row, &mut adapter);
        }
        let held_at_holder = matches!(row.escalation, Escalation::Waiting { .. });
        check_records(
            row.number,
            adapter.waits(),
            &core.inspection_wait_records(row),
            held_at_holder,
            moment,
            wall,
            |kind| core.wait_floor(kind),
        )?;
    }
    Ok(())
}

/// Judge one task's snapshot. Closing and batch waits are not inputs; they
/// describe settlement and admission, not a live task's own waits.
pub fn check_records(
    task: u64,
    waits: &[Wait],
    records: &[Record],
    held_at_holder: bool,
    moment: Moment,
    wall: Wall,
    floor: impl Fn(WaitKind) -> Option<Wall>,
) -> Result<(), String> {
    let finding = |what: &str| Inspection::new("jig", crate::KINDS).finding(moment, &format!("task {task}"), what);
    let live_delegates = waits.iter().any(|wait| wait.kind == "on its delegates" && live_tasks(wait).next().is_some());
    for wait in waits {
        if let Some(missing) = missing(wait, records, held_at_holder, live_delegates) {
            return Err(finding(&format!("{}: missing {missing}", wait.kind)));
        }
    }
    for record in records {
        let kind = name(record.kind);
        let since = record.since.as_nanos();
        let problem = if record.since > wall {
            format!("since {since} ns is after the moment's wall time {} ns", wall.as_nanos())
        } else if let Some(floor) = floor(record.kind)
            && record.since < floor
        {
            format!("since {since} ns is before its floor {} ns", floor.as_nanos())
        } else if let Bound::At { at, .. } = record.bound
            && at < record.since
        {
            format!("bound {} ns is before its since {since} ns", at.as_nanos())
        } else {
            // A missing bound already carries a reason in Bound::None.
            continue;
        };
        return Err(finding(&format!("{kind} {:?}: {problem}", record.object)));
    }
    Ok(())
}

fn live_tasks(wait: &Wait) -> impl Iterator<Item = u64> + '_ {
    wait.wakes.iter().filter_map(|wake| match wake {
        Wake::Waiter(waiter) => waiter.strip_prefix("task ")?.parse().ok(),
        Wake::Ended(_) | Wake::Party(_) | Wake::Timer(_) => None,
    })
}

fn missing(wait: &Wait, records: &[Record], held_at_holder: bool, live_delegates: bool) -> Option<String> {
    let kind = wait.kind.strip_suffix(", with no deadline armed").unwrap_or(&wait.kind);
    let has = |kind| records.iter().any(|record| record.kind == kind);
    let party = |accept: fn(EntryKind) -> bool| {
        records.iter().any(|record| {
            matches!(record, Record { kind: WaitKind::Party, object: WaitObject::Entry { kind, .. }, .. } if accept(*kind))
        })
    };
    let expected = match kind {
        "on its dependencies" | "on its delegates" => {
            let expected = if kind == "on its dependencies" { WaitKind::Dependency } else { WaitKind::Delegate };
            return live_tasks(wait)
                .find(|target| {
                    !records.iter().any(|record| record.kind == expected && record.object == WaitObject::Task(*target))
                })
                .map(|target| format!("{} record for task {target}", name(expected)));
        }
        "in a resource's queue" if !has(WaitKind::Resource) => "resource record",
        "on its requester" if !has(WaitKind::Question) && !party(|kind| matches!(kind, EntryKind::Question { .. })) => {
            "question record"
        }
        "on a person's answer" if !party(|kind| matches!(kind, EntryKind::PersonTask)) => {
            "party record for the person task"
        }
        "on its proposal's decision" if !party(|kind| matches!(kind, EntryKind::Proposal { .. })) => {
            "party record for the proposal"
        }
        kind if kind.starts_with("held:")
            && held_at_holder
            && kind != "held: placed"
            && !party(|kind| matches!(kind, EntryKind::Escalation { .. })) =>
        {
            "party record for the escalation"
        }
        "on a watch" | "on a timer" | "on its party" | "paced"
            if !live_delegates
                && !records.iter().any(|record| {
                    matches!(
                        record.kind,
                        WaitKind::News | WaitKind::Step | WaitKind::Room | WaitKind::Turn | WaitKind::SignOff
                    )
                }) =>
        {
            "news or procedure wait record"
        }
        "backing off" if !has(WaitKind::Account) => "account record",
        "suspended" if !has(WaitKind::Host) => "host record",
        _ => return None,
    };
    Some(expected.into())
}

fn name(kind: WaitKind) -> &'static str {
    match kind {
        WaitKind::Party => "party",
        WaitKind::Question => "question",
        WaitKind::Dependency => "dependency",
        WaitKind::Resource => "resource",
        WaitKind::Account => "account",
        WaitKind::Host => "host",
        WaitKind::Delegate => "delegate",
        WaitKind::News => "news",
        WaitKind::Step => "step",
        WaitKind::Room => "room",
        WaitKind::Turn => "turn",
        WaitKind::SignOff => "sign-off",
    }
}
