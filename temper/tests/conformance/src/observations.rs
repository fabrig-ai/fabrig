//! Store and forge translations for jig's outside referee.
use jig_conformance::referee as r;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use r::EffectKey;
use std::collections::{BTreeMap, BTreeSet};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;

#[must_use]
pub fn value(bytes: &[u8]) -> u64 {
    // Whole byte strings are reduced consistently at both independent boundaries.
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |n, b| (n ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3))
}
#[must_use]
pub fn repository() -> r::Name {
    r::Name {
        connector: 0,
        path: vec![b"forge".to_vec(), b"forge.example".to_vec(), b"org".to_vec(), b"repo".to_vec()],
    }
}
pub fn parts(write: &skein_forge::api::Write) -> (u16, Vec<r::Name>, Option<u64>, u64, u64) {
    let mut name = repository();
    let (kind, condition, target, state) = match write {
        skein_forge::api::Write::Status { commit, context, check, .. } => {
            (6, None, value(format!("{context:?}:{check:?}").as_bytes()), value(commit))
        }
        skein_forge::api::Write::CreateIssue { title, body, .. } => {
            (8, None, value(format!("{title:?}:{body:?}").as_bytes()), 0)
        }
        skein_forge::api::Write::Post { number, body, .. } => {
            name.path.extend([b"issue".to_vec(), number.to_string().into_bytes()]);
            (7, None, value(body), 0)
        }
        skein_forge::api::Write::Edit { number, title, body } => {
            name.path.extend([b"issue".to_vec(), number.to_string().into_bytes()]);
            (13, None, value(format!("{title:?}:{body:?}").as_bytes()), 0)
        }
        skein_forge::api::Write::Close { number } => {
            name.path.extend([b"issue".to_vec(), number.to_string().into_bytes()]);
            (13, None, 1, 0)
        }
        skein_forge::api::Write::Merge { number, head, .. } => {
            name.path.extend([b"branch".to_vec(), b"main".to_vec()]);
            (4, Some(value(head)), *number, value(head))
        }
        skein_forge::api::Write::OpenPull { title, body, head, base } => {
            (3, None, value(format!("{title:?}:{body:?}:{head:?}:{base:?}").as_bytes()), 0)
        }
        skein_forge::api::Write::CreateBranch { branch, commit } => {
            name.path.extend([b"branch".to_vec(), branch.to_vec()]);
            (9, None, value(commit), 0)
        }
        skein_forge::api::Write::DeleteBranch { branch } => {
            name.path.push(b"branch".to_vec());
            name.path.extend(branch.split(|byte| *byte == b'/').map(<[u8]>::to_vec));
            (15, None, 0, 0)
        }
        other => panic!("unbound forge effect {other:?}"),
    };
    (kind, vec![name], condition, target, state)
}
#[must_use]
pub fn key(store: &Store, entry: &skein_forge_client::Entry) -> EffectKey {
    let first = core::CallKey { task: 0, attempt: 0, completion: 0, position: 0 };
    let last = core::CallKey { task: u64::MAX, attempt: u64::MAX, completion: u32::MAX, position: u32::MAX };
    let range = root::Key::Core(core::Key::Core(core::CoreKey::Call(first)))
        ..=root::Key::Core(core::Key::Core(core::CoreKey::Call(last)));
    let call = store.rows.range(range).find_map(|(_, row)| match row {
        root::Record::Core(core::Record::Core(core::CoreRecord::Call(row)))
            if matches!(row.part, core::CallPart::Effect { entry: number, .. } if number == entry.number) => Some(row.key),
        _ => None,
    });
    let key = EffectKey {
        deployment: [31; 16], task: entry.task, purpose: entry.number,
        origin: call.map_or_else(|| {
            let tracked = [tasks::Key::Live(entry.task), tasks::Key::Ended(entry.task)]
                .into_iter().any(|key| matches!(
                    store.rows.get(&root::Key::Core(core::Key::Tasks(key))),
                    Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))))
                        if task.number == entry.task && task.tracked.is_some()
                ));
            if tracked { r::EffectOrigin::Projection } else { r::EffectOrigin::Procedure }
        }, |call| r::EffectOrigin::Call {
            attempt: call.attempt, completion: call.completion, position: call.position,
        }),
    };
    #[cfg(test)]
    assert_eq!(key, whole_key(store, entry), "indexed effect identity");
    key
}

