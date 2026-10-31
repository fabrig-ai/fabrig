//! The testing root's connector and peer evidence for jig's conformance core
//! (`jig/docs/design/domain/testing.md`, 5).
use crate::{Store, referee as r};
use jig_conformance::core as conformance;
use jig_test_connector as connector;
use jig_test_domain as root;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Observer {
    inner: conformance::Observer,
    application_requests: bool,
    effects: [usize; 2],
    reads: [usize; 2],
    programs: BTreeMap<(u16, u16), Box<[connector::ProcedureAction]>>,
}

impl std::ops::Deref for Observer {
    type Target = conformance::Observer;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::ops::DerefMut for Observer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Translate the test connector's stable identity at the outside boundary.
#[must_use]
pub fn effect_key(key: connector::Key) -> r::EffectKey {
    r::EffectKey {
        deployment: key.deployment,
        task: key.task,
        purpose: key.purpose,
        origin: match key.origin {
            connector::Purpose::Procedure => r::EffectOrigin::Procedure,
            connector::Purpose::Projection => r::EffectOrigin::Projection,
            connector::Purpose::Call { attempt, completion, position } => {
                r::EffectOrigin::Call { attempt, completion, position }
            }
        },
    }
}

fn connector_policy(policy: &mut conformance::ConnectorPolicy, number: u16, connector: &connector::Config) {
    let parts = connector.prefix.segments();
    // The deployment prefix owns its complete subtree, but not the prefix object itself.
    policy.owned.push((
        number,
        r::Pattern { base: parts.iter().map(|part| part.to_vec()).collect(), terminal: Vec::new(), open: true },
    ));
    for kind in &connector.kinds {
        policy.kinds.insert(
            (number, kind.kind),
            r::Kind {
                price: kind.price.unwrap_or(0),
                recovery: match kind.recovery {
                    connector::Recovery::Keyed => r::Recovery::Keyed,
                    connector::Recovery::Conditional => r::Recovery::Conditional,
                    connector::Recovery::Idempotent => r::Recovery::Idempotent,
                    connector::Recovery::Unrecoverable => r::Recovery::Unrecoverable,
                },
            },
        );
    }
}

impl Observer {
    #[must_use]
    pub fn new(configuration: &root::Config) -> Self {
        let mut connectors = conformance::ConnectorPolicy::default();
        for (number, config) in [(1, &configuration.first), (2, &configuration.second)] {
            connector_policy(&mut connectors, number, config);
        }
        let programs = [(1, &configuration.first), (2, &configuration.second)]
            .into_iter()
            .flat_map(|(number, config)| {
                config.procedures.iter().map(move |spec| ((number, spec.number), spec.actions.clone()))
            })
            .collect();
        Self {
            inner: conformance::Observer::new(&configuration.core, connectors),
            application_requests: false,
            effects: [0; 2],
            reads: [0; 2],
            programs,
        }
    }
    #[must_use]
    pub fn into_referee(self) -> r::Referee {
        self.inner.into_referee()
    }

