//! Core rows and released outputs translated into referee evidence
//! (`jig/docs/design/domain/testing.md`, 5–6).
pub mod acceptance;
pub mod intents;
pub mod naming;
pub mod peers;
mod projects;
mod snapshot;
pub use snapshot::{proposal_decisions, snapshot};

use std::collections::{BTreeMap, BTreeSet};

use crate::referee as r;
mod evidence;
pub use evidence::*;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

fn pattern(base: &[Box<[u8]>], terminal: &[u8], open: bool) -> r::Pattern {
    r::Pattern { base: base.iter().map(|part| part.to_vec()).collect(), terminal: terminal.to_vec(), open }
}

fn authority_pattern(value: &authority::Pattern) -> r::Pattern {
    match &value.last {
        authority::Last::Exact(last) => pattern(&value.segments, last, false),
        authority::Last::Open(last) => pattern(&value.segments, last, true),
    }
}

fn task_pattern(value: &tasks::Pattern) -> r::Pattern {
    match &value.last {
        tasks::Last::Exact(last) => pattern(&value.segments, last, false),
        tasks::Last::Open(last) => pattern(&value.segments, last, true),
    }
}

/// Translate configured authority for an application's independent referee.
#[must_use]
pub fn scope(value: &authority::Authority) -> r::Scope {
    r::Scope {
        budget: value.budget.spend,
        deadline: value.budget.deadline.map(skein_lib::Wall::as_nanos),
        tools: value.tools.0,
        notes: value.notes.0,
        grants: authority_grants(&value.grants),
        executors: value
            .delegation
            .kinds
            .iter()
            .map(|kind| match kind {
                authority::Executor::Charter(number) => (0, *number),
                authority::Executor::Procedure(number) => (1, *number),
                authority::Executor::Role(number) => (2, *number),
                authority::Executor::Job(number) => (3, *number),
            })
            .collect(),
        descendants: value.delegation.tasks,
        depth: value.delegation.depth,
        note_resources: value
            .note_resources
            .iter()
            .map(|resource| (resource.connector, authority_pattern(&resource.pattern)))
            .collect(),
    }
}

fn authority_grants(grants: &[authority::Grant]) -> Vec<r::Grant> {
    grants
        .iter()
        .map(|grant| r::Grant {
            connector: grant.connector,
            kind: grant.kind,
            pattern: authority_pattern(&grant.pattern),
        })
        .collect()
}

fn task_scope(value: &tasks::Authority) -> r::Scope {
    r::Scope {
        budget: value.budget.spend,
        deadline: value.budget.deadline.map(skein_lib::Wall::as_nanos),
        tools: value.tools.0,
        notes: value.notes.0,
        grants: value
            .grants
            .iter()
            .map(|grant| r::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: task_pattern(&grant.pattern),
            })
            .collect(),
        executors: value
            .delegation
            .kinds
            .iter()
            .map(|kind| match kind {
                tasks::AuthorityExecutor::Charter(number) => (0, *number),
                tasks::AuthorityExecutor::Procedure(number) => (1, *number),
                tasks::AuthorityExecutor::Role(number) => (2, *number),
                tasks::AuthorityExecutor::Job(number) => (3, *number),
            })
            .collect(),
        descendants: value.delegation.tasks,
        depth: value.delegation.depth,
        note_resources: value
            .note_resources
            .iter()
            .map(|resource| (resource.connector, task_pattern(&resource.pattern)))
            .collect(),
    }
}

fn name(value: &tasks::Name) -> r::Name {
    r::Name { connector: value.connector, path: value.path.iter().map(|part| part.to_vec()).collect() }
}

fn source(value: tasks::Party) -> r::Source {
    match value {
        tasks::Party::Task(task) => r::Source::Task(task),
        tasks::Party::Person(person) => r::Source::Person(person),
        tasks::Party::Deployment { .. } => r::Source::Deployment,
    }
}

fn phase(value: &tasks::Phase) -> r::Phase {
    match value {
        tasks::Phase::Waiting => r::Phase::Waiting,
        tasks::Phase::Active(_) => r::Phase::Active,
        tasks::Phase::Closing(_) => r::Phase::Closing,
        tasks::Phase::Held { .. } => r::Phase::Held,
        tasks::Phase::Ended(tasks::Ending::Done(_)) => r::Phase::Done,
        tasks::Phase::Ended(tasks::Ending::Failed { .. } | tasks::Ending::Cancelled { .. }) => r::Phase::Ended,
    }
}