#[cfg(test)]
fn whole_key(store: &Store, entry: &skein_forge_client::Entry) -> EffectKey {
    let call = store.rows.values().find_map(|row| match row {
        root::Record::Core(core::Record::Core(core::CoreRecord::Call(row))) if matches!(row.part, core::CallPart::Effect { entry: n, .. } if n == entry.number) => Some(row.key),
        _ => None,
    });
    EffectKey {
        deployment: [31; 16],
        task: entry.task,
        purpose: entry.number,
        origin: call.map_or_else(
            || {
                if store.rows.values().any(|record| {
                    matches!(record,
                root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task)))
                    if task.number == entry.task && task.tracked.is_some())
                }) {
                    r::EffectOrigin::Projection
                } else {
                    r::EffectOrigin::Procedure
                }
            },
            |call| r::EffectOrigin::Call {
                attempt: call.attempt,
                completion: call.completion,
                position: call.position,
            },
        ),
    }
}
#[must_use]
pub fn policy(
    seed: u64,
    scenario: Option<jig_conformance::scenarios::Scenario>,
    story: Option<super::stories::Story>,
) -> r::Policy {
    let (config, _) = super::config::root(seed, scenario, story);
    let rules = &config.core.defaults.rules;
    let project = &config.core.seed_projects[0].policy.policy;
    r::Policy {
        deployment: jig_conformance::core::scope(&rules.ceiling),
        projects: BTreeMap::from([(1, jig_conformance::core::scope(&project.ceiling))]),
        projections: BTreeMap::from([(
            1,
            project
                .projections
                .iter()
                .map(|g| r::Grant { connector: g.connector, kind: g.kind, pattern: pattern(&g.pattern) })
                .collect(),
        )]),
        people: BTreeMap::from([((1, 1), jig_conformance::core::scope(&project.roles[0].authority))]),
        effect_accepters: BTreeSet::from([(1, 1)]),
        requirements: Vec::new(),
        implications: BTreeSet::new(),
        kinds: BTreeMap::from(
            [
                (2, r::Recovery::Unrecoverable),
                (3, r::Recovery::Conditional),
                (4, r::Recovery::Conditional),
                (5, r::Recovery::Unrecoverable),
                (6, r::Recovery::Idempotent),
                (7, r::Recovery::Unrecoverable),
                (8, r::Recovery::Unrecoverable),
                (9, r::Recovery::Unrecoverable),
                (10, r::Recovery::Idempotent),
                (11, r::Recovery::Idempotent),
                (12, r::Recovery::Idempotent),
                (13, r::Recovery::Idempotent),
                (14, r::Recovery::Idempotent),
                (15, r::Recovery::Idempotent),
            ]
            .map(|(kind, recovery)| ((0, kind), r::Kind { recovery, price: 0 })),
        ),
        owned: vec![(0, r::Pattern { base: repository().path, terminal: Vec::new(), open: true })],
        participating: BTreeSet::from([repository()]),
        mechanics: BTreeSet::new(),
        delivery_bound: 60_000_000_000,
        uncertainty_bound: 60_000_000_000,
        story_steps: 1_000,
    }
}
#[must_use]
pub fn pattern(p: &jig_domain_authority::Pattern) -> r::Pattern {
    let (terminal, open) = match &p.last {
        jig_domain_authority::Last::Exact(x) => (x.to_vec(), false),
        jig_domain_authority::Last::Open(x) => (x.to_vec(), true),
    };
    r::Pattern { base: p.segments.iter().map(|x| x.to_vec()).collect(), terminal, open }
}
pub type Record = root::Record;
pub type Store = temper_engine_domain_world::store::Store;

#[must_use]
pub fn snapshot(store: &Store) -> r::Snapshot {
    let records = store
        .rows
        .values()
        .filter_map(|record| match record {
            root::Record::Core(record) => Some((record.key(), record)),
            root::Record::Forge { .. } => None,
        })
        .collect();
    let snapshot = jig_conformance::core::snapshot(&records);
    connector_snapshot(store, snapshot, jig_conformance::core::proposal_decisions(&records))
}

/// A commit's keys update the core evidence; connector facts remain independent.
#[must_use]
pub fn projected(store: &Store, projection: &mut jig_conformance::core::Projection) -> r::Snapshot {
    let snapshot = projection.snapshot_by(
        store.applied,
        store.rows.values().filter_map(|record| match record {
            root::Record::Core(record) => Some((record.key(), record)),
            root::Record::Forge { .. } => None,
        }),
        |key| match store.rows.get(&root::Key::Core(key.clone())) {
            Some(root::Record::Core(row)) => Some(row),
            Some(root::Record::Forge { .. }) | None => None,
        },
    );
    #[cfg(test)]
    {
        let records = store
            .rows
            .values()
            .filter_map(|record| match record {
                root::Record::Core(record) => Some((record.key(), record)),
                root::Record::Forge { .. } => None,
            })
            .collect();
        assert_eq!(snapshot, jig_conformance::core::snapshot(&records), "commit {}", store.applied);
    }
    connector_snapshot(store, snapshot, projection.proposal_decisions())
}

