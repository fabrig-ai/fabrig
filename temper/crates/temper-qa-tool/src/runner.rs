//! A single-home service, with ordered saved steps per tick (qa/README.md, 4–5).
use crate::faces::{
    ClockFace, DirectoryMode, Environment, FilesFace, GitFace, LockMode, Moment, ProcessCommand, Processes, RemoveMode,
    Signal, WriteOptions, read_text, remove_missing,
};
use crate::{
    Exit, Home, Ledger, PassState, Refusal, SessionKind, brief::Inputs, pass_brief, pass_files, refused,
    watch::watch_with_fetch,
};
use std::{collections::BTreeMap, path::PathBuf};

struct Active {
    kind: SessionKind,
    bound: u64,
    process: u64,
    started: u64,
    bounded: bool,
    exit: Option<Exit>,
}
struct Failure {
    since: u64,
    lined: bool,
    retry_at: u64,
}

pub struct Run<G, P, C, F: FilesFace> {
    pub home: Home,
    pub git: G,
    pub processes: P,
    pub clock: C,
    pub files: F,
    environment: Environment,
    events: crate::events::Events,
    listener: Option<F::Listener>,
    next_fetch: Option<u64>,
    place_retry: Option<u64>,
    state_retry: Option<u64>,
    holders: Vec<PathBuf>,
    _lock: F::Lock,
    active: BTreeMap<u64, Active>,
    trees: Vec<PathBuf>,
    reports: Vec<u64>,
    kills: Vec<(u64, u64)>,
    notifications: Vec<u64>,
    failures: BTreeMap<&'static str, Failure>,
}
impl<G: GitFace, P: Processes, C: ClockFace, F: FilesFace> Run<G, P, C, F> {
    pub fn start(
        home: Home,
        git: G,
        processes: P,
        clock: C,
        mut files: F,
        environment: Environment,
    ) -> Result<Self, Refusal> {
        let lock =
            files.lock(&home.run_lock, LockMode::Try, false).map_err(|why| refused(format!("run.lock: {why}")))?;
        let Some(lock) = lock else {
            return Err(refused(format!("another temper-qa run holds {}", home.directory.display())));
        };
        let listener = crate::events::bind(&home.directory, &mut files)?;
        let events = crate::events::Events::default();
        let mut processes = processes;
        processes.wake_on_exit(events.waker());
        let mut run = Self {
            home,
            git,
            processes,
            clock,
            files,
            environment,
            events,
            listener: Some(listener),
            next_fetch: None,
            place_retry: None,
            state_retry: None,
            holders: Vec::new(),
            _lock: lock,
            active: BTreeMap::new(),
            trees: Vec::new(),
            reports: Vec::new(),
            kills: Vec::new(),
            notifications: Vec::new(),
            failures: BTreeMap::new(),
        };
        let now = run.clock.now();
        run.transact(|run, ledger| {
            let numbers: Vec<_> = ledger.running_sessions().iter().map(|session| session.number).collect();
            for number in numbers {
                let output =
                    read_text(&mut run.files, &run.home.session_directory(number).join("output")).unwrap_or_default();
                run.end_record(ledger, number, Exit::Lost, parse_tokens(&output), now.wall)?;
            }
            Ok(())
        })?;
        let result = run.clean_passes();
        run.outcome("triage", result, now);
        Ok(run)
    }

    fn transact<T>(
        &mut self,
        operation: impl FnOnce(&mut Self, &mut Ledger) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let (_lock, mut ledger) = self.home.begin(&mut self.files)?;
        let result = operation(self, &mut ledger)?;
        self.home.save(&mut self.files, &ledger)?;
        Ok(result)
    }

