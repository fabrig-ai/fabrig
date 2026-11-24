//! Agents and scripted programs (temper/docs/design/testing/qa/contracts.md, 4).
use super::{Connection, Tree, path_io};
use crate::{
    CaseState, Exit, FindingState, Home, Refusal,
    events::Waker,
    faces::{
        CommandAnswer, DriveProcess, EnvironmentEntry, FilesFace, LockMode, Mode, OutputCommand, PathKind,
        ProcessCommand, ProcessOutput, Processes, RealFiles, RealProcesses, Signal, WriteMode, WriteOptions,
    },
    refused,
};
use std::{
    cell::Cell,
    collections::{BTreeMap, VecDeque},
    ffi::{OsStr, OsString},
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Clone, Copy)]
pub enum Agent {
    None,
    Case,
    Empty,
    Never,
    IgnoreTerm,
    Unstarted,
    Pass,
    Fail,
    Note,
    Gone,
    Blocked,
}
struct Process {
    command: ProcessCommand,
    agent: Agent,
    ended: bool,
    notify: bool,
}
/// A test-controlled exit for a holder or drive session. It runs until the test answers it.
#[derive(Clone, Default)]
pub struct Child(Rc<Cell<Option<Exit>>>);

impl Child {
    pub fn exit(&self, exit: Exit) {
        self.0.set(Some(exit));
    }
}

pub struct HolderCommand<'a> {
    pub log: &'a Path,
    pub ctl: bool,
    pub environment: &'a [EnvironmentEntry],
}

pub struct DriveStreams {
    pub child: Child,
    pub input: Connection,
    pub output: Connection,
}

