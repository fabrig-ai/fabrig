//! One root input per pass, with completions passing held new work (protocol/README.md, 4).
mod diagnostics;
use crate::{CoreWait, End, route, stop, store::route::Up};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Wall};
use temper_engine_domain as root;

// A pass keeps the shipped small profile's 22 release/resume cycles.
const RELEASE_CYCLES: u32 = 22;

#[expect(
    clippy::struct_excessive_bools,
    reason = "fact-drain readiness is independent of start, release and input admission"
)]
#[expect(missing_debug_implementations, reason = "the engine owns account token records")]
pub struct Engine {
    pub(crate) domain: root::Domain,
    pub(crate) upgrade: Option<crate::upgrade::State>,
    pub(crate) env: Env<root::Limits>,
    pub(crate) store: crate::store::Store,
    pub(crate) accounts: Option<crate::accounts::Accounts>,
    pub(crate) workers: crate::workers::Workers,
    pub(crate) clients: crate::clients::Clients,
    pub(crate) demo: Option<crate::settings::Demo>,
    pub(crate) box_link: route::box_link::Link,
    pub(crate) setup: crate::setup::Layer,
    pub(crate) forge: crate::forge::Layer,
    pub(crate) forge_records: crate::forge::records::Loader,
    pub(crate) admission: Option<skein_lib::Token>,
    pub(crate) facts: u64,
    facts_waiting: bool,
    pub(crate) values: crate::values::Values,
    pub(crate) events: Queue<root::Event>,
    pub(crate) requests: Queue<root::Request>,
    pub(crate) end: Option<End>,
    pub(crate) listening: Queue<crate::Note>,
    pub(crate) notes: Queue<crate::Note>,
    pub(crate) note_bound: u32,
    pub(crate) last_hello_refusal: Option<Box<[u8]>>,
    pub(crate) first_periods: List<(u32, u64)>,
    pub(crate) drain: Option<stop::Drain>,
    pub(crate) blocked: bool,
    pub(crate) pending: Option<Box<root::Event>>,
    // A refused completion that tried to pass pending new work. It goes first.
    passing: Option<Box<root::Event>>,
    begin: bool,
    core_wait: crate::core_wait::Watch,
    pub(crate) credentials: List<u32>,
    pub(crate) ready: bool,
    settled_revision: Option<u64>,
    aborted: Option<()>,
    #[cfg(test)]
    pub(crate) release_trace: Option<ReleaseTrace>,
}

/// Test observations of the release/dispatch order (skein/docs/foundation/programming-model.md, 7.2).
#[cfg(test)]
pub(crate) enum ReleaseObservation {
    Released { ordinal: u64, requests: List<Box<[u8]>>, waiting_answers: u32 },
    Dispatched { ordinal: Option<u64>, request: Box<[u8]> },
}

#[cfg(test)]
pub(crate) struct ReleaseTrace {
    pub(crate) observations: Queue<ReleaseObservation>,
    pub(crate) inputs: Queue<root::Event>,
    pub(crate) cycles: u32,
    ordinal: u64,
    pending: Queue<u64>,
}

#[cfg(test)]
impl ReleaseTrace {
    pub(crate) fn new() -> Self {
        Self {
            observations: Queue::uncapped(),
            inputs: Queue::uncapped(),
            cycles: 0,
            ordinal: 0,
            pending: Queue::uncapped(),
        }
    }

    fn released(&mut self, requests: &Queue<root::Request>, before: u32, waiting_answers: u32) {
        self.ordinal = self.ordinal.checked_add(1).expect("test release ordinals");
        let mut batch = List::with_capacity(requests.len().checked_sub(before).expect("release appends requests"));
        for request in requests.iter().skip(usize::try_from(before).expect("test queue length")) {
            batch.push(release_snapshot(request)).expect("released batch length");
            self.pending.push(self.ordinal);
        }
        self.observations.push(ReleaseObservation::Released {
            ordinal: self.ordinal,
            requests: batch,
            waiting_answers,
        });
    }

    fn dispatched(&mut self, request: &root::Request) {
        self.observations
            .push(ReleaseObservation::Dispatched { ordinal: self.pending.pop(), request: release_snapshot(request) });
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    reason = "test-only observation copies the request's Debug representation without cloning or changing its ownership"
)]
fn release_snapshot(request: &root::Request) -> Box<[u8]> {
    std::format!("{request:?}").into_bytes().into_boxed_slice()
}

