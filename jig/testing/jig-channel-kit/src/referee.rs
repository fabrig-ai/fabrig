//! Channel-wide safety and liveness (protocol/channel/README.md, 11).
use crate::{
    processes::Proc,
    world::{Store, answer_parties, answer_store, diagnostics, terminal, turns, worker_wire},
};
use jig_domain_core as core;
use jig_fake_parties as parties;
use skein_lib::Time;
use std::{cell::RefCell, rc::Rc};
/// When `observe_roots` formats a host's roots. A root's state can newly hold
/// a grant only once the grant exists, so a host is due when the wire's count
/// of grants has moved since it was last checked, and at its next root step
/// after that, when the receiving root has taken the grant up. `every_step`
/// keeps the check at each root step, for `fuzzy::channel`.
pub struct RootChecks {
    every_step: bool,
    hosts: std::collections::BTreeMap<u64, RootCheck>,
}

#[derive(Default)]
struct RootCheck {
    grants: usize,
    steps: u64,
    next_step: bool,
}

impl RootChecks {
    #[must_use]
    pub fn new(every_step: bool) -> Self {
        Self { every_step, hosts: std::collections::BTreeMap::new() }
    }

    /// Whether this host, at this many root steps and with this many grants
    /// seen on the wire, is due; records that it was checked when it is.
    pub fn due(&mut self, grants: usize, host: u64, steps: u64) -> bool {
        let checked = self.hosts.entry(host).or_default();
        let new_grant = grants != checked.grants;
        if !new_grant && !((self.every_step || checked.next_step) && steps != checked.steps) {
            return false;
        }
        checked.grants = grants;
        checked.steps = steps;
        checked.next_step = new_grant;
        true
    }
}

#[expect(clippy::struct_excessive_bools, reason = "independent party, injection and settlement controls")]
pub struct Referee {
    pub(crate) redial_ceiling: skein_lib::Duration,
    pub(crate) cut_ready: Rc<std::cell::Cell<u32>>,
    pub(crate) accounts: crate::account_setup::Setup,
    /// Owned outside every engine process, so restart cannot drop durable rows.
    pub(crate) store: Rc<RefCell<Store>>,
    pub(crate) party: parties::Party,
    pub(crate) period: bool,
    pub(crate) passed: bool,
    pub(crate) stopped: bool,
    pub(crate) rules: Rules,
    pub(crate) observed: usize,
    pub(crate) expected_turns: u32,
    pub(crate) expected_answers: usize,
    pub(crate) story: crate::stories::Story,
    pub(crate) injected: bool,
    pub(crate) calibrating_held_turn: bool,
    pub(crate) resumed_attempt: u64,
    pub(crate) engine_observed: usize,
    pub(crate) root_checks: RootChecks,
    pub(crate) formatted: String,
    pub(crate) engine_instance: Option<u64>,
    pub(crate) views: Rc<RefCell<crate::world::Views>>,
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        self.accounts.act(procs, self.story, self.passed);
        if !self.passed {
            self.accounts.party(now, procs, &mut self.party, &mut self.views.borrow_mut(), self.story);
        }
        self.inject(now, procs);