/// Translate a durable task row without consulting live engine state.
#[must_use]
pub fn task(value: &tasks::TaskRecord) -> r::Task {
    r::Task {
        settings: r::SettingsRevision { task: value.settings.task, run: value.settings.run },
        tracked: value.tracked.is_some(),
        number: value.number,
        project: value.project,
        source: source(value.requester),
        scope: task_scope(&value.authority),
        executor: match value.executor {
            tasks::Executor::Job { kind } => (3, kind),
            tasks::Executor::Agent { charter } => (0, charter),
            tasks::Executor::Procedure { code, .. } => (1, code),
            tasks::Executor::Person(tasks::PersonAddress::Role(role)) => (2, role),
            tasks::Executor::Person(tasks::PersonAddress::Person(_)) => (2, 0),
        },
        budget: value.numbers.budget,
        spent: value.numbers.spent,
        spent_below: value.numbers.spent_below,
        reserved: value.numbers.reserved,
        run_reserved: value.run_reserved,
        run_spent: value.run_spent,
        attempt: value.attempt,
        period: value.recurring.as_ref().map(|recurring| recurring.last_period),
        phase: phase(&value.phase),
        spec_revision: value.revisions.spec,
        hold_revision: value.revisions.hold,
        placed: value.placed,
        placed_reason: matches!(value.phase, tasks::Phase::Held { why: tasks::Hold::Placed { .. }, .. }),
        dependencies: value.dependencies.to_vec(),
        delegates: value.delegates.to_vec(),
        pools: if value.holds_taken && !matches!(value.phase, tasks::Phase::Ended(_)) {
            value
                .holdings
                .iter()
                .filter_map(|holding| match holding {
                    tasks::Holding::Slot { pool, .. } => Some(name(pool)),
                    tasks::Holding::Write { .. } => None,
                })
                .collect()
        } else {
            Vec::new()
        },
        inbox: value.inbox.iter().map(|word| word.number).collect(),
        proposal: value.proposal.as_ref().map(|proposal| {
            let tasks::ProposalState::Pending { holder, .. } = proposal.state else {
                panic!("live proposal is pending")
            };
            (
                proposal.number,
                match holder {
                    tasks::ProposalHolder::Task(task) => r::Source::Task(task),
                    tasks::ProposalHolder::Person(person) => r::Source::Person(person),
                    tasks::ProposalHolder::Policy { .. } => r::Source::Deployment,
                },
            )
        }),
    }
}

fn call(key: core::CallKey) -> r::Receipt {
    r::Receipt::Call(key.task, key.attempt, key.completion, key.position)
}

#[derive(Clone, Debug)]
struct SeenFence {
    after: u64,
    position: Option<people::Seen>,
}

impl SeenFence {
    fn new(store: &Store<'_>, person: u64, seen: people::Seen) -> Self {
        let mut current = 0;
        let mut ended = false;
        let records = store.rows.values().copied().chain(store.pending.iter().flat_map(|pending| {
            pending.writes.iter().filter_map(|write| match write {
                Write::Save(record) => Some(*record),
                Write::Other | Write::Erase(_) => None,
            })
        }));
        for record in records {
            match seen {
                people::Seen::Results { .. } => {
                    if let core::Record::People(people::Stored::ReadResults { party, through }) = record
                        && *party == person
                    {
                        current = current.max(*through);
                    }
                }
                people::Seen::Conversation { task, .. } => {
                    if let core::Record::Tasks(tasks::Stored::Ended(row)) = record {
                        ended |= row.number == task;
                    }
                    if let core::Record::People(
                        people::Stored::ReadConversation { party, task: chat, through }
                        | people::Stored::EndedRead { party, task: chat, through },
                    ) = record
                        && *party == person
                        && *chat == task
                    {
                        current = current.max(*through);
                    }
                }
            }
        }
        // Results and live conversation marks at or below the current position
        // write nothing. Ended marks always save max(old, wanted), even if
        // forgetting erases that position in the same commit.
        let position = match seen {
            people::Seen::Results { through } => (through > current).then_some(seen),
            people::Seen::Conversation { task, through } => (ended || through > current)
                .then_some(people::Seen::Conversation { task, through: current.max(through) }),
        };
        Self { after: store.pending.back().map_or(store.applied, |pending| pending.number), position }
    }
}