pub fn submitted(store: &Store, projection: &mut jig_conformance::core::Projection) {
    use jig_conformance::core::{Commit, Write};
    if let Some((number, writes)) = store.pending.back() {
        let writes = writes
            .iter()
            .map(|write| match write {
                root::Write::Save(root::Record::Core(record)) => Write::Save(record),
                root::Write::Erase(root::Key::Core(key)) => Write::Erase(key.clone()),
                root::Write::CopySettingsRevision(_) | root::Write::DropTranscript { .. } => Write::Refresh,
                root::Write::Save(root::Record::Forge { .. })
                | root::Write::Erase(root::Key::Forge(_) | root::Key::Opening(_) | root::Key::Wires)
                | root::Write::Opening(_)
                | root::Write::Wires => Write::Other,
            })
            .collect();
        projection.submitted(&Commit { number: *number, writes });
    }
}

fn connector_snapshot(
    store: &Store,
    mut snapshot: r::Snapshot,
    decisions: Vec<core::ProposalDecisionRecord>,
) -> r::Snapshot {
    for record in store.rows.values() {
        match record {
            root::Record::Forge { row } => {
                let forge::Stored::Entry { entry: row, held: false } = row.as_ref() else { continue };
                let key = key(store, row);
                let (kind, resources, condition, target, state) = parts(&row.effect.write);
                if let Some(attempt) = row.attempt {
                    snapshot.receipts.insert(r::Receipt::Outbox(
                        0,
                        key,
                        u32::try_from(attempt.sent.as_nanos() / 1_000_000).expect("bounded attempt timestamp"),
                    ));
                    snapshot.attempts.insert(
                        (0, key),
                        (
                            u32::try_from(attempt.sent.as_nanos() / 1_000_000).expect("bounded attempt timestamp"),
                            attempt.expires.as_nanos(),
                        ),
                    );
                }
                snapshot.decisions.push(r::Decision {
                    projection: key.origin == r::EffectOrigin::Projection,
                    connector: 0,
                    key,
                    kind,
                    resources,
                    condition,
                    target,
                    judged_state: state,
                    accepted_by: None,
                });
            }
            root::Record::Core(_) => {}
        }
    }
    for accepted in decisions {
        for decision in &mut snapshot.decisions {
            if let people::Proposer::Task(task) = accepted.proposer
                && decision.key.task == task
            {
                decision.accepted_by = Some(r::Source::Person(accepted.by));
                snapshot.acceptances.insert(
                    (0, decision.key),
                    r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
                );
            }
        }
    }
    snapshot
}

/// Entry identity and projection ownership retained from actual durable rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EffectEvidence {
    pub kind: u16,
    pub projection: bool,
}