        if self.views.borrow().query_submissions.is_none() && procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some())) {
            self.views.borrow_mut().query_submissions = procs.iter().find_map(|p| if let Proc::Engine(e) = p { Some(e.submitted) } else { None });
        }
        for proc in procs {
            match proc {
                Proc::Engine(host) => {
                    let mut store = self.store.borrow_mut();
                    if self.engine_instance.is_some_and(|instance| instance != host.revision.0) {
                        if matches!(self.story, crate::stories::Story::DurableRestart) {
                            let mut views = self.views.borrow_mut();
                            views.durable_at_restart = store.applied;
                            views.held_at_restart = store.pending.front().map(|commit| commit.number);
                            self.injected = true;
                        }
                        if matches!(self.story, crate::stories::Story::FailedStarts { .. })
                            && host.failed_starts.ended == 0
                        {
                            host.failed_starts.arm();
                        }
                        store.crash();
                        self.period = false;
                    }
                    self.engine_instance = Some(host.revision.0);
                    answer_store(
                        host,
                        &mut store,
                        &mut self.rules,
                        matches!(self.story, crate::stories::Story::Facts)
                            && self.injected
                            && self.views.borrow().reply.prefixes.len() < crate::live_text::PIECES
                            || matches!(self.story, crate::stories::Story::Input)
                                && self.views.borrow().input.opened
                                && !self.views.borrow().input.seen
                            || matches!(self.story, crate::stories::Story::DurableRestart) && !self.injected
                            || matches!(self.story, crate::stories::Story::HeldCommit | crate::stories::Story::Drain)
                                && now < Time::from_nanos(1_000_000_000),
                        self.story,
                        now,
                    );
                    if host.holding && crate::world::pending_turn(&store) {
                        let mut views = self.views.borrow_mut();
                        if views.held_turn_submissions.is_none() {
                            views.held_turn_submissions = Some(host.submitted);
                            if self.calibrating_held_turn {
                                self.injected = true;
                                host.holding = false;
                            }
                        }
                    }
                    let engine = host.service.engine().expect("engine layer");
                    self.views.borrow_mut().messages_ended = self.rules.message_ends.len();
                    self.views.borrow_mut().refusals_kept = !self.rules.refused_messages.is_empty();
                    answer_parties(engine, &mut self.party, &mut self.views.borrow_mut());
                    if !self.period && self.party.sign_in.is_some() {
                        engine.parties_in().push(core::Event::Period {
                            project: 1,
                            period: 1,
                            budget: 100,
                            next: None,
                        });
                        self.period = true;
                    }
                    if self.passed && !host.stopping {
                        host.stopping = true;
                        host.service.shutdown();
                    }
                }
                Proc::Worker(host) if self.passed && !host.stopping => {
                    host.stopping = true;
                    host.service.shutdown();
                }
                Proc::Peer(peer) if self.passed => peer.shutdown(),
                Proc::Probe(peer) if self.passed => peer.shutdown(),
                Proc::Worker(_)
                | Proc::Peer(_)
                | Proc::Auth(_)
                | Proc::Client(_)
                | Proc::Agent { .. }
                | Proc::Probe(_) => {}
            }
        }
        self.stopped = self.passed;
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the referee settles the channel story and its application read together"
    )]
    fn observe(&mut self, now: Time, procs: &[Proc]) {
        if self.observe_probe(procs) {
            return;
        }
        let worker = worker_wire(procs);
        self.observe_worker(worker);
        self.observe_roots(worker, procs);
        let engine = procs
            .iter()
            .find_map(|proc| if let Proc::Engine(host) = proc { Some(host) } else { None })
            .expect("engine");
        self.observe_engine(now, engine);
        let store = self.store.borrow();
        if matches!(self.story, crate::stories::Story::Providers)
            && self.views.borrow().engine_drained
            && !store.recovered.is_empty()
        {
            let choices = crate::world::provider_choices(&store);
            if !choices.1.is_empty() {
                self.views.borrow_mut().providers_after = choices;
            }
        }
        let turns = turns(&store);
        assert!(turns.len() <= self.expected_turns as usize, "bounded committed turns for the script");
        let engine = procs
            .iter()
            .find_map(|proc| match proc {
                Proc::Engine(engine) => Some(engine),
                Proc::Worker(_)
                | Proc::Agent { .. }
                | Proc::Peer(_)
                | Proc::Auth(_)
                | Proc::Client(_)
                | Proc::Probe(_) => None,
            })
            .expect("engine process");
        assert!(worker.acknowledgements.len() <= self.expected_answers, "one received answer acknowledgement");
        assert!(engine.wire.acknowledgements.len() <= self.expected_answers, "one sent answer acknowledgement");
        for &(task, attempt) in &worker.acknowledgements {
            assert!(
                terminal(&store, task, attempt, self.story),
                "acknowledgement follows a durable successful terminal: {task}/{attempt} at {now:?}; replies {:?}; turns {:?}; diagnostics {}",
                self.party.replies,
                crate::world::turns(&store).iter().map(|turn| (turn.attempt, turn.turn, turn.read)).collect::<Vec<_>>(),
                diagnostics(procs)
            );
        }
        let replied = matches!(
            self.story,
            crate::stories::Story::Refusals(
                crate::stories::MessageRefusal::Full | crate::stories::MessageRefusal::NameInUse
            )
        ) || procs.iter().any(|proc| match proc {
            Proc::Peer(peer) => {
                matches!(
                    self.story,
                    crate::stories::Story::InvalidCharter
                        | crate::stories::Story::Shutdown
                        | crate::stories::Story::Drain
                        | crate::stories::Story::BeyondGrace
                ) || smith_agent_process_world::fake::replied(peer)
            }
            Proc::Engine(_)
            | Proc::Worker(_)
            | Proc::Agent { .. }
            | Proc::Probe(_)
            | Proc::Auth(_)
            | Proc::Client(_) => false,
        });
        let ended = if matches!(self.story, crate::stories::Story::InvalidCharter) {
            self.rules.refused_ends.len() == 1 && worker.acknowledgements.is_empty()
        } else if matches!(self.story, crate::stories::Story::InvalidTranscript) {
            self.rules.refused_ends.len() == 1 && transcript_recovered(worker, &store)
        } else {
            worker.acknowledgements.len() == self.expected_answers
        };
        if self.party.quiescent()
            && turns.len() == self.expected_turns as usize
            && ended
            && replied
            && self.rules.complete()
            && self.provisional_committed()
            && (!matches!(self.story, crate::stories::Story::Ask) || self.views.borrow().application_answer.is_some())
            && (!matches!(self.story, crate::stories::Story::Providers)
                || !self.views.borrow().providers_after.1.is_empty())
        {
            if matches!(self.story, crate::stories::Story::ApplicationGeneration) {
                assert!(self.injected, "generation 2 announced while the agent was running");
                application_generation(worker);
            }
            if matches!(self.story, crate::stories::Story::Chat | crate::stories::Story::Providers) {
                assert_eq!(&*presented(worker, procs, false).1, crate::fixtures::TOKEN);
            }
            if let crate::stories::Story::Key { replaced } = self.story {
                assert_eq!(presented(worker, procs, true), (2, crate::fixtures::KEY.into()));
                if replaced {
                    assert!(self.injected, "generation 3 put while the agent was running");
                    key_replaced(worker);
                }
            }
            self.passed = true;
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.passed {
            return (!self.stopped).then_some(Time::ZERO);
        }
        if matches!(self.story, crate::stories::Story::HeldCommit) && !self.injected
            || matches!(self.story, crate::stories::Story::Drain) && !self.store.borrow().pending.is_empty()
        {
            Some(Time::from_nanos(1_000_000_000))
        } else {
            Some(self.horizon())
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.passed && now >= self.horizon()).then(|| {
            format!(
                "story {:?} completes; received hellos {}; replies {:?}, durable turns {}, rules {:?}",
                self.story,
                self.cut_ready.get(),
                self.party.replies,
                turns(&self.store.borrow()).len(),
                self.rules
            ) + &self.formatted
        })
    }
    fn passed(&self) -> bool {
        self.passed && self.stopped
    }
}

