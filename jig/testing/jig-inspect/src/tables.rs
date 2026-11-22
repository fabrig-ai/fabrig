//! The owning tables' capacities, release rules and admission waits.
use crate::waits::alarm;
use jig_domain_core::Core;
use jig_domain_tasks::{Domain, HoldKind, Holding, Name, Phase, SubscriptionKind};
use skein_lib::Time;
use skein_world::inspect::{Entry, Inspection, Refused, Release, Table};
use std::collections::BTreeSet;

const SUBSCRIPTION: &str = "the subscriber's end, or unsubscribing";
const REPORT: &str = "the batch's answer; pinned while a batch that hasn't answered needs it; evictable otherwise";
const HOLDER: &str = "the holder's end";

fn name(holding: &Holding) -> &Name {
    match holding {
        Holding::Write { resource, .. } => resource,
        Holding::Slot { pool, .. } => pool,
    }
}

fn ended(domain: &Domain, task: u64) -> bool {
    domain.task(task).is_none_or(|row| matches!(row.phase, Phase::Ended(_)))
}

pub(crate) fn blocked(domain: &Domain, name: &Name) -> bool {
    let count = domain.inspection_holders().iter().filter(|((held, _), ())| held == name).count();
    match domain.resource_hold(name) {
        Some(HoldKind::Exclusive { .. }) => count != 0,
        Some(HoldKind::Pooled { .. }) => domain
            .inspection_pools()
            .get(name)
            .is_some_and(|pool| u64::try_from(count).unwrap_or(u64::MAX) >= u64::from(pool.slots)),
        Some(HoldKind::Shared) | None => false,
    }
}

pub(crate) fn tasks(domain: &Domain, core: Option<&Core>, into: &mut Inspection) {
    let mut reports = BTreeSet::new();
    let mut named = BTreeSet::new();
    for (resource, _) in domain.inspection_reports() {
        reports.insert(resource);
    }
    for (_, pins) in domain.inspection_report_pins() {
        reports.extend(pins);
    }
    let mut after = None;
    while let Some(row) = domain.inspection_task_after(after) {
        after = Some(row.number);
        named.extend(row.holdings.iter().map(name));
        into.table(Table {
            name: "subscriptions",
            capacity: u64::from(domain.inspection_subscription_capacity()),
            entries: row
                .subscriptions
                .iter()
                .map(|subscription| {
                    let due = matches!(subscription.kind, SubscriptionKind::Timer { period: None, .. })
                        && alarm(domain.inspection_timers(), subscription.number).is_none();
                    Entry {
                        key: format!("task {} subscription {}", row.number, subscription.number),
                        rule: SUBSCRIPTION,
                        release: if due { Release::Due } else { Release::Pending },
                    }
                })
                .collect(),
            refused: vec![],
        });
    }
    into.table(Table {
        name: "creation reports",
        capacity: u64::from(domain.inspection_reports().capacity()),
        entries: reports
            .into_iter()
            .map(|resource| {
                let mut pinned = false;
                let mut stale = false;
                for (owner, pins) in domain.inspection_report_pins() {
                    if pins.contains(resource) {
                        pinned = true;
                        stale |= core.is_some_and(|core| core.inspection_batch(*owner).is_none());
                    }
                }
                Entry {
                    key: resource_key(resource),
                    rule: REPORT,
                    release: if stale {
                        Release::Due
                    } else if pinned || named.contains(resource) {
                        Release::Pending
                    } else {
                        Release::Kept
                    },
                }
            })
            .collect(),
        refused: vec![],
    });
    for (resource, pool) in domain.inspection_pools() {
        holding_table(domain, core, resource, "pools", u64::from(pool.slots), into);
    }
    let mut writes = BTreeSet::new();
    for ((resource, _), ()) in domain.inspection_holders() {
        if matches!(domain.resource_hold(resource), Some(HoldKind::Exclusive { .. })) {
            writes.insert(resource);
        }
    }
    for resource in writes {
        holding_table(domain, core, resource, "write holds", 1, into);
    }
}

fn holding_table(
    domain: &Domain,
    core: Option<&Core>,
    resource: &Name,
    table: &'static str,
    capacity: u64,
    into: &mut Inspection,
) {
    let entries = domain
        .inspection_holders()
        .iter()
        .filter(|((held, _), ())| held == resource)
        .map(|((_, holder), ())| Entry {
            key: format!("task {holder}"),
            rule: HOLDER,
            release: if ended(domain, *holder) { Release::Due } else { Release::Pending },
        })
        .collect();
    let mut refused = Vec::new();
    for ((waiting, task), ()) in domain.inspection_waiters() {
        if waiting != resource {
            continue;
        }
        let blockers: Vec<_> = domain
            .inspection_waiters()
            .iter()
            .filter_map(|((waiting, number), ())| (*number == *task && blocked(domain, waiting)).then_some(waiting))
            .collect();
        if blockers.as_slice() == [resource] {
            refused.push(Refused {
                key: format!("task {task}"),
                // The child's saved wait start is Wall, not monotonic Time (testing.md, 7.1).
                since: Time::ZERO,
                bound: if core.is_some_and(|core| core.inspection_timer_withheld(jig_domain_core::Timer::Tasks)) {
                    None
                } else {
                    alarm(domain.inspection_hold_alarms(), *task)
                },
            });
        }
    }
    into.table(Table { name: table, capacity, entries, refused });
}

fn resource_key(name: &Name) -> String {
    let path: Vec<_> = name.path.iter().map(|part| String::from_utf8_lossy(part)).collect();
    format!("connector {} resource {}", name.connector, path.join("/"))
}

fn hex(key: &[u8; 16]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(32);
    for byte in key {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    text
}

fn request_key(key: &jig_domain_people::RequestKey) -> String {
    format!("person {} request {}", key.person, hex(&key.key))
}

fn act_key(key: &jig_domain_people::ActKey) -> String {
    format!("connector {} request {}", key.connector, hex(&key.key))
}

fn answer(key: String, due: Time, now: Time, withheld: bool) -> Entry {
    Entry {
        key,
        rule: "their retention's expiry",
        release: if due <= now && !withheld { Release::Due } else { Release::Kept },
    }
}

pub(crate) fn answers(core: &Core, into: &mut Inspection) {
    let people = &core.people;
    let now = core.inspection_now();
    let withheld = core.inspection_timer_withheld(jig_domain_core::Timer::People);
    let capacity = people.inspection_answer_capacity();
    let mut entries = Vec::new();
    for index in 0..people.inspection_answer_count().max(people.inspection_act_answer_count()) {
        if let Some((key, due)) = people.inspection_answer(index) {
            entries.push(answer(request_key(key), due, now, withheld));
        }
        if let Some((key, due)) = people.inspection_act_answer(index) {
            entries.push(answer(act_key(key), due, now, withheld));
        }
    }
    for index in 0..people.inspection_request_count() {
        if let Some((key, count)) = people.inspection_answer_reservation(index) {
            for place in 0..count {
                let key = if place == 0 { request_key(key) } else { format!("{} written answer", request_key(key)) };
                entries.push(Entry { key, rule: "its answer", release: Release::Pending });
            }
        }
    }
    for index in 0..people.inspection_act_count() {
        if let Some(key) = people.inspection_act_reservation(index) {
            entries.push(Entry { key: act_key(key), rule: "its answer", release: Release::Pending });
        }
    }
    into.table(Table { name: "kept answers", capacity: u64::from(capacity), entries, refused: vec![] });
}