pub(crate) struct Children {
    pub(crate) accounts: Option<crate::accounts::Accounts>,
    pub(crate) workers: crate::workers::Workers,
    pub(crate) clients: crate::clients::Clients,
    pub(crate) demo: Option<crate::settings::Demo>,
    pub(crate) setup: crate::setup::Layer,
    pub(crate) forge: crate::forge::Layer,
    pub(crate) limits: crate::limits::Limits,
    pub(crate) notes: Queue<crate::Note>,
}
impl Engine {
    /// Membership errors are unreachable after `startup::checks::root`; keep the
    /// typed refusal for callers constructing an engine directly.
    pub(crate) fn new(
        config: root::Config,
        limits: &root::Limits,
        mut store: crate::store::Store,
        children: Children,
        values: crate::values::Values,
        first_periods: List<(u32, u64)>,
    ) -> Result<Self, jig_protocol_config::Problems> {
        let forge_records = crate::forge::records::Loader::new(&children.limits);
        let Children { accounts, workers, mut clients, demo, forge, setup, notes, limits: own } = children;
        clients.query_bytes = own.routes.query_bytes;
        let credentials = values.identities.numbers();
        store.routes.patch_context().executors.clone_from(&config.core.defaults.rules.ceiling.delegation.kinds);
        let mut domain = match root::Domain::new(config, limits) {
            Ok(domain) => domain,
            Err(problem) => {
                let (path, what) = crate::startup::checks::membership(problem);
                let mut problems = jig_protocol_config::Problems::empty();
                problems.push(path, what);
                return Err(problems);
            }
        };
        domain.connection_limits(own.forge.forges, own.forge_component.installations);
        let mut engine = Self {
            upgrade: None,
            credentials,
            domain,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            store,
            accounts,
            workers,
            clients,
            demo,
            forge,
            setup,
            box_link: route::box_link::Link::new(0, false),
            forge_records,
            admission: None,
            facts: 0,
            facts_waiting: false,
            values,
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            end: None,
            listening: Queue::with_capacity(3),
            note_bound: notes.capacity(),
            notes,
            last_hello_refusal: None,
            first_periods,
            drain: None,
            blocked: false,
            pending: None,
            passing: None,
            begin: true,
            core_wait: crate::core_wait::Watch::new(),
            ready: true,
            settled_revision: None,
            aborted: None,
            #[cfg(test)]
            release_trace: None,
        };
        let mut documents =
            core::mem::replace(&mut engine.workers.forges, crate::forges::Forges::with_capacity(own.forge.forges));
        for number in &documents.numbers() {
            let forge = documents.remove(*number).expect("document forge");
            crate::forges::keep(&mut engine, forge);
        }
        Ok(engine)
    }
    pub(crate) fn room(&self) -> bool {
        self.clients.commit_room()
    }
    fn release_pending(&self) -> bool {
        self.end.is_none()
            && (!self.requests.is_empty() || self.ready || self.settled_revision != Some(self.domain.revision()))
    }
    pub(crate) fn start_runtime(&self) -> bool {
        use jig_domain_core::start::Phase;
        match self.domain.start_phase() {
            Some(Phase::AdoptRuns | Phase::ReadAfresh { .. } | Phase::SettleOutbox { .. } | Phase::Open) => true,
            Some(
                Phase::LoadCore
                | Phase::LoadInbox
                | Phase::RestoreConnector { .. }
                | Phase::Link
                | Phase::Sweep
                | Phase::Switch
                | Phase::Wires,
            ) => false,
            None => !self.begin,
        }
    }
    pub(crate) fn step(&mut self, event: root::Event) {
        let watcher = watcher(&event);
        let party = party_input(&event);
        if let Err(event) = root::try_step(&mut self.domain, &self.env, event) {
            if self.pending.is_some() {
                assert!(
                    completes_pending(self) == Some(false) && root::completes(&event),
                    "only completions pass held new work"
                );
                assert!(self.passing.is_none(), "nothing passes a held completion");
                self.passing = Some(Box::new(event));
            } else {
                self.pending = Some(Box::new(event));
            }
            self.blocked = true;
            if party {
                self.observe_party(false);
            }
        } else {
            self.admission = watcher;
            if party {
                self.observe_party(true);
            }
        }
        self.drain_notes();
    }
    fn holding(&self) -> Option<CoreWait> {
        if self.begin || !self.domain.ready() {
            Some(CoreWait::Starting { step: self.domain.start_phase() })
        } else if !self.forge_records.done() {
            Some(CoreWait::ForgeRecords)
        } else if !self.first_periods.is_empty() {
            Some(CoreWait::FirstPeriods)
        } else if !self.forge.resolving.is_empty() {
            Some(CoreWait::Resolving)
        } else if self.domain.commits_in_flight() >= u64::from(self.env.limits.journal.commits) {
            Some(CoreWait::Store {
                oldest: self.domain.oldest_commit_in_flight().expect("a full commit window is nonempty"),
                in_flight: self.domain.commits_in_flight(),
            })
        } else if self.domain.backlog() >= u64::from(self.env.limits.watermark) {
            Some(CoreWait::Watermark { backlog: self.domain.backlog(), watermark: self.env.limits.watermark })
        } else {
            None
        }
    }
    fn observe_party(&mut self, entered: bool) {
        if self.notes.room() == 0 {
            return;
        }
        let note = if entered {
            self.core_wait.passed()
        } else {
            match self.holding() {
                Some(why) => self.core_wait.kept(self.env.now, why),
                None => None,
            }
        };
        if let Some(note) = note {
            self.notes.push(note);
        }
    }
    fn can_pass(&self) -> bool {
        self.passing.is_none()
            && completes_pending(self) == Some(false)
            && match self.events.iter().next() {
                Some(event) => root::completes(event),
                None => false,
            }
    }
    pub(crate) fn drain_worker_notes(&mut self) {
        for _ in 0..self.notes.room() {
            let Some(note) = self.workers.notes.pop() else { break };
            self.notes.push(note);
        }
    }
    fn drain_notes(&mut self) {
        self.drain_worker_notes();
        for _ in 0..self.notes.room() {
            let note = if let Some(diagnostic) = self.domain.pop_run_diagnostic() {
                crate::Note::RunFailed(diagnostic)
            } else if let Some(diagnostic) = self.domain.pop_run_refusal() {
                crate::Note::RunRefused(diagnostic)
            } else if let Some(diagnostic) = self.domain.pop_job_diagnostic() {
                crate::Note::JobAnswered(diagnostic)
            } else {
                break;
            };
            self.notes.push(note);
        }
        self.facts_waiting = false;
        for _ in 0..root::FACTS {
            if self.notes.room() == 0 {
                self.facts_waiting = true;
                break;
            }
            let Some(fact) = self.domain.pop_fact() else { break };
            match fact {
                root::Fact::ForgeLink { forge, note } => {
                    self.notes.push(crate::Note::ForgeLink { forge, note });
                }
                root::Fact::ReadRefused { task, attempt, tool, layer, reason, findings, error, job } => {
                    self.notes.push(crate::Note::ReadRefused {
                        task,
                        attempt,
                        tool,
                        layer,
                        reason,
                        findings,
                        error,
                        job,
                    });
                }
                root::Fact::BriefReadRefused { task, section, stage, reason, heads, error } => {
                    self.notes.push(crate::Note::BriefReadRefused { task, section, stage, reason, heads, error });
                }
                root::Fact::HeadRefused { task, words } => {
                    self.notes.push(crate::Note::HeadRefused { task, words });
                }
                root::Fact::BriefFailed { task, counts, reason } => {
                    self.notes.push(crate::Note::BriefFailed { task, counts, reason });
                }
                root::Fact::CallAdmissionRefused { task, attempt, tool, layer, reason } => {
                    self.notes.push(crate::Note::CallAdmissionRefused { task, attempt, tool, layer, reason });
                }
                root::Fact::PreparationFailed { .. } | root::Fact::StepRefused { .. } => {}
            }
            self.facts = self.facts.saturating_add(1);
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = skein_lib::earliest(self.store.host.next_deadline(), self.workers.host.next_deadline());
        if self.end.is_none() {
            next = skein_lib::earliest(next, self.domain.next_deadline(&self.env));
            next = skein_lib::earliest(next, self.values.accounts.next_expiry());
        }
        next = skein_lib::earliest(next, self.setup.next_deadline());
        next = skein_lib::earliest(next, self.clients.next_deadline());
        next = skein_lib::earliest(next, self.forge.component.next_deadline());
        skein_lib::earliest(
            next,
            match &self.accounts {
                Some(accounts) => accounts.next_deadline(),
                None => None,
            },
        )
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.pending_with_room(now, self.notes.room())
    }
    /// Read readiness using the notes still held by the joined drain.
    #[must_use]
    pub fn work_pending_with_notes(&self, now: Time, held: u32) -> bool {
        self.pending_with_room(now, self.note_bound.saturating_sub(held).saturating_sub(self.notes.len()))
    }
    fn pending_with_room(&self, now: Time, notes_room: u32) -> bool {
        let effects = self.store.host.mark_durable() || self.end.is_some();
        let secrets_ready = self.end.is_none() && self.store.host.mark_durable();
        let components = (self.store.opened || self.end.is_some())
            && self.clients.work_pending_with_secrets(now, secrets_ready)
            || effects && self.setup.work_pending(now)
            || effects && self.setup.lifecycle_ready(self) && self.events.room() >= 4
            || self.store.work_pending(now)
            || effects && self.forge.work_pending(now)
            || (self.store.opened || self.end.is_some()) && self.workers.host.work_pending(now)
            || !self.workers.notes.is_empty() && notes_room > 0
            || effects
                && match &self.accounts {
                    Some(accounts) => accounts.work_pending(now, true),
                    None => false,
                };
        if self.end.is_some() || !self.store.opened {
            return components;
        }
        let due = match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        };
        components
            || (self.facts_waiting && notes_room > 0)
            || self.room()
                && (self.begin
                    || self.release_pending()
                    || self.start_runtime()
                        && (!self.credentials.is_empty()
                            || self.can_pass()
                            || !self.blocked
                                && (self.passing.is_some()
                                    || self.pending.is_some()
                                    || !self.events.is_empty()
                                    || due
                                    || self.domain.ready() && !self.first_periods.is_empty())))
    }
    pub fn abort_if_ended(&mut self) {
        if self.end.is_some() {
            // An ended owner owes no root continuation. Late component terminals
            // still close their IO, but cannot release another domain decision.
            self.ready = false;
            self.settled_revision = Some(self.domain.revision());
            self.pending = None;
            self.passing = None;
            for _ in 0..self.events.len() {
                let _event = self.events.pop();
            }
            for _ in 0..self.requests.len() {
                let _request = self.requests.pop();
            }
            self.setup.close();
        }
        if self.end.is_some() {
            self.forge.down(skein_forge_connection::Request::Close);
        }
        if let Some(End::Aborted | End::Stopped(_)) = self.end
            && self.aborted.is_none()
        {
            self.aborted = Some(());
            self.store.close();
            self.clients.abort(!self.store.opened);
            self.workers.abort(self.env.now, &mut self.events);
            if let Some(accounts) = &mut self.accounts {
                accounts.abort();
            }
        }
    }
}