    /// Exit results are collected outside the ledger lock: the fake, like an
    /// agent's commands, may commit answers before reporting its exit.
    pub fn tick(&mut self) {
        let now = self.clock.now();
        let home = self.home.clone();
        let fetch = home.configuration.git.fetch.is_some() && self.next_fetch.is_none_or(|due| now.monotonic >= due);
        if fetch {
            self.next_fetch = Some(now.monotonic.saturating_add(home.configuration.bounds.poll));
        }
        let result = home.transact_loop(&mut self.files, |ledger, _| {
            watch_with_fetch(&mut self.git, &home.configuration.git, ledger, now.wall, fetch)
        });
        self.outcome("watch", result, now);
        let result =
            self.probe().and_then(|()| self.transact(|run, ledger| Ok(run.triage(ledger))).and_then(|result| result));
        self.outcome("triage", result, now);
        let result = self.passes();
        self.outcome("passes", result, now);
        let result = home
            .transact_loop(&mut self.files, |ledger, files| {
                Ok(crate::requests::step(&home, ledger, &mut self.processes, files, &self.environment, now.wall))
            })
            .and_then(|result| result);
        self.outcome("requests", result, now);
        let result = home.transact_loop(&mut self.files, |ledger, _| {
            ledger.waiting_lines(home.configuration.bounds.build_wait, now.wall, &home.configuration.owner);
            Ok(())
        });
        self.outcome("waiting cases", result, now);
        // Files already removed and sessions already ended must stay recorded
        // even if a later effect in their step fails.
        let result = self.transact(|run, ledger| Ok(run.delete_builds(ledger, now.wall))).and_then(|result| result);
        self.outcome("builds", result, now);
        let result = self.transact(|run, ledger| Ok(run.give_lines(ledger))).and_then(|result| result);
        self.outcome("lines", result, now);
        let result = home.read(&mut self.files, |ledger| {
            self.place_retry =
                ledger.place_waits.last().filter(|wait| wait.to.is_none()).map(|_| now.monotonic.saturating_add(60));
            self.state_retry = (home.configuration.request.state.is_some()
                && ledger.passes.iter().any(|pass| {
                    pass.state == PassState::Waiting
                        && pass.requests.last().is_some_and(|request| {
                            request.ended.is_none()
                                && request
                                    .goal
                                    .is_some_and(|goal| !ledger.landings.iter().any(|landing| landing.goal == goal))
                        })
                }))
            .then(|| now.monotonic.saturating_add(60));
            Ok(())
        });
        self.outcome("waits", result, now);
    }

    /// Accepts and child waiters feed one coalescing channel; only deadlines time out.
    pub fn serve(&mut self) -> Result<(), Refusal> {
        let listener = self.listener.take().expect("one serving loop");
        let waker = self.waker();
        self.files.serve_wakes(listener, waker)?;
        self.tick();
        loop {
            self.cycle();
        }
    }

    #[must_use]
    pub fn waker(&self) -> crate::events::Waker {
        self.events.waker()
    }

    /// Wait for one coalesced event or the nearest deadline, then run one tick.
    pub fn cycle(&mut self) {
        let timeout = self.wait_timeout().unwrap_or_else(|why| {
            self.files.diagnostic(&format!("temper-qa: wait: {why}"));
            Some(60)
        });
        self.clock.wait(&self.events, timeout);
        self.tick();
    }

    pub fn wait_timeout(&mut self) -> Result<Option<u64>, Refusal> {
        let now = self.clock.now();
        let deadline = self
            .active
            .values()
            .filter(|active| active.exit.is_none() && !active.bounded)
            .map(|active| active.started.saturating_add(active.bound))
            .chain(self.kills.iter().map(|(_, due)| *due))
            .chain(self.next_fetch)
            .chain(self.place_retry)
            .chain(self.state_retry)
            .chain(self.failures.values().map(|failure| {
                let line = failure.since.saturating_add(3600);
                failure.retry_at.min(if failure.lined || line <= now.monotonic { u64::MAX } else { line })
            }))
            .min();
        self.home.read(&mut self.files, |ledger| {
            let build = ledger
                .build_wait_deadline(self.home.configuration.bounds.build_wait)
                .map(|wall| now.monotonic.saturating_add(wall.saturating_sub(now.wall)));
            Ok(deadline.into_iter().chain(build).min().map(|due| due.saturating_sub(now.monotonic)))
        })
    }

