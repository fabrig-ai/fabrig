//! Conformance over decoded client/agent bytes and independently read durable rows.
pub(super) mod agent;
use crate::world::Proc;
use jig_channel_kit::world::store::Store;
use jig_client_domain::contract as client;
use jig_client_protocol_kit::native::Received;
use jig_conformance::referee as rules;
use jig_domain_core as core;
use jig_test_domain as root;
use skein_lib::Time;
use skein_shell::Host;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Scenario {
    Answer,
    MidRun,
    Restart,
    MarkGate,
    Fault { cut: skein_world::Cut, before: u32 },
}
/// Read-only evidence captured before shutdown changes the task.
#[derive(Clone, Debug)]
pub struct ChatOutcome {
    pub calls: usize,
    pub claims: u64,
    pub tries: jig_domain_tasks::Tries,
    pub phase: jig_domain_tasks::Phase,
    pub last_reply: u64,
    pub reply: Vec<u8>,
    pub mode: Option<smith_charter::Mode>,
    pub idle_elapsed: bool,
}
/// The world's external observer survives every service lifetime.
pub struct Referee {
    finding: Option<String>,
    chat: crate::fixtures::Chat,
    chat_outcome: Rc<RefCell<Option<ChatOutcome>>>,
    idle_until: Option<Time>,
    handed: bool,
    done: bool,
    scenario: Scenario,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    rules: Rc<RefCell<rules::Referee>>,
    applied: u64,
    clients: BTreeMap<usize, usize>,
    keys: BTreeSet<usize>,
    ready: BTreeSet<usize>,
    agent_seen: usize,
    commits_seen: usize,
    rows: BTreeMap<root::Key, root::Record>,
    requests: BTreeMap<(usize, skein_lib::Token), [u8; 16]>,
    parties: BTreeMap<usize, u64>,
    status: Vec<(u32, u32, bool)>,
    attached: bool,
    cold: Option<()>,
    drained: Option<()>,
    pub(super) restarts: Rc<RefCell<u32>>,
}
impl Referee {
    pub(super) fn new(
        scenario: Scenario,
        controls: skein_world::Controls,
        mut store: Store,
        rules: Rc<RefCell<rules::Referee>>,
        restarts: Rc<RefCell<u32>>,
        chat: crate::fixtures::Chat,
        chat_outcome: Rc<RefCell<Option<ChatOutcome>>>,
    ) -> Self {
        let applied = store.applied;
        let rows = store.rows.clone();
        let finding = if applied > 0 {
            let finding = rules
                .borrow_mut()
                .observe(0, rules::Observed::Restart)
                .err()
                .map(|why| format!("new service lifetime: {why:?}"));
            store.recovering();
            finding
        } else {
            None
        };
        Self {
            finding,
            chat,
            chat_outcome,
            idle_until: None,
            handed: false,
            done: false,
            scenario,
            controls,
            store: Rc::new(RefCell::new(store)),
            rules,
            applied,
            clients: BTreeMap::new(),
            keys: BTreeSet::new(),
            ready: BTreeSet::new(),
            agent_seen: 0,
            commits_seen: 0,
            rows,
            requests: BTreeMap::new(),
            parties: BTreeMap::new(),
            status: Vec::new(),
            attached: false,
            cold: (applied > 0).then_some(()),
            drained: None,
            restarts,
        }
    }
    fn mark_gate(&self, now: Time, service: &mut crate::world::ServiceHost) -> Result<bool, String> {
        let source = self
            .rows
            .values()
            .find_map(|row| match row {
                root::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Source { number, .. })) => {
                    Some(*number)
                }
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            })
            .ok_or("the previous start kept its source")?;
        let state = service.service.account_startup(source, 1, now).ok_or("accounts component")?;
        if !service.held_mark.is_empty() {
            if state.durable {
                return Err(String::from("assertion failed: !state.durable"));
            }
            if state.driver_made {
                return Err(String::from("driver is made only after the mark"));
            }
            // jig/docs/design/domain/engine/restart.md: the core holds account requests until durable Wires.
            {
                let (observed_value, wanted) = (&(state.restores_waiting), &(0));
                if observed_value != wanted {
                    return Err(format!(
                        "source restore is held by the core; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                    ));
                }
            }
            {
                let (observed_value, wanted) = (&(state.wants_waiting), &(0));
                if observed_value != wanted {
                    return Err(format!(
                        "the restored source's want is held by the core; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                    ));
                }
            }
            if state.granted {
                return Err(String::from("assertion failed: !state.granted"));
            }
            {
                let (observed_value, wanted) = (&(service.early_connections), &(0));
                if observed_value != wanted {
                    return Err(format!(
                        "no provider connection or token request; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                    ));
                }
            }
            service.mark_gate_checked = true;
            service.hold_mark = false;
        }
        Ok(service.mark_gate_checked
            && state.durable
            && state.driver_made
            && state.granted
            && state.restores_waiting == 0
            && state.wants_waiting == 0)
    }

    fn expectation_deadline(&self) -> Time {
        Time::from_nanos(if self.chat == crate::fixtures::Chat::Finite { 10_000_000_000 } else { 60_000_000_000 })
    }
    fn accounts(&mut self, procs: &mut [Proc]) {
        for (index, proc) in procs.iter_mut().enumerate() {
            if let Proc::Client(client) = proc {
                if client.account_sign_in_ready() && self.keys.insert(index) {
                    client.account_request(jig_client::accounts::add_key(
                        jig_client::Key { asker: jig_client::Asker(77), number: 1 },
                        [61; 16],
                        jig_client::contract::AskAccountHolder::Deployment,
                        1,
                        None,
                        skein_lib::Secret::new(jig_channel_kit::fixtures::TOKEN.into()),
                    ));
                }
                if !self.ready.contains(&index)
                    && client.received.iter().any(|r| {
                        matches!(
                            r,
                            Received::Answer {
                                answer: jig_client::contract::Answer::Done(jig_client::contract::Outcome::Document(
                                    jig_client::contract::DocumentOutcome::AccountKeyKept { .. }
                                )),
                                ..
                            }
                        )
                    })
                {
                    self.ready.insert(index);
                    client.resume_after_account_setup();
                }
            }
        }
    }
    fn text_reached(&mut self, now: Time, service: &crate::world::ServiceHost, other: &[Proc]) -> Result<bool, String> {
        use crate::fixtures::Chat;
        use jig_domain_tasks::{Active, Phase, Stored};
        let calls = other
            .iter()
            .filter_map(|proc| {
                if let Proc::Peer(peer) = proc {
                    Some(
                        peer.observations()
                            .iter()
                            .filter(|seen| matches!(seen, skein_fake_peers::llm::Observation::Query { .. }))
                            .count(),
                    )
                } else {
                    None
                }
            })
            .sum();
        let store = self.store.borrow();
        let task = store.rows.values().find_map(|row| {
            if let root::Record::Core(core::Record::Tasks(Stored::Live(task))) = row { Some(task) } else { None }
        });
        let Some(task) = task else { return Ok(false) };
        let parked = task.phase == Phase::Active(Active::Idle);
        let backing_off = matches!(task.phase, Phase::Active(Active::BackingOff { .. }));
        if (self.chat == Chat::OpenText && parked || self.chat == Chat::FiniteText && backing_off)
            && self.chat_outcome.borrow().is_none()
        {
            let mut reply = Vec::new();
            for row in store.rows.values() {
                if let root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) = row {
                    let config = jig_channel_kit::fixtures::agent_configuration();
                    let decoded = smith_protocol_channel::decode_turn(
                        &turn.transcript,
                        turn.turn,
                        &config.limits.channel.transcript,
                        &config.channel_endpoints,
                    )
                    .map_err(|why| format!("committed turn bytes: {why:?}"))?;
                    for message in &decoded.messages {
                        if message.role == smith_domain::llm::Role::Assistant {
                            for block in &message.content {
                                if let smith_domain::session::llm::Block::Text { text, .. } = block {
                                    reply.extend_from_slice(text);
                                }
                            }
                        }
                    }
                }
            }
            *self.chat_outcome.borrow_mut() = Some(ChatOutcome {
                calls,
                claims: task.attempt,
                tries: task.tries,
                phase: task.phase.clone(),
                last_reply: task.last_reply,
                reply,
                mode: service.wire.mode.clone(),
                idle_elapsed: false,
            });
            self.idle_until = Some(now.saturating_add(crate::fixtures::idle_interval()));
        }
        let mut outcome = self.chat_outcome.borrow_mut();
        let Some(outcome) = outcome.as_mut() else { return Ok(false) };
        outcome.claims = task.attempt;
        outcome.calls = calls;
        Ok(if self.chat == Chat::OpenText {
            {
                let (observed_value, wanted) = (&(calls), &(1));
                if observed_value != wanted {
                    return Err(format!(
                        "an idle Open chat makes no further LLM call; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                    ));
                }
            }
            {
                let (observed_value, wanted) = (&(task.attempt), &(1));
                if observed_value != wanted {
                    return Err(format!(
                        "an idle Open chat takes no further claim; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                    ));
                }
            }
            outcome.idle_elapsed = self.idle_until.is_some_and(|until| now >= until);
            outcome.idle_elapsed && parked
        } else {
            task.attempt == 2
        })
    }
    fn say(&mut self, now: Time, event: rules::Observed) -> Result<(), String> {
        self.rules
            .borrow_mut()
            .observe(now.as_nanos(), event)
            .map_err(|why| format!("one-process conformance over bytes and durable rows: {why:?}"))
    }
}
pub(super) fn conformance() -> Rc<RefCell<rules::Referee>> {
    Rc::new(RefCell::new(rules::Referee::new(policy())))
}
fn policy() -> rules::Policy {
    let scope = rules::Scope {
        budget: 100,
        executors: BTreeSet::from([(0, 1)]),
        descendants: 8,
        depth: 8,
        ..rules::Scope::default()
    };
    rules::Policy {
        deployment: scope.clone(),
        projects: BTreeMap::from([(1, scope.clone())]),
        people: BTreeMap::from([((1, 1), scope)]),
        projections: BTreeMap::new(),
        effect_accepters: BTreeSet::new(),
        requirements: vec![],
        implications: BTreeSet::new(),
        kinds: BTreeMap::new(),
        owned: vec![],
        participating: BTreeSet::new(),
        mechanics: BTreeSet::new(),
        delivery_bound: 10_000_000_000,
        uncertainty_bound: 10_000_000_000,
        story_steps: 100_000,
    }
}
pub(super) fn terminals(store: &Store) -> u32 {
    store
        .rows
        .values()
        .filter(|row| {
            matches!(row,
                root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))
                if matches!(terminal.end, jig_domain_tasks::End::Finished { .. })
            )
        })
        .count()
        .try_into()
        .expect("fixture terminals")
}
pub(super) fn settled(store: &Store) -> u32 {
    store
        .rows
        .values()
        .filter(|row| {
            matches!(row,
                root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))
                if matches!(terminal.end, jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Lost))
            )
        })
        .count()
        .try_into()
        .expect("settled worker claims")
}
pub(super) fn interrupted(store: &Store) -> u32 {
    store
        .rows
        .values()
        .filter(|row| {
            matches!(row,
        root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))
        if terminal.end == jig_domain_tasks::End::Recheck)
        })
        .count()
        .try_into()
        .expect("interrupted claims")
}
pub(super) fn lost_tries(store: &Store) -> u32 {
    store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(task))) => Some(task.tries.lost),
            _ => None,
        })
        .sum()
}
fn snapshot(rows: &BTreeMap<root::Key, root::Record>) -> rules::Snapshot {
    let mut snapshot = rules::Snapshot::default();
    for row in rows.values() {
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Revision {
            history: false,
            revision,
        }))) = row
        {
            snapshot.newest = snapshot.newest.max(revision.head.number);
        }
        if let root::Record::Core(core::Record::Tasks(
            jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
        )) = row
        {
            snapshot.tasks.insert(task.number, jig_conformance::core::task(task));
            snapshot.receipts.insert(rules::Receipt::Task(task.number));
            for word in &task.inbox {
                snapshot.receipts.insert(rules::Receipt::Word(task.number, word.number));
            }
        }
        if let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = row {
            snapshot.receipts.insert(rules::Receipt::Claim(proof.task, proof.attempt));
        }
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) = row {
            snapshot.receipts.insert(rules::Receipt::Turn(turn.task, turn.attempt, turn.turn));
        }
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(end))) = row {
            snapshot.receipts.insert(rules::Receipt::Terminal(end.task, end.attempt));
        }
        if let root::Record::Core(core::Record::People(
            jig_domain_people::Stored::ReadConversation { party, task, through }
            | jig_domain_people::Stored::EndedRead { party, task, through },
        )) = row
        {
            snapshot.read_positions.insert((*party, *task), *through);
        }
        if let root::Record::Core(core::Record::People(jig_domain_people::Stored::SignIn { number, .. })) = row {
            snapshot.receipts.insert(rules::Receipt::SignIn(*number));
        }
        if let root::Record::Core(core::Record::People(jig_domain_people::Stored::Answer { key, .. })) = row {
            snapshot.receipts.insert(rules::Receipt::Party(key.person, key.key));
        }
    }
    snapshot
}
fn received(seen: &Received, store: &Store) -> Result<Vec<rules::Receipt>, String> {
    if let Received::Answer {
        answer: client::Answer::Done(client::Outcome::Document(client::DocumentOutcome::Created { first, last, .. })),
        ..
    } = seen
    {
        return Ok((*first..=*last).map(rules::Receipt::Task).collect());
    }
    if let Received::SignedIn { person } = seen {
        let receipts: Vec<_> = store
            .rows
            .values()
            .filter_map(|row| match row {
                root::Record::Core(core::Record::People(jig_domain_people::Stored::SignIn {
                    number,
                    person: owner,
                    ..
                })) if owner == person => Some(rules::Receipt::SignIn(*number)),
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            })
            .collect();
        if receipts.is_empty() {
            return Err(String::from("signed-in response preceded a durable sign-in"));
        }
        return Ok(receipts);
    }
    if let Received::Delivery(client::Delivery::RunState(run)) = seen
        && run.committed_turn > 0
    {
        return Ok(vec![rules::Receipt::Turn(run.task, run.attempt.ok_or("committed run")?, run.committed_turn)]);
    }
    if let Received::Read(client::ReadResult::Value(client::ReadValue::Activity(page))) = seen {
        return Ok(page
            .rows
            .iter()
            .filter_map(|row| {
                row.committed_turn().map(|turn| rules::Receipt::Turn(row.source.task, turn.attempt, turn.number))
            })
            .collect());
    }
    Ok(Vec::new())
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, now: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        let Proc::Service(service) = procs.first().ok_or("one service process")? else {
            return Err(String::from("one service process"));
        };
        if !service.attached {
            return Ok(());
        }
        let durable = service.store.borrow();
        for commit in &service.commits[self.commits_seen..] {
            if commit.number <= self.applied {
                continue;
            }
            for write in &commit.writes {
                match write {
                    root::Write::DropTranscript { task } => {
                        self.rows.retain(|key, _| {
                            if let root::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Turn {
                                task: owner,
                                ..
                            })) = key
                            {
                                owner != task
                            } else {
                                true
                            }
                        });
                    }
                    root::Write::Wires => {}
                    root::Write::CopySettingsRevision(number) => {
                        use jig_domain_core as core;
                        let key = root::Key::Core(core::Key::Core(core::CoreKey::Settings(core::settings::Key::Held(
                            *number,
                        ))));
                        let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                            core::settings::Record::Revision { revision, .. },
                        ))) = self.rows.get(&key).ok_or("held settings revision")?.clone()
                        else {
                            return Err(String::from("revision row"));
                        };
                        let row = root::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                            core::settings::Record::Revision { history: true, revision },
                        )));
                        self.rows.insert(row.key(), row);
                    }
                    root::Write::Save(record) => {
                        self.rows.insert(record.key(), record.clone());
                    }
                    root::Write::Erase(key) => {
                        self.rows.remove(key);
                    }
                }
            }
            self.applied = commit.number;
            self.say(now, rules::Observed::Durable { number: commit.number, snapshot: snapshot(&self.rows) })?;
        }
        self.commits_seen = service.commits.len();
        if self.cold.is_some() && durable.applied >= self.applied && durable.applied > 0 {
            self.say(now, rules::Observed::ColdStore { receipts: snapshot(&durable.rows).receipts })?;
            self.cold = None;
        }
        for event in &service.wire.events[self.agent_seen..] {
            self.say(now, event.clone())?;
        }
        self.agent_seen = service.wire.events.len();
        for (index, proc) in procs.iter().enumerate() {
            if let Proc::Client(client) = proc {
                let offset = self.clients.get(&index).copied().unwrap_or(0);
                for event in &client.received[offset..] {
                    if let Received::Sent { attempt, key } = event {
                        self.requests.insert((index, *attempt), *key);
                    }
                    if let Received::SignedIn { person } = event {
                        self.parties.insert(index, *person);
                    }
                    let mut requires = received(event, &durable)?;
                    if let Received::Answer { attempt, .. } = event {
                        requires.push(rules::Receipt::Party(
                            *self.parties.get(&index).ok_or("signed-in native client")?,
                            *self.requests.get(&(index, *attempt)).ok_or("the answered request's actual key")?,
                        ));
                    }
                    if !requires.is_empty() {
                        self.say(now, rules::Observed::Released { requires })?;
                    }
                }
                self.clients.insert(index, client.received.len());
            }
        }
        if self.done && self.drained.is_none() && service.is_empty() {
            if !durable.pending.is_empty() {
                return Err(String::from("a drained shutdown leaves no commit waiting"));
            }
            self.say(now, rules::Observed::Drained { at: self.applied })?;
            self.drained = Some(());
        }
        self.say(now, rules::Observed::Tick)?;
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.done {
            None
        } else {
            Some(
                self.idle_until
                    .filter(|_| self.chat == crate::fixtures::Chat::OpenText)
                    .unwrap_or(self.expectation_deadline()),
            )
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        if !self.done && now >= self.expectation_deadline() {
            Some(format!(
                "one-process expectation not met: restarts{} link{}, finished{} lost{}, durable{}, clients{:?}",
                *self.restarts.borrow(),
                self.handed,
                terminals(&self.store.borrow()),
                settled(&self.store.borrow()),
                self.store.borrow().applied,
                self.status
            ))
        } else {
            None
        }
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.done
    }
}