/// Core evidence retained across commits and cold starts.
#[derive(Clone, Debug)]
pub struct Observer {
    pub referee: r::Referee,
    pub intents: intents::Intents,
    project_makes: projects::Makes,
    policy_generation: u64,
    roles: BTreeMap<(u32, u32), r::Scope>,
    decisions_by_role: BTreeMap<(u32, u32), u8>,
    applied: u64,
    activity: BTreeMap<(u64, u64), tasks::ActivityLine>,
    activity_erases: BTreeMap<u64, Vec<(u64, u64)>>,
    // Seen uses a commit fence instead of a new Answer receipt.
    requests: BTreeMap<u64, (u64, [u8; 16], Option<SeenFence>)>,
    proposals: BTreeMap<u64, (u16, Effect)>,
    accepted: BTreeMap<(u16, r::EffectKey), r::Source>,
    settled_calls: BTreeSet<(u64, u64, u64)>,
    claims: BTreeSet<(u64, u64)>,
    // Durable fresh marks fence these attempts: their traffic is acknowledged
    // and discarded without a turn or terminal row (engine.md, 7.2 and 8).
    crossed: BTreeSet<(u64, u64)>,
    cancelled: BTreeSet<(u64, u64)>,
    discarded: BTreeSet<(u64, u64)>,
    words: BTreeSet<(u64, u64)>,
    policy_changes: BTreeMap<(u64, [u8; 16]), (u32, people::PolicyChange)>,
    applied_changes: BTreeSet<(u64, [u8; 16])>,
    forgetting: BTreeMap<u64, Vec<u64>>,
    recorded: Option<Vec<r::Observed>>,
}

impl Observer {
    /// Capture policy from scenario configuration before constructing the root.
    #[must_use]
    pub fn new(configuration: &core::Config, connectors: ConnectorPolicy) -> Self {
        let deployment = &configuration.defaults.rules;
        let mut policy = r::Policy {
            deployment: scope(&deployment.ceiling),
            projects: BTreeMap::new(),
            projections: BTreeMap::new(),
            people: BTreeMap::new(),
            effect_accepters: BTreeSet::new(),
            requirements: Vec::new(),
            implications: BTreeSet::new(),
            kinds: BTreeMap::new(),
            owned: Vec::new(),
            participating: BTreeSet::new(),
            mechanics: BTreeSet::new(),
            delivery_bound: 60_000_000_000,
            uncertainty_bound: 60_000_000_000,
            story_steps: 30_000,
        };
        let mut roles = BTreeMap::new();
        let mut decisions_by_role = BTreeMap::new();
        for seed in &configuration.seed_projects {
            let project = seed.number;
            {
                let value = &seed.policy.policy;
                policy.projects.insert(project, scope(&value.ceiling));
                policy.projections.insert(project, authority_grants(&value.projections));
                for role in &value.roles {
                    roles.insert((project, role.number), scope(&role.authority));
                    decisions_by_role.insert((project, role.number), role.decides.0);
                }
                for (requirement, scope) in deployment
                    .requirements
                    .iter()
                    .map(|r| (r, None))
                    .chain(value.requirements.iter().map(|r| (r, Some(project))))
                {
                    policy.requirements.push(r::Requirement {
                        project: scope,
                        connector: requirement.connector,
                        kind: requirement.kind,
                        pattern: authority_pattern(&requirement.pattern),
                        judge: requirement.judge.connector,
                        freshness: match requirement.guard {
                            authority::Guard::Guarded => None,
                            authority::Guard::Observed { freshness } => Some(freshness.as_nanos()),
                        },
                    });
                }
            }
        }
        policy.kinds = connectors.kinds;
        policy.owned = connectors.owned;
        // Enumerate the finite configured kind vocabulary rather than reuse an
        // authority decision function when checking a later observed effect.
        for &(connector, given) in policy.kinds.keys() {
            for &(other, needed) in policy.kinds.keys() {
                if connector == other && given != needed && deployment.implies.allows(connector, needed, given) {
                    policy.implications.insert((connector, given, needed));
                }
            }
        }
        Self {
            referee: r::Referee::new(policy),
            policy_generation: 0,
            roles,
            decisions_by_role,
            applied: 0,
            activity: BTreeMap::new(),
            activity_erases: BTreeMap::new(),
            requests: BTreeMap::new(),
            proposals: BTreeMap::new(),
            accepted: BTreeMap::new(),
            settled_calls: BTreeSet::new(),
            claims: BTreeSet::new(),
            crossed: BTreeSet::new(),
            cancelled: BTreeSet::new(),
            discarded: BTreeSet::new(),
            words: BTreeSet::new(),
            policy_changes: BTreeMap::new(),
            applied_changes: BTreeSet::new(),
            forgetting: BTreeMap::new(),
            recorded: None,
            intents: intents::Intents::default(),
            project_makes: projects::Makes::new(configuration),
        }
    }

    /// Submit one translated observation, identifying the first broken promise.
    pub fn observe(&mut self, now: u64, observed: r::Observed) {
        if let Some(recorded) = &mut self.recorded {
            recorded.push(observed);
        } else {
            self.referee.observe(now, observed).unwrap_or_else(|error| panic!("referee {error:?} at {now}"));
        }
    }

    /// Record evidence for a referee owned by an enclosing harness.
    pub fn record(&mut self) {
        self.recorded = Some(Vec::new());
    }

