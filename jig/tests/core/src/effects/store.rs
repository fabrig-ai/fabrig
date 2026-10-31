//! Store transport, fixture writes and cold restart.
use super::{Fault, StoreOut, Up, World, connector, core, fixture, root, tasks};
impl World {
    /// Serve the phase selected by the core (jig/docs/design/domain/engine.md, 6).
    pub(super) fn start_step(&mut self, phase: core::start::Phase, census: Option<core::Census>) {
        use core::start::Phase;
        if phase == Phase::Sweep && self.restoring.is_some_and(|(selected, _)| selected == Phase::Sweep) {
            return;
        }
        self.restart_steps.push(phase);
        match phase {
            Phase::LoadCore | Phase::LoadInbox | Phase::RestoreConnector { .. } | Phase::Sweep => {
                if phase == Phase::LoadCore {
                    self.restore_ranges.extend(core::RestoreRange::LIVE);
                    let env = self.env();
                    for row in self.fixture_rows.clone() {
                        root::start_record(&mut self.domain, &env, row);
                    }
                }
                self.restoring = Some((phase, 0));
                self.store.pause_pages(self.hold_restore_pages);
                self.store.ask(StoreOut::Start { phase, census });
            }
            Phase::Switch if self.hold_switch => {}
            Phase::Switch => {
                if self.store.generation_pending {
                    self.store.ask(StoreOut::Start { phase, census: None });
                } else {
                    self.events.push_back(root::Event::StartDone(phase));
                }
            }
            Phase::Link
            | Phase::Wires
            | Phase::AdoptRuns
            | Phase::ReadAfresh { .. }
            | Phase::SettleOutbox { .. }
            | Phase::Open => panic!("root performs its synchronous phase"),
        }
    }

    /// Serve a previously captured Switch after inspecting the restored copy.
    pub fn finish_switch(&mut self) {
        self.hold_switch = false;
        self.restart_steps.pop();
        self.start_step(core::start::Phase::Switch, None);
        self.drain();
    }

    pub(super) fn restore_pages(&mut self) {
        for answer in self.store.answers() {
            match answer {
                Up::Root(event) => self.events.push_back(event),
                Up::Ask(_) => panic!("adapter follows its own page continuations"),
            }
        }
    }