    fn probe(&mut self) -> Result<(), Refusal> {
        let now = self.clock.now().monotonic;
        let mut remaining = Vec::new();
        for (process, due) in &self.kills {
            if now >= *due {
                self.processes.signal(*process, Signal::Kill)?;
            } else {
                remaining.push((*process, *due));
            }
        }
        self.kills = remaining;
        for active in self.active.values_mut().filter(|active| active.exit.is_none()) {
            active.exit = self.processes.poll(active.process)?;
            if active.exit.is_none() && !active.bounded && now.saturating_sub(active.started) >= active.bound {
                self.processes.signal(active.process, Signal::Term)?;
                active.bounded = true;
                self.kills.push((active.process, now.saturating_add(10)));
                active.exit = self.processes.poll(active.process)?;
            }
        }
        Ok(())
    }

    fn triage(&mut self, ledger: &mut Ledger) -> Result<(), Refusal> {
        let now = self.clock.now();
        if let Some((&number, active)) = self.active.iter().find(|(_, active)| active.kind == SessionKind::Triage)
            && let Some(exit) = active.exit
        {
            let output =
                read_text(&mut self.files, &self.home.session_directory(number).join("output")).unwrap_or_default();
            ledger.end_session(
                number,
                if active.bounded { Exit::Bound } else { exit },
                parse_tokens(&output),
                now.wall,
                &self.home.configuration.owner,
            )?;
            self.active.remove(&number);
        }
        self.clean_trees()?;
        if self.active.values().any(|active| active.kind == SessionKind::Triage) {
            return Ok(());
        }
        crate::watch::close_gaps(&mut self.git, ledger, now.wall)?;
        let Some(landing) = ledger.next_landing().cloned() else { return Ok(()) };
        let build =
            ledger.triage_build(landing.number).map_or_else(|| landing.commit.clone(), |build| build.commit.clone());
        let configuration = &self.home.configuration;
        let kinds: Vec<_> = configuration.places.iter().map(|place| configuration.place_kind(place)).collect();
        let inputs = Inputs::read(&mut self.git, ledger, &landing, &configuration.surfaces, &kinds, &build)?;
        let number = ledger.start_session(
            landing.number,
            &self.home.configuration.agent.triage,
            inputs.pages.clone(),
            inputs.setups.clone(),
            inputs.deployments.clone(),
            now.wall,
        )?;
        let directory = self.home.session_directory(number);
        let tree = self.home.triage_tree();
        let mut start = || -> Result<ProcessCommand, Refusal> {
            self.files
                .directory(&directory, DirectoryMode::Parents)
                .map_err(|why| refused(format!("sessions/{number}: {why}")))?;
            let brief = directory.join("brief");
            self.files
                .write(
                    &brief,
                    inputs.render(&self.home, number, &landing, ledger).as_bytes(),
                    WriteOptions::REPLACE,
                    false,
                )
                .map_err(|why| refused(format!("brief: {why}")))?;
            self.files
                .write(&directory.join("diff.patch"), &inputs.patch, WriteOptions::REPLACE, false)
                .map_err(|why| refused(format!("diff.patch: {why}")))?;
            Ok(self.agent_command(number, &self.home.configuration.agent.triage, tree.clone()))
        };
        let result = start().and_then(|command| {
            self.prepare_tree(&tree, &build)?;
            self.processes.start(&command)
        });
        match result {
            Ok(process) => {
                self.active.insert(
                    number,
                    Active {
                        kind: SessionKind::Triage,
                        bound: self.home.configuration.bounds.triage,
                        process,
                        started: now.monotonic,
                        bounded: false,
                        exit: None,
                    },
                );
            }
            Err(why) => {
                ledger.end_session(number, Exit::Unstarted, None, now.wall, &self.home.configuration.owner)?;
                return Err(why);
            }
        }
        Ok(())
    }
    fn agent_command(&self, number: u64, effort: &str, tree: PathBuf) -> ProcessCommand {
        let directory = self.home.session_directory(number);
        let mut environment = self.environment.clone();
        environment.retain(|(name, _)| {
            name.as_ref() != crate::HOME.name.as_bytes() && name.as_ref() != crate::SESSION.name.as_bytes()
        });
        environment
            .push((crate::HOME.name.as_bytes().into(), self.home.directory.as_os_str().as_encoded_bytes().into()));
        environment.push((crate::SESSION.name.as_bytes().into(), number.to_string().into_bytes().into()));
        ProcessCommand {
            arguments: self
                .home
                .configuration
                .agent
                .command
                .iter()
                .map(|argument| argument.replace("{effort}", effort))
                .collect(),
            directory: tree,
            stdin: directory.join("brief"),
            output: directory.join("output"),
            environment,
        }
    }