fn assignment(worker: &crate::processes::Wire) -> jig_protocol_worker_wire::Assignment {
    use jig_protocol_worker_wire::{Assignment, CEILINGS};
    worker
        .frames
        .iter()
        .find_map(|(_, kind, bytes)| {
            (*kind == 0x0180)
                .then(|| Assignment::decode(&CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("assignment"))
        })
        .expect("live assignment")
}

/// The live assignment's account grant is a credential in its scheme, and the
/// provider saw it presented so: Anthropic's path tells a key from a bearer.
/// Returns the grant's generation and bytes.
fn presented(worker: &crate::processes::Wire, procs: &[Proc], key: bool) -> (u64, Box<[u8]>) {
    use jig_protocol_worker_wire::{GrantValue, Scheme};
    let assignment = assignment(worker);
    let (grant, value) = assignment
        .grants()
        .iter()
        .find_map(|grant| match grant.value() {
            GrantValue::Credential(value) => Some((grant, value)),
            GrantValue::Application(_) | GrantValue::Program(_) => None,
        })
        .expect("assignment holds the account's credential");
    assert_eq!(*value.scheme(), Scheme::Key);
    let (route, scheme) = if key {
        (skein_llm::Provider::Anthropic, skein_llm::Scheme::Key)
    } else {
        (skein_llm::Provider::OpenAi, skein_llm::Scheme::Bearer)
    };
    let peer = procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Peer(peer) => Some(peer),
            Proc::Engine(_)
            | Proc::Worker(_)
            | Proc::Agent { .. }
            | Proc::Probe(_)
            | Proc::Auth(_)
            | Proc::Client(_) => None,
        })
        .expect("provider");
    assert!(
        peer.observations().iter().any(|observation| matches!(
            observation,
            skein_fake_peers::llm::Observation::Credential { route: seen, scheme: Some(presented), index: Some(0), .. }
                if *seen == route && *presented == scheme
        )),
        "the provider matched the granted credential in its scheme"
    );
    (grant.generation(), value.bytes().expose().into())
}

/// The key's generation 3 reached the live attempt's connection before its answer.
fn key_replaced(worker: &crate::processes::Wire) {
    use jig_protocol_worker_wire::{GrantValue, Scheme};
    let assignment = assignment(worker);
    let (fd, granted) = worker
        .granted
        .iter()
        .find(|(_, frame)| frame.grant().generation() == 3)
        .expect("generation 3 granted on the live attempt's connection (0x0187)");
    assert_eq!((granted.task(), granted.attempt()), (assignment.task(), assignment.attempt()));
    assert_eq!(worker.placements.get(&(granted.task(), granted.attempt())), Some(fd));
    assert_eq!(granted.grant().number(), 0);
    let GrantValue::Credential(value) = granted.grant().value() else { panic!("the account's grant is a credential") };
    assert_eq!(*value.scheme(), Scheme::Key);
    assert_eq!(value.bytes().expose(), crate::fixtures::NEXT_KEY);
    let grant = worker.frames.iter().position(|(_, kind, _)| *kind == 0x0187).expect("granted frame");
    let answer = worker.frames.iter().position(|(_, kind, _)| *kind == 0x0103).expect("same attempt answers");
    assert!(grant < answer, "the key arrives before the live attempt finishes");
}

fn application_generation(worker: &crate::processes::Wire) {
    use jig_protocol_worker_wire::GrantValue;
    let assignment = assignment(worker);
    let (initial, secret) = assignment
        .grants()
        .iter()
        .find_map(|grant| match grant.value() {
            GrantValue::Application(value) => Some((grant, value.secret())),
            GrantValue::Credential(_) | GrantValue::Program(_) => None,
        })
        .expect("assignment holds the application credential");
    assert_eq!(initial.number(), 1);
    assert_eq!(initial.generation(), 1);
    assert_eq!(secret.expose(), b"application-token");
    let (fd, granted, secret) = worker
        .granted
        .iter()
        .find_map(|(fd, frame)| match frame.grant().value() {
            GrantValue::Application(value) if frame.grant().generation() == 2 => Some((fd, frame, value.secret())),
            GrantValue::Application(_) | GrantValue::Credential(_) | GrantValue::Program(_) => None,
        })
        .expect("generation 2 Application grant received on the live attempt's connection (0x0187)");
    assert_eq!((granted.task(), granted.attempt()), (assignment.task(), assignment.attempt()));
    assert_eq!(worker.placements.get(&(granted.task(), granted.attempt())), Some(fd));
    assert_eq!(granted.grant().number(), 1);
    assert_eq!(secret.expose(), b"application-token-2");
    assert!(granted.grant().valid() > skein_lib::Duration::ZERO);
    let grant = worker.frames.iter().position(|(_, kind, _)| *kind == 0x0187).expect("granted frame");
    let answer = worker.frames.iter().position(|(_, kind, _)| *kind == 0x0103).expect("same attempt answers");
    assert!(grant < answer, "the grant arrives before the live attempt finishes");
}