type Program<F> = Box<dyn FnMut(&OutputCommand<'_>, &mut F) -> io::Result<ProcessOutput>>;
type StartHolder<F> = Box<dyn FnOnce(&HolderCommand<'_>, &mut F) -> Result<Child, Refusal>>;
type StartDrive<F> = Box<dyn FnOnce(&OutputCommand<'_>, &mut F) -> Result<DriveStreams, Refusal>>;
type EndHolder<F> = Box<dyn FnOnce(&mut F)>;

pub struct Agents<F = Tree> {
    files: F,
    real: Option<RealProcesses>,
    programs: BTreeMap<OsString, Program<F>>,
    holders: VecDeque<StartHolder<F>>,
    drives: VecDeque<StartDrive<F>>,
    end_actions: BTreeMap<PathBuf, VecDeque<EndHolder<F>>>,
    children: BTreeMap<u64, Child>,
    next_process: u64,
    pub ended_holders: Vec<PathBuf>,
    pub next: VecDeque<Agent>,
    pub commands: Vec<ProcessCommand>,
    pub notifications: Vec<Vec<u8>>,
    pub signals: Vec<(u64, Signal)>,
    pub bounded: Vec<(ProcessCommand, u64)>,
    pub command_bound: bool,
    pub answers: VecDeque<CommandAnswer>,
    processes: BTreeMap<u64, Process>,
    waker: Option<Waker>,
}
impl<F: Default> Default for Agents<F> {
    fn default() -> Self {
        Self::new(F::default())
    }
}

impl Agents<RealFiles> {
    pub fn on_disk(files: RealFiles) -> Self {
        Self { real: Some(RealProcesses::default()), ..Self::new(files) }
    }
}

impl<F> Agents<F> {
    pub fn new(files: F) -> Self {
        Self {
            files,
            real: None,
            programs: BTreeMap::new(),
            holders: VecDeque::new(),
            drives: VecDeque::new(),
            end_actions: BTreeMap::new(),
            children: BTreeMap::new(),
            next_process: 1,
            ended_holders: Vec::new(),
            next: VecDeque::new(),
            commands: Vec::new(),
            notifications: Vec::new(),
            signals: Vec::new(),
            bounded: Vec::new(),
            command_bound: false,
            answers: VecDeque::new(),
            processes: BTreeMap::new(),
            waker: None,
        }
    }

    /// Answer `run`, `output` and `version`. Lookup tries the command's full program, then its basename.
    pub fn register(
        &mut self,
        program: impl Into<OsString>,
        handler: impl FnMut(&OutputCommand<'_>, &mut F) -> io::Result<ProcessOutput> + 'static,
    ) {
        self.programs.insert(program.into(), Box::new(handler));
    }

    /// Answer the next holder launch, after the caller has released its lock probe.
    pub fn script_holder(
        &mut self,
        start: impl FnOnce(&HolderCommand<'_>, &mut F) -> Result<Child, Refusal> + 'static,
    ) {
        self.holders.push_back(Box::new(start));
    }

    /// Answer the next drive launch with its child and the holder's ends of the two streams.
    pub fn script_drive(
        &mut self,
        start: impl FnOnce(&OutputCommand<'_>, &mut F) -> Result<DriveStreams, Refusal> + 'static,
    ) {
        self.drives.push_back(Box::new(start));
    }

    /// Act on the next `end` sent at this place, e.g. feed the peer's reply and drop its lock.
    pub fn script_end_holder(&mut self, place: impl Into<PathBuf>, end: impl FnOnce(&mut F) + 'static) {
        self.end_actions.entry(place.into()).or_default().push_back(Box::new(end));
    }

    fn scripted(&mut self, command: &OutputCommand<'_>) -> Option<io::Result<ProcessOutput>> {
        let program = if self.programs.contains_key(command.program) {
            command.program
        } else {
            Path::new(command.program).file_name()?
        };
        Some(self.programs.get_mut(program)?(command, &mut self.files))
    }

    fn process_number(&mut self) -> u64 {
        let number = self.next_process;
        self.next_process += 1;
        number
    }

    fn start_child(&mut self, child: Child) -> u64 {
        let number = self.process_number();
        self.children.insert(number, child);
        number
    }

    pub fn finish(&mut self, number: u64, agent: Agent) {
        self.processes.get_mut(&number).expect("running process").agent = agent;
        if let Some(waker) = &self.waker {
            waker.poke();
        }
    }
}
impl<F: FilesFace> Processes for Agents<F> {
    type Connection = Connection;

    fn output(&mut self, command: &OutputCommand<'_>) -> io::Result<ProcessOutput> {
        let Some(output) = self.scripted(command) else {
            return self.real.as_mut().ok_or(ErrorKind::NotFound)?.output(command);
        };
        let ProcessOutput { exit, status, stdout, stderr } = output?;
        if let Some(log) = command.log {
            let mut log = self.files.open_write(log, WriteOptions::private(WriteMode::Append))?;
            log.write_all(&stdout)?;
            log.write_all(&stderr)?;
            Ok(ProcessOutput { exit, status, stdout: Vec::new(), stderr: Vec::new() })
        } else {
            Ok(ProcessOutput { exit, status, stdout, stderr })
        }
    }
    fn version(
        &mut self,
        executable: &Path,
        capture: &Path,
        environment: &[EnvironmentEntry],
    ) -> Result<String, Refusal> {
        let command = OutputCommand {
            program: executable.as_os_str(),
            arguments: &["--version".into()],
            directory: None,
            environment,
            log: None,
        };
        if let Some(output) = self.scripted(&command) {
            return Ok(String::from_utf8_lossy(&path_io(executable, output)?.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .into());
        }
        let real = self.real.as_mut().ok_or_else(|| unregistered(command.program))?;
        real.version(executable, capture, environment)
    }
    fn holder(
        &mut self,
        log: &Path,
        ctl: bool,
        environment: &[EnvironmentEntry],
        release: impl FnOnce(),
    ) -> Result<u64, Refusal> {
        let start = self.holders.pop_front().ok_or_else(|| refused("no scripted holder"))?;
        path_io(log, self.files.write(log, b"", WriteOptions::private(WriteMode::Replace), false))?;
        release();
        let child = start(&HolderCommand { log, ctl, environment }, &mut self.files)?;
        Ok(self.start_child(child))
    }
    fn drive(
        &mut self,
        program: &OsStr,
        arguments: &[OsString],
        environment: &[EnvironmentEntry],
    ) -> Result<DriveProcess<Connection>, Refusal> {
        let start = self.drives.pop_front().ok_or_else(|| refused("no scripted drive"))?;
        let command = OutputCommand { program, arguments, directory: None, environment, log: None };
        let DriveStreams { child, input, output } = start(&command, &mut self.files)?;
        Ok(DriveProcess { process: self.start_child(child), input, output })
    }

    fn end_holder(&mut self, place: PathBuf, waker: Waker) {
        self.ended_holders.push(place.clone());
        if let Some(real) = self.real.as_mut() {
            real.end_holder(place, waker);
            return;
        }
        let Ok(Some(mut connection)) = self.files.connect(&place.join("drive.sock")) else { return };
        if connection.write_all(b"end\n").is_ok() {
            if let Some(end) = self.end_actions.get_mut(&place).and_then(VecDeque::pop_front) {
                end(&mut self.files);
            }
            // There is no background waiter. A scripted end settles during this call.
            match connection.read_to_end(&mut Vec::new()) {
                Ok(_) if matches!(self.files.lock(&place.join("drive.lock"), LockMode::Try, false), Ok(None)) => return,
                Err(why) if why.kind() == ErrorKind::WouldBlock => return,
                Ok(_) | Err(_) => (),
            }
        }
        waker.poke();
    }

    fn wake_on_exit(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }
    fn run(&mut self, command: &ProcessCommand, seconds: u64) -> Result<CommandAnswer, Refusal> {
        self.bounded.push((command.clone(), seconds));
        if let Some(answer) = self.answers.pop_front() {
            return Ok(answer);
        }
        if self.command_bound {
            return Ok(CommandAnswer { exit: Exit::Bound, last_stdout: None });
        }
        let (program, arguments) = command.arguments.split_first().ok_or_else(|| refused("command needs a program"))?;
        let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        let request = OutputCommand {
            program: OsStr::new(program),
            arguments: &arguments,
            directory: Some(&command.directory),
            environment: &command.environment,
            log: Some(&command.output),
        };
        let Some(output) = self.scripted(&request) else {
            let real = self.real.as_mut().ok_or_else(|| unregistered(request.program))?;
            real.environment.clone_from(&command.environment);
            return real.run(command, seconds);
        };
        let ProcessOutput { exit, stdout, mut stderr, .. } =
            output.map_err(|why| refused(format!("{program}: {why}")))?;
        let text = String::from_utf8_lossy(&stdout);
        let last_stdout =
            text.lines().rev().find(|line| !line.trim().is_empty()).map(|line| line.trim_end_matches('\r').to_owned());
        stderr.extend_from_slice(&stdout);
        path_io(
            &command.output,
            self.files.write(&command.output, &stderr, WriteOptions::private(WriteMode::Replace), false),
        )?;
        Ok(CommandAnswer { exit, last_stdout })
    }
    fn start(&mut self, command: &ProcessCommand) -> Result<u64, Refusal> {
        self.commands.push(command.clone());
        assert!(self.files.facts(&command.stdin, true).is_ok_and(|facts| facts.kind == PathKind::File));
        let number = self.process_number();
        let notify = command.arguments.first().is_some_and(|arg| arg == "notify");
        let agent = if notify { Agent::Empty } else { self.next.pop_front().unwrap_or(Agent::Never) };
        match agent {
            Agent::Unstarted => return Err(Refusal("agent not found".into())),
            Agent::None
            | Agent::Case
            | Agent::Empty
            | Agent::Never
            | Agent::IgnoreTerm
            | Agent::Pass
            | Agent::Fail
            | Agent::Note
            | Agent::Gone
            | Agent::Blocked => (),
        }
        if notify {
            self.notifications.push(self.files.read(&command.stdin).expect("notify stdin"));
        }
        self.processes.insert(number, Process { command: command.clone(), agent, ended: false, notify });
        if !matches!(agent, Agent::Never | Agent::IgnoreTerm)
            && let Some(waker) = &self.waker
        {
            waker.poke();
        }
        Ok(number)
    }
    fn poll(&mut self, process: u64) -> Result<Option<Exit>, Refusal> {
        if let Some(child) = self.children.get(&process) {
            return Ok(child.0.get());
        }
        let process = self.processes.get_mut(&process).expect("process");
        if process.ended {
            return Ok(Some(Exit::Signal(15)));
        }
        if process.notify {
            process.ended = true;
            return Ok(Some(Exit::Code(9)));
        }
        match process.agent {
            Agent::Never | Agent::IgnoreTerm => return Ok(None),
            Agent::Unstarted => unreachable!("not started"),
            Agent::None
            | Agent::Case
            | Agent::Empty
            | Agent::Pass
            | Agent::Fail
            | Agent::Note
            | Agent::Gone
            | Agent::Blocked => (),
        }
        let env = |key: &str| {
            process
                .command
                .environment
                .iter()
                .find(|(name, _)| name.as_ref() == key.as_bytes())
                .map(|(_, value)| String::from_utf8(value.to_vec()).expect("env"))
        };
        let home = Home::open(Path::new(&env("TEMPER_INTERNAL_QA_HOME").expect("home env")), &mut self.files)
            .expect("agent's home");
        let number = env("TEMPER_INTERNAL_QA_SESSION").expect("session env").parse().expect("number");
        home.transact(&mut self.files, |ledger, _| {
            match process.agent {
                Agent::None => {
                    ledger.answer_none(number, "internal refactor")?;
                }
                Agent::Case => {
                    ledger.add_case(number, &super::case(), 0)?;
                }
                Agent::Pass | Agent::Blocked => {
                    let pass = ledger.pass_command_session(number)?.pass.expect("pass");
                    let cases: Vec<_> = ledger
                        .cases
                        .iter()
                        .filter(|case| case.pass == Some(pass) && case.state == CaseState::Assigned)
                        .map(|case| case.number)
                        .collect();
                    for case in cases {
                        let result = if matches!(process.agent, Agent::Pass) { "passed" } else { "blocked" };
                        ledger.record_outcome(number, case, result, "agent's outcome", &[], 0)?;
                    }
                }
                Agent::Fail => {
                    let pass = ledger.pass_command_session(number)?.pass.expect("pass");
                    let case = ledger
                        .cases
                        .iter()
                        .find(|case| case.pass == Some(pass) && case.state == CaseState::Assigned)
                        .expect("assigned case")
                        .number;
                    let finding = ledger.record_finding(number, &super::passes::finding(case), &[], "product", 0)?;
                    ledger.record_outcome(number, case, "failed", "no goal", &[finding.number], 0)?;
                }
                Agent::Note => {
                    ledger.record_note(number, "off the path", 0)?;
                }
                Agent::Gone => {
                    let pass = ledger.pass_command_session(number)?.pass.expect("pass");
                    let findings: Vec<_> = ledger
                        .findings
                        .iter()
                        .filter(|finding| finding.pass == pass && finding.state == FindingState::Open)
                        .map(|finding| finding.number)
                        .collect();
                    for finding in findings {
                        ledger.record_check(number, finding, "gone", "fixed", 0)?;
                    }
                }
                Agent::Empty => (),
                Agent::Never | Agent::IgnoreTerm | Agent::Unstarted => unreachable!("not ending"),
            }
            Ok(())
        })?;
        let output = &process.command.output;
        let options = WriteOptions { way: WriteMode::Replace, mode: Mode::Default, read: false };
        path_io(output, self.files.write(output, b"agent output\ntokens 1234\n\n", options, false))?;
        process.ended = true;
        Ok(Some(Exit::Code(0)))
    }
    fn signal(&mut self, process: u64, signal: Signal) -> Result<(), Refusal> {
        self.signals.push((process, signal));
        if self.children.contains_key(&process) {
            return Ok(());
        }
        let process = self.processes.get_mut(&process).expect("process");
        match (process.agent, signal) {
            (Agent::IgnoreTerm, Signal::Term) => (),
            (
                Agent::None
                | Agent::Case
                | Agent::Empty
                | Agent::Never
                | Agent::IgnoreTerm
                | Agent::Unstarted
                | Agent::Pass
                | Agent::Fail
                | Agent::Note
                | Agent::Gone
                | Agent::Blocked,
                Signal::Kill,
            )
            | (
                Agent::None
                | Agent::Case
                | Agent::Empty
                | Agent::Never
                | Agent::Unstarted
                | Agent::Pass
                | Agent::Fail
                | Agent::Note
                | Agent::Gone
                | Agent::Blocked,
                Signal::Term,
            ) => {
                process.ended = true;
            }
        }
        Ok(())
    }
}

fn unregistered(program: &OsStr) -> Refusal {
    refused(format!("{}: no scripted program", program.to_string_lossy()))
}