/// Follow each durable entry to its named answer or connector erasure. A later
/// hold cannot relabel a key already observed as settled.
pub fn resolved(
    store: &Store,
    snapshot: &mut r::Snapshot,
    seen: &mut BTreeMap<EffectKey, EffectEvidence>,
    held: &mut BTreeMap<EffectKey, u16>,
    settled: &mut BTreeSet<EffectKey>,
) {
    #[cfg(test)]
    let (expected, expected_seen, expected_held, expected_settled) = {
        let mut expected = r::Snapshot {
            held_effects: snapshot.held_effects.clone(),
            settled_effects: snapshot.settled_effects.clone(),
            ..r::Snapshot::default()
        };
        let mut expected_seen = seen.clone();
        let mut expected_held = held.clone();
        let mut expected_settled = settled.clone();
        whole_resolved(store, &mut expected, &mut expected_seen, &mut expected_held, &mut expected_settled);
        (expected, expected_seen, expected_held, expected_settled)
    };
    let mut live = BTreeSet::new();
    let entries = root::Key::Forge(forge::Key::Entry(0))..=root::Key::Forge(forge::Key::Entry(u64::MAX));
    for (_, record) in store.rows.range(entries) {
        if let root::Record::Forge { row } = record
            && let forge::Stored::Entry { entry, held: false } = row.as_ref()
        {
            let key = key(store, entry);
            live.insert(key);
            let evidence = EffectEvidence {
                kind: parts(&entry.effect.write).0,
                projection: issue(store, key.task).is_some_and(|issue| issue.pending == Some(key.purpose)),
            };
            seen.entry(key)
                .and_modify(|old| {
                    assert_eq!(old.kind, evidence.kind, "durable entry changed kind");
                    old.projection |= evidence.projection;
                })
                .or_insert(evidence);
        }
    }
    for (&key, evidence) in seen.iter() {
        if held.contains_key(&key) || settled.contains(&key) {
            continue;
        }
        let answer = call_outcome(store, &key);
        match answer {
            Some(Some(core::effects::connector::OutboxOutcome::Held { .. })) => {
                held.insert(key, evidence.kind);
            }
            // A write past its lifetime found not made is settled too (D14).
            Some(Some(
                core::effects::connector::OutboxOutcome::Made
                | core::effects::connector::OutboxOutcome::NotMade
                | core::effects::connector::OutboxOutcome::Failed
                | core::effects::connector::OutboxOutcome::Withdrawn,
            )) => {
                settled.insert(key);
            }
            Some(None | Some(core::effects::connector::OutboxOutcome::Uncertain)) => {}
            None if live.contains(&key) => {}
            None => {
                let projection_failed = evidence.projection && issue(store, key.task).is_some_and(|issue| issue.failed);
                if projection_failed {
                    held.insert(key, evidence.kind);
                } else {
                    // A non-named connector entry has reached its terminal
                    // before its root-owned projection/procedure erases it.
                    settled.insert(key);
                }
            }
        }
    }
    snapshot.held_effects.extend(held.keys().map(|key| (0, *key)));
    snapshot.settled_effects.extend(settled.iter().map(|key| (0, *key)));
    #[cfg(test)]
    {
        assert_eq!(*seen, expected_seen, "indexed entry evidence");
        assert_eq!(*held, expected_held, "indexed held evidence");
        assert_eq!(*settled, expected_settled, "indexed settled evidence");
        assert_eq!(snapshot.held_effects, expected.held_effects);
        assert_eq!(snapshot.settled_effects, expected.settled_effects);
    }
}

fn issue(store: &Store, task: u64) -> Option<&forge::IssueRow> {
    let found: Option<&forge::IssueRow> = match store.rows.get(&root::Key::Forge(forge::Key::Issue(task))) {
        Some(root::Record::Forge { row }) => match row.as_ref() {
            forge::Stored::Issue(issue) if issue.goal == task => Some(issue),
            _ => None,
        },
        _ => None,
    };
    #[cfg(test)]
    {
        let whole = store.rows.values().find_map(|record| match record {
            root::Record::Forge { row } => match row.as_ref() {
                forge::Stored::Issue(issue) if issue.goal == task => Some(issue),
                _ => None,
            },
            root::Record::Core(_) => None,
        });
        assert_eq!(
            found.map(|row| (row.pending, row.failed)),
            whole.map(|row| (row.pending, row.failed)),
            "indexed issue evidence"
        );
    }
    found
}