    /// Consume the observer and retain its independently configured referee.
    #[must_use]
    pub fn into_referee(self) -> r::Referee {
        self.referee
    }

    /// Take observations in the order the neighbours and store produced them.
    #[must_use]
    pub fn take(&mut self) -> Vec<r::Observed> {
        std::mem::take(self.recorded.as_mut().expect("recording observer"))
    }

    /// Observe inbound peer traffic, before the root may decide it.
    pub fn inbound(&mut self, store: &Store<'_>, now: u64, event: &mut core::Event) {
        self.project_makes.inbound(store, event);
        for observed in self.intents.inbound_core(store, event) {
            self.observe(now, observed);
        }
        if let core::Event::People(people::Event::Ask { models: _, reply_to, sign_in, key, ask }) = event {
            let person = store.rows.values().copied().find_map(|row| match row {
                core::Record::People(people::Stored::SignIn { number, person, .. }) if number == sign_in => {
                    Some(*person)
                }
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Notes(_)
                | core::Record::Accounts(_) => None,
            });
            if let Some(person) = person {
                let to = destination(reply_to);
                let seen = if let people::Ask::MarkSeen { seen } = ask {
                    Some(SeenFence::new(store, person, *seen))
                } else {
                    None
                };
                self.requests.insert(to, (person, *key, seen));
                if let people::Ask::ChangePolicy { project, change } = ask {
                    self.policy_changes.insert((person, *key), (*project, change.clone()));
                }
            }
        }
    }

    /// Revision of the independently observed policy used by the enclosing referee.
    #[must_use]
    pub const fn policy_generation(&self) -> u64 {
        self.policy_generation
    }

    /// A restart changes future makes' configured policies, never an existing project's.
    pub fn configure_projects(&mut self, config: &core::Config) {
        self.project_makes.configure(config);
    }

    fn made_project(&mut self, project: u32, policy: &authority::Policy) {
        self.policy_generation = self.policy_generation.checked_add(1).expect("policy generation fits");
        self.referee.policy.projects.insert(project, scope(&policy.ceiling));
        self.referee.policy.projections.insert(project, authority_grants(&policy.projections));
        for role in &policy.roles {
            self.roles.insert((project, role.number), scope(&role.authority));
            self.decisions_by_role.insert((project, role.number), role.decides.0);
        }
        for requirement in &policy.requirements {
            self.referee.policy.requirements.push(r::Requirement {
                project: Some(project),
                connector: requirement.connector,
                kind: requirement.kind,
                pattern: authority_pattern(&requirement.pattern),
                judge: requirement.judge.connector,
                freshness: match requirement.guard {
                    authority::Guard::Guarded => None,
                    authority::Guard::Observed { freshness } => Some(freshness.as_nanos()),
                },
            });
        }
    }

    fn policy_change(&mut self, project: u32, change: &people::PolicyChange) {
        self.policy_generation = self.policy_generation.checked_add(1).expect("policy generation fits");
        match change {
            people::PolicyChange::Projections { grants } => {
                self.referee.policy.projections.insert(
                    project,
                    grants
                        .iter()
                        .map(|grant| r::Grant {
                            connector: grant.connector,
                            kind: grant.kind,
                            pattern: person_pattern(&grant.pattern),
                        })
                        .collect(),
                );
            }
            people::PolicyChange::Role(role) => {
                self.roles.insert((project, role.number), person_scope(&role.authority));
                self.decisions_by_role.insert((project, role.number), role.decides);
            }
            people::PolicyChange::Requirements { requirements } => {
                self.referee.policy.requirements.retain(|r| r.project != Some(project));
                for requirement in requirements {
                    self.referee.policy.requirements.push(r::Requirement {
                        project: Some(project),
                        connector: requirement.connector,
                        kind: requirement.kind,
                        pattern: person_pattern(&requirement.pattern),
                        judge: requirement.judge.connector,
                        freshness: match requirement.guard {
                            people::Guard::Guarded => None,
                            people::Guard::Observed { freshness } => Some(freshness.as_nanos()),
                        },
                    });
                }
            }
            people::PolicyChange::RoleSpend { .. }
            | people::PolicyChange::ProjectSpend { .. }
            | people::PolicyChange::Permissions { .. }
            | people::PolicyChange::Templates { .. } => {}
        }
    }

