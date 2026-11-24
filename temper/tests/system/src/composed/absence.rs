//! Absences across the composed roots (temper/docs/design/domain/worker/README.md, 2 and 9).
//! The link's grace is scripted at the worker root; jig's worker-link world owns redials.
use super::{Protocol, RunWatch, Script, Time, VecDeque, World, engine, host, open_config, parties, tasks, worker};
use skein_fake_llm_domain::api::Query;
use skein_lib::Duration;

#[expect(
    clippy::struct_excessive_bools,
    reason = "each root and their link can be stopped independently of recording the story"
)]
pub(super) struct Controls {
    pub enabled: bool,
    pub engine_running: bool,
    pub worker_running: bool,
    pub link_joined: bool,
    lost: Option<Time>,
    grace: Duration,
    pub told: Option<Duration>,
    pub drained: bool,
    pub hellos: Vec<Vec<jig_domain_link::Hosting>>,
    admissions: usize,
    pub hello_pending: bool,
    pub outbound: VecDeque<engine::Link>,
    refused_starts: usize,
    watching: bool,
    assignments: Vec<engine::Assignment>,
    prompts: Vec<Query>,
    pub answers: Vec<(u64, u64, host::Ending)>,
    tries: Vec<tasks::Tries>,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            enabled: false,
            engine_running: true,
            worker_running: true,
            link_joined: true,
            lost: None,
            grace: Duration::from_secs(1),
            told: None,
            drained: false,
            hellos: Vec::new(),
            admissions: 0,
            hello_pending: false,
            outbound: VecDeque::new(),
            refused_starts: 0,
            watching: false,
            assignments: Vec::new(),
            prompts: Vec::new(),
            answers: Vec::new(),
            tries: Vec::new(),
        }
    }
}

impl World {
    /// Open a chat and hold Smith after its selected model query, before its response.
    #[must_use]
    pub fn absence_chat(seed: u64, script: Script, words: &[u8], query: usize) -> Self {
        let mut world = Self::open_chat(seed, script);
        world.absence.enabled = true;
        world.process.held_query = Some(query);
        world.party = super::signed_in_party(words);
        world
    }

    /// The deployment's long engine-away bound, with a model response held across it.
    #[must_use]
    pub fn absence_chat_with_grace(seed: u64, script: Script, words: &[u8], query: usize, grace: Duration) -> Self {
        let mut world = Self::absence_chat(seed, script, words, query);
        world.absence.grace = grace;
        world.worker_env.limits.stop_bound = grace.saturating_add(Duration::from_secs(26));
        world.worker_env.limits.agent.no_progress = world.env.limits.fleet.restart_wait;
        world.env.limits.fleet.grace = world.worker_env.limits.stop_bound.saturating_add(Duration::from_secs(1));
        world.env.limits.people.sign_in_lifetime = Duration::from_secs(3600);
        world.worker = worker::Domain::new(&world.worker_env.limits, Box::new([]));
        world.replace_root(open_config(seed));
        world
    }