#[expect(clippy::option_option, reason = "a retained call without an outcome differs from no call")]
fn call_outcome(store: &Store, key: &EffectKey) -> Option<Option<core::effects::connector::OutboxOutcome>> {
    let found = if let r::EffectOrigin::Call { attempt, completion, position } = key.origin {
        let wanted = core::CallKey { task: key.task, attempt, completion, position };
        match store.rows.get(&root::Key::Core(core::Key::Core(core::CoreKey::Call(wanted)))) {
            Some(root::Record::Core(core::Record::Core(core::CoreRecord::Call(call)))) if call.key == wanted => {
                match call.part {
                    core::CallPart::Effect { entry, outcome, .. } if entry == key.purpose => Some(outcome),
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    };
    #[cfg(test)]
    assert_eq!(found, whole_answer(store, key), "indexed effect outcome");
    found
}

#[cfg(test)]
#[expect(clippy::option_option, reason = "a retained call without an outcome differs from no call")]
fn whole_answer(store: &Store, key: &EffectKey) -> Option<Option<core::effects::connector::OutboxOutcome>> {
    store.rows.values().find_map(|record| match record {
        root::Record::Core(core::Record::Core(core::CoreRecord::Call(call)))
            if call.key.task == key.task
                && key.origin
                    == (r::EffectOrigin::Call {
                        attempt: call.key.attempt,
                        completion: call.key.completion,
                        position: call.key.position,
                    }) =>
        {
            match call.part {
                core::CallPart::Effect { entry, outcome, .. } if entry == key.purpose => Some(outcome),
                _ => None,
            }
        }
        _ => None,
    })
}

#[cfg(test)]
fn whole_resolved(
    store: &Store,
    snapshot: &mut r::Snapshot,
    seen: &mut BTreeMap<EffectKey, EffectEvidence>,
    held: &mut BTreeMap<EffectKey, u16>,
    settled: &mut BTreeSet<EffectKey>,
) {
    let mut live = BTreeSet::new();
    for record in store.rows.values() {
        if let root::Record::Forge { row } = record
            && let forge::Stored::Entry { entry, held: false } = row.as_ref()
        {
            let key = whole_key(store, entry);
            live.insert(key);
            let evidence = EffectEvidence {
                kind: parts(&entry.effect.write).0,
                projection: store.rows.values().any(|record| {
                    matches!(record,
                    root::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Issue(issue)
                        if issue.goal == key.task && issue.pending == Some(key.purpose)))
                }),
            };
            seen.entry(key)
                .and_modify(|old| {
                    assert_eq!(old.kind, evidence.kind, "durable entry changed kind");
                    old.projection |= evidence.projection;
                })
                .or_insert(evidence);
        }
    }
    for (&key, evidence) in seen.iter() {
        if held.contains_key(&key) || settled.contains(&key) {
            continue;
        }
        let answer = store.rows.values().find_map(|record| match record {
            root::Record::Core(core::Record::Core(core::CoreRecord::Call(call)))
                if call.key.task == key.task
                    && key.origin
                        == (r::EffectOrigin::Call {
                            attempt: call.key.attempt,
                            completion: call.key.completion,
                            position: call.key.position,
                        }) =>
            {
                match call.part {
                    core::CallPart::Effect { entry, outcome, .. } if entry == key.purpose => Some(outcome),
                    _ => None,
                }
            }
            _ => None,
        });
        match answer {
            Some(Some(core::effects::connector::OutboxOutcome::Held { .. })) => {
                held.insert(key, evidence.kind);
            }
            // A write past its lifetime found not made is settled too (D14).
            Some(Some(
                core::effects::connector::OutboxOutcome::Made
                | core::effects::connector::OutboxOutcome::NotMade
                | core::effects::connector::OutboxOutcome::Failed
                | core::effects::connector::OutboxOutcome::Withdrawn,
            )) => {
                settled.insert(key);
            }
            Some(None | Some(core::effects::connector::OutboxOutcome::Uncertain)) => {}
            None if live.contains(&key) => {}
            None => {
                let projection_failed = evidence.projection
                    && store.rows.values().any(|record| {
                        matches!(record,
                    root::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Issue(issue)
                        if issue.goal == key.task && issue.failed))
                    });
                if projection_failed {
                    held.insert(key, evidence.kind);
                } else {
                    // A non-named connector entry has reached its terminal
                    // before its root-owned projection/procedure erases it.
                    settled.insert(key);
                }
            }
        }
    }
    snapshot.held_effects.extend(held.keys().map(|key| (0, *key)));
    snapshot.settled_effects.extend(settled.iter().map(|key| (0, *key)));
}

/// Report every lost turn before task totals, preserving the map's task and
/// attempt order. Sorted attempts can be summed without a temporary task map.
pub(crate) fn expenses(
    expenses: &BTreeMap<(u64, u64), jig_fake_workers::Expense>,
    observations: &mut Vec<r::Observed>,
) {
    for (&(task, attempt), expense) in expenses {
        if expense.lost_turns > 0 {
            observations.push(r::Observed::LostTurns { task, attempt, turns: expense.lost_turns, spent: expense.lost });
        }
    }
    let mut total = None;
    for (&(task, _), expense) in expenses {
        match &mut total {
            Some((prior, spent, lost)) if *prior == task => {
                *spent += expense.spent;
                *lost += expense.lost;
            }
            Some((prior, spent, lost)) => {
                observations.push(r::Observed::Spend { task: *prior, spent: *spent, lost_bound: *lost });
                *prior = task;
                *spent = expense.spent;
                *lost = expense.lost;
            }
            None => total = Some((task, expense.spent, expense.lost)),
        }
    }
    if let Some((task, spent, lost_bound)) = total {
        observations.push(r::Observed::Spend { task, spent, lost_bound });
    }
}