    /// Compare the cold store with the receipts that outside observers saw.
    pub fn cold(&mut self, now: u64, store: &Store<'_>) {
        self.requests.retain(|_, (_, _, seen)| seen.is_none());
        // An uncommitted forget is lost with the old root. A start's mark may
        // reuse its number without forgetting the task.
        self.forgetting.retain(|commit, _| *commit <= store.applied);
        self.activity_erases.retain(|commit, _| *commit <= store.applied);
        let receipts = store
            .rows
            .values()
            .copied()
            .filter_map(|row| match row {
                core::Record::Tasks(tasks::Stored::Live(value) | tasks::Stored::Ended(value)) => {
                    Some(r::Receipt::Task(value.number))
                }
                core::Record::Core(core::CoreRecord::RunProof(value)) => {
                    self.claims.insert((value.task, value.attempt));
                    if value.crossed {
                        self.crossed.insert((value.task, value.attempt));
                    }
                    Some(r::Receipt::Claim(value.task, value.attempt))
                }
                core::Record::Core(core::CoreRecord::Turn(value)) => {
                    Some(r::Receipt::Turn(value.task, value.attempt, value.turn))
                }
                core::Record::Core(core::CoreRecord::Terminal(value)) => {
                    Some(r::Receipt::Terminal(value.task, value.attempt))
                }
                core::Record::Core(core::CoreRecord::Call(value)) => Some(call(value.key)),
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Notes(_)
                | core::Record::Accounts(_) => None,
            })
            .chain(store.rows.values().copied().filter_map(|row| match row {
                core::Record::Tasks(tasks::Stored::Ended(value)) => Some(r::Receipt::Ended(value.number)),
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Notes(_)
                | core::Record::Accounts(_) => None,
            }))
            .collect();
        self.observe(now, r::Observed::ColdStore { receipts });
    }

    /// Observe the application records the store has just received.
    pub fn submitted(&mut self, store: &Store<'_>) {
        self.intents.submitted(store);
        if let Some(pending) = store.pending.back() {
            for (person, _, seen) in self.requests.values_mut() {
                let Some(fence) = seen else { continue };
                let Some(position) = fence.position else { continue };
                for write in &pending.writes {
                    let moves_position = match position {
                        people::Seen::Results { through } => matches!(write,
                            Write::Save(core::Record::People(people::Stored::ReadResults { party, through: written }))
                                if party == person && *written == through
                        ),
                        people::Seen::Conversation { task, through } => matches!(write,
                            Write::Save(core::Record::People(
                                people::Stored::ReadConversation { party, task: chat, through: written }
                                | people::Stored::EndedRead { party, task: chat, through: written }
                            )) if party == person && *chat == task && *written == through
                        ),
                    };
                    if moves_position {
                        fence.after = fence.after.max(pending.number);
                        // A later mark or live-to-ended copy cannot retag this reply.
                        fence.position = None;
                        break;
                    }
                }
            }
            let mut tasks = Vec::new();
            for write in &pending.writes {
                if let Write::Erase(core::Key::Tasks(tasks::Key::Activity { task, position })) = write
                    && store.rows.contains_key(&core::Key::Tasks(tasks::Key::Ended(*task)))
                {
                    self.activity_erases.entry(pending.number).or_default().push((*task, *position));
                }
                if let Write::Erase(core::Key::Tasks(tasks::Key::Ended(task))) = write {
                    for key in store.rows.keys() {
                        if matches!(key, core::Key::People(people::Key::ReadConversation { task: chat, .. }) if chat == task)
                        {
                            assert!(
                                pending.writes.contains(&Write::Erase(key.clone())),
                                "read marker erases with its owner"
                            );
                        }
                    }
                    tasks.push(*task);
                }
            }
            if !tasks.is_empty() {
                self.forgetting.insert(pending.number, tasks);
            }
        }
    }

    /// Snapshot only the store's durable rows, including held acknowledgements.
    pub fn durable(&mut self, now: u64, store: &Store<'_>) {
        self.durable_at(now, store, store.applied);
    }

    /// Upgrade metadata and the start's mark do not change the referee's
    /// snapshot. At bootstrap, these repeat the seed's view.
    pub fn fresh_mark(&mut self, now: u64, store: &Store<'_>) {
        for number in self.applied + 1..=store.applied {
            self.durable_at(now, store, number);
        }
    }