/// Every child sees this iteration's time before an input can arm a deadline.
pub(crate) fn clock(engine: &mut Engine, now: Time, wall: Wall) {
    engine.drain_worker_notes();
    if engine.facts_waiting && engine.notes.room() > 0 {
        engine.drain_notes();
    }
    engine.env.now = now;
    engine.env.wall = wall;
    engine.store.host.clock(now, wall);
    if let Some(accounts) = &mut engine.accounts {
        accounts.clock(now, wall);
    }
    engine.clients.clock(now, wall);
    engine.forge.env.now = now;
    engine.forge.env.wall = wall;
    engine.setup.env.now = now;
    engine.setup.env.wall = wall;
}

pub(crate) fn iterate(engine: &mut Engine, now: Time, wall: Wall) {
    clock(engine, now, wall);
    engine.store.iterate();
    answer(engine);
    if crate::upgrade::advance(engine) {
        engine.abort_if_ended();
        return;
    }
    if !engine.store.opened && engine.end.is_none() {
        return;
    }
    if let Some(accounts) = &mut engine.accounts {
        crate::accounts::iterate(accounts, now, wall, &mut engine.values, &mut engine.events, &mut engine.store);
    }
    crate::forge::gone::advance(engine);
    crate::forge::records::advance(engine);
    if engine.store.host.mark_durable()
        && (!engine.forge_records.loading() || engine.end.is_some() || stop::requested(engine))
    {
        engine.forge.iterate(now, wall);
    }
    // Reconcile must observe the root's restored repositories.
    if !engine.begin && engine.store.host.mark_durable() && engine.forge_records.done() {
        crate::setup::lifecycle::restored(engine);
    }
    if engine.store.host.mark_durable() {
        crate::setup::advance(engine);
    }
    forge_pass(engine);
    engine.workers.iterate(now, &mut engine.events);
    engine.drain_worker_notes();
    let open = !engine.begin
        && engine.domain.ready()
        && engine.forge_records.done()
        && engine.first_periods.is_empty()
        && engine.forge.resolving.is_empty();
    if open {
        engine.forge.serving = Some(());
    }
    engine.clients.playground = engine.domain.playground().copied();
    engine.clients.starting = if open { None } else { Some(()) };
    engine.clients.iterate(
        now,
        wall,
        &mut engine.events,
        &engine.workers.forges,
        engine.demo.as_ref(),
        &mut engine.accounts,
    );
    clients_pass(engine);
    for _ in 0..engine.clients.early.len() {
        if engine.events.room() == 0 {
            break;
        }
        let event = engine.clients.early.pop().expect("staged client input");
        if open {
            engine.events.push(event);
        } else if let root::Event::Party(value) = event {
            engine.observe_party(false);
            engine.clients.not_ready(value);
        }
    }
    stop::advance(engine);
    if engine.end.is_some() {
        engine.abort_if_ended();
        return;
    }
    if !engine.room() {
        return;
    }
    // Public restore runs before secrets. Once marked, finish secret bootstrap
    // before releasing the script into adoption and fresh forge reads.
    if engine.store.host.mark_durable() && !engine.forge_records.done() && !stop::requested(engine) {
        return;
    }
    input_pass(engine, now);
    if engine.blocked
        && let Some(event) = engine.pending.as_deref()
        && party_input(event)
    {
        engine.observe_party(false);
    }
    release(engine);
    if crate::upgrade::advance(engine) {
        engine.abort_if_ended();
        return;
    }
    engine.forge.down(skein_forge_connection::Request::Offer);
    clients_pass(engine);
}
fn input_pass(engine: &mut Engine, now: Time) {
    if engine.begin {
        engine.begin = false;
        engine.step(root::Event::Restart { pending: engine.store.pending_generation });
    } else if !engine.start_runtime() {
        // Store answers and held start continuations advance through release.
        // Operational inputs wait until the Wires continuation has resumed.
    } else if !engine.credentials.is_empty() {
        let last = engine.credentials.len().checked_sub(1).expect("first credential");
        let number = *engine.credentials.get(last).expect("configured identity");
        engine.credentials.truncate(last);
        engine.step(root::Event::Credential {
            number,
            generation: 1,
            valid: skein_lib::Duration::from_nanos(u64::MAX),
        });
    } else if !engine.blocked && engine.passing.is_some() {
        let event = engine.passing.take().expect("held passing completion");
        engine.step(*event);
    } else if engine.can_pass() {
        let event = engine.events.pop().expect("next completion");
        engine.step(event);
    } else if !engine.blocked && engine.pending.is_some() {
        let event = engine.pending.take().expect("held input");
        engine.step(*event);
    } else if !engine.blocked && engine.domain.ready() && !engine.first_periods.is_empty() {
        let last = engine.first_periods.len().checked_sub(1).expect("first period");
        let (project, budget) = *engine.first_periods.get(last).expect("configured project");
        engine.first_periods.truncate(last);
        engine.step(root::Event::Party(root::PartyInput::Period { project, period: 1, budget, next: None }));
    } else if !engine.blocked && !engine.events.is_empty() {
        let event = engine.events.pop().expect("ready input");
        engine.step(event);
    } else if !engine.blocked
        && let Some(deadline) = engine.domain.next_deadline(&engine.env)
        && deadline <= now
    {
        root::fire(&mut engine.domain, &engine.env);
        engine.drain_notes();
    }
}