/// Event ledger used by the live referee and its planted-breach tests.
/// Replayed wire transmissions are not new durable decisions.
#[derive(Debug, Default)]
pub struct Rules {
    turns: std::collections::BTreeSet<(u64, u64, u32)>,
    turn_acks: std::collections::BTreeSet<(u64, u64, u32)>,
    answers: std::collections::BTreeSet<(u64, u64)>,
    refused_starts: std::collections::BTreeSet<(u64, u64)>,
    refused_ends: std::collections::BTreeSet<(u64, u64)>,
    answer_acks: std::collections::BTreeSet<(u64, u64)>,
    lost_ends: std::collections::BTreeSet<(u64, u64)>,
    calls: std::collections::BTreeSet<(u64, u64, Box<[u8]>)>,
    call_answers: std::collections::BTreeSet<(u64, u64, Box<[u8]>)>,
    messages: std::collections::BTreeSet<(u64, u64, u64)>,
    refused_messages: std::collections::BTreeSet<(u64, u64, u64)>,
    message_ends: std::collections::BTreeSet<(u64, u64, u64)>,
    running: std::collections::BTreeMap<u64, (u64, Time)>,
    failed: Option<String>,
}
impl Rules {
    pub fn turned(&mut self, task: u64, attempt: u64, number: u32) {
        if !self.turns.insert((task, attempt, number)) {
            self.fail("a turn reaches the durable engine once");
        }
    }
    pub fn turn_acknowledged(&mut self, task: u64, attempt: u64, number: u32) {
        let key = (task, attempt, number);
        if !self.turns.contains(&key) || !self.turn_acks.insert(key) {
            self.fail("a turn is acknowledged once, after its durable commit");
        }
    }
    pub fn answered(&mut self, task: u64, attempt: u64) {
        let unread: Vec<_> = self
            .messages
            .range((task, attempt, 0)..=(task, attempt, u64::MAX))
            .filter(|key| !self.message_ends.contains(key))
            .copied()
            .collect();
        for (_, _, name) in unread {
            self.message_ended(task, attempt, name);
        }
        // The worker's terminal tail reports that the agent has already gone.
        self.running.remove(&task);
        if self.refused_starts.contains(&(task, attempt)) {
            if !self.refused_ends.insert((task, attempt)) {
                self.fail("a refused start ends once");
            }
        } else if !self.answers.insert((task, attempt)) {
            self.fail("an answer reaches the durable engine once");
        }
    }
    pub fn lost_ended(&mut self, task: u64, attempt: u64, now: Time) {
        if self.running.get(&task).is_some_and(|(_, until)| now < *until) {
            self.fail("a lost attempt ends only after its worker's stop bound");
        }
        if !self.lost_ends.insert((task, attempt)) {
            self.fail("a lost attempt ends once");
        }
        self.running.remove(&task);
        let unread: Vec<_> = self
            .messages
            .range((task, attempt, 0)..=(task, attempt, u64::MAX))
            .filter(|key| !self.message_ends.contains(key))
            .copied()
            .collect();
        for (_, _, name) in unread {
            self.message_ended(task, attempt, name);
        }
        if self.refused_starts.contains(&(task, attempt)) {
            self.refused_ends.insert((task, attempt));
        }
    }
    pub fn refused(&mut self, task: u64, attempt: u64) {
        self.running.remove(&task);
        if !self.refused_starts.insert((task, attempt)) {
            self.fail("a refused start reaches the engine at most once");
        }
    }
    pub fn acknowledged(&mut self, task: u64, attempt: u64) {
        let key = (task, attempt);
        if self.refused_starts.contains(&key) {
            self.fail("a refused start is never acknowledged");
        }
        if !self.answers.contains(&key) || !self.answer_acks.insert(key) {
            self.fail("an answer is acknowledged once, after its durable commit");
        }
        self.running.remove(&task);
    }
    pub fn called(&mut self, task: u64, attempt: u64, name: &[u8]) {
        self.calls.insert((task, attempt, name.into()));
    }
    pub fn call_answered(&mut self, task: u64, attempt: u64, name: &[u8]) {
        let key = (task, attempt, name.into());
        if !self.calls.contains(&key) || !self.call_answers.insert(key) {
            self.fail("each relayed call is answered once");
        }
    }
    pub fn sent(&mut self, task: u64, attempt: u64, name: u64) {
        self.messages.insert((task, attempt, name));
    }
    pub fn message_ended(&mut self, task: u64, attempt: u64, name: u64) {
        let key = (task, attempt, name);
        if !self.messages.contains(&key) || !self.message_ends.insert(key) {
            self.fail("each sent message ends once");
        }
    }
    pub fn message_refused(&mut self, task: u64, attempt: u64, name: u64) {
        self.refused_messages.insert((task, attempt, name));
        if !self.message_ends.contains(&(task, attempt, name)) {
            self.message_ended(task, attempt, name);
        }
    }
    pub fn read(&mut self, task: u64, attempt: u64, through: Option<u64>) {
        let Some(through) = through else { return };
        let read: Vec<_> = self
            .messages
            .range((task, attempt, 0)..=(task, attempt, through))
            .filter(|key| !self.message_ends.contains(key))
            .copied()
            .collect();
        for (_, _, name) in read {
            self.message_ended(task, attempt, name);
        }
    }
    fn complete(&self) -> bool {
        self.turns == self.turn_acks
            && self.answers == self.answer_acks
            && self.calls == self.call_answers
            && self.messages == self.message_ends
            && self.refused_starts == self.refused_ends
            && self.running.is_empty()
    }
    pub fn placed(&mut self, task: u64, attempt: u64, now: Time) {
        if self.running.get(&task).is_some_and(|(last, until)| *last != attempt && now < *until) {
            self.fail("no new attempt while a worker may run the last");
        }
        self.running.insert(task, (attempt, Time::from_nanos(u64::MAX)));
    }
    pub fn lost(&mut self, task: u64, attempt: u64, may_run_until: Time) {
        if let Some((current, until)) = self.running.get_mut(&task)
            && *current == attempt
        {
            *until = may_run_until;
        }
    }
    pub fn roots(&mut self, state: &str, grants: &[&[u8]]) {
        for value in grants {
            let bytes = format!("{value:?}");
            if !value.is_empty() && state.contains(&bytes) {
                self.fail("a credential grant value appears in a root's state");
            }
        }
    }
    fn fail(&mut self, why: &str) {
        if self.failed.is_none() {
            self.failed = Some(why.into());
        }
    }
    #[must_use]
    pub fn verdict(&self) -> skein_world::domain::Verdict {
        use skein_world::domain::{Failure, Verdict};
        if let Some(why) = &self.failed {
            Verdict::Failed(Failure { at: Time::ZERO, why: why.clone(), pending: Vec::new() })
        } else {
            Verdict::Passed
        }
    }
}