impl Referee {
    fn check_act(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        self.accounts(procs);
        let clients_done = procs
            .iter()
            .filter_map(|proc| match proc {
                Proc::Client(client) => Some(client.is_empty()),
                Proc::Service(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
            })
            .all(|done| done);
        self.status = procs
            .iter()
            .filter_map(|proc| match proc {
                Proc::Client(client) => Some((client.signed_in, client.chats_made, client.is_empty())),
                Proc::Service(_) | Proc::Agent { .. } | Proc::Peer(_) => None,
            })
            .collect();
        let agent_started = procs.iter().any(|proc| matches!(proc, Proc::Agent { .. }));
        let (service, other) = procs.split_first_mut().ok_or("one service process")?;
        let Proc::Service(service) = service else { return Err(String::from("one service process")) };
        if !service.attached {
            service.store = self.store.clone();
            service.attached = true;
            service.hold_mark = self.scenario == Scenario::MarkGate;
            if self.attached {
                self.handed = false;
                self.agent_seen = 0;
                self.commits_seen = 0;
                self.say(now, rules::Observed::Restart)?;
                self.store.borrow_mut().recovering();
                self.cold = Some(());
                *self.restarts.borrow_mut() += 1;
                if matches!(self.scenario, Scenario::Fault { cut: skein_world::Cut::PowerLoss, .. }) {
                    self.keys.clear();
                    self.ready.clear();
                    self.clients.clear();
                    self.requests.clear();
                    self.parties.clear();
                }
            }
            self.attached = true;
        }
        if !self.handed
            && (self.cold.is_none() || !self.attached)
            && let Some(link) = service.service.links().pop()
        {
            for proc in other.iter_mut() {
                if let Proc::Client(client) = proc
                    && client.signed_in == 0
                {
                    client.sign_in(&link).map_err(|why| format!("printed link: {why:?}"))?;
                }
            }
            self.handed = true;
        }
        let gate_ready = self.scenario == Scenario::MarkGate && self.mark_gate(now, service)?;
        let reached = match self.scenario {
            Scenario::Answer => {
                clients_done
                    && if self.chat == crate::fixtures::Chat::Finite {
                        terminals(&self.store.borrow()) > 0
                    } else {
                        self.text_reached(now, service, other)?
                    }
            }
            Scenario::MidRun => {
                clients_done
                    && agent_started
                    && self
                        .store
                        .borrow()
                        .rows
                        .values()
                        .any(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Turn(_)))))
            }
            Scenario::Restart => {
                self.handed
                    && self.cold.is_none()
                    && (settled(&self.store.borrow()) > 0 || interrupted(&self.store.borrow()) > 0)
            }
            Scenario::MarkGate => gate_ready,
            Scenario::Fault { .. } => *self.restarts.borrow() > 0 && clients_done && self.cold.is_none(),
        };
        if !self.done && reached {
            service.draining_after = Some(service.commits.len());
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            self.done = true;
        }
        if self.done {
            for proc in other {
                if let Proc::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
        }

        Ok(())
    }
}