fn completes_pending(engine: &Engine) -> Option<bool> {
    match engine.pending.as_deref() {
        Some(event) => Some(root::completes(event)),
        None => None,
    }
}
fn clients_pass(engine: &mut Engine) {
    engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
    if engine.end.is_none() && engine.store.host.mark_durable() {
        engine.clients.secrets_step(&mut engine.store);
    }
    engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
    if engine.clients.take_linked_changed() {
        engine.domain.mark_forges();
    }
}
pub(crate) fn release(engine: &mut Engine) {
    if engine.end.is_none() && engine.room() && !dispatch(engine) {
        return;
    }
    // Clocks are inputs too: Resume can observe a due timer even when normal
    // input admission is blocked. Recompute wall-to-monotonic mapping now.
    if !engine.release_pending() {
        let due = match engine.domain.next_deadline(&engine.env) {
            Some(deadline) => deadline <= engine.env.now,
            None => false,
        };
        if !due {
            // No root mutation since its last reclaim means no newly retired slots.
            // Child IO may still have closed, so shutdown must advance on this pass.
            stop::advance(engine);
            return;
        }
    }
    engine.ready = true;
    let mut empty = 0_u32;
    for _ in 0..RELEASE_CYCLES {
        if !engine.room() || engine.end.is_some() {
            break;
        }
        // Route every request from the previous batch before releasing again.
        assert!(engine.requests.is_empty(), "release follows routing of the previous batch");
        #[cfg(test)]
        if let Some(trace) = &mut engine.release_trace {
            trace.cycles = trace.cycles.checked_add(1).expect("test release cycles");
        }
        #[cfg(test)]
        let before = engine.requests.len();
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        #[cfg(test)]
        if let Some(trace) = &mut engine.release_trace {
            trace.released(&engine.requests, before, engine.store.host.above.len());
        }
        if engine.requests.is_empty() {
            root::try_step(&mut engine.domain, &engine.env, root::Event::Resume).expect("owner continuation admitted");
            engine.drain_notes();
            #[cfg(test)]
            let before = engine.requests.len();
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
            #[cfg(test)]
            if let Some(trace) = &mut engine.release_trace {
                trace.released(&engine.requests, before, engine.store.host.above.len());
            }
        }
        engine.drain_notes();
        if !engine.requests.is_empty() || engine.domain.quiescent() {
            engine.blocked = false;
        }
        if engine.requests.is_empty() {
            empty = empty.checked_add(1).expect("two empty passes");
        } else {
            empty = 0;
        }
        if !dispatch(engine) {
            break;
        }
        if empty == 2 {
            engine.ready = false;
            break;
        }
    }
    if let Some(watcher) = engine.admission.take() {
        engine.clients.down(jig_protocol_engine_clients::Event::Admitted { watcher });
    }
    engine.domain.reclaim();
    if !engine.ready {
        engine.settled_revision = Some(engine.domain.revision());
    }
    stop::advance(engine);
}