pub(crate) struct Roots<'a>(pub(crate) &'a jig_test_service::Service);
impl std::fmt::Debug for Roots<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_roots(formatter)
    }
}

impl Referee {
    /// Once the provider is first asked, the next generation of the grant the
    /// story replaces mid-run: the application's credential, or the key.
    fn inject_grant(&mut self, _now: Time, procs: &mut [Proc]) {
        let key = matches!(self.story, crate::stories::Story::Key { replaced: true });
        if !(key || matches!(self.story, crate::stories::Story::ApplicationGeneration)) || self.injected
            || !procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some()))
        {
            return;
        }
        self.injected = true;
        if key {
            self.accounts.replace(procs);
            return;
        }
        for proc in procs.iter_mut() {
            if let Proc::Engine(host) = proc {
                let engine = host.service.engine().expect("engine");
                if key {
                    unreachable!("LLM key replacement goes through the client");
                }
                engine.put_application(1, 2, skein_lib::Secret::new(b"application-token-2".as_slice().into()));
                engine.parties_in().push(core::Event::Account(jig_domain_accounts::Event::Credential {
                    number: 1,
                    generation: 2,
                    valid: skein_lib::Duration::from_secs(60),
                }));
            }
        }
    }

    fn inject_ask(&mut self, procs: &mut [Proc]) {
        if matches!(self.story, crate::stories::Story::Ask)
            && procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some()))
        {
            for proc in procs.iter_mut() {
                if let Proc::Worker(host) = proc {
                    if !self.injected && let Some((&(run, attempt), _)) = host.wire.placements.iter().next() {
                        host.service.worker(0).expect("reference worker").ask(skein_lib::Token::new(run), skein_lib::Token::new(attempt), b"x".as_slice().into());
                        self.injected = true;
                    }
                    if let Some(answer) = host.service.worker(0).expect("reference worker").pop_asked() {
                        assert!(self.views.borrow().application_answer.is_none(), "the worker's root hears one answer");
                        self.views.borrow_mut().application_answer = Some(answer);
                    }
                }
            }
        }
    }
    fn inject(&mut self, now: Time, procs: &mut [Proc]) {
        self.inject_ask(procs);
        if matches!(self.story, crate::stories::Story::HeldCommit) && now >= Time::from_nanos(1_000_000_000) {
            self.injected = true;
        }
        if matches!(self.story, crate::stories::Story::FailedStarts { .. }) {
            for proc in procs.iter_mut() {
                if let Proc::Engine(host) = proc {
                    crate::failed_starts::advance(host);
                }
            }
        }
        self.drain_engine(procs);
        self.resume_runs(procs);
        self.inject_refusal(procs);
        self.inject_grant(now, procs);
        if matches!(self.story, crate::stories::Story::Inbox) && !self.injected {
            self.injected = true;
            for proc in procs.iter_mut() {
                if let Proc::Worker(host) = proc {
                    host.relay.until = Some(Time::from_nanos(1_000_000_000));
                }
            }
        }
        self.input_story(procs);

        if matches!(self.story, crate::stories::Story::Facts) && !self.injected
            && procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some())) {
            self.injected = true;
            for proc in procs.iter_mut() {
                if let Proc::Client(client) = proc {
                    for (number, subject) in [(70, jig_client::contract::Subject::Task { task: 1 }), (71, jig_client::contract::Subject::Run { task: 1, turn: None, offset: 0 })] {
                        client.account_request(jig_client::Event::Watch { handle: jig_client::Handle { asker: jig_client::Asker(78), number }, subject: jig_client::Watchable::Jig(subject) });
                    }
                }
            }
        }

        if matches!(self.story, crate::stories::Story::Facts) && self.injected {
            self.views.borrow_mut().reply.act(procs, &self.store.borrow());
        }

        self.shutdown_worker(procs);
        let (engine, worker) = match self.story {
            crate::stories::Story::Partition { engine, worker } => (engine, worker),
            crate::stories::Story::BeyondGrace => {
                (skein_lib::Duration::from_secs(40), skein_lib::Duration::from_secs(40))
            }
            crate::stories::Story::Ask
            | crate::stories::Story::Chat
            | crate::stories::Story::ApplicationGeneration
            | crate::stories::Story::Key { .. }
            | crate::stories::Story::Stalled
            | crate::stories::Story::Quiet
            | crate::stories::Story::Replaced
            | crate::stories::Story::Cuts { .. }
            | crate::stories::Story::Reconnect
            | crate::stories::Story::Shutdown
            | crate::stories::Story::Drain
            | crate::stories::Story::Refusals(_)
            | crate::stories::Story::Opening(_)
            | crate::stories::Story::Progress
            | crate::stories::Story::Facts
            | crate::stories::Story::Input
            | crate::stories::Story::Inbox
            | crate::stories::Story::Call
            | crate::stories::Story::Wait
            | crate::stories::Story::InvalidTranscript
            | crate::stories::Story::MidFrame
            | crate::stories::Story::Window
            | crate::stories::Story::InvalidCharter
            | crate::stories::Story::Restart { .. }
            | crate::stories::Story::FailedStarts { .. }
            | crate::stories::Story::HeldCommit
            | crate::stories::Story::DurableRestart
            | crate::stories::Story::Providers => return,
        };
        if self.injected
            || !procs.iter().any(|proc| match proc {
                Proc::Worker(host) => host.wire.frames.iter().any(|(_, kind, _)| *kind == 0x0180),
                Proc::Engine(_)
                | Proc::Agent { .. }
                | Proc::Peer(_)
                | Proc::Auth(_)
                | Proc::Client(_)
                | Proc::Probe(_) => false,
            })
        {
            return;
        }
        self.injected = true;
        for proc in procs {
            match proc {
                Proc::Engine(host) => {
                    host.relay.until = Some(now.saturating_add(engine));
                    host.relay.channel = host.wire.channel;
                }
                Proc::Worker(host) => {
                    host.relay.until = Some(now.saturating_add(worker));
                    host.relay.channel = host.wire.channel;
                }
                Proc::Agent { .. } | Proc::Peer(_) | Proc::Auth(_) | Proc::Client(_) | Proc::Probe(_) => {}
            }
        }
    }
}