    /// End records are committed before reports take the ledger's read lock.
    /// Keep pending effects until they succeed, including after an unstarted end.
    fn passes(&mut self) -> Result<(), Refusal> {
        let home = self.home.clone();
        self.transact(|run, ledger| {
            let ended: Vec<_> = run
                .active
                .iter()
                .filter_map(|(&number, active)| {
                    (active.kind == SessionKind::Pass).then_some((number, active.exit, active.bounded))
                })
                .collect();
            let end = || -> Result<(), Refusal> {
                for (number, exit, bounded) in ended {
                    if let Some(exit) = exit {
                        let output = read_text(&mut run.files, &home.session_directory(number).join("output"))
                            .unwrap_or_default();
                        run.end_record(
                            ledger,
                            number,
                            if bounded { Exit::Bound } else { exit },
                            parse_tokens(&output),
                            run.clock.now().wall,
                        )?;
                        run.active.remove(&number);
                    }
                }
                Ok(())
            };
            Ok(end())
        })??;
        self.clean_passes()?;
        for place in home.configuration.places.clone() {
            // An effect failure must not undo an earlier session's end, or an
            // unstarted rollback, in this step.
            let result = self.transact(|run, ledger| Ok(run.start_pass(ledger, &place)))?;
            let cleanup = self.clean_passes();
            result?;
            cleanup?;
        }
        home.transact_loop(&mut self.files, |ledger, files| {
            let mut free = false;
            for place in &home.configuration.places {
                if !ledger.passes.iter().any(|pass| pass.state == PassState::Running && &pass.place == place) {
                    let directory = home.configuration.place_directory(&home.directory, place)?;
                    if pass_files::free_lock(&directory, home.configuration.place_kind(place), files)?.is_some() {
                        free = true;
                        break;
                    }
                }
            }
            ledger.update_place_wait(&home.configuration.surfaces, free, self.clock.now().wall);
            Ok(())
        })
    }

    fn end_record(
        &mut self,
        ledger: &mut Ledger,
        number: u64,
        exit: Exit,
        tokens: Option<u64>,
        now: u64,
    ) -> Result<(), Refusal> {
        let session = ledger.sessions.iter().find(|session| session.number == number).expect("running session");
        if let Some(number) = session.pass {
            let pass = ledger.passes.iter().find(|pass| pass.number == number).expect("session's pass");
            let recipes = if exit == Exit::Unstarted {
                Vec::new()
            } else {
                pass_files::changed_setups(
                    &mut self.git,
                    &pass.build,
                    &pass_files::setups(&self.home, number),
                    &mut self.files,
                )?
            };
            let place = self.home.configuration.place_directory(&self.home.directory, &pass.place)?;
            ledger.end_pass_session(session.number, exit, tokens, now, &self.home.configuration.owner, &recipes)?;
            self.holders.push(place);
            self.reports.push(number);
        } else {
            ledger.end_session(number, exit, tokens, now, &self.home.configuration.owner)?;
        }
        let legacy = self.home.session_directory(number).join("tree");
        if self.files.facts(&legacy, true).is_ok() {
            self.trees.push(legacy);
        }
        Ok(())
    }