/// A full diagnostic sink leaves the batch ahead of the next release (protocol/cluster.md, 3).
fn dispatch(engine: &mut Engine) -> bool {
    for _ in 0..engine.requests.len() {
        if engine.notes.room() == 0 {
            return false;
        }
        let request = engine.requests.pop().expect("root outputs");
        #[cfg(test)]
        if let Some(trace) = &mut engine.release_trace {
            trace.dispatched(&request);
        }
        route::request(engine, request);
        // A scripted neighbour keeps the real root productive across release cycles.
        #[cfg(test)]
        if let Some(trace) = &mut engine.release_trace
            && let Some(input) = trace.inputs.pop()
        {
            engine.step(input);
        }
        if engine.end.is_some() {
            break;
        }
    }
    true
}

#[expect(clippy::too_many_lines, reason = "each store terminal routes to its owner, including upgrade progress")]
pub(crate) fn answer(engine: &mut Engine) {
    if engine.end.is_some() {
        for _ in 0..engine.store.host.above.len() {
            let _late = engine.store.host.above.pop();
        }
        for _ in 0..engine.store.routed.len() {
            let _late = engine.store.routed.pop();
        }
        return;
    }
    for _ in 0..engine.store.host.above.len() {
        if engine.notes.room() == 0 {
            break;
        }
        let event = engine.store.host.above.pop().expect("store terminals");
        match event {
            jig_protocol_store::Event::Opened { installation, pending, .. } => {
                engine.store.opened = true;
                engine.store.pending_generation = pending;
                engine.store.installation = installation;
            }
            jig_protocol_store::Event::OpenRefused { why, .. } => open_refused(engine, why),
            jig_protocol_store::Event::Corrupt { .. } if !engine.store.opened => {
                // Opening has no load route; corruption is the open's terminal.
                if let Some(state) = &mut engine.upgrade {
                    state.report.open_failure = engine.store.host.component.start_open_failure().cloned();
                }
                stop::open(engine, &jig_protocol_store::OpenRefusal::Failed);
            }
            event @ (jig_protocol_store::Event::Decoded { .. }
            | jig_protocol_store::Event::Undecodable { .. }
            | jig_protocol_store::Event::Rehearsing { .. }
            | jig_protocol_store::Event::Back { .. }
            | jig_protocol_store::Event::BackRefused { .. }
            | jig_protocol_store::Event::Closed { .. }) => {
                let state = engine.upgrade.as_mut().expect("upgrade store event");
                if let jig_protocol_store::Event::Undecodable { .. } = &event {
                    state.undecodable = true;
                }
                if let jig_protocol_store::Event::Closed { .. } = &event {
                    state.closed = true;
                }
                if let jig_protocol_store::Event::Back { .. } | jig_protocol_store::Event::BackRefused { .. } = &event {
                    state.back = true;
                }
                state.report.events.push(event);
            }
            jig_protocol_store::Event::Patched { owner, of, patch, counts } => {
                if let Some(state) = &mut engine.upgrade {
                    state.report.events.push(jig_protocol_store::Event::Patched { owner, of, patch, counts });
                }
                engine.notes.try_push(crate::Note::Patched { of, patch, counts }).expect("patch progress note room");
            }
            event @ jig_protocol_store::Event::Site { .. } => {
                crate::store::route::site(
                    &mut engine.store.routes,
                    &mut engine.store.host.component,
                    event,
                    &mut engine.store.routed,
                );
            }
            jig_protocol_store::Event::Erased { owner } if crate::forge::gone::owns(owner) => {
                crate::forge::gone::answer(engine, owner);
            }
            jig_protocol_store::Event::Corrupt { owner } if crate::forge::gone::owns(owner) => {
                stop::unavailable(engine, b"forge secrets erase");
            }
            jig_protocol_store::Event::Corrupt { owner } if owner == crate::accounts::WRONG_KIND => {
                stop::unavailable(engine, b"accounts secrets");
            }
            event @ (jig_protocol_store::Event::SecretsNotReady { owner }
            | jig_protocol_store::Event::Kept { owner }
            | jig_protocol_store::Event::Stale { owner }
            | jig_protocol_store::Event::Erased { owner }
            | jig_protocol_store::Event::Carried { owner, .. }
            | jig_protocol_store::Event::Secret { owner, .. }
            | jig_protocol_store::Event::SecretsLoaded { owner, .. }) => {
                if crate::forge::gone::owns(owner) {
                    stop::unavailable(engine, b"forge secrets erase");
                } else if owner == crate::forge::records::LOAD {
                    if let jig_protocol_store::Event::SecretsLoaded { rows, next, .. } = event {
                        engine.forge_records.loaded(rows, next);
                    } else if let jig_protocol_store::Event::Erased { .. } = event {
                        engine.forge_records.erased();
                    } else {
                        stop::unavailable(engine, b"forge records");
                    }
                } else if crate::tokens::prefix(owner) == 11 {
                    crate::setup::answer(engine, event);
                } else if crate::accounts::owns(owner) {
                    engine.store.host.sources.push(event);
                } else {
                    engine.store.host.secrets.push(event);
                }
            }
            event @ jig_protocol_store::Event::Switched { .. } => {
                engine.store.pending_generation = false;
                crate::store::route::up(&mut engine.store.routes, event, &mut engine.store.routed);
            }
            event @ (jig_protocol_store::Event::Committed { .. }
            | jig_protocol_store::Event::Failed { .. }
            | jig_protocol_store::Event::FrameTooLarge { .. }
            | jig_protocol_store::Event::Page { .. }
            | jig_protocol_store::Event::PageRefused { .. }
            | jig_protocol_store::Event::Loaded { .. }
            | jig_protocol_store::Event::Read { .. }
            | jig_protocol_store::Event::Got { .. }
            | jig_protocol_store::Event::Turns { .. }
            | jig_protocol_store::Event::Unread { .. }
            | jig_protocol_store::Event::Corrupt { .. }) => {
                if let jig_protocol_store::Event::Corrupt { owner } = &event
                    && crate::tokens::prefix(*owner) == 11
                {
                    crate::setup::answer(engine, event);
                } else if let jig_protocol_store::Event::Corrupt { owner } = &event
                    && *owner == crate::forge::records::LOAD
                {
                    stop::unavailable(engine, b"forge records");
                } else {
                    crate::store::route::up(&mut engine.store.routes, event, &mut engine.store.routed);
                }
            }
        }
        if engine.end.is_some() {
            return;
        }
    }
    answer_routed(engine);
}