impl Referee {
    fn horizon(&self) -> Time {
        // A cut assignment leaves an unhosted claim adrift until the fleet's grace.
        // Allow one redial plus 30 s for startup, retry placement and the chat's turn.
        let grace = crate::fixtures::store_limits().root.core.fleet.grace.as_nanos();
        let after_grace = grace
            .checked_add(self.redial_ceiling.as_nanos())
            .and_then(|nanos| nanos.checked_add(30_000_000_000))
            .expect("grace and recovery fit the clock");
        let base = 300_000_000_000_u64.max(after_grace);
        let extra = match self.story {
            crate::stories::Story::Cuts { times } => {
                self.redial_ceiling.as_nanos().checked_mul(u64::from(times)).expect("story cuts fit the clock")
            }
            _ => 0,
        };
        Time::from_nanos(base.checked_add(extra).expect("story horizon fits the clock"))
    }

    fn observe_probe(&mut self, procs: &[Proc]) -> bool {
        if let crate::stories::Story::Cuts { times } = self.story {
            let engine =
                procs.iter().find_map(|p| if let Proc::Engine(e) = p { Some(e) } else { None }).expect("engine");
            self.cut_ready.set(engine.wire.admitted_hellos);
            self.passed = engine.wire.admitted_hellos == times.checked_add(1).expect("cut count fits");
            assert!(engine.wire.placements.is_empty(), "cuts host no run");
            return true;
        }
        if let crate::stories::Story::Opening(case) = self.story {
            let probe = procs
                .iter()
                .find_map(|proc| if let Proc::Probe(peer) = proc { Some(peer) } else { None })
                .expect("probe");
            let expected = match case {
                crate::stories::Opening::Secret => Some(256),
                crate::stories::Opening::Wires => Some(257),
                crate::stories::Opening::Version => Some(1),
                crate::stories::Opening::Terms => Some(2),
                crate::stories::Opening::Family | crate::stories::Opening::Garbage => Some(3),
            };
            if let Some(reason) = probe.refused {
                assert_eq!(Some(reason), expected, "opening {case:?}");
                self.passed = true;
            }
            if matches!(case, crate::stories::Opening::Family | crate::stories::Opening::Garbage) && probe.ended {
                self.passed = true;
            }
            return true;
        }
        if matches!(self.story, crate::stories::Story::Replaced) {
            self.passed = procs.iter().filter(|proc| matches!(proc, Proc::Probe(peer) if peer.ready)).count() == 2
                && procs.iter().any(|proc| matches!(proc, Proc::Probe(peer) if peer.ended));
            return true;
        }
        if matches!(self.story, crate::stories::Story::Stalled | crate::stories::Story::Quiet) {
            self.passed = procs.iter().any(|proc| matches!(proc, Proc::Engine(host) if host.wire.closed))
                && procs.iter().any(|proc| matches!(proc, Proc::Probe(peer) if peer.ready));
            return true;
        }
        false
    }
}

impl Referee {
    fn observe_worker(&mut self, worker: &crate::processes::Wire) {
        for (_, kind, bytes) in &worker.frames[self.observed..] {
            let mut reader = skein_lib::Reader::new(bytes);
            match *kind {
                0x0183 => {
                    let kept =
                        jig_protocol_worker_wire::TurnKept::decode(&jig_protocol_worker_wire::CEILINGS, &mut reader)
                            .expect("kept turn");
                    self.rules.turn_acknowledged(kept.task(), kept.attempt(), kept.number());
                }
                0x0182 => {
                    let reply =
                        jig_protocol_worker_wire::Relayed::decode(&jig_protocol_worker_wire::CEILINGS, &mut reader)
                            .expect("relayed call");
                    if matches!(reply.answer(), jig_protocol_worker_wire::RelayAnswer::Text(_)) {
                        self.rules.call_answered(reply.task(), reply.attempt(), reply.name());
                    }
                }
                0x0185 => {
                    let ack =
                        jig_protocol_worker_wire::Acknowledge::decode(&jig_protocol_worker_wire::CEILINGS, &mut reader)
                            .expect("answer ack");
                    self.rules.acknowledged(ack.task(), ack.attempt());
                }
                _ => {}
            }
        }
        self.observed = worker.frames.len();
    }
}

impl Referee {
    fn observe_roots(&mut self, worker: &crate::processes::Wire, procs: &[Proc]) {
        let grants: Vec<_> = worker.grants.iter().map(AsRef::as_ref).collect();
        for proc in procs {
            if let Proc::Engine(host) | Proc::Worker(host) = proc
                && !grants.is_empty()
                && self.root_checks.due(grants.len(), host.revision.0, host.service.root_steps())
            {
                use std::fmt::Write;
                self.formatted.clear();
                write!(&mut self.formatted, "{:?}", Roots(&host.service)).expect("root state");
                self.rules.roots(&self.formatted, &grants);
            }
        }
        assert_eq!(self.rules.verdict(), skein_world::domain::Verdict::Passed);
    }
}