    pub(super) fn start_input(&mut self, event: &root::Event) {
        if let root::Event::StartRecord(row) = event {
            let (phase, count) = self.restoring.as_mut().expect("one start load");
            *count += 1;
            if *phase == core::start::Phase::LoadInbox
                && matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(_))))
            {
                self.census_ended += 1;
            }
            if self.tracing && *count % 2 == 0 {
                self.trace.push(format!("store page {phase:?} 2 rows"));
            }
        } else if let root::Event::StartDone(phase) = event
            && self.restoring.is_some_and(|(selected, _)| selected == *phase)
        {
            let (_, count) = self.restoring.take().expect("start load completion");
            if self.tracing && count % 2 != 0 {
                self.trace.push(format!("store page {phase:?} 1 rows"));
            }
        }
    }

    /// Deliver every outstanding original restore page through the usual route.
    pub fn finish_restore_pages(&mut self) {
        self.hold_restore_pages = false;
        self.store.pause_pages(false);
        self.drain();
    }

    /// Save fixture rows through a durable, externally observed numbered commit.
    pub fn seed(&mut self, rows: Box<[root::Record]>) {
        self.fixture_writes(rows.into_vec().into_iter().map(root::Write::Save).collect());
    }
    /// Erase a fixture through the same durable transaction path.
    pub fn erase(&mut self, key: root::Key) -> Option<root::Record> {
        let prior = self.store.rows.get(&key).cloned();
        self.fixture_writes(vec![root::Write::Erase(key)]);
        prior
    }
    fn fixture_writes(&mut self, mut writes: Vec<root::Write>) {
        let header_key = root::Key::Core(core::Key::Core(core::CoreKey::Deployment));
        let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(mut header))) =
            self.store.rows[&header_key].clone()
        else {
            panic!("durable header");
        };
        // Direct fixture policy seeds must carry the same high-water mark as a first start.
        for write in &writes {
            if let root::Write::Save(root::Record::Core(core::Record::People(jig_domain_people::Stored::Policy {
                project,
                ..
            }))) = write
            {
                header.projects = header.projects.max(u64::from(*project));
            }
        }
        header.commits = self.store.applied + 1;
        writes.push(root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))));
        self.store.submit(header.commits, writes.into_boxed_slice(), 0);
        self.observer.submitted(self.wall, &self.store);
        assert_eq!(self.store.tick(false), Ok(Some(header.commits)));
        self.observer.durable(self.wall, &self.store);
        self.refresh_fixture_header();
    }
    fn refresh_fixture_header(&mut self) {
        let header = self.store.rows[&root::Key::Core(core::Key::Core(core::CoreKey::Deployment))].clone();
        let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = header else {
            unreachable!("fixture header is the deployment");
        };
        self.domain.core_mut().counters = core::Counters::new(header);
        *self.domain.journal_mut() = skein_lib::Journal::uncapped(self.limits.journal.release, header.commits);
    }

    /// Install the connector's fixed hold vocabulary, retained across cold starts.
    pub fn configure_holds(&mut self, connector: u16, kinds: Box<[tasks::Kind]>) {
        self.hold_kinds.push((connector, kinds.clone()));
        self.send(root::Event::Core(core::Event::Tasks(tasks::Event::Kinds { connector, kinds })));
    }

    /// Let every previously durable completion through its ordered barrier.
    pub fn release_commits(&mut self) {
        while let Some(number) = self.store.release_held() {
            self.events.push_back(root::Event::Committed { number, fill: self.fill });
        }
        self.drain();
    }

    /// Read each lost writer's resource and deliver its answer through the root.
    pub fn finish_lost_reads(&mut self) {
        self.hold_lost_reads = false;
        for (connector, resource, call) in std::mem::take(&mut self.pending_lost_reads) {
            let event = self.systems[usize::from(connector - 1)].answer(call, Fault::None);
            self.events.push_back(root::Event::WriterRead { connector, resource, event });
        }
        self.drain();
    }

    pub(super) fn answer_restart_reads(&mut self) {
        for (number, call) in std::mem::take(&mut self.pending_reads) {
            if self.restart_reads.contains_key(&number) {
                let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
                self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
            } else {
                self.pending_reads.push((number, call));
            }
        }
        for (_, phase) in std::mem::take(&mut self.restart_reads) {
            self.events.push_back(root::Event::StartDone(phase));
        }
    }

    pub fn finish_restart_reads(&mut self) {
        self.answer_restart_reads();
        self.hold_restart_reads = false;
        self.drain();
    }

    pub fn restart(&mut self, seed: u64, requirement: bool) {
        let mut config = fixture(seed, requirement).0;
        for kind in &mut config.first.kinds {
            if kind.kind == 5 {
                kind.recovery = self.recovery;
            }
        }
        self.restart_with(config);
    }

    /// Restart under the next binary's patch sets.
    pub fn restart_with_patch_sets(&mut self, seed: u64, requirement: bool, sets: jig_protocol_store::PatchSets) {
        self.store.patch_sets(sets);
        self.restart(seed, requirement);
    }

    /// Run the next binary through Switch and its durable mark, holding the
    /// acknowledgement so no restored claim is adopted before the crash.
    pub fn switch_patch_sets(&mut self, seed: u64, sets: jig_protocol_store::PatchSets) {
        self.hold_switch = true;
        self.restart_with_patch_sets(seed, false, sets);
        self.store.fault(jig_test_store_files::Fault::Hold { commits: 1 });
        self.finish_switch();
    }

    /// Cold start after the owner lowers limits between processes.
    pub fn restart_with_limits(&mut self, config: root::Config, limits: root::Limits) {
        self.limits = limits;
        self.restart_with(config);
    }

    /// Cold start with the same scenario configuration, including its host slots.
    /// Change the application's startup setup for its replacement core.
    pub fn restart_with_startup(&mut self, config: root::Config, setup: fn(&mut root::Domain)) {
        self.startup = Some(setup);
        self.restart_with(config);
    }

    pub fn restart_with(&mut self, config: root::Config) {
        self.restart_config = (crate::oracle::configuration(&config), self.limits);
        self.observer.observe(self.wall, crate::referee::Observed::Restart);
        self.store.reconfigure(18, &config.core.settings.models);
        // The observer may resume A after a crashed pending copy whose
        // process-local commits were discarded. Its own position is the fence.
        self.observer.fresh_mark(self.wall, &self.store);
        self.observer.configure_projects(&config);
        if let Some(watches) = &mut self.watches {
            watches.restart(self.store.applied);
        }
        if let Some(reads) = &mut self.reads {
            reads.restart(self.store.applied);
        }
        self.observer.cold(self.wall, &self.store);
        self.stopped = false;
        self.period_started = false;
        self.restoring = None;
        self.move_provider = config.core.settings.llm_providers.providers.first().map(|provider| provider.number);
        self.domain = root::Domain::new(config, &self.limits);
        if let Some(setup) = self.startup {
            setup(&mut self.domain);
        }
        for (connector, kinds) in self.hold_kinds.clone() {
            self.domain.configure_holds(&self.env(), connector, kinds);
        }
        self.events.clear();
        self.pending_reads.clear();
        self.pending_lost_reads.clear();
        self.pending_writes.clear();
        self.pending_lookups.clear();
        self.restart_reads.clear();
        self.restart_steps.clear();
        self.moved.clear();
        self.restore_ranges.clear();
        self.census_ended = 0;
        if let Some(peers) = &mut self.peers {
            peers.restart();
        }
        self.cut = None;
        self.events.push_back(root::Event::StartBegin { pending: self.store.generation_pending });
        self.drain();
        if self.domain.ready() {
            self.hello_engine();
            self.drain();
        }
        self.lose_answer = false;
    }
}
