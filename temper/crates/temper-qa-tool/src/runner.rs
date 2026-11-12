//! A single-home service, with ordered saved steps per tick (qa/README.md, 4–5).
use crate::faces::{ClockFace, Environment, GitFace, Moment, ProcessCommand, Processes, Signal};
use crate::{
    Exit, Home, Ledger, PassState, Refusal, SessionKind, brief::Inputs, pass_brief, pass_files, refused, watch::watch,
};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions, TryLockError},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};

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
}

pub struct Run<G, P, C> {
    pub home: Home,
    pub git: G,
    pub processes: P,
    pub clock: C,
    environment: Environment,
    _lock: File,
    active: BTreeMap<u64, Active>,
    trees: Vec<PathBuf>,
    reports: Vec<u64>,
    kills: Vec<(u64, u64)>,
    notifications: Vec<u64>,
    failures: BTreeMap<&'static str, Failure>,
}
impl<G: GitFace, P: Processes, C: ClockFace> Run<G, P, C> {
    pub fn start(home: Home, git: G, processes: P, clock: C, environment: Environment) -> Result<Self, Refusal> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&home.run_lock)
            .map_err(|why| refused(format!("run.lock: {why}")))?;
        match lock.try_lock() {
            Ok(()) => (),
            Err(TryLockError::WouldBlock) => {
                return Err(refused(format!("another temper-qa run holds {}", home.directory.display())));
            }
            Err(TryLockError::Error(why)) => return Err(refused(format!("run.lock: {why}"))),
        }
        let mut run = Self {
            home,
            git,
            processes,
            clock,
            environment,
            _lock: lock,
            active: BTreeMap::new(),
            trees: Vec::new(),
            reports: Vec::new(),
            kills: Vec::new(),
            notifications: Vec::new(),
            failures: BTreeMap::new(),
        };
        let now = run.clock.now();
        let home = run.home.clone();
        home.transact(|ledger| {
            let numbers: Vec<_> = ledger.running_sessions().iter().map(|session| session.number).collect();
            for number in numbers {
                let output = fs::read_to_string(home.session_directory(number).join("output")).unwrap_or_default();
                run.end_record(ledger, number, Exit::Lost, parse_tokens(&output), now.wall)?;
            }
            Ok(())
        })?;
        let result = run.clean_passes();
        run.outcome("triage", result, now);
        Ok(run)
    }

    /// The process probes are outside the ledger lock: the fake, like an agent's
    /// commands, may commit answers through Home before reporting its exit.
    pub fn tick(&mut self) {
        let now = self.clock.now();
        let home = self.home.clone();
        let result = home.transact(|ledger| watch(&mut self.git, &home.configuration.git, ledger, now.wall));
        self.outcome("watch", result, now);
        let result =
            self.probe().and_then(|()| home.transact(|ledger| Ok(self.triage(ledger))).and_then(|result| result));
        self.outcome("triage", result, now);
        let result = self.passes();
        self.outcome("passes", result, now);
        let result = home
            .transact(|ledger| {
                Ok(crate::requests::step(&home, ledger, &mut self.processes, &self.environment, now.wall))
            })
            .and_then(|result| result);
        self.outcome("requests", result, now);
        let result = home.transact(|ledger| {
            ledger.waiting_lines(home.configuration.bounds.build_wait, now.wall, &home.configuration.owner);
            Ok(())
        });
        self.outcome("waiting cases", result, now);
        // Files already removed and sessions already ended must stay recorded
        // even if a later effect in their step fails.
        let result = home.transact(|ledger| Ok(self.delete_builds(ledger, now.wall))).and_then(|result| result);
        self.outcome("builds", result, now);
        let result = home.transact(|ledger| Ok(self.give_lines(ledger))).and_then(|result| result);
        self.outcome("lines", result, now);
    }

    /// One-second probes wake a full tick at an exit or a deadline, rather than
    /// waiting for poll. No probe waits for the agent's words.
    pub fn serve(&mut self) -> Result<(), Refusal> {
        loop {
            self.cycle();
        }
    }

    /// A tick and its wait, exposed so a fake clock can drive the service's cadence.
    pub fn cycle(&mut self) {
        let started = self.clock.now().monotonic;
        self.tick();
        if self.home.configuration.bounds.poll == 0 {
            return;
        }
        let mut probe_failed = false;
        loop {
            self.clock.sleep(1);
            let now = self.clock.now().monotonic;
            if now.saturating_sub(started) >= self.home.configuration.bounds.poll {
                break;
            }
            if !probe_failed {
                match self.probe() {
                    Ok(()) if self.active.values().any(|active| active.exit.is_some()) => break,
                    Ok(()) => (),
                    Err(why) => {
                        eprintln!("temper-qa: sessions: {why}");
                        probe_failed = true;
                    }
                }
            }
        }
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
            let output = fs::read_to_string(self.home.session_directory(number).join("output")).unwrap_or_default();
            ledger.end_session(
                number,
                if active.bounded { Exit::Bound } else { exit },
                parse_tokens(&output),
                now.wall,
                &self.home.configuration.owner,
            )?;
            self.trees.push(self.home.session_directory(number).join("tree"));
            self.active.remove(&number);
        }
        self.clean_trees()?;
        if self.active.values().any(|active| active.kind == SessionKind::Triage) {
            return Ok(());
        }
        let Some(landing) = ledger.next_landing().cloned() else { return Ok(()) };
        let inputs = Inputs::read(&mut self.git, &landing)?;
        let number = ledger.start_session(
            landing.number,
            &self.home.configuration.agent.triage,
            inputs.pages.clone(),
            inputs.setups.clone(),
            now.wall,
        )?;
        let directory = self.home.session_directory(number);
        let tree = directory.join("tree");
        let start = || -> Result<ProcessCommand, Refusal> {
            fs::create_dir_all(&directory).map_err(|why| refused(format!("sessions/{number}: {why}")))?;
            let brief = directory.join("brief");
            fs::write(&brief, inputs.render(&self.home, number, &landing, ledger))
                .map_err(|why| refused(format!("brief: {why}")))?;
            fs::write(directory.join("diff.patch"), inputs.patch)
                .map_err(|why| refused(format!("diff.patch: {why}")))?;
            Ok(self.agent_command(number, &self.home.configuration.agent.triage))
        };
        let result = start().and_then(|command| {
            self.git.add_tree(&tree, &landing.commit)?;
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
                self.trees.push(tree);
                if let Err(cleanup) = self.clean_trees() {
                    eprintln!("temper-qa: triage cleanup: {cleanup}");
                }
                return Err(why);
            }
        }
        Ok(())
    }
    fn agent_command(&self, number: u64, effort: &str) -> ProcessCommand {
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
            directory: directory.join("tree"),
            stdin: directory.join("brief"),
            output: directory.join("output"),
            environment,
        }
    }

    /// End records are committed before reports take the ledger's read lock.
    /// Keep pending effects until they succeed, including after an unstarted end.
    fn passes(&mut self) -> Result<(), Refusal> {
        let home = self.home.clone();
        home.transact(|ledger| {
            let ended: Vec<_> = self
                .active
                .iter()
                .filter_map(|(&number, active)| {
                    (active.kind == SessionKind::Pass).then_some((number, active.exit, active.bounded))
                })
                .collect();
            let end = || -> Result<(), Refusal> {
                for (number, exit, bounded) in ended {
                    if let Some(exit) = exit {
                        let output =
                            fs::read_to_string(home.session_directory(number).join("output")).unwrap_or_default();
                        self.end_record(
                            ledger,
                            number,
                            if bounded { Exit::Bound } else { exit },
                            parse_tokens(&output),
                            self.clock.now().wall,
                        )?;
                        self.active.remove(&number);
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
            let result = home.transact(|ledger| Ok(self.start_pass(ledger, &place)))?;
            let cleanup = self.clean_passes();
            result?;
            cleanup?;
        }
        home.transact(|ledger| {
            let mut free = false;
            for place in &home.configuration.places {
                if !ledger.passes.iter().any(|pass| pass.state == PassState::Running && &pass.place == place) {
                    let directory = home.configuration.place_directory(&home.directory, place)?;
                    if pass_files::free_lock(&directory)?.is_some() {
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
                pass_files::changed_setups(&mut self.git, &pass.build, &pass_files::setups(&self.home, number))?
            };
            ledger.end_pass_session(session.number, exit, tokens, now, &self.home.configuration.owner, &recipes)?;
            self.reports.push(number);
        } else {
            ledger.end_session(number, exit, tokens, now, &self.home.configuration.owner)?;
        }
        self.trees.push(self.home.session_directory(number).join("tree"));
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
        let mut recorded = ledger.clone();
        let Some(start) = recorded
            .resume_pass(place, &configuration.surfaces, &configuration.agent.pass, now.wall)
            .or_else(|| recorded.start_pass(place, &configuration.surfaces, &configuration.agent.pass, now.wall))
        else {
            return Ok(());
        };
        let directory = configuration.place_directory(&self.home.directory, place)?;
        let Some(lock) = pass_files::free_lock(&directory)? else { return Ok(()) };
        pass_files::install(&self.home.builds.join(&start.build), &start.build, &directory)?;
        pass_files::write_setups(&mut self.git, &start.build, &pass_files::setups(&self.home, start.pass))?;
        *ledger = recorded;
        let command = self.agent_command(start.session, &configuration.agent.pass);
        let mut prepare = || -> Result<(), Refusal> {
            let directory = self.home.session_directory(start.session);
            fs::create_dir_all(&directory).map_err(|why| refused(format!("{}: {why}", directory.display())))?;
            let brief = pass_brief::render(&mut self.git, &self.home, &start, ledger)?;
            fs::write(&command.stdin, brief).map_err(|why| refused(format!("brief: {why}")))?;
            self.git.add_tree(&command.directory, &start.build)
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
        while let Some(&number) = self.reports.first() {
            self.home.write_pass_report(number)?;
            self.reports.remove(0);
        }
        self.clean_trees()
    }

    fn clean_trees(&mut self) -> Result<(), Refusal> {
        while let Some(tree) = self.trees.first() {
            self.git.remove_tree(tree)?;
            self.trees.remove(0);
        }
        self.git.prune()
    }
    fn delete_builds(&self, ledger: &mut Ledger, now: u64) -> Result<(), Refusal> {
        let mut builds: Vec<_> =
            ledger.deletable_builds().iter().map(|build| (build.order, build.commit.clone())).collect();
        builds.sort();
        for (_, commit) in builds {
            let directory = self.home.builds.join(&commit);
            match fs::remove_dir_all(&directory) {
                Ok(()) => (),
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
                Err(why) => return Err(refused(format!("{}: {why}", directory.display()))),
            }
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
                Ok(Some(exit)) => eprintln!("temper-qa: notify: {exit:?}"),
                Err(why) => {
                    eprintln!("temper-qa: notify: {why}");
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
            let prepare = || -> Result<(), Refusal> {
                fs::create_dir_all(&directory).map_err(|why| refused(format!("notify: {why}")))?;
                fs::write(&input, format!("{}: {}\n", line.area, line.words))
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
                Err(why) => eprintln!("temper-qa: notify: {why}"),
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
                eprintln!("temper-qa: {step}: {why}");
                let failure = self.failures.entry(step).or_insert(Failure { since: now.monotonic, lined: false });
                if !failure.lined && now.monotonic.saturating_sub(failure.since) >= 3600 {
                    match self.home.transact(|ledger| {
                        ledger.add_line(
                            &self.home.configuration.owner,
                            &format!("{step} has failed for an hour: {why}"),
                            now.wall,
                        );
                        Ok(())
                    }) {
                        Ok(()) => failure.lined = true,
                        Err(why) => eprintln!("temper-qa: recording {step} failure: {why}"),
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