fn answer_routed(engine: &mut Engine) {
    for _ in 0..engine.store.routed.len() {
        match engine.store.routed.pop().expect("store routing") {
            Up::Root(event) => {
                if let root::Event::Store(root::StoreInput::Committed { number, .. }) = event.as_ref() {
                    engine.clients.committed(*number);
                }
                if let root::Event::Store(root::StoreInput::Loaded { rows, .. }) = event.as_ref() {
                    for row in rows {
                        if row.key()
                            == root::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment))
                        {
                            engine.first_periods.truncate(0);
                        }
                    }
                }
                if let root::Event::Store(root::StoreInput::Uncommitted { number }) = *event {
                    stop::failed(engine, number);
                } else {
                    engine.blocked = false;
                    engine.step(*event);
                }
            }
            Up::Ask(request) => engine.store.host.ask(request),
            Up::FrameTooLarge { number, task, bytes, limit } => {
                stop::frame_too_large(engine, number, task, bytes, limit);
            }
        }
        if engine.end.is_some() {
            break;
        }
    }
}

fn open_refused(engine: &mut Engine, why: jig_protocol_store::OpenRefusal) {
    use jig_protocol_store::OpenRefusal;
    #[expect(clippy::match_same_arms, reason = "each store refusal keeps its own startup stop line")]
    match why {
        why @ OpenRefusal::Format { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Deployment => stop::open(engine, &why),
        why @ OpenRefusal::Numbers { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Failed => stop::open(engine, &why),
        why @ OpenRefusal::OlderBuild { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Generations { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Ahead { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Patch { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Set { .. } => stop::open(engine, &why),
        why @ OpenRefusal::InFlight { .. } => stop::open(engine, &why),
        why @ OpenRefusal::Check { .. } => stop::open(engine, &why),
    }
}

fn party_input(event: &root::Event) -> bool {
    match event {
        root::Event::Party(root::PartyInput::Period { .. })
        | root::Event::Worker(_)
        | root::Event::Store(_)
        | root::Event::Account(_)
        | root::Event::Forge(_)
        | root::Event::Released(_)
        | root::Event::Credential { .. }
        | root::Event::Setup { .. }
        | root::Event::Restart { .. }
        | root::Event::Stop
        | root::Event::Resume => false,
        root::Event::Party(_) => true,
    }
}

fn watcher(event: &root::Event) -> Option<skein_lib::Token> {
    match event {
        root::Event::Party(root::PartyInput::Watch { watcher, .. } | root::PartyInput::WatchRun { watcher, .. }) => {
            Some(*watcher)
        }
        root::Event::Party(_)
        | root::Event::Worker(_)
        | root::Event::Store(_)
        | root::Event::Account(_)
        | root::Event::Forge(_)
        | root::Event::Released(_)
        | root::Event::Credential { .. }
        | root::Event::Setup { .. }
        | root::Event::Restart { .. }
        | root::Event::Stop
        | root::Event::Resume => None,
    }
}

pub(crate) fn forge_pass(engine: &mut Engine) {
    for _ in 0..engine.forge.above.len() {
        if engine.notes.room() == 0 {
            break;
        }
        if engine.end.is_none()
            && (!engine.room() || engine.passing.is_some() || completes_pending(engine) == Some(true))
        {
            break;
        }
        match engine.forge.above.pop().expect("forge outputs") {
            skein_forge_connection::Event::Answered(answer) => forge_answer(engine, answer),
            skein_forge_connection::Event::Hint(hinted) => forge_hint(engine, hinted),
            skein_forge_connection::Event::Connected { forge } => crate::setup::lifecycle::connected(engine, forge),
            skein_forge_connection::Event::Installation { forge, installation, account, change } => {
                crate::setup::lifecycle::installation(engine, forge, installation, account, change);
            }
            skein_forge_connection::Event::GitToken { forge, installation, account, token, expires } => {
                crate::forge::tokens::token(engine, forge, installation, account, token, expires);
            }
            skein_forge_connection::Event::Fact { forge, fact: skein_forge_connection::Fact::Origin { origin } } => {
                crate::forge::origins::refused(engine, forge, origin);
            }
            skein_forge_connection::Event::Fact { fact: skein_forge_connection::Fact::Gap, .. } => {}
            skein_forge_connection::Event::Fact {
                forge,
                fact: skein_forge_connection::Fact::Unkept { installation },
            } => {
                if engine.notes.room() > 0 {
                    engine.notes.push(crate::Note::ForgeUnkept { number: forge, installation });
                }
            }
            skein_forge_connection::Event::Listening { addr } => {
                if let Some(addr) = addr.socket() {
                    engine.setup.webhooks_listening(addr);
                    let note = crate::Note::Listening { listener: crate::Listening::Webhooks, addr };
                    engine.listening.try_push(note).expect("one note per listener bind");
                }
            }
            skein_forge_connection::Event::ListenerLost => stop::hooks(engine),
            skein_forge_connection::Event::Stopped { forge, why } => forge_stopped(engine, forge, why),
            skein_forge_connection::Event::Closed => engine.forge.closed = true,
        }
    }
}

fn forge_answer(engine: &mut Engine, answer: skein_forge_connection::Answered) {
    if engine.end.is_some() {
        engine.forge.down(skein_forge_connection::Request::Returned(answer));
        return;
    }
    if let Err(root::Event::Forge(event)) = root::try_step(&mut engine.domain, &engine.env, crate::forge::event(answer))
    {
        engine.forge.down(skein_forge_connection::Request::Refused(
            crate::forge::back(event).expect("the routing hands the root only the component's answers"),
        ));
    }
    engine.drain_notes();
}
fn forge_hint(engine: &mut Engine, hinted: skein_forge_connection::hooks::Hinted) {
    if engine.drain.is_some() || engine.end.is_some() || engine.domain.forge_ended(hinted.hint.repository.forge) {
        return;
    }
    let connection = hinted.connection;
    if let Err(root::Event::Forge(event)) =
        root::try_step(&mut engine.domain, &engine.env, crate::forge::hinted(hinted))
    {
        let temper_engine_domain_forge::Event::Hint { hint } = *event else {
            unreachable!("hint retained by admission")
        };
        engine.forge.down(skein_forge_connection::Request::RefusedHint(skein_forge_connection::hooks::Hinted {
            connection,
            hint,
        }));
    }
    engine.drain_notes();
}
fn forge_stopped(engine: &mut Engine, forge: u16, why: skein_forge_connection::Stopped) {
    crate::forge::tokens::accepted(engine, forge, &[]);
    crate::setup::lifecycle::stopped_connection(engine, forge);
    crate::setup::lifecycle::state(
        engine,
        forge,
        temper_engine_domain_forge::Connection::Stopped { reason: crate::setup::lifecycle::stopped_text(why) },
    );
    if engine.notes.room() > 0 {
        engine.notes.push(crate::Note::ForgeStopped { number: forge, why });
    }
}

impl Engine {
    pub fn clock(&mut self, now: Time, wall: Wall) {
        clock(self, now, wall);
    }
    pub fn iterate(&mut self, now: Time, wall: Wall) {
        iterate(self, now, wall);
    }
}

#[cfg(test)]
mod release_tests {
    use super::*;

    #[test]
    fn an_empty_release_stays_settled_until_the_root_changes() {
        let mut service = crate::tests::fixtures::ready();
        let engine = service.engine.as_mut().expect("engine");
        engine.step(root::Event::Resume);
        assert!(engine.release_pending());
        release(engine);
        assert!(!engine.ready);
        assert!(!engine.release_pending());
        assert!(engine.requests.is_empty());
        release(engine);
        assert!(!engine.release_pending());
        assert_eq!(engine.settled_revision, Some(engine.domain.revision()));
        let revision = engine.domain.revision();
        root::step(&mut engine.domain, &engine.env, root::Event::Resume);
        assert!(engine.domain.revision() > revision);
        assert!(engine.release_pending(), "direct root inputs wake the owner too");
    }
    #[test]
    fn returned_joined_note_credit_wakes_waiting_facts() {
        let mut service = crate::tests::fixtures::ready();
        let engine = service.engine.as_mut().expect("engine");
        let now = Time::from_nanos(1);
        assert!(!engine.work_pending(now), "settled fixture");
        let held = engine.note_bound;
        engine.held_notes(held);
        engine.facts_waiting = true;
        assert!(!engine.work_pending_with_notes(now, held), "the joined sink is full");
        assert!(engine.work_pending_with_notes(now, 0), "returned credit wakes waiting facts before the next turn");
        assert!(engine.pending_with_notes(now, 0).work_pending, "the snapshot uses the same credit");
        engine.held_notes(0);
        assert!(engine.work_pending(now));
    }
}
