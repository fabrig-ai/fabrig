//! Store transport, fixture writes and cold restart.
use super::{Fault, StoreOut, Up, World, connector, core, fixture, root, tasks};
use core::engine::store as engine_store;

impl World {
    pub(super) fn point_load(&mut self, out: core::engine::store::Out) {
        use core::engine::store::{Out, Range};
        let Out::Load { load, range } = &out else { unreachable!("switch is not a point load") };
        assert!(self.internal_reads.insert(*load), "one internal point load");
        if matches!(range, Range::Record(core::Key::Tasks(tasks::Key::Ended(_))))
            && let Some(pending) = &mut self.pending_ended_loads
        {
            pending.push(out);
        } else {
            self.store.ask(StoreOut::Engine(out));
        }
    }

    /// Observe the start's logical walks, then carry the released store output.
    pub(super) fn start_load(&mut self, out: engine_store::Out) {
        use engine_store::{Out, Range};
        let phase = self.domain.start_phase().expect("engine reported the phase before the load");
        match &out {
            Out::Switch => {
                self.restart_steps.push(phase);
                if self.hold_switch {
                    self.switch_held = true;
                    return;
                }
            }
            Out::Load { load, range: Range::Start { ranges, after } } => {
                let live = ranges.as_ref() == core::RestoreRange::LIVE;
                if after.is_none() {
                    self.restart_steps.push(phase);
                    if live {
                        self.restore_ranges.extend(ranges.iter().cloned());
                    }
                }
                if live && !self.fixture_rows.is_empty() {
                    self.fixture_page(*load, after.as_ref());
                    return;
                }
                self.store.pause_pages(self.hold_restore_pages);
            }
            Out::Load { .. } => unreachable!("only the start's store outputs"),
        }
        self.store.ask(StoreOut::Engine(out));
    }

    // First-start fixture rows take the place of the empty store's live walk.
    // They still become durable only with the first seed commit.
    fn fixture_page(&mut self, load: skein_lib::Token, after: Option<&engine_store::Cursor>) {
        let offset = after
            .map_or(0, |cursor| usize::from_be_bytes(cursor.after.as_ref().try_into().expect("fixture page offset")));
        let items: Vec<_> = self
            .fixture_rows
            .iter()
            .skip(offset)
            .take(2)
            .map(|row| match row {
                root::Record::Core(row) => engine_store::Item::Record(row.clone()),
                root::Record::Connector { .. } => panic!("first-start fixture rows belong to the core"),
            })
            .collect();
        let offset = offset + items.len();
        let next = (offset < self.fixture_rows.len())
            .then(|| engine_store::Cursor { interval: 0, after: Box::new(offset.to_be_bytes()) });
        let result = engine_store::Answer::Rows(engine_store::Rows::Start { items: items.into_boxed_slice(), next });
        self.events.push_back(root::Event::Store(engine_store::In::Loaded { load, result }));
    }

    pub(super) fn connector_load(&mut self, number: u16, after: Option<Box<[u8]>>) {
        if after.is_none() {
            self.restart_steps.push(self.domain.start_phase().expect("engine reported the connector phase"));
        }
        self.store.pause_pages(self.hold_restore_pages);
        self.store.ask(StoreOut::ConnectorLoad { number, after });
    }