impl Referee {
    fn observe_engine(&mut self, now: Time, engine: &crate::processes::ServiceHost) {
        if now >= self.horizon() {
            self.formatted = format!(
                "engine frames {:?}; roots {:?}",
                engine.wire.frames.iter().map(|(_, kind, _)| *kind).collect::<Vec<_>>(),
                Roots(&engine.service)
            );
        }
        for (&(task, attempt), &fd) in &engine.wire.placements {
            if let Some(until) = engine.wire.lost.get(&fd) {
                self.rules.lost(task, attempt, *until);
            }
        }
        if self.engine_observed > engine.wire.frames.len() {
            self.engine_observed = 0;
        }
        for (at, kind, bytes) in &engine.wire.frames[self.engine_observed..] {
            if *kind == 0x0181 {
                let message = jig_protocol_worker_wire::Inbound::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("inbound message");
                self.rules.sent(message.task(), message.attempt(), message.name());
            }
            if *kind == 0x0103 {
                let answer = jig_protocol_worker_wire::Answer::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("worker answer");
                if matches!(answer.ending(), jig_protocol_worker_wire::Ending::Refused(_)) {
                    assert!(
                        !matches!(self.story, crate::stories::Story::Window),
                        "the compaction fixture must admit its start: {:?}",
                        answer.ending()
                    );
                    self.rules.refused(answer.task(), answer.attempt());
                }
            }
            if *kind == 0x0102 {
                let call = jig_protocol_worker_wire::Call::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("call");
                self.rules.called(call.task(), call.attempt(), call.name());
            }
            if *kind == 0x0180 {
                let assignment = jig_protocol_worker_wire::Assignment::decode(
                    &jig_protocol_worker_wire::CEILINGS,
                    &mut skein_lib::Reader::new(bytes),
                )
                .expect("assignment");
                self.rules.placed(assignment.task(), assignment.attempt(), *at);
                for message in assignment.messages() {
                    self.rules.sent(assignment.task(), assignment.attempt(), message.name());
                }
            }
        }
        self.engine_observed = engine.wire.frames.len();
        assert_eq!(self.rules.verdict(), skein_world::domain::Verdict::Passed, "at {now:?}");
    }
}

impl Referee {
    fn shutdown_worker(&mut self, procs: &mut [Proc]) {
        if !matches!(self.story, crate::stories::Story::Shutdown) {
            return;
        }
        if !self.injected
            && !turns(&self.store.borrow()).is_empty()
            && procs.iter().any(
                |proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).count() >= 2),
            )
        {
            self.injected = true;
            for proc in procs.iter_mut() {
                if let Proc::Worker(host) = proc {
                    host.service.shutdown();
                    host.stopping = true;
                }
            }
        }
        for proc in procs.iter_mut() {
            if let Proc::Worker(host) = proc
                && self.injected
                && host.service.is_empty()
                && let Some(root) = host.restart_root.take()
            {
                let Proc::Worker(replacement) = crate::processes::worker(root) else {
                    unreachable!("replacement worker")
                };
                host.service = replacement.service;
                host.wire.restarted();
                host.stopping = false;
            }
        }
        if self.views.borrow().resumed_tries.is_none()
            && worker_wire(procs).frames.iter().filter(|(_, kind, _)| *kind == 0x0180).count() >= 2
        {
            self.views.borrow_mut().resumed_tries = self.store.borrow().rows.values().find_map(|row| {
                if let jig_test_domain::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(task))) = row {
                    Some(task.tries)
                } else {
                    None
                }
            });
        }
    }

    fn resume_runs(&mut self, procs: &[Proc]) {
        let parked = self
            .store
            .borrow()
            .rows
            .values()
            .filter_map(|row| match row {
                jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer)))
                    if answer.end == jig_domain_tasks::End::Parked =>
                {
                    Some(answer.attempt)
                }
                jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
            })
            .max();
        if matches!(self.story, crate::stories::Story::Window | crate::stories::Story::InvalidTranscript)
            && (!matches!(self.story, crate::stories::Story::InvalidTranscript) || self.resumed_attempt == 0)
            && parked.is_some_and(|at| at > self.resumed_attempt)
            && self.party.quiescent()
        {
            self.injected = true;
            self.resumed_attempt = parked.expect("parked attempt");
            self.party.extend(Box::new([parties::Act::Request {
                key: [u8::try_from(self.resumed_attempt + 2).expect("few attempts"); 16],
                ask: parties::Ask::Say {
                    task: parties::Task::Last,
                    words: b"Continue now".as_slice().into(),
                    release: None,
                },
            }]));
        }
        if matches!(self.story, crate::stories::Story::Wait)
            && !self.injected
            && procs.iter().any(|proc| matches!(proc, Proc::Worker(host) if host.wire.waited))
        {
            self.injected = true;
            self.party.extend(Box::new([parties::Act::Request {
                key: [2; 16],
                ask: parties::Ask::Say {
                    task: parties::Task::Last,
                    words: b"Continue now".as_slice().into(),
                    release: None,
                },
            }]));
        }
    }
    fn inject_refusal(&mut self, procs: &mut [Proc]) {
        if let crate::stories::Story::Refusals(reason) = self.story {
            if matches!(reason, crate::stories::MessageRefusal::Replay)
                && self.views.borrow().refusal_message_submissions.is_none()
                && worker_wire(procs)
                    .frames
                    .iter()
                    .any(|(_, kind, _)| *kind == jig_protocol_worker_wire::Kind::Message.number())
                && procs.iter().any(|proc| {
                    matches!(proc, Proc::Engine(host)
                    if host.wire.frames.iter().any(|(_, kind, _)| {
                        *kind == jig_protocol_worker_wire::Kind::Message.number()
                    }))
                })
            {
                self.views.borrow_mut().refusal_message_submissions = procs
                    .iter()
                    .find_map(|proc| if let Proc::Engine(host) = proc { Some(host.submitted) } else { None });
            }
            let ending = matches!(reason, crate::stories::MessageRefusal::Ending);
            if ending && !self.injected {
                for proc in procs.iter_mut() {
                    if let Proc::Worker(host) = proc {
                        host.relay.until = Some(Time::from_nanos(1_000_000_000));
                        host.relay.wait_only = true;
                    }
                }
            }
            let ready = procs.iter().any(|proc| matches!(proc, Proc::Engine(host) if host.wire.frames.iter().any(|(_, kind, _)| *kind == if ending { 0x0101 } else { 0x0180 })));
            if !self.injected && ready {
                self.injected = true;
                let count = 1;
                self.party.extend(
                    (0..count)
                        .map(|at| parties::Act::Request {
                            key: [at + 2; 16],
                            ask: parties::Ask::Say {
                                task: parties::Task::Last,
                                words: if matches!(
                                    reason,
                                    crate::stories::MessageRefusal::TooLarge | crate::stories::MessageRefusal::Replay
                                ) {
                                    vec![b'x'; 256].into()
                                } else {
                                    b"From the person".as_slice().into()
                                },
                                release: None,
                            },
                        })
                        .collect(),
                );
            }
        }
    }
}