    #[must_use]
    pub fn recording(configuration: &root::Config) -> Self {
        let mut observer = Self::new(configuration);
        observer.inner.record();
        observer
    }
    pub fn systems(&mut self, now: u64, systems: &[jig_test_system::System; 2]) {
        for (index, system) in systems.iter().enumerate() {
            let connector = u16::try_from(index + 1).expect("two systems");
            for read in &system.reads()[self.reads[index]..] {
                self.observe(
                    now,
                    r::Observed::Read {
                        resource: r::Name { connector, path: read.resource.path.clone() },
                        state: read.state,
                        at: read.at,
                    },
                );
            }
            self.reads[index] = system.reads().len();
            for effect in &system.observed()[self.effects[index]..] {
                self.observe(
                    now,
                    r::Observed::Effect {
                        connector,
                        effect: r::EffectObserved {
                            kind: effect.kind,
                            key: effect.key,
                            resources: effect
                                .resources
                                .iter()
                                .map(|name| r::Name { connector, path: name.path.clone() })
                                .collect(),
                            condition: effect.condition,
                            target: effect.target,
                            before: effect.before.clone(),
                            applied: effect.applied,
                            copy: effect.copy,
                        },
                    },
                );
            }
            self.effects[index] = system.observed().len();
        }
    }
    pub fn workers(&mut self, now: u64, workers: &[jig_fake_workers::Worker]) {
        let mut totals: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
        for worker in workers {
            for ((task, attempt), expense) in &worker.expenses {
                if expense.lost_turns > 0 {
                    self.observe(
                        now,
                        r::Observed::LostTurns {
                            task: *task,
                            attempt: *attempt,
                            turns: expense.lost_turns,
                            spent: expense.lost,
                        },
                    );
                }
                let total = totals.entry(*task).or_default();
                total.0 += expense.spent;
                total.1 += expense.lost;
            }
        }
        for (task, (spent, lost_bound)) in totals {
            self.observe(now, r::Observed::Spend { task, spent, lost_bound });
        }
    }
    pub fn configure_projects(&mut self, config: &root::Config) {
        self.inner.configure_projects(&config.core);
    }
    pub fn inbound(&mut self, store: &Store, now: u64, event: &mut root::Event) {
        if let root::Event::Core(event) = event {
            self.inner.inbound(&store_view(store), now, event);
        }
        if let root::Event::Answer { task, attempt, end, .. } = event {
            for observed in self.inner.intents.terminal(*task, *attempt, end) {
                self.observe(now, observed);
            }
        }
        if let root::Event::EffectCall { key, proposal: Some(_), .. } = event {
            for observed in self.inner.intents.propose(*key) {
                self.observe(now, observed);
            }
        }
        if let root::Event::Turn { task, attempt, cumulative, .. }
        | root::Event::Answer { task, attempt, cumulative, .. } = event
        {
            self.inner.completion(now, *task, *attempt, *cumulative);
        }
        if let root::Event::Connector {
            number,
            event:
                connector::Event::System(connector::SystemEvent::Applied {
                    entry,
                    result: connector::ApplyResult::Uncertain,
                    ..
                }),
        } = event
            && let Some(key) = store.rows.values().find_map(|row| match row {
                root::Record::Connector { number: owner, record: connector::Record::Outbox(outbox) }
                    if owner == number && outbox.number == *entry =>
                {
                    Some(outbox.key)
                }
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            })
        {
            self.observe(now, r::Observed::Uncertain { connector: *number, key: effect_key(key) });
        }
        if let Some(observed) = crate::application_observations::requested(store, event) {
            self.application_requests = true;
            self.observe(now, observed);
        }
    }
    pub fn submitted(&mut self, now: u64, store: &Store) {
        let mut view = store_view(store);
        if let Some(commit) = store.pending.back() {
            for write in &commit.writes {
                if let root::Write::Save(root::Record::Connector {
                    number,
                    record: connector::Record::Procedure(state),
                }) = write
                    && state.awaiting
                    && let Some(task) = state.delegate
                    && let Some(actions) = self.programs.get(&(*number, state.number))
                    && let Some(connector::ProcedureAction::Amend { priority, .. }) =
                        actions.get(usize::from(state.index))
                {
                    view.named.push(conformance::NamedRevision {
                        connector: *number,
                        owner: state.task,
                        step: state.steps,
                        task,
                        revision: state.revision,
                        priority: priority.is_some(),
                    });
                }
            }
        }
        self.inner.submitted(&view);
        if self.application_requests {
            for observed in crate::application_observations::submitted(store) {
                self.observe(now, observed);
            }
        }
    }
    pub fn durable(&mut self, now: u64, store: &Store) {
        self.inner.durable(now, &store_view(store));
    }
    pub fn fresh_mark(&mut self, now: u64, store: &Store) {
        self.inner.fresh_mark(now, &store_view(store));
    }
    pub fn cold(&mut self, now: u64, store: &Store) {
        self.inner.cold(now, &store_view(store));
    }
    pub fn delivered(&mut self, now: u64, delivery: &mut root::Delivery) {
        let mut requires = Vec::new();
        match delivery {
            root::Delivery::Assigned { assignment, .. } => {
                self.inner.assigned(
                    now,
                    conformance::Assignment {
                        task: assignment.task,
                        attempt: assignment.attempt,
                        conversation: assignment.conversation,
                        resumed: !assignment.transcript.is_empty(),
                        budget: assignment.run.budget,
                        settings: assignment.run.revision,
                    },
                    &assignment.messages,
                );
            }
            root::Delivery::Core(held) => {
                self.inner.delivered(now, held);
                return;
            }
            root::Delivery::CallAnswer { task, attempt, call: settled, writes, .. } => {
                self.inner.call_answer(*task, *attempt, settled.serial, *writes);
            }
            root::Delivery::Message { task: run, word, .. } => {
                requires.push(r::Receipt::Word(*run, word.number));
                self.observe(now, r::Observed::Reached(r::Receipt::Word(*run, word.number)));
            }
            root::Delivery::System { connector, call: connector::SystemRequest::Apply { key, attempt, .. } } => {
                requires.push(r::Receipt::Outbox(*connector, effect_key(*key), *attempt));
            }
            root::Delivery::Procedure { task, .. } => {
                requires.push(r::Receipt::Task(*task));
            }
            root::Delivery::LostRead { task, attempt, .. } => {
                requires.push(r::Receipt::Terminal(*task, *attempt));
            }
            root::Delivery::Application { to, number, outcome } => {
                self.observe(
                    now,
                    r::Observed::ApplicationAnswered {
                        reply: conformance::destination(to),
                        connector: *number,
                        outcome: crate::application_observations::encoded(outcome),
                    },
                );
            }
            root::Delivery::Fleet(_)
            | root::Delivery::ApplicationDocument { .. }
            | root::Delivery::ApplicationRead { .. }
            | root::Delivery::Projection { .. }
            | root::Delivery::Link { .. }
            | root::Delivery::ProjectMade { .. }
            | root::Delivery::CallBusy { .. }
            | root::Delivery::Start(_)
            | root::Delivery::System { .. } => {}
        }
        self.observe(now, r::Observed::Released { requires });
    }
}
/// Borrow the durable core rows and translate only the test connector's facts.
#[must_use]
pub fn store_view(store: &Store) -> conformance::Store<'_> {
    let mut view = conformance::Store {
        applied: store.applied,
        rows: store
            .rows
            .values()
            .filter_map(|row| match row {
                root::Record::Core(row) => Some((row.key(), row)),
                root::Record::Connector { .. } => None,
            })
            .collect(),
        pending: store
            .pending
            .iter()
            .map(|pending| conformance::Commit {
                number: pending.number,
                writes: pending
                    .writes
                    .iter()
                    .map(|write| match write {
                        root::Write::Save(root::Record::Core(row)) => conformance::Write::Save(row),
                        root::Write::Erase(root::Key::Core(key)) => conformance::Write::Erase(key.clone()),
                        root::Write::Save(root::Record::Connector { .. })
                        | root::Write::Erase(root::Key::Connector { .. })
                        | root::Write::Wires
                        | root::Write::CopySettingsRevision(_) => conformance::Write::Other,
                    })
                    .collect(),
            })
            .collect(),
        ..conformance::Store::default()
    };
    let snapshot = &mut view.effects;
    for row in store.rows.values() {
        match row {
            root::Record::Connector {
                number,
                record: connector::Record::Proposal { number: proposal, effect, .. },
            } => {
                snapshot.proposals.insert(
                    *proposal,
                    (
                        *number,
                        conformance::Effect { kind: effect.kind, target: effect.target, purpose: effect.purpose },
                    ),
                );
            }
            root::Record::Connector { number, record: connector::Record::Outbox(value) } => {
                if let Some(attempt) = value.attempt {
                    snapshot.receipts.insert(r::Receipt::Outbox(*number, effect_key(value.key), attempt.number));
                    snapshot
                        .attempts
                        .insert((*number, effect_key(value.key)), (attempt.number, attempt.deadline.as_nanos()));
                }
                if value.phase == connector::EffectPhase::Held {
                    snapshot.held_effects.insert((*number, effect_key(value.key)));
                }
                snapshot.decisions.push(r::Decision {
                    projection: value.key.origin == connector::Purpose::Projection,
                    connector: *number,
                    key: effect_key(value.key),
                    kind: value.effect.kind,
                    resources: value
                        .effect
                        .resources
                        .iter()
                        .map(|resource| r::Name {
                            connector: *number,
                            path: resource.segments().iter().map(|part| part.to_vec()).collect(),
                        })
                        .collect(),
                    condition: value.effect.condition,
                    target: value.effect.target,
                    judged_state: value.effect.state,
                    accepted_by: None,
                });
            }
            root::Record::Connector { number, record: connector::Record::Made { key, .. } } => {
                snapshot.settled_effects.insert((*number, effect_key(*key)));
            }
            root::Record::Core(_) | root::Record::Connector { .. } => {}
        }
    }
    view
}