    /// Serve a previously captured switch after inspecting the restored copy.
    pub fn finish_switch(&mut self) {
        self.hold_switch = false;
        if std::mem::take(&mut self.switch_held) {
            self.store.ask(StoreOut::Engine(engine_store::Out::Switch));
        }
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
        let Some(phase) = self.domain.start_phase() else { return };
        let rows = match event {
            root::Event::Store(engine_store::In::Loaded {
                result: engine_store::Answer::Rows(engine_store::Rows::Start { items, .. }),
                ..
            }) => {
                if phase == core::start::Phase::LoadInbox {
                    for item in items {
                        if matches!(item, engine_store::Item::Record(core::Record::Tasks(tasks::Stored::Ended(_)))) {
                            self.census_ended += 1;
                        }
                    }
                }
                items.len()
            }
            root::Event::ConnectorPage { records, .. } => records.len(),
            _ => return,
        };
        if self.tracing && rows != 0 {
            self.trace.push(format!("store page {phase:?} {rows} rows"));
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
        self.try_seed(rows).unwrap_or_else(|finding| self.fail(&finding));
    }

    /// Save fixture rows, returning a refused durable store to its story.
    pub fn try_seed(&mut self, rows: Box<[root::Record]>) -> Result<(), String> {
        let mut writes = Vec::new();
        for row in rows {
            if let root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(task))) = &row {
                writes.push(root::Write::Save(root::Record::Core(core::Record::Tasks(
                    jig_domain_tasks::Stored::LiveCold {
                        number: task.number,
                        spec: Box::new(task.spec.clone()),
                        contract: task.contract.clone(),
                        authority: task.authority.clone(),
                        instructions: task.instructions.clone(),
                    },
                ))));
            }
            writes.push(root::Write::Save(row));
        }
        self.fixture_writes(writes)
    }
    /// Erase a fixture through the same durable transaction path.
    pub fn erase(&mut self, key: root::Key) -> Option<root::Record> {
        let prior = self.store.rows.get(&key).cloned();
        self.fixture_writes(vec![root::Write::Erase(key)]).unwrap_or_else(|finding| self.fail(&finding));
        prior
    }
    /// Model an upgrade from a build with no wait floors, leaving all wait facts intact.
    pub fn clear_wait_floors_for_upgrade(&mut self) {
        let header = core::Deployment { wait_floors: None, ..self.fixture_header() };
        self.commit_fixture_writes(Vec::new(), header).unwrap_or_else(|finding| self.fail(&finding));
    }

    fn fixture_header(&self) -> core::Deployment {
        let key = root::Key::Core(core::Key::Core(core::CoreKey::Deployment));
        let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = &self.store.rows[&key]
        else {
            panic!("durable header");
        };
        *header
    }

    fn fixture_writes(&mut self, writes: Vec<root::Write>) -> Result<(), String> {
        let mut header = self.fixture_header();
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
        self.commit_fixture_writes(writes, header)
    }

    fn commit_fixture_writes(
        &mut self,
        mut writes: Vec<root::Write>,
        mut header: core::Deployment,
    ) -> Result<(), String> {
        header.commits = self.store.applied + 1;
        writes.push(root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))));
        self.store.submit(header.commits, writes.into_boxed_slice(), 0);
        self.observer.submitted(self.wall, &self.store)?;
        assert_eq!(self.store.tick(false), Ok(Some(header.commits)));
        self.observer.durable(self.wall, &self.store)?;
        self.check_durable()?;
        if self.fork_cut(super::Cut::Durable(header.commits))? {
            self.reached_cut = self.cut.take();
        }
        self.refresh_fixture_header();
        Ok(())
    }
    fn refresh_fixture_header(&mut self) {
        let header = self.fixture_header();
        self.domain.core_mut().counters = core::Counters::new(header);
        *self.domain.journal_mut() = skein_lib::Journal::uncapped(self.limits.journal.release, header.commits);
    }

    /// Let every previously durable completion through its ordered barrier.
    pub fn release_commits(&mut self) {
        self.try_release_commits().unwrap_or_else(|finding| self.fail(&finding));
    }

    pub(crate) fn try_release_commits(&mut self) -> Result<(), String> {
        while let Some(number) = self.store.release_held() {
            self.events.push_back(root::Event::Committed { number, fill: self.fill });
        }
        self.try_drain()
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
            if self.restart_reads.contains(&number) {
                let event = self.systems[usize::from(number - 1)].answer(call, Fault::None);
                self.events.push_back(root::Event::Connector { number, event: connector::Event::System(event) });
            } else {
                self.pending_reads.push((number, call));
            }
        }
        self.restart_reads.clear();
    }

    pub fn finish_restart_reads(&mut self) {
        self.answer_restart_reads();
        self.hold_restart_reads = false;
        self.drain();
    }

    /// Cold restart from the fixture with the same resource kind declarations.
    pub fn restart(&mut self, seed: u64, requirement: bool) {
        let mut config = fixture(seed, requirement).0;
        for kind in &mut config.first.kinds {
            if kind.kind == 5 {
                kind.recovery = self.recovery;
            }
        }
        self.restart_from_fixture(config);
    }

    /// Keep the running connector's declared resource kinds when rebuilding a fixture.
    pub(crate) fn restart_from_fixture(&mut self, mut config: root::Config) {
        config.resource_kinds.clone_from(&self.restart_config.0.resource_kinds);
        self.restart_with(config);
    }

    /// Restart under the next binary's owner sets.
    pub fn restart_with_owner_sets(&mut self, seed: u64, requirement: bool, sets: jig_protocol_store::OwnerSets) {
        self.store.owner_sets(sets);
        self.restart(seed, requirement);
    }

    /// Run the next binary through Switch and its durable mark, holding the
    /// acknowledgement so no restored claim is adopted before the crash.
    pub fn switch_owner_sets(&mut self, seed: u64, sets: jig_protocol_store::OwnerSets) {
        self.hold_switch = true;
        self.restart_with_owner_sets(seed, false, sets);
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
        self.try_restart_with(config).unwrap_or_else(|finding| self.fail(&finding));
    }

    /// Cold start, returning the sweep child's finding at its checked cut.
    pub fn try_restart_with(&mut self, mut config: root::Config) -> Result<(), String> {
        for worker in &mut config.core.defaults.workers {
            worker.places = self.limits.core.fleet.slots;
        }
        self.restart_config = (super::Rc::new(crate::oracle::configuration(&config)), self.limits);
        self.restart_configured(config)
    }

    /// A cold start under `config`, which `restart_config` already holds a copy of.
    pub(crate) fn restart_configured(&mut self, config: root::Config) -> Result<(), String> {
        self.observer.observe(self.wall, crate::referee::Observed::Restart)?;
        if matches!(self.cut, Some(super::Cut::Disk { .. }))
            || matches!(self.reached_cut, Some(super::Cut::Disk { .. }))
        {
            self.reached_cut = None;
        }
        if let Some(super::Cut::Disk { from, operation }) = self.cut {
            self.store.fault(jig_test_store_files::Fault::Kill { from, operation });
        }
        self.store
            .reconfigure(18, &config.core.settings.models)
            .map_err(|finding| format!("restart readback: {finding:?}"))?;
        if !self.sweep.is_empty() {
            self.collect_disk_forks();
        }
        if self.store.reached_kill() {
            self.reached_cut = self.cut.take();
            return Ok(());
        }
        // The observer may resume A after a crashed pending copy whose
        // process-local commits were discarded. Its own position is the fence.
        self.observer.fresh_mark(self.wall, &self.store)?;
        self.observer.configure_projects(&config);
        if let Some(watches) = &mut self.watches {
            watches.restart(self.store.applied);
        }
        if let Some(reads) = &mut self.reads {
            reads.restart(self.store.applied);
        }
        self.observer.cold(self.wall, &self.store)?;
        self.stopped = false;
        self.period_started = false;
        self.switch_held = false;
        self.move_provider = config.core.settings.llm_providers.providers.first().map(|provider| provider.number);
        self.domain = root::Domain::new(config, &self.limits);
        if let Some((_, active)) = &mut self.restart_check {
            *active = true;
        }
        if let Some(setup) = self.startup {
            setup(&mut self.domain);
        }
        self.events.clear();
        self.internal_reads.clear();
        self.pending_reads.clear();
        self.restart_reads.clear();
        self.pending_lost_reads.clear();
        self.pending_writes.clear();
        self.pending_lookups.clear();
        self.restart_steps.clear();
        self.moved.clear();
        self.restore_ranges.clear();
        self.census_ended = 0;
        if let Some(peers) = &mut self.peers {
            peers.restart();
        }
        if !matches!(self.cut, Some(super::Cut::Disk { .. })) {
            self.cut = None;
        }
        self.events.push_back(root::Event::StartBegin { pending: self.store.generation_pending });
        self.try_drain_restarting(true)?;
        self.check_restarted_child()?;
        if self.domain.ready() {
            self.hello_engine();
            self.try_drain_restarting(true)?;
        }
        self.lose_answer = false;
        Ok(())
    }
}