    #[expect(clippy::too_many_lines, reason = "one boundary translation projects every durable family")]
    fn durable_at(&mut self, now: u64, store: &Store<'_>, number: u64) {
        if number == self.applied {
            return;
        }
        for observed in self.intents.durable(store) {
            self.observe(now, observed);
        }
        for (project, policy) in self.project_makes.durable(store) {
            if !self.referee.policy.projects.contains_key(&project) {
                self.made_project(project, &policy);
            }
        }
        // Apply only an externally requested policy edit whose durable answer
        // says it succeeded. Saved policy carriers are never an authority oracle.
        for row in store.rows.values().copied() {
            if let core::Record::People(people::Stored::Answer {
                key,
                outcome: people::Outcome::PolicyChanged { .. },
                ..
            }) = row
                && self.applied_changes.insert((key.person, key.key))
                && let Some((project, change)) = self.policy_changes.get(&(key.person, key.key)).cloned()
            {
                self.policy_change(project, &change);
            }
        }
        self.discarded.clear();
        let mut snapshot = snapshot(&store.rows);
        snapshot.naming.extend(store.effects.naming.iter().cloned());
        snapshot.receipts.extend(store.effects.receipts.iter().cloned());
        snapshot.attempts.extend(store.effects.attempts.iter().map(|(key, value)| (*key, *value)));
        snapshot.held_effects.extend(store.effects.held_effects.iter().copied());
        snapshot.settled_effects.extend(store.effects.settled_effects.iter().copied());
        snapshot.decisions.clone_from(&store.effects.decisions);
        self.proposals.extend(store.effects.proposals.clone());
        for decision in &mut snapshot.decisions {
            decision.accepted_by = self.accepted.get(&(decision.connector, decision.key)).copied();
        }
        let decisions = proposal_decisions(&store.rows);
        let mut task_acceptances = Vec::new();
        let mut words = Vec::new();
        let mut reached = Vec::new();
        for row in store.rows.values().copied() {
            match row {
                core::Record::Tasks(tasks::Stored::Live(value) | tasks::Stored::Ended(value)) => {
                    if matches!(
                        value.phase,
                        tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_)
                    ) {
                        self.discarded.insert((value.number, value.attempt));
                    }
                    for word in &value.inbox {
                        if matches!(word.from, tasks::Party::Person(_))
                            && matches!(word.kind, tasks::MessageKind::Words | tasks::MessageKind::Answer { .. })
                            && self.words.insert((value.number, word.number))
                        {
                            words.push((value.number, word.number));
                        }
                        if matches!(word.kind, tasks::MessageKind::Result(_))
                            && let tasks::Party::Task(child) = word.from
                        {
                            reached.push(r::Receipt::Ended(child));
                        }
                    }
                }
                core::Record::Tasks(tasks::Stored::Activity(value)) => {
                    let kept = self.activity.entry((value.task, value.position)).or_insert_with(|| value.clone());
                    assert_eq!(kept, value, "immutable activity row changed");
                }
                core::Record::Tasks(tasks::Stored::History(value)) => {
                    if value.change == tasks::Change::ProposalAccepted
                        && let Some(proposal) = &value.proposal
                        && let tasks::ProposalAction::Effect { connector, attempt, completion, position, .. } =
                            proposal.action
                    {
                        task_acceptances.push((
                            connector,
                            value.task,
                            attempt,
                            completion,
                            position,
                            source(value.by),
                            proposal.number,
                        ));
                    }
                }
                core::Record::People(people::Stored::Roles { project, holdings }) => {
                    self.policy_generation = self.policy_generation.checked_add(1).expect("policy generation fits");
                    self.referee.policy.people.retain(|(number, _), _| number != project);
                    self.referee.policy.effect_accepters.retain(|(number, _)| number != project);
                    for holding in holdings {
                        if let Some(scope) = self.roles.get(&(*project, holding.role.number())) {
                            self.referee.policy.people.insert((*project, holding.person), scope.clone());
                            if self
                                .decisions_by_role
                                .get(&(*project, holding.role.number()))
                                .is_some_and(|bits| bits & 2 != 0)
                            {
                                self.referee.policy.effect_accepters.insert((*project, holding.person));
                            }
                        }
                    }
                }
                core::Record::People(people::Stored::Answer { ask, outcome, .. }) => {
                    if let people::Ask::Say { task, .. } = ask.as_ref()
                        && let people::Outcome::Said { task: answered, message } = outcome
                    {
                        assert_eq!(task, answered, "party's words answered for another task");
                        if self.words.insert((*task, *message)) {
                            words.push((*task, *message));
                        }
                    }
                }
                core::Record::Core(core::CoreRecord::RunProof(value)) => {
                    self.claims.insert((value.task, value.attempt));
                    if value.crossed {
                        self.crossed.insert((value.task, value.attempt));
                    }
                }
                core::Record::Core(core::CoreRecord::Call(value)) => {
                    if let Some(settled) = &value.settled {
                        self.settled_calls.insert((value.key.task, value.key.attempt, settled.serial));
                    }
                }
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Notes(_)
                | core::Record::Accounts(_) => {}
            }
        }
        for accepted in decisions {
            if let Some((connector, effect)) = self.proposals.get(&accepted.proposal) {
                for decision in &mut snapshot.decisions {
                    if decision.connector == *connector
                        && decision.kind == effect.kind
                        && decision.target == effect.target
                        && decision.key.task
                            == match accepted.proposer {
                                people::Proposer::Task(task) => task,
                                people::Proposer::Party(_) => 0,
                            }
                        && decision.key.purpose == effect.purpose
                    {
                        decision.accepted_by = Some(r::Source::Person(accepted.by));
                        snapshot.acceptances.insert(
                            (decision.connector, decision.key),
                            r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
                        );
                        self.accepted.insert((*connector, decision.key), r::Source::Person(accepted.by));
                    }
                }
            }
        }
        for (connector, task, attempt, completion, position, holder, proposal) in task_acceptances {
            for decision in &mut snapshot.decisions {
                if decision.connector == connector
                    && decision.key.task == task
                    && decision.key.origin == (r::EffectOrigin::Call { attempt, completion, position })
                {
                    decision.accepted_by = Some(holder);
                    snapshot.acceptances.insert((connector, decision.key), r::Acceptance { proposal, by: holder });
                }
            }
        }
        while let Some((&commit, _)) = self.activity_erases.first_key_value() {
            if commit > number {
                break;
            }
            for key in self.activity_erases.remove(&commit).expect("submitted activity erasure") {
                self.activity.remove(&key);
            }
        }
        for ((task, position), kept) in &self.activity {
            let key = core::Key::Tasks(tasks::Key::Activity { task: *task, position: *position });
            assert!(
                matches!(store.rows.get(&key), Some(core::Record::Tasks(tasks::Stored::Activity(row))) if row == kept),
                "immutable activity row disappeared"
            );
        }
        self.observe(now, r::Observed::Durable { number, snapshot });
        self.applied = number;
        let applied: Vec<_> = self.forgetting.keys().copied().filter(|commit| *commit <= number).collect();
        for number in applied {
            for task in self.forgetting.remove(&number).expect("submitted retention") {
                self.observe(now, r::Observed::Forgotten { task });
            }
        }
        for (task, message) in words {
            self.observe(now, r::Observed::Words { task, message });
        }
        for receipt in reached {
            self.observe(now, r::Observed::Reached(receipt));
        }
    }

    /// Observe a released core output.
    pub fn delivered(&mut self, now: u64, delivery: &mut core::Held) {
        if let core::Held::PeopleReply { to, reply } = &mut *delivery {
            for observed in self.intents.reply(skein_lib::Token::new(destination(to)), reply) {
                self.observe(now, observed);
            }
        }
        let mut requires = Vec::new();
        match delivery {
            core::Held::Cancel { run, attempt, .. } => {
                self.cancelled.insert((run.raw(), attempt.raw()));
            }
            core::Held::AcknowledgeTurn { run, attempt, turn, .. } => {
                let receipt = if self.crossed.contains(&(run.raw(), attempt.raw())) {
                    r::Receipt::Claim(run.raw(), attempt.raw())
                } else if self.cancelled.contains(&(run.raw(), attempt.raw()))
                    && self.discarded.contains(&(run.raw(), attempt.raw()))
                {
                    // A cancelled attempt's discarded body has no Turn row (fleet's turn.rs).
                    requires.push(r::Receipt::Claim(run.raw(), attempt.raw()));
                    r::Receipt::Task(run.raw())
                } else {
                    r::Receipt::Turn(run.raw(), attempt.raw(), *turn)
                };
                requires.push(receipt);
            }
            core::Held::Acknowledge { run, attempt, .. } => {
                requires.extend(self.terminal_receipt(run.raw(), attempt.raw()));
            }
            core::Held::CallAnswer { key, .. } => {
                requires.push(call(*key));
            }
            core::Held::PeopleReply { to, reply } => {
                self.reply_receipts(to, reply, &mut requires);
                if let people::Reply::Outcome(people::Outcome::ProjectRemoved { project }) = reply {
                    self.observe(now, r::Observed::ProjectRemoved { project: *project });
                }
            }
            core::Held::Acted { outcome: people::Outcome::ProjectRemoved { project }, .. } => {
                self.observe(now, r::Observed::ProjectRemoved { project: *project });
            }
            core::Held::Result { person, task, .. } => {
                requires.push(r::Receipt::Ended(*task));
                self.observe(now, r::Observed::Result { task: *task, to: r::Source::Person(*person) });
            }
            core::Held::Acted { .. }
            | core::Held::Drained
            | core::Held::Load(_)
            | core::Held::ApplicationDelivery { .. }
            | core::Held::ApplicationWatchEnded { .. }
            | core::Held::ReadLoad { .. }
            | core::Held::Read { .. }
            | core::Held::PublicRead { .. }
            | core::Held::PublicWait { .. }
            | core::Held::Applied { .. }
            | core::Held::CarriedUnused { .. }
            | core::Held::Delivery { .. }
            | core::Held::WatchEnded { .. }
            | core::Held::MakeEffect { .. }
            | core::Held::SettledCall { .. }
            | core::Held::Assign { .. }
            | core::Held::Sources { .. }
            | core::Held::CheckSource { .. }
            | core::Held::SignOutSource { .. }
            | core::Held::SourceSignedOut { .. }
            | core::Held::Grant { .. }
            | core::Held::Relayed { .. }
            | core::Held::Inbound { .. }
            | core::Held::Relay { .. }
            | core::Held::TaskTerminalAcknowledged { .. }
            | core::Held::TaskTurnKept { .. }
            | core::Held::Refuse { .. }
            | core::Held::TurnBusy { .. }
            | core::Held::StopRun { .. }
            | core::Held::NotesLoad { .. }
            | core::Held::NotesWritten { .. }
            | core::Held::NotesDeleted { .. } => {}
        }
        self.observe(now, r::Observed::Released { requires });
    }
    /// A host's independently reported completion charge.
    pub fn completion(&mut self, now: u64, task: u64, attempt: u64, cumulative: u64) {
        if self.claims.contains(&(task, attempt)) {
            self.observe(now, r::Observed::Completion { task, attempt, cumulative });
        }
    }
    /// A prepared assignment released to a host.
    pub fn assigned(&mut self, now: u64, assignment: Assignment, messages: &[tasks::Word]) {
        let Assignment { task, attempt, conversation, resumed, budget, settings } = assignment;
        self.observe(now, r::Observed::Assigned { task, attempt, conversation, resumed, budget, settings });
        for word in messages {
            self.observe(now, r::Observed::Reached(r::Receipt::Word(task, word.number)));
        }
    }
    /// A settled named call's body must be durable before its answer.
    pub fn call_answer(&self, task: u64, attempt: u64, serial: u64, writes: bool) {
        assert!(
            !writes || serial == 0 || self.settled_calls.contains(&(task, attempt, serial)),
            "answer preceded its durable named-call record"
        );
    }

    fn terminal_receipt(&self, run: u64, attempt: u64) -> Option<r::Receipt> {
        // Cancelling and forgetting an unknown host listing produces no terminal.
        if self.cancelled.contains(&(run, attempt)) && !self.claims.contains(&(run, attempt)) {
            return None;
        }
        Some(if self.crossed.contains(&(run, attempt)) {
            r::Receipt::Claim(run, attempt)
        } else {
            r::Receipt::Terminal(run, attempt)
        })
    }

    fn reply_receipts(&mut self, to: &mut skein_lib::ReplyTo, reply: &people::Reply, requires: &mut Vec<r::Receipt>) {
        match reply {
            people::Reply::SignedIn { sign_in, .. } => requires.push(r::Receipt::SignIn(*sign_in)),
            people::Reply::Signer(_)
            | people::Reply::Full { .. }
            | people::Reply::Again(_)
            | people::Reply::SignedOut
            | people::Reply::Refused(_) => {
                let to = destination(to);
                if self.requests.get(&to).is_some_and(|(_, _, seen)| seen.is_some()) {
                    self.requests.remove(&to);
                }
            }
            people::Reply::Outcome(_) => {
                let to = destination(to);
                if let Some((person, key, seen)) = self.requests.remove(&to) {
                    if let Some(fence) = seen {
                        assert!(self.applied >= fence.after, "Seen reply preceded its durable position/forget commit");
                    } else {
                        requires.push(r::Receipt::Party(person, key));
                    }
                }
            }
        }
    }
}