// The most held of this world's stories, all binaries counting:
// 9,195,220 bytes at jig 4cd5d46966, in conformance's
// client_settings::an_owner_moves_running_work_through_the_client_and_hears_both_counts.
// The core crate's own most is 880,068 bytes.
// A ceiling, a quarter above, rounded up to two significant digits
// (skein's dynamic-memory draft, 7.1); it moves by hand only in the commit
// of a real change that passes it, never on a type's size.
const CORE_WORLD_HEAP_CEILING: u64 = 12_000_000;

/// A conservative heap observation at each testing-root iteration. Test peers'
/// allocations after construction are counted too; passing this check therefore
/// bounds the engine even while those peers retain their independent ledgers.
/// The engine's own tighter step peaks are measured separately in memory tests.
pub struct Heap {
    meter: skein_world::domain::heap::Meter,
    maximum: u64,
}

impl std::fmt::Debug for Heap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Heap").field("maximum", &self.maximum).finish_non_exhaustive()
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    /// Start before root construction, after the scenario built its configuration.
    #[must_use]
    pub fn new() -> Self {
        Self { meter: skein_world::domain::heap::Meter::new(), maximum: CORE_WORLD_HEAP_CEILING }
    }

    /// Heap observation of allocator-held bytes since construction.
    #[must_use]
    pub fn observed(&self) -> r::Observed {
        r::Observed::Heap { held: self.meter.held(), maximum: self.maximum }
    }
}

#[cfg(test)]
mod activity_tests {
    use super::*;
    use jig_domain_core as core;
    use jig_domain_tasks as tasks;

    #[test]
    fn activity_referee_rejects_changed_or_erased_lines() {
        for erased in [false, true] {
            let configuration = crate::world::fixture(507, false).0;
            let mut observer = Observer::new(&configuration);
            let mut store = Store::new(&jig_test_profile::limits(), &configuration.core.settings.models);
            let row = root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
                task: 1,
                position: 1,
                at: skein_lib::Wall::EPOCH,
                line: tasks::Line::Milestone { position: 1 },
            })));
            let key = row.key();
            store.rows.insert(key.clone(), row);
            store.applied = 1;
            observer.durable(0, &store);
            if erased {
                store.rows.remove(&key);
            } else {
                store.rows.insert(
                    key,
                    root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
                        task: 1,
                        position: 1,
                        at: skein_lib::Wall::EPOCH,
                        line: tasks::Line::Milestone { position: 2 },
                    }))),
                );
            }
            store.applied = 2;
            let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| observer.durable(1, &store)));
            assert!(rejected.is_err(), "referee must reject an immutable row changing or disappearing");
        }
    }
}