impl Referee {
    fn drain_engine(&mut self, procs: &mut [Proc]) {
        if !matches!(self.story, crate::stories::Story::Drain | crate::stories::Story::Providers) {
            return;
        }
        let grants = worker_wire(procs).grants.clone();
        for proc in procs {
            if let Proc::Engine(host) = proc {
                if !self.injected && crate::world::pending_turn(&self.store.borrow()) {
                    self.injected = true;
                    self.views.borrow_mut().providers_before = crate::world::provider_choices(&self.store.borrow());
                    host.service.shutdown();
                }
                if self.injected && !self.views.borrow().engine_drained && host.service.is_empty() {
                    use std::fmt::Write;
                    assert_eq!(host.service.end(), Some(&jig_test_service::End::Drained));
                    let mut views = self.views.borrow_mut();
                    views.engine_drained = true;
                    views.commits_waiting_at_drain = self.store.borrow().pending.len();
                    assert_eq!(views.commits_waiting_at_drain, 0);
                    self.formatted.clear();
                    write!(&mut self.formatted, "{:?}", Roots(&host.service)).expect("root state");
                    self.rules.roots(&self.formatted, &grants.iter().map(AsRef::as_ref).collect::<Vec<_>>());
                    assert_eq!(self.rules.verdict(), skein_world::domain::Verdict::Passed);
                    host.service = jig_test_service::startup(
                        &if matches!(self.story, crate::stories::Story::Providers) {
                            crate::fixtures::providers_document()
                        } else {
                            crate::fixtures::engine_document()
                        },
                        &crate::fixtures::files(false),
                        None,
                        &crate::fixtures::seeds(),
                        None,
                        None,
                    )
                    .expect("restart after drained");
                    let root = host.restart_root.take().expect("independent directory for the drained restart");
                    host.service.adopt_store(root, 1000).expect("restart store directory");
                    host.files.root(root);
                    host.revision.0 += 1_000_000;
                }
            }
        }
    }
}

/// Check every surviving host's roots once more after teardown has settled.
pub(crate) fn roots_at_end(procs: &[Proc]) {
    let grants: Vec<_> = worker_wire(procs).grants.iter().map(AsRef::as_ref).collect();
    let mut rules = Rules::default();
    for proc in procs {
        if let Proc::Engine(host) | Proc::Worker(host) = proc {
            rules.roots(&format!("{:?}", Roots(&host.service)), &grants);
        }
    }
    assert_eq!(rules.verdict(), skein_world::domain::Verdict::Passed);
}

impl Referee {
    fn provisional_committed(&self) -> bool {
        (!matches!(self.story, crate::stories::Story::Facts) || self.views.borrow().reply.committed)
            && (!matches!(self.story, crate::stories::Story::Input) || self.views.borrow().input.committed)
    }
    fn input_story(&mut self, procs: &mut [Proc]) {
        if matches!(self.story, crate::stories::Story::Input) && !self.injected
            && procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if smith_agent_process_world::fake::queries(peer).next().is_some())) {
            self.injected=true;
            crate::live_input::Input::watch(procs);
        }
        if matches!(self.story, crate::stories::Story::Input) && self.injected {
            self.views.borrow_mut().input.act(procs, &self.store.borrow());
        }
    }
}

fn transcript_recovered(worker: &crate::processes::Wire, store: &Store) -> bool {
    use jig_protocol_worker_wire::{Assignment, CEILINGS};
    if worker.acknowledgements.len() != 2 {
        return false;
    }
    let assignments: Vec<_> = worker
        .frames
        .iter()
        .filter(|(_, kind, _)| *kind == 0x0180)
        .map(|(_, _, bytes)| Assignment::decode(&CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("assignment"))
        .collect();
    assert_eq!(assignments.len(), 3, "one fresh retry follows the refused resume");
    let fresh = assignments.last().expect("fresh retry");
    assert_eq!((fresh.task(), fresh.attempt()), (1, 3));
    assert!(fresh.transcript().is_empty(), "the retry resumes nothing");
    let message = fresh
        .messages()
        .iter()
        .find(|message| message.words() == b"Continue now")
        .expect("the refused resume retains the pending person's message");
    assert!(
        turns(store).iter().any(|turn| turn.attempt == 3 && turn.read == Some(message.name())),
        "the fresh agent takes the pending person's message"
    );
    assert!(
        terminal(store, 1, 2, crate::stories::Story::InvalidTranscript),
        "the refused resume commits a transient task failure"
    );
    assert!(
        terminal(store, 1, 3, crate::stories::Story::InvalidTranscript),
        "the fresh retry commits its parked terminal"
    );
    true
}
