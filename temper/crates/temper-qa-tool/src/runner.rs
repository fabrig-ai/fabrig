//! A single-home service, with five saved steps per tick (qa/README.md, 4–5).
use crate::faces::{ClockFace, Environment, GitFace, Moment, ProcessCommand, Processes, Signal};
use crate::{Exit, Home, Ledger, Refusal, brief::Inputs, refused, watch::watch};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions, TryLockError},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};

struct Active {
    session: u64,
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
    active: Option<Active>,
    trees: Vec<PathBuf>,
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
            active: None,
            trees: Vec::new(),
            kills: Vec::new(),
            notifications: Vec::new(),
            failures: BTreeMap::new(),
        };
        let now = run.clock.now();
        run.trees = run.home.transact(|ledger| {
            let numbers: Vec<_> = ledger.running_sessions().iter().map(|session| session.number).collect();
            let mut trees = Vec::new();
            for number in numbers {
                ledger.end_session(number, Exit::Lost, None, now.wall, &run.home.configuration.owner)?;
                trees.push(run.home.session_directory(number).join("tree"));
            }
            Ok(trees)
        })?;
        let result = run.clean_trees();
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
                    Ok(()) if self.active.as_ref().is_some_and(|active| active.exit.is_some()) => break,
                    Ok(()) => (),
                    Err(why) => {
                        eprintln!("temper-qa: triage: {why}");
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
        if let Some(active) = &mut self.active
            && active.exit.is_none()
        {
            active.exit = self.processes.poll(active.process)?;
            if active.exit.is_none()
                && !active.bounded
                && now.saturating_sub(active.started) >= self.home.configuration.bounds.triage
            {
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
        if let Some(active) = &self.active
            && let Some(exit) = active.exit
        {
            let number = active.session;
            let output = fs::read_to_string(self.home.session_directory(number).join("output")).unwrap_or_default();
            ledger.end_session(
                number,
                if active.bounded { Exit::Bound } else { exit },
                parse_tokens(&output),
                now.wall,
                &self.home.configuration.owner,
            )?;
            self.trees.push(self.home.session_directory(number).join("tree"));
            self.active = None;
        }
        self.clean_trees()?;
        if self.active.is_some() {
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
            let mut environment = self.environment.clone();
            environment.retain(|(name, _)| {
                name.as_ref() != crate::HOME.name.as_bytes() && name.as_ref() != crate::SESSION.name.as_bytes()
            });
            environment
                .push((crate::HOME.name.as_bytes().into(), self.home.directory.as_os_str().as_encoded_bytes().into()));
            environment.push((crate::SESSION.name.as_bytes().into(), number.to_string().into_bytes().into()));
            Ok(ProcessCommand {
                arguments: self
                    .home
                    .configuration
                    .agent
                    .command
                    .iter()
                    .map(|argument| argument.replace("{effort}", &self.home.configuration.agent.triage))
                    .collect(),
                directory: tree.clone(),
                stdin: brief,
                output: directory.join("output"),
                environment,
            })
        };
        let result = start().and_then(|command| {
            self.git.add_tree(&tree, &landing.commit)?;
            self.processes.start(&command)
        });
        match result {
            Ok(process) => {
                self.active =
                    Some(Active { session: number, process, started: now.monotonic, bounded: false, exit: None });
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