    fn start_pass(&mut self, ledger: &mut Ledger, place: &str) -> Result<(), Refusal> {
        if ledger.passes.iter().any(|pass| pass.state == PassState::Running && pass.place == place) {
            return Ok(());
        }
        let configuration = &self.home.configuration;
        let now = self.clock.now();
        // Run the ledger's selection on a snapshot. Preparation failures record
        // nothing; no caller duplicates the ledger's eligibility rules.
        let kind = configuration.place_kind(place);
        let mut recorded = ledger.clone();
        let Some(start) = recorded
            .resume_pass(place, kind, &configuration.surfaces, &configuration.agent.pass, now.wall)
            .or_else(|| recorded.start_pass(place, kind, &configuration.surfaces, &configuration.agent.pass, now.wall))
        else {
            return Ok(());
        };
        let directory = configuration.place_directory(&self.home.directory, place)?;
        let Some(lock) = pass_files::free_lock(&directory, kind, &mut self.files)? else { return Ok(()) };
        crate::installation::install(
            &self.home.builds.join(&start.build),
            &start.build,
            &directory,
            kind,
            &mut self.files,
        )?;
        pass_files::write_setups(
            &mut self.git,
            &start.build,
            &pass_files::setups(&self.home, start.pass),
            &mut self.files,
        )?;
        pass_files::prepare_files(&self.home, start.pass, &mut self.files)?;
        *ledger = recorded;
        let command = self.agent_command(start.session, &configuration.agent.pass, self.home.pass_tree(place));
        let mut prepare = || -> Result<(), Refusal> {
            let directory = self.home.session_directory(start.session);
            self.files
                .directory(&directory, DirectoryMode::Parents)
                .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
            let brief = pass_brief::render(&mut self.git, &self.home, &start, ledger)?;
            self.files
                .write(&command.stdin, brief.as_bytes(), WriteOptions::REPLACE, false)
                .map_err(|why| refused(format!("brief: {why}")))?;
            self.prepare_tree(&command.directory, &start.build)
        };
        let result = prepare().and_then(|()| self.processes.start(&command));
        drop(lock);
        match result {
            Ok(process) => {
                self.active.insert(
                    start.session,
                    Active {
                        kind: SessionKind::Pass,
                        bound: self.home.configuration.bounds.pass,
                        process,
                        started: now.monotonic,
                        bounded: false,
                        exit: None,
                    },
                );
                Ok(())
            }
            Err(why) => {
                self.end_record(ledger, start.session, Exit::Unstarted, None, now.wall)?;
                Err(why)
            }
        }
    }

    fn clean_passes(&mut self) -> Result<(), Refusal> {
        for place in std::mem::take(&mut self.holders) {
            self.processes.end_holder(place, self.waker());
        }
        while let Some(&number) = self.reports.first() {
            self.home.write_pass_report(number, &mut self.files)?;
            self.reports.remove(0);
        }
        self.clean_trees()
    }

    fn clean_trees(&mut self) -> Result<(), Refusal> {
        if self.trees.is_empty() {
            return Ok(());
        }
        for tree in &self.trees {
            self.git.remove_tree(tree)?;
        }
        self.git.prune()?;
        self.trees.clear();
        Ok(())
    }