    /// Await the selected query and all preceding durable turn acknowledgements.
    pub fn until_model_held(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if self.process.held_query.is_some_and(|query| {
                !self.process.prepared()
                    && self.agent().prompts().len() == query
                    && self.agent().turns().len() + 1 == query
                    && self.worker.retained_turns() == 0
            }) && self.store.idle()
                && self.root_events.is_empty()
                && self.worker_events.is_empty()
            {
                return;
            }
        }
        panic!("model response not held: {:?}; answers {:?}", self.observed, self.absence.answers);
    }

    /// Ask the process face to send the held response as one live reply, then its turn.
    pub fn watch_absence_reply(&mut self) {
        self.process.in_pieces(1);
        self.run_watch = Some(RunWatch::default());
        self.watch_absence_run();
    }

    fn watch_absence_run(&mut self) {
        if self.run_watch.is_some() && self.absence_watch_needed() {
            self.root_events.push_back(engine::Event::Party(engine::PartyInput::WatchRun {
                watcher: super::RUN_WATCH,
                sign_in: self.party.sign_in.expect("authenticated party"),
                task: self.assignment().task,
                turn: None,
                offset: 0,
            }));
        }
    }

    /// Continue to a later model query, holding its response.
    pub fn continue_until_model(&mut self, query: usize) {
        assert!(self.process.held_query.is_some(), "a held model response");
        self.process.held_query = Some(query);
    }

    pub fn release_model(&mut self) {
        assert!(self.process.held_query.take().is_some(), "a held model response");
    }

    /// Both roots hear the loss; undelivered link records no longer reach the peer.
    pub fn cut_link(&mut self) {
        assert!(self.absence.link_joined);
        self.root_step(engine::Event::Worker(engine::WorkerInput::Link {
            channel: super::CHANNEL,
            link: engine::Link::Lost,
        }));
        self.worker_events.push_back(worker::Event::Lost);
        self.absence.link_joined = false;
        self.absence.hello_pending = false;
        self.absence.outbound.clear();
        self.absence.lost.get_or_insert(self.env.now);
    }

    /// The worker says its hello at the same configured position. Admission, not Ready,
    /// stops the grace, and the engine adopts the listed attempts from its restored store.
    pub fn join_link(&mut self) {
        assert!(self.absence.engine_running && self.absence.worker_running);
        self.absence.link_joined = true;
        self.worker_events.push_back(worker::Event::Ready);
        let before = self.absence.admissions;
        for _ in 0..10000 {
            self.tick();
            if self.absence.admissions > before {
                self.watch_absence_run();
                return;
            }
        }
        panic!(
            "engine did not admit the worker hello: admissions {}, before {before}, hellos {:?}, ready {}, observed {:?}",
            self.absence.admissions,
            self.absence.hellos,
            self.root.ready(),
            self.observed
        );
    }

    /// Settle the engine and hear its word before the scripted link is cut.
    pub fn settle_engine(&mut self) {
        self.root_step(engine::Event::Stop);
        for _ in 0..10000 {
            self.tick();
            if self.absence.drained {
                return;
            }
        }
        panic!("engine did not drain");
    }

    /// Move the scripted clock during an absence; the next tick observes expired clocks.
    pub fn elapse_absence(&mut self, duration: Duration) {
        assert!(!self.absence.engine_running && !self.absence.link_joined);
        self.jump(duration);
        self.tick();
    }

    #[must_use]
    pub fn restart_word(&self) -> Option<Duration> {
        self.absence.told
    }

    /// Drop the engine's queues and crash its store, retaining the worker and disk.
    pub fn stop_engine(&mut self) {
        assert!(!self.absence.link_joined, "cut the link before stopping the engine");
        self.absence.engine_running = false;
        self.absence.watching = false;
        self.root_events.clear();
        self.root_out.clear();
        self.store.crash(self.seed);
        self.protocol = Protocol::default();
        // The stopped root owns no live waits; retain any finding on the world.
        self.checks = super::Checks::new(self.seed);
    }

    /// Reopen only the engine root on its store. No worker Ready is generated here.
    pub fn reopen_engine(&mut self) {
        self.reopen_with_config(open_config(self.seed));
    }

    pub(super) fn reopen_with_config(&mut self, config: engine::Config) {
        assert!(!self.absence.engine_running);
        self.replace_root(config);
        self.root_events = super::opening_events();
        self.absence.engine_running = true;
        self.absence.drained = false;
        self.opened = false;
        self.ends.restarted(self.env.now);
    }

    /// A root started and dropped before it can admit a hello is still one absence.
    pub fn refuse_engine_start(&mut self) {
        self.reopen_engine();
        self.ends.no_opening(self.env.now);
        self.stop_engine();
        self.absence.refused_starts += 1;
    }

    /// Cancel for shutdown, then wait for every run, answer and acknowledgement before exit.
    pub fn stop_worker(&mut self) {
        self.worker_events.push_back(worker::Event::Shutdown { drain: false });
        for _ in 0..10000 {
            self.tick();
            if !self.absence.worker_running {
                self.cut_link();
                self.worker_events.clear();
                self.worker_out.clear();
                return;
            }
        }
        panic!("worker did not stop: {:?}", self.observed);
    }

    /// Start one worker at its position over the same checkout directories.
    pub fn start_worker(&mut self) {
        assert!(!self.absence.worker_running);
        self.worker = worker::Domain::new(&self.worker_env.limits, Box::new([]));
        self.absence.worker_running = true;
        self.join_link();
    }

    /// Wait for the cancelled process to be reaped while its worker retains the answer.
    pub fn until_worker_keeps_answer(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if self.worker.kept_answers() == 1 && self.observed.gone == 1 {
                return;
            }
        }
        panic!("worker did not retain its cancelled answer: {:?}", self.observed);
    }

    /// Close through the authenticated party and await the durable task ending.
    pub fn close_absence_chat(&mut self) {
        self.act(parties::Ask::Cancel { task: parties::Task::Last, grace: None }, [42; 16]);
        let ended = super::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(self.assignment().task)));
        for _ in 0..10000 {
            self.tick();
            if self.store.rows.contains_key(&ended)
                && self.party.quiescent()
                && self.root.quiescent()
                && self.worker_events.is_empty()
                && self.root_events.is_empty()
                && self.store.idle()
            {
                return;
            }
        }
        panic!("the closed chat did not end");
    }

    #[must_use]
    pub fn engine_away_grace(&self) -> Duration {
        self.absence.grace
    }

    #[must_use]
    pub fn absence_assignments(&self) -> &[engine::Assignment] {
        &self.absence.assignments
    }

    /// Prompts seen by Smith's worlds across all processes and attempts.
    #[must_use]
    pub fn absence_prompts(&self) -> Vec<&Query> {
        self.absence.prompts.iter().chain(self.agent().prompts()).collect()
    }

    #[must_use]
    pub fn absence_hellos(&self) -> &[Vec<jig_domain_link::Hosting>] {
        &self.absence.hellos
    }

    #[must_use]
    pub fn absence_admissions(&self) -> usize {
        self.absence.admissions
    }

    #[must_use]
    pub fn refused_engine_starts(&self) -> usize {
        self.absence.refused_starts
    }

    #[must_use]
    pub fn absence_answers(&self) -> &[(u64, u64, host::Ending)] {
        &self.absence.answers
    }

    /// Every distinct durable retry count, including before a successful park resets it.
    #[must_use]
    pub fn absence_tries(&self) -> &[tasks::Tries] {
        &self.absence.tries
    }

    pub(super) fn absence_watch_needed(&mut self) -> bool {
        if !self.absence.enabled {
            return true;
        }
        !core::mem::replace(&mut self.absence.watching, true)
    }

    pub(super) fn absence_clock(&mut self) {
        if !self.absence.enabled {
            return;
        }
        for row in self.store.rows.values() {
            if let engine::Record::Core(jig_domain_core::Record::Tasks(
                tasks::Stored::Live(task) | tasks::Stored::Ended(task),
            )) = row
                && self.absence.tries.last() != Some(&task.tries)
            {
                self.absence.tries.push(task.tries);
            }
        }
        if self.absence.worker_running
            && self.absence.lost.is_some_and(|lost| {
                self.env.now.saturating_since(lost)
                    >= self.absence.told.unwrap_or(Duration::ZERO).max(self.absence.grace)
            })
        {
            self.absence.lost = None;
            self.worker_events.push_back(worker::Event::GracePassed);
        }
    }

    pub(super) fn absence_admitted(&mut self) {
        if self.absence.hello_pending {
            self.absence.hello_pending = false;
            self.absence.admissions += 1;
            self.absence.lost = None;
            self.absence.told = None;
            while let Some(link) = self.absence.outbound.pop_front() {
                self.up(link);
            }
        }
    }

    pub(super) fn absence_assignment(&mut self, assignment: &engine::Assignment) {
        if !self.absence.enabled || self.workload.is_some() {
            return;
        }
        if self.assignment.is_some() {
            self.absence.prompts.extend_from_slice(self.process.agent().prompts());
            let script = self.process.script.clone();
            if self.agent().turns().is_empty() {
                self.process = super::Process::new(self.seed, script);
            } else {
                self.process.resume(Script { cue: Box::default(), turns: Box::default() });
                self.process.fresh = Some(script);
            }
            self.process.held_query = Some(1);
            self.process.in_pieces(1);
        }
        self.absence.assignments.push(assignment.clone());
    }
}