pub fn destination(to: &mut skein_lib::ReplyTo) -> u64 {
    let token = std::mem::replace(to, skein_lib::ReplyTo::new(skein_lib::Token::new(0))).into_token();
    *to = skein_lib::ReplyTo::new(token);
    token.raw()
}

fn person_pattern(value: &people::Pattern) -> r::Pattern {
    match &value.last {
        people::Last::Exact(last) => pattern(&value.segments, last, false),
        people::Last::Open(last) => pattern(&value.segments, last, true),
    }
}

fn person_scope(value: &people::Authority) -> r::Scope {
    r::Scope {
        budget: value.spend,
        deadline: value.deadline.map(skein_lib::Wall::as_nanos),
        tools: value.tools,
        notes: value.notes,
        grants: value
            .grants
            .iter()
            .map(|grant| r::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: person_pattern(&grant.pattern),
            })
            .collect(),
        executors: value
            .delegation
            .kinds
            .iter()
            .map(|kind| match kind {
                people::Executor::Charter(n) => (0, *n),
                people::Executor::Procedure(n) => (1, *n),
                people::Executor::Role(n) => (2, *n),
                people::Executor::Job(n) => (3, *n),
            })
            .collect(),
        descendants: value.delegation.tasks,
        depth: value.delegation.depth,
        note_resources: value
            .note_resources
            .iter()
            .map(|resource| (resource.connector, person_pattern(&resource.pattern)))
            .collect(),
    }
}