    fn prepare_tree(&mut self, tree: &std::path::Path, commit: &str) -> Result<(), Refusal> {
        if let Some(parent) = tree.parent() {
            crate::private::directory(parent, &mut self.files)?;
        }
        if self.files.facts(tree, true).is_ok() {
            if self.git.move_tree(tree, commit).is_ok() {
                return Ok(());
            }
            self.git.remove_tree(tree)?;
            self.git.prune()?;
        }
        if let Err(why) = self.git.add_tree(tree, commit) {
            self.git.remove_tree(tree)?;
            self.git.prune()?;
            return Err(why);
        }
        Ok(())
    }
    fn delete_builds(&mut self, ledger: &mut Ledger, now: u64) -> Result<(), Refusal> {
        let mut builds: Vec<_> =
            ledger.deletable_builds().iter().map(|build| (build.order, build.commit.clone())).collect();
        builds.sort();
        for (_, commit) in builds {
            let directory = self.home.builds.join(&commit);
            remove_missing(&mut self.files, &directory, RemoveMode::Tree)
                .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
            ledger.mark_build_deleted(&commit, now)?;
        }
        Ok(())
    }
    fn give_lines(&mut self, ledger: &mut Ledger) -> Result<(), Refusal> {
        let mut running = Vec::new();
        for process in &self.notifications {
            match self.processes.poll(*process) {
                Ok(None) => running.push(*process),
                Ok(Some(Exit::Code(0))) => (),
                Ok(Some(exit)) => self.files.diagnostic(&format!("temper-qa: notify: {exit:?}")),
                Err(why) => {
                    self.files.diagnostic(&format!("temper-qa: notify: {why}"));
                    running.push(*process);
                }
            }
        }
        self.notifications = running;
        let Some(arguments) = &self.home.configuration.notify else { return Ok(()) };
        let lines: Vec<_> = ledger.ungiven_lines().into_iter().cloned().collect();
        for line in lines {
            let directory = self.home.directory.join("notifications").join(line.number.to_string());
            let input = directory.join("input");
            let mut prepare = || -> Result<(), Refusal> {
                self.files
                    .directory(&directory, DirectoryMode::Parents)
                    .map_err(|why| refused(format!("notify: {why}")))?;
                self.files
                    .write(&input, format!("{}: {}\n", line.area, line.words).as_bytes(), WriteOptions::REPLACE, false)
                    .map_err(|why| refused(format!("notify: {why}")))
            };
            prepare()?;
            let result = self.processes.start(&ProcessCommand {
                arguments: arguments.clone(),
                directory: self.home.directory.clone(),
                stdin: input,
                output: directory.join("output"),
                environment: self.environment.clone(),
            });
            ledger.mark_line_given(line.number)?;
            match result {
                Ok(process) => self.notifications.push(process),
                Err(why) => self.files.diagnostic(&format!("temper-qa: notify: {why}")),
            }
        }
        Ok(())
    }
    fn outcome(&mut self, step: &'static str, result: Result<(), Refusal>, now: Moment) {
        match result {
            Ok(()) => {
                self.failures.remove(step);
            }
            Err(why) => {
                self.files.diagnostic(&format!("temper-qa: {step}: {why}"));
                let failure = self.failures.entry(step).or_insert(Failure {
                    since: now.monotonic,
                    lined: false,
                    retry_at: now.monotonic.saturating_add(60),
                });
                failure.retry_at = now.monotonic.saturating_add(60);
                if !failure.lined && now.monotonic.saturating_sub(failure.since) >= 3600 {
                    match self.home.transact_loop(&mut self.files, |ledger, _| {
                        ledger.add_line(
                            &self.home.configuration.owner,
                            &format!("{step} has failed for an hour: {why}"),
                            now.wall,
                        );
                        Ok(())
                    }) {
                        Ok(()) => failure.lined = true,
                        Err(why) => self.files.diagnostic(&format!("temper-qa: recording {step} failure: {why}")),
                    }
                }
            }
        }
    }
}

#[must_use]
pub fn parse_tokens(output: &str) -> Option<u64> {
    let value = output.lines().rev().find(|line| !line.trim().is_empty())?.strip_prefix("tokens ")?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}
