//! Drives the box through its pipes, signals and supervised process tree.
use super::{Process, StdoutReader, patience::Patience, printed_link};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Expectation, Mode, Streams},
    real,
};
use std::{
    cell::Cell,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    rc::Rc,
};

/// The patience that ends a hung box story, never its guide: a story's time is measured idle
/// (testing/checks.md 1). Keep a duration under test separate from this horizon.
pub const BOX_DEADLINE: Duration = Duration::from_secs(240);

/// The box's own log of this run, empty before its first line (protocol/setup/box.md, 2).
#[must_use]
pub fn box_log(data: &Path) -> String {
    match std::fs::read_to_string(data.join("box/logs/box.log")) {
        Ok(log) => log,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => panic!("box log: {error}"),
    }
}

/// Serve has listened on the clients' listener this run: the moment the first link used to print (protocol/setup/box.md, 2).
#[must_use]
pub fn serve_listening(data: &Path) -> bool {
    match std::fs::read_to_string(data.join("box/logs/serve.log")) {
        Ok(log) => log.contains("listening clients "),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => panic!("serve log: {error}"),
    }
}

/// What the referee does after the link arrives and serve listens.
#[derive(Clone, Copy, Debug)]
pub enum Plan {
    /// Probe Forgejo and ask the box to stop.
    Stop,
    /// Wait for the box's own exit; send it no signal.
    Exit,
    /// Kill Forgejo, wait for its replacement, and check the engine before stopping.
    KillForgejo,
}
/// Evidence retained after every process and pipe settles.
pub struct Outcome {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub forgejo_at_link: Option<u16>,
    pub left: Option<Vec<u32>>,
    pub forgejo_pids: Option<(u32, u32)>,
    pub engine_served: Option<bool>,
}
#[derive(Clone, Copy)]
enum Phase {
    AwaitingServe,
    Linked,
    Killed { engine: u32, forgejo: u32 },
    Stopping,
}
struct Judge {
    data: Option<PathBuf>,
    plan: Plan,
    forgejo: Option<SocketAddr>,
    forgejo_at_link: Rc<Cell<Option<u16>>>,
    forgejo_pids: Rc<Cell<Option<(u32, u32)>>>,
    engine_served: Rc<Cell<Option<bool>>>,
    temper_program: PathBuf,
    forgejo_program: Option<PathBuf>,
    phase: Phase,
    next_probe: Time,
    cleanup: bool,
    deadline: Patience,
    settled: bool,
}
impl Judge {
    fn advance(&mut self, binary: &mut Binary, address: SocketAddr) {
        let pids = &binary.tree_status().live_pids;
        match self.phase {
            Phase::AwaitingServe => {
                // Plain serve prints its own link once listening; a box prints before it starts.
                if self.data.as_deref().is_none_or(serve_listening) {
                    self.phase = Phase::Linked;
                    self.advance(binary, address);
                }
            }
            Phase::Linked => {
                self.forgejo_at_link.set(self.forgejo.and_then(|address| status(address, "/api/v1/version")));
                match self.plan {
                    Plan::Stop => self.stop(binary),
                    Plan::Exit => {}
                    Plan::KillForgejo => {
                        if let Some(engine) = find(pids, &self.temper_program, b"serve")
                            && let Some(forgejo) =
                                find(pids, self.forgejo_program.as_deref().expect("Forgejo restart plan"), b"web")
                        {
                            let killed = std::process::Command::new("/usr/bin/kill")
                                .args(["-KILL", &forgejo.to_string()])
                                .status()
                                .expect("kill Forgejo");
                            assert!(killed.success(), "Forgejo's pid accepted the kill");
                            self.phase = Phase::Killed { engine, forgejo };
                        }
                    }
                }
            }
            Phase::Killed { engine, forgejo } => {
                if let Some(restarted) =
                    find(pids, self.forgejo_program.as_deref().expect("Forgejo restart plan"), b"web")
                    && restarted != forgejo
                    && self.forgejo.and_then(|address| status(address, "/api/v1/version")) == Some(200)
                {
                    self.forgejo_pids.set(Some((forgejo, restarted)));
                    self.engine_served.set(Some(
                        find(pids, &self.temper_program, b"serve") == Some(engine) && status(address, "/").is_some(),
                    ));
                    self.stop(binary);
                }
            }
            Phase::Stopping => {}
        }
    }
    fn stop(&mut self, binary: &mut Binary) {
        if binary.exit_status().is_none() {
            binary.signal(Signal::Terminate);
        }
        self.phase = Phase::Stopping;
    }
}
pub fn find(pids: &[u32], program: &Path, argument: &[u8]) -> Option<u32> {
    pids.iter().copied().find(|pid| {
        std::fs::read_link(format!("/proc/{pid}/exe")).ok().as_deref() == Some(program)
            && std::fs::read(format!("/proc/{pid}/cmdline"))
                .is_ok_and(|bytes| bytes.split(|b| *b == 0).any(|arg| arg == argument))
    })
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        let engine = printed_link(&stdout.bytes)
            .and_then(|link| std::str::from_utf8(link).ok()?.strip_prefix("http://")?.split('/').next()?.parse().ok());
        let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
        if now >= self.next_probe {
            if let Some(engine) = engine {
                self.advance(binary, engine);
            }
            self.next_probe = now.saturating_add(Duration::from_millis(200));
        }
        let left = binary.tree_status().live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty());
        if !self.cleanup && left && !binary.is_empty() {
            binary.signal_tree(Signal::Kill);
            self.cleanup = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = procs.iter().all(Host::is_empty);
        if !self.settled
            && let Some(figures) = self.deadline.failed()
        {
            let Process::Binary(binary) = &procs[0] else { panic!("binary") };
            panic!("box did not finish and settle: {}; {figures}", String::from_utf8_lossy(binary.stderr()));
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.cleanup || self.settled { None } else { self.deadline.until().map(|until| until.min(self.next_probe)) }
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Opt in only for a story whose assertions read a step's diagnostic bytes.
#[must_use]
pub fn echo_steps(mut flags: Vec<String>) -> Vec<String> {
    flags.extend(["--echo".into(), "steps".into()]);
    flags
}
#[must_use]
pub fn run_echo_steps(directory: &Path, forgejo: Option<SocketAddr>, plan: Plan, deadline: Duration) -> Outcome {
    let mut command = Command {
        program: skein_scratch::paths::binary("temper-box"),
        arguments: super::configuration::flags(directory),
        directory: directory.into(),
        environment: Vec::new(),
    };
    command.arguments.extend(["--echo".into(), "steps".into()]);
    run_command(command, forgejo, plan, deadline)
}
/// Starts the shipped box and drives its referee within the supplied deadline.
#[must_use]
pub fn run(directory: &Path, forgejo: Option<SocketAddr>, plan: Plan, deadline: Duration) -> Outcome {
    run_command(
        Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: super::configuration::flags(directory),
            directory: directory.into(),
            environment: Vec::new(),
        },
        forgejo,
        plan,
        deadline,
    )
}
/// Drives the same box referee with an isolation wrapper around the shipped binary.
#[must_use]
pub fn run_command(command: Command, forgejo: Option<SocketAddr>, plan: Plan, deadline: Duration) -> Outcome {
    let data = if command.arguments.first().is_some_and(|word| word == "serve") {
        None
    } else {
        command.arguments.windows(2).find(|pair| pair[0] == "--data").map(|pair| PathBuf::from(&pair[1]))
    };
    let clock = Clock::new();
    let mut binary =
        Binary::start_with_tree(dying_with_test(command), Mode::Pipes, 262_144, Expectation::Supervise, false)
            .expect("temper box starts");
    let streams = binary.take_streams().expect("binary streams");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let forgejo_at_link = Rc::new(Cell::new(None));
    let forgejo_pids = Rc::new(Cell::new(None));
    let engine_served = Rc::new(Cell::new(None));
    let forgejo_program = if matches!(plan, Plan::KillForgejo) {
        Some(std::fs::canonicalize(super::owned::binaries().0).expect("Forgejo path"))
    } else {
        None
    };
    let mut world = real::World::new(Judge {
        data,
        plan,
        forgejo,
        forgejo_at_link: forgejo_at_link.clone(),
        forgejo_pids: forgejo_pids.clone(),
        engine_served: engine_served.clone(),
        temper_program: std::fs::canonicalize(skein_scratch::paths::binary("temper-box")).expect("temper path"),
        forgejo_program,
        phase: Phase::AwaitingServe,
        next_probe: Time::ZERO,
        cleanup: false,
        deadline: Patience::watching(
            "the box's action",
            Duration::from_nanos(
                deadline.as_nanos().saturating_sub(Duration::from_secs(10).as_nanos()).max(deadline.as_nanos() / 2),
            ),
            &binary,
        ),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Stdout(Box::new(StdoutReader::new(input, output))));
    let ended = world.run(&clock, deadline);
    let Process::Binary(binary) = &ended.procs[0] else { panic!("binary") };
    let Process::Stdout(stdout) = &ended.procs[1] else { panic!("stdout reader") };
    Outcome {
        exit: binary.exit_status(),
        stdout: stdout.bytes.clone(),
        stderr: binary.stderr().to_vec(),
        forgejo_at_link: forgejo_at_link.get(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        forgejo_pids: forgejo_pids.get(),
        engine_served: engine_served.get(),
    }
}
/// The status line of a plain request, or none when no server answers.
#[must_use]
pub fn status(address: SocketAddr, target: &str) -> Option<u16> {
    match status_reply(address, target) {
        Ok(status) => status,
        Err(error) => {
            if matches!(error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) {
                eprintln!("HTTP status probe did not answer: {error}");
            }
            None
        }
    }
}
fn status_reply(address: SocketAddr, target: &str) -> std::io::Result<Option<u16>> {
    let within = Duration::from_millis(100);
    let mut connect = Patience::arm("the box status probe's connection", within);
    let mut socket = loop {
        let result = match connect.socket_timeout() {
            Some(timeout) => TcpStream::connect_timeout(&address, timeout),
            None => TcpStream::connect(address),
        };
        match result {
            Ok(socket) => break socket,
            Err(error) => connect.socket_retry(error)?,
        }
    };
    let mut write = Patience::arm("the box status probe's request", within);
    let request = format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    let mut written = 0;
    while written < request.len() {
        socket.set_write_timeout(write.socket_timeout())?;
        match socket.write(&request.as_bytes()[written..]) {
            Ok(0) => return Err(std::io::ErrorKind::WriteZero.into()),
            Ok(count) => written += count,
            Err(error) => write.socket_retry(error)?,
        }
    }
    let mut read = Patience::arm("the box status probe's reply", within);
    let mut bytes = [0_u8; 128];
    let length = loop {
        socket.set_read_timeout(read.socket_timeout())?;
        match socket.read(&mut bytes) {
            Ok(count) => break count,
            Err(error) => read.socket_retry(error)?,
        }
    };
    Ok(std::str::from_utf8(&bytes[..length]).ok().and_then(|line| line.split_whitespace().nth(1)?.parse().ok()))
}

fn action_deadline(what: &'static str, budget: Duration, binary: &Binary) -> Patience {
    let within = Duration::from_nanos(budget.as_nanos().saturating_sub(Duration::from_secs(10).as_nanos()));
    Patience::watching(what, within, binary)
}

/// The first run's evidence, including a check made while the box still serves.
pub struct StoryOutcome<T> {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub left: Option<Vec<u32>>,
    pub finished: bool,
    pub dormant_checked: bool,
    pub checked: T,
}
struct StoryJudge<'a, F, T> {
    check: Option<F>,
    ending: Signal,
    checked: &'a Cell<Option<T>>,
    dormant_checked: &'a Cell<bool>,
    logs: PathBuf,
    stopping: bool,
    cleanup: bool,
    settled: bool,
    deadline: Patience,
}
impl<F: FnOnce(&Binary) -> T, T> Referee<Process> for StoryJudge<'_, F, T> {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        let finished = person.finished && person.client.as_ref().is_some_and(Host::is_empty);
        let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
        if !self.stopping && finished {
            self.checked.set(Some(self.check.take().expect("one check before stopping")(binary)));
            if self.ending == Signal::Kill {
                binary.signal_tree(Signal::Kill);
                self.cleanup = true;
            } else {
                binary.signal(Signal::Terminate);
            }
            self.stopping = true;
        }
        let left = binary.tree_status().live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty());
        if !self.cleanup && left && !binary.is_empty() {
            binary.signal_tree(Signal::Kill);
            self.cleanup = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = self.stopping && procs.iter().all(Host::is_empty);
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        let stderr = String::from_utf8_lossy(binary.stderr());
        if person.choosing_playground() {
            for name in ["fake-llm.log", "fake-forge-1.log", "prepare-1.log"] {
                let path = self.logs.join(name);
                assert!(
                    std::fs::read(&path).is_ok_and(|bytes| bytes.is_empty()) || !path.exists(),
                    "the demo's fakes stay dormant at the welcome: {}",
                    path.display()
                );
            }
            self.dormant_checked.set(true);
        }
        if person.failed.is_some() {
            for node in person.client.as_ref().expect("native client").tree().nodes() {
                eprintln!(
                    "{:?}: {} {}",
                    node.role,
                    String::from_utf8_lossy(node.name.as_deref().unwrap_or_default()),
                    node.text.iter().map(|run| String::from_utf8_lossy(&run.text)).collect::<String>()
                );
            }
        }
        assert!(person.failed.is_none(), "{:?}; temper box stderr: {stderr}", person.failed);
        assert!(binary.exit_status().is_none() || self.stopping, "temper box ended before the person: {stderr}");
        assert!(
            binary
                .exit_status()
                .is_none_or(|exit| exit == if self.ending == Signal::Kill { Exit::Signal(9) } else { Exit::Code(0) }),
            "temper box failed: {stderr}"
        );
        assert!(!binary.stderr_overflowed(), "temper box overflowed stderr: {stderr}");
        if !self.settled
            && let Some(figures) = self.deadline.failed()
        {
            panic!("native person did not settle: {stderr}; {figures}");
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.settled { None } else { self.deadline.until() }
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Runs the shipped box on its pipes with a fresh client rooted at `client`.
/// Checks the forge after the person finishes and their client settles, before SIGTERM.
#[must_use]
pub fn run_story<T>(
    directory: &Path,
    flags: Vec<String>,
    client: &Path,
    script: super::person::Script,
    deadline: Duration,
    check: impl FnOnce() -> T,
) -> StoryOutcome<T> {
    run_story_with_box(directory, flags, client, StoryPlan { script, kill: false }, deadline, |_| check())
}

/// The person's script and the signal sent after the live check.
pub struct StoryPlan {
    pub script: super::person::Script,
    pub kill: bool,
}
/// Checks the live supervised box after the person's scripted change lands.
#[must_use]
pub fn run_story_with_box<T>(
    directory: &Path,
    flags: Vec<String>,
    client: &Path,
    plan: StoryPlan,
    deadline: Duration,
    check: impl FnOnce(&Binary) -> T,
) -> StoryOutcome<T> {
    let StoryPlan { script, kill } = plan;
    let data = flags.windows(2).find(|pair| pair[0] == "--data").expect("explicit data directory");
    let data = PathBuf::from(&data[1]);
    let logs = data.join("box/logs");
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        dying_with_test(Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: flags.into_iter().map(Into::into).collect(),
            directory: directory.into(),
            // Init's optional git/claude discovery is exercised without PATH.
            // The demo uses the written link to this binary, and needs neither.
            environment: Vec::new(),
        }),
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("temper box starts");
    let streams = binary.take_streams().expect("binary pipes");
    let mut fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let root = skein_shell::open_root(client).expect("client's parent root");
    fds.push(root);
    let checked = Cell::new(None);
    let dormant_checked = Cell::new(false);
    let mut world = real::World::new(StoryJudge {
        check: Some(check),
        checked: &checked,
        dormant_checked: &dormant_checked,
        ending: if kill { Signal::Kill } else { Signal::Terminate },
        logs,
        stopping: false,
        cleanup: false,
        settled: false,
        deadline: action_deadline("the box person's settlement", deadline, &binary),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Person(Box::new(super::person::Person::new(root, input, output, script).with_serve_data(&data)))
    });
    let ended = world.run(&clock, deadline);
    assert!(ended.procs.iter().all(Host::is_empty), "all real io settles");
    let Process::Binary(binary) = &ended.procs[0] else { panic!("binary") };
    let Process::Person(person) = &ended.procs[1] else { panic!("native person") };
    StoryOutcome {
        exit: binary.exit_status(),
        stdout: person.stdout().to_vec(),
        stderr: binary.stderr().to_vec(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        finished: person.finished,
        dormant_checked: dormant_checked.get(),
        checked: checked.take().expect("the finished person's forge check ran"),
    }
}

/// A box driven through ctl, with evidence taken before its stop.
pub struct CtlOutcome<T> {
    pub model: Option<super::forge_land::Seen>,
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub left: Option<Vec<u32>>,
    pub calls: Vec<super::Ended>,
    pub checked: Option<T>,
    pub visits: Vec<Vec<super::sign_in_browser::Hop>>,
}
struct CtlJudge<'a, F, T> {
    data: PathBuf,
    resuming_playground: Option<[PathBuf; 2]>,
    fail_on_panic: Option<()>,
    expected_exits: Vec<(usize, u8)>,
    release_answers: Option<usize>,
    next_release: Option<Time>,
    restart_at: Option<Time>,
    face: Option<super::forge_land::FaceProcs>,
    visit: Visit,
    tab: Option<usize>,
    tab_visit: Visit,
    visits: &'a Cell<Vec<Vec<super::sign_in_browser::Hop>>>,
    reply: Option<super::forge_land::Reply>,
    on_link: Option<Box<dyn FnOnce()>>,
    check: Option<F>,
    checked: &'a Cell<Option<T>>,
    stopping: bool,
    cleanup: bool,
    settled: bool,
    deadline: Patience,
    cleanup_deadline: Time,
}
pub fn visit_address(call: &super::Ended) -> Option<Vec<u8>> {
    super::sign_in::authorization_address(call).map(<[u8]>::to_vec).or_else(|| {
        let answer: serde_json::Value = serde_json::from_slice(&call.stdout).ok()?;
        Some(answer.pointer("/outcome/done/document/account_sign_in_started/address")?.as_str()?.as_bytes().to_vec())
    })
}

enum Visit {
    AwaitingAddress,
    Handed,
}
fn report_face_stop(calls: &super::ctl::Calls, stderr: &[u8], failed: bool, expired: bool) {
    eprintln!("face box stop: idle {}, failed {failed}, expired {expired}, calls {}", calls.idle(), calls.ends().len());
    for (index, call) in calls.ends().iter().enumerate().rev().take(3) {
        eprintln!(
            "face call {index}: {:?}; stdout {}; stderr {}",
            call.exit,
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    eprintln!("face box stderr: {}", String::from_utf8_lossy(stderr));
}
fn release_llm(calls: &mut super::ctl::Calls, peers: &mut [Process], release_answers: &mut Option<usize>) {
    use skein_fake_peers::llm::Observation;
    let Some((name, cue)) = calls.llm_release() else { return };
    let peer = peers
        .iter_mut()
        .find_map(|proc| match proc {
            Process::Peer(peer) => Some(peer),
            Process::Calls(_)
            | Process::Binary(_)
            | Process::Stdout(_)
            | Process::Person(_)
            | Process::Issuer(_)
            | Process::Visitor(_) => None,
        })
        .expect("scripted model owns the hold");
    let observations = peer.observations();
    let before = *release_answers.get_or_insert(observations.len());
    let answered = observations.iter().enumerate().skip(before).any(|(index, event)| {
        let Observation::Answered { connection: Some(connection), .. } = event else { return false };
        observations[..index]
            .iter()
            .rev()
            .find_map(|event| {
                if let Observation::Query { connection: queried, query } = event
                    && queried == connection
                {
                    Some(skein_lib::bytes::find(&query.system, cue).is_some() || super::llm::request_has(query, cue))
                } else {
                    None
                }
            })
            .unwrap_or(false)
    });
    if answered {
        eprintln!("scripted response released: {}", String::from_utf8_lossy(name));
        calls.llm_released();
        *release_answers = None;
    } else {
        // The review task can be visible before its model reaches the hold.
        peer.release(name).expect("release scripted review");
    }
}
fn restart_engine(
    calls: &mut super::ctl::Calls,
    binary: &mut Binary,
    now: Time,
    restart_at: &mut Option<Time>,
    expected_exits: &mut Vec<(usize, u8)>,
) {
    if calls.restarting_engine() {
        // Let the started caller enter its wait before stopping the engine.
        // This is an action's schedule, not a failure deadline.
        let at = *restart_at.get_or_insert(now.saturating_add(Duration::from_secs(1)));
        if now >= at {
            let engine = find(&binary.tree_status().live_pids, &skein_scratch::paths::binary("temper-box"), b"serve")
                .expect("box engine");
            let status = std::process::Command::new("/usr/bin/kill")
                .args(["-TERM", &engine.to_string()])
                .status()
                .expect("stop engine");
            assert!(status.success(), "engine accepts a clean stop");
            let index = calls.engine_restarting();
            expected_exits.push((index, 3));
            *restart_at = None;
        }
    }
}
impl<F, T> CtlJudge<'_, F, T> {
    fn resume_playground(&mut self, binary: &Binary) {
        if find(&binary.tree_status().live_pids, &skein_scratch::paths::binary("temper-box"), b"serve").is_some()
            && let Some([config, client]) = self.resuming_playground.take()
        {
            let parent = super::playground::interrupt_reset(binary, &config, &client);
            let continued = std::process::Command::new("/usr/bin/kill")
                .args(["-CONT", &parent.to_string()])
                .status()
                .expect("resume box parent");
            assert!(continued.success());
        }
    }

    fn bound(&mut self, calls: &super::ctl::Calls, binary: &Binary, failed: bool) {
        if !self.stopping
            && !calls.idle()
            && !failed
            && binary.exit_status().is_none()
            && let Some(figures) = self.deadline.failed()
        {
            if let Some(read) = calls.last_unmet() {
                eprintln!("last unmet ctl read: {}", String::from_utf8_lossy(&read.stdout));
            }
            report_face_stop(calls, binary.stderr(), false, true);
            panic!("box ctl story did not finish: {figures}");
        }
    }
}
impl<F: FnOnce(&Binary) -> T, T> Referee<Process> for CtlJudge<'_, F, T> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Binary(binary) = &procs[0] else { panic!("box") };
        self.resume_playground(binary);
        let Process::Calls(calls) = &procs[1] else { panic!("ctl") };
        let failed = calls.ends().iter().enumerate().any(|(index, call)| {
            let expected = self.expected_exits.iter().find(|(at, _)| *at == index).map_or(0, |(_, code)| *code);
            calls.observed_at()[index].is_some() && call.exit != Some(Exit::Code(expected))
        });
        if let Some(face) = &self.face {
            let Process::Calls(calls) = &mut procs[1] else { panic!("ctl") };
            if matches!(self.visit, Visit::AwaitingAddress)
                && let Some(address) = calls.ends().iter().find_map(super::sign_in::authorization_address)
            {
                let address = address.to_vec();
                let Process::Visitor(visitor) = &mut procs[face.visitor] else { panic!("visitor") };
                visitor.hand(&address);
                self.visit = Visit::Handed;
            }
            let Process::Calls(calls) = &mut procs[1] else { panic!("ctl") };
            if !failed && let Some(reply) = &mut self.reply {
                reply.advance(calls, &mut self.expected_exits);
            }
        }
        if let Some(index) = self.tab {
            let (drivers, visitors) = procs.split_at_mut(index);
            let Process::Calls(calls) = &mut drivers[1] else { panic!("ctl") };
            let Process::Visitor(visitor) = &mut visitors[0] else { panic!("tab visitor") };
            if calls.visiting() {
                if matches!(self.tab_visit, Visit::Handed) && visitor.is_empty() {
                    let hops = visitor.hops().to_vec();
                    calls.visited(hops.last().expect("landing hop").status);
                    let mut visits = self.visits.take();
                    visits.push(hops);
                    self.visits.set(visits);
                    self.tab_visit = Visit::AwaitingAddress;
                } else if matches!(self.tab_visit, Visit::AwaitingAddress) {
                    let address =
                        calls.ends().last().and_then(visit_address).expect("last act printed a browser address");
                    visitor.hand(&address);
                    calls.visit_handed();
                    self.tab_visit = Visit::Handed;
                }
            }
        }
        let (drivers, peers) = procs.split_at_mut(2);
        let [Process::Binary(binary), Process::Calls(calls)] = drivers else { panic!("box and ctl") };
        restart_engine(calls, binary, now, &mut self.restart_at, &mut self.expected_exits);
        release_llm(calls, peers, &mut self.release_answers);
        // A release before the response reaches its hold changes nothing.
        // Keep driving it even when the held peer has no io or timer ready
        // (testing/process-tier.md, the scripts' process cadence).
        self.next_release = calls.llm_release().map(|_| now.saturating_add(Duration::from_millis(20)));
        if self.on_link.is_some()
            && printed_link(calls.stdout()).is_some()
            && serve_listening(&self.data)
            && let Some(on_link) = self.on_link.take()
        {
            on_link();
        }
        calls.observe_box_stderr(binary.stderr(), now);
        // The box's stderr may end a call: judge the calls again after it.
        let failed = calls.ends().iter().enumerate().any(|(index, call)| {
            let expected = self.expected_exits.iter().find(|(at, _)| *at == index).map_or(0, |(_, code)| *code);
            calls.observed_at()[index].is_some() && call.exit != Some(Exit::Code(expected))
        });
        let failed = failed
            || self.fail_on_panic.is_some()
                && binary.stderr().windows(b"panicked at".len()).any(|part| part == b"panicked at");
        self.bound(calls, binary, failed);
        if !self.stopping && (calls.idle() || failed || binary.exit_status().is_some()) {
            if self.face.is_some() || failed {
                report_face_stop(calls, binary.stderr(), failed, false);
                eprintln!("pending scripted release: {:?}", calls.llm_release());
            }
            if calls.idle() && !failed && binary.exit_status().is_none() {
                assert!(self.resuming_playground.is_none(), "the kept reset was observed after restart");
                self.checked.set(Some(self.check.take().expect("one live check")(binary)));
            }
            // A held batch keeps its root open for the reply referee. Release
            // it on failure too, so cancellation drains that descriptor.
            calls.release();
            calls.cancel();
            for proc in peers {
                match proc {
                    Process::Peer(peer) => peer.shutdown(),
                    Process::Issuer(issuer) => issuer.shutdown(),
                    Process::Binary(_)
                    | Process::Calls(_)
                    | Process::Person(_)
                    | Process::Stdout(_)
                    | Process::Visitor(_) => {}
                }
            }
            if binary.exit_status().is_none() {
                binary.signal(Signal::Terminate);
            }
            self.stopping = true;
        }
        let left = binary.tree_status().live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty());
        if !self.cleanup && (left || now >= self.cleanup_deadline) && !binary.is_empty() {
            binary.signal_tree(Signal::Kill);
            self.cleanup = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = self.stopping && procs.iter().all(Host::is_empty);
        if self.face.is_some() && self.cleanup && !self.settled {
            eprintln!("face drain: {:?}", procs.iter().map(Host::is_empty).collect::<Vec<_>>());
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.settled {
            None
        } else if self.stopping {
            Some(self.cleanup_deadline)
        } else {
            [self.deadline.until(), self.next_release, self.restart_at].into_iter().flatten().min()
        }
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Uses the box's printed link for ctl sign-in, then drains the supervised tree.
/// A refused call stops the story immediately and retains its diagnostic.
/// The live check borrows the world's binary to watch its tree in classified
/// waits. It defers starvation to the world and never passes from the callback.
#[must_use]
pub fn run_ctl<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce(&Binary) -> T,
) -> CtlOutcome<T> {
    run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        check,
        CtlPeers { program: None, expected_exits: Vec::new(), model: CtlModel::None },
    )
    .0
}
/// Start an installed box at an explicit path, keeping ctl and tree drainage unchanged.
#[must_use]
pub fn run_ctl_from<T>(
    program: &Path,
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    check: impl FnOnce(&Binary) -> T,
) -> CtlOutcome<T> {
    run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        BOX_DEADLINE,
        check,
        CtlPeers { program: Some(program.into()), expected_exits: Vec::new(), model: CtlModel::None },
    )
    .0
}
/// Observe the kept reset while the restarted box's acknowledgement is held.
#[must_use]
pub fn run_ctl_resuming_playground<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce(&Binary) -> T,
) -> CtlOutcome<T> {
    let config = flags.windows(2).find(|pair| pair[0] == "--config").expect("explicit configuration");
    let paths = [PathBuf::from(&config[1]), client_root.into()];
    run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        check,
        CtlPeers { program: None, expected_exits: Vec::new(), model: CtlModel::ResumingPlayground(paths) },
    )
    .0
}
/// Continues through the exact expected nonzero calls, keeping the normal early-stop rule otherwise.
#[must_use]
pub fn run_ctl_with_exits<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce() -> T,
    expected_exits: &[(usize, u8)],
) -> CtlOutcome<T> {
    run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        |_| check(),
        CtlPeers { program: None, expected_exits: expected_exits.to_vec(), model: CtlModel::None },
    )
    .0
}
/// An independent fake and a fixture hook after the box has written its files.
pub struct CtlLlm {
    pub expected_exits: Vec<(usize, u8)>,
    pub peer: super::llm::Listening,
    pub on_link: Box<dyn FnOnce()>,
}
struct CtlPeers {
    program: Option<PathBuf>,
    expected_exits: Vec<(usize, u8)>,
    model: CtlModel,
}
enum CtlModel {
    ResumingPlayground([PathBuf; 2]),
    None,
    Llm(Box<CtlLlm>),
    Face(Box<super::forge_land::CtlFace>),
    Account(Box<AccountProbe>),
}
/// Process-tier account restart controls and a one-shot listener fault.
pub struct AccountProbe {
    pub face: super::forge_land::Face,
    pub cut_after_listening: Option<PathBuf>,
}
/// Drive a caller's steps with a signed-in fake account, without a reply sketch.
#[must_use]
pub fn run_ctl_with_account<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce() -> T,
    face: super::forge_land::Face,
) -> CtlOutcome<T> {
    run_ctl_with_account_probe(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        check,
        AccountProbe { face, cut_after_listening: None },
    )
}
#[must_use]
pub fn run_ctl_with_account_probe<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce() -> T,
    probe: AccountProbe,
) -> CtlOutcome<T> {
    run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        |_| check(),
        CtlPeers { program: None, expected_exits: Vec::new(), model: CtlModel::Account(Box::new(probe)) },
    )
    .0
}
/// Drive the first task without changing the box's template or demo.
#[must_use]
pub fn run_ctl_with_llm<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce(&[u8]) -> T,
    llm: CtlLlm,
) -> CtlOutcome<T> {
    run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        |binary| check(binary.stderr()),
        CtlPeers { program: None, expected_exits: llm.expected_exits.clone(), model: CtlModel::Llm(Box::new(llm)) },
    )
    .0
}
/// Drive the owned forge with a plan issuer, redirect visitor and bearer peer.
#[must_use]
pub fn run_ctl_with_face<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce() -> T,
    face: super::forge_land::CtlFace,
) -> (CtlOutcome<T>, super::forge_land::Seen) {
    let (outcome, seen) = run_ctl_peers(
        directory,
        flags,
        client_root,
        steps,
        deadline,
        |_| check(),
        CtlPeers { program: None, expected_exits: Vec::new(), model: CtlModel::Face(Box::new(face)) },
    );
    (outcome, seen.expect("face observations"))
}
fn account_environment(probe: bool, cut: Option<&Path>) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut environment = vec![("PATH".into(), "/usr/bin:/bin".into())];
    if probe {
        environment.push(("TEMPER_INTERNAL_WAIT_CHECK".into(), "1".into()));
    }
    if let Some(marker) = cut {
        environment.push(("TEMPER_INTERNAL_CUT_AT_LISTEN".into(), marker.as_os_str().to_owned()));
    }
    environment
}

fn configure_face(face: Option<&super::forge_land::Face>, flags: &[String]) {
    if let Some(super::forge_land::Face::Fake { peer, .. }) = face {
        let config = flags.windows(2).find(|pair| pair[0] == "--config").expect("explicit config directory");
        peer.configure(Path::new(&config[1]));
    }
}

fn resuming_playground(model: &CtlModel) -> Option<[PathBuf; 2]> {
    match model {
        CtlModel::ResumingPlayground(paths) => Some(paths.clone()),
        CtlModel::None | CtlModel::Llm(_) | CtlModel::Face(_) | CtlModel::Account(_) => None,
    }
}

#[expect(clippy::too_many_lines, reason = "Keeps the box, ctl and peers wired together.")]
fn run_ctl_peers<T>(
    directory: &Path,
    flags: Vec<String>,
    client_root: &Path,
    steps: Vec<super::ctl::Step>,
    deadline: Duration,
    check: impl FnOnce(&Binary) -> T,
    peers: CtlPeers,
) -> (CtlOutcome<T>, Option<super::forge_land::Seen>) {
    let fake_backends = flags.iter().any(|flag| flag == "--fake-backends");
    let pair = flags.windows(2).find(|pair| pair[0] == "--data").expect("explicit data directory");
    let data = PathBuf::from(&pair[1]);
    let data = if fake_backends { data.join("box/fake-backends/data") } else { data };
    let resuming_playground = resuming_playground(&peers.model);
    let account_probe = matches!(&peers.model, CtlModel::Account(_));
    let cut = match &peers.model {
        CtlModel::Account(probe) => probe.cut_after_listening.clone(),
        CtlModel::None | CtlModel::Llm(_) | CtlModel::Face(_) | CtlModel::ResumingPlayground(_) => None,
    };
    let fail_on_panic = account_probe.then_some(());
    let (peer, on_link, face, reply) = match peers.model {
        CtlModel::Llm(llm) => (Some(llm.peer), Some(llm.on_link), None, None),
        CtlModel::Face(story) => {
            (None, None, Some(story.face), Some(super::forge_land::Reply::new(story.first, story.reply)))
        }
        CtlModel::None | CtlModel::ResumingPlayground(_) => (None, None, None, None),
        CtlModel::Account(probe) => (None, None, Some(probe.face), None),
    };
    configure_face(face.as_ref(), &flags);
    let tab = steps.iter().any(|step| matches!(step, super::ctl::Step::Visit));
    assert!(!tab || face.is_none(), "box-owned peers or an independent face");
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        dying_with_test(Command {
            program: peers.program.unwrap_or_else(|| skein_scratch::paths::binary("temper-box")),
            arguments: flags.into_iter().map(Into::into).collect(),
            directory: directory.into(),
            // Real repository stories, including a fake box's owned Forgejo,
            // need init to discover system git for their first chat's checkout.
            environment: {
                let mut environment = if on_link.is_some() || face.is_some() || fake_backends {
                    account_environment(account_probe, cut.as_deref())
                } else {
                    Vec::new()
                };
                if fake_backends {
                    environment.push(chosen_fake_ports());
                }
                environment
            },
        }),
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("temper box starts");
    let streams = binary.take_streams().expect("box pipes");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(client_root).expect("client root");
    fds.push(root);
    let checked = Cell::new(None);
    let visits = Cell::new(Vec::new());
    let follow_reply = reply.is_some();
    let mut world = real::World::new(CtlJudge {
        data: data.clone(),
        resuming_playground,
        fail_on_panic,
        expected_exits: peers.expected_exits,
        release_answers: None,
        next_release: None,
        restart_at: None,
        face: face.as_ref().map(|_| super::forge_land::FaceProcs { issuer: 2, visitor: 3, peer: 4 }),
        visit: Visit::AwaitingAddress,
        tab: tab.then_some(2),
        tab_visit: Visit::AwaitingAddress,
        visits: &visits,
        reply,
        on_link,
        check: Some(check),
        checked: &checked,
        stopping: false,
        cleanup: false,
        settled: false,
        // Leave ten seconds for cancellation, pipe drain and reaping.
        deadline: action_deadline("the box ctl story's actions", deadline, &binary),
        cleanup_deadline: clock.now().now.saturating_add(deadline),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        let mut calls =
            super::ctl::Calls::new(root, steps, Some(&streams), None).with_serve_data(&data).with_box_stderr();
        if follow_reply {
            calls.hold();
        }
        Process::Calls(Box::new(calls))
    });
    if tab {
        world.spawn(|| Process::Visitor(Box::default()));
    }
    let face_procs = face.map(|face| face.spawn(&mut world));

    let peer = peer.map(|peer| peer.spawn(&mut world));
    let ended = world.run(&clock, deadline);
    assert!(ended.procs.iter().all(Host::is_empty), "box, ctl and peer io settles");
    let [Process::Binary(binary), Process::Calls(calls), ..] = ended.procs.as_slice() else { panic!("box and ctl") };
    (
        CtlOutcome {
            model: peer.map(|index| super::forge_land::peer_seen(&ended.procs, index)),
            exit: binary.exit_status(),
            stdout: calls.stdout().to_vec(),
            stderr: binary.stderr().to_vec(),
            left: binary.tree_status().live_at_leader_exit.clone(),
            calls: calls
                .ends()
                .iter()
                .map(|call| super::Ended { exit: call.exit, stdout: call.stdout.clone(), stderr: call.stderr.clone() })
                .collect(),
            checked: checked.take(),
            visits: visits.take(),
        },
        face_procs.as_ref().map(|at| super::forge_land::seen(&ended.procs, at)),
    )
}

/// Spawn on the calling test thread, which drives the world until teardown.
/// Linux's death signal follows that thread's lifetime, even across box's exec.
#[must_use]
pub fn dying_with_test(mut command: Command) -> Command {
    if cfg!(target_os = "linux") {
        let mut arguments = vec![
            "internal".into(),
            "forge-child".into(),
            "--parent".into(),
            std::process::id().to_string().into(),
            // Terminate, so the box stops its steps as on any stop.
            "--signal".into(),
            "terminate".into(),
            "--".into(),
            command.program.into(),
        ];
        arguments.extend(command.arguments);
        command.program = skein_scratch::paths::binary("temper-box");
        command.arguments = arguments;
    }
    command
}

/// Drive a crash story; the probe either cuts at its boundary or lets ctl finish.
/// The keeper drains every descendant before the deployment is started again.
#[must_use]
pub fn run_ctl_with_cut(
    directory: &Path,
    flags: Vec<String>,
    steps: Vec<super::ctl::Step>,
    mut cut: impl FnMut(&Binary, &super::ctl::Calls) -> bool,
) -> CtlOutcome<()> {
    struct CutJudge<'a, F> {
        inner: CtlJudge<'a, fn(&Binary), ()>,
        cut: &'a mut F,
        killed: &'a Cell<bool>,
        next_probe: Time,
        armed: bool,
    }
    impl<F: FnMut(&Binary, &super::ctl::Calls) -> bool> Referee<Process> for CutJudge<'_, F> {
        fn act(&mut self, now: Time, procs: &mut [Process]) {
            let [Process::Binary(binary), Process::Calls(calls), ..] = procs else { panic!("box and ctl") };
            if self.armed && !self.killed.get() && calls.idle() {
                binary.signal(Signal::Kill);
                self.killed.set(true);
            } else if !self.armed && binary.exit_status().is_none() && (self.cut)(binary, calls) {
                // Keep the boundary while the caller reaps; it shares the observer's cgroup.
                binary.signal(Signal::Stop);
                calls.cancel();
                self.inner.stopping = true;
                self.armed = true;
            }
            self.next_probe = now.saturating_add(Duration::from_millis(2));
            self.inner.act(now, procs);
        }
        fn observe(&mut self, now: Time, procs: &[Process]) {
            self.inner.observe(now, procs);
        }
        fn next_deadline(&self) -> Option<Time> {
            if self.inner.settled { None } else { Some(self.next_probe) }
        }
        fn passed(&self) -> bool {
            self.inner.passed()
        }
    }
    fn check(_: &Binary) {}
    let clock = Clock::new();
    let pair = flags.windows(2).find(|pair| pair[0] == "--data").expect("explicit data directory");
    let data = PathBuf::from(&pair[1]);
    let mut binary = cut_binary(directory, flags);
    let streams = binary.take_streams().expect("box pipes");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(directory).expect("client root");
    fds.push(root);
    let checked = Cell::new(None);
    let visits = Cell::new(Vec::new());
    let killed = Cell::new(false);
    let mut world = real::World::new(CutJudge {
        inner: CtlJudge {
            data: data.clone(),
            fail_on_panic: Some(()),
            expected_exits: Vec::new(),
            release_answers: None,
            next_release: None,
            restart_at: None,
            resuming_playground: None,
            face: None,
            visit: Visit::AwaitingAddress,
            tab: None,
            tab_visit: Visit::AwaitingAddress,
            visits: &visits,
            reply: None,
            on_link: None,
            check: Some(check as fn(&Binary)),
            checked: &checked,
            stopping: false,
            cleanup: false,
            settled: false,
            deadline: action_deadline("the box's named cut", BOX_DEADLINE, &binary),
            cleanup_deadline: clock.now().now.saturating_add(BOX_DEADLINE),
        },
        cut: &mut cut,
        killed: &killed,
        next_probe: Time::ZERO,
        armed: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Calls(Box::new(
            super::ctl::Calls::new(root, steps, Some(&streams), None).with_serve_data(&data).with_box_stderr(),
        ))
    });
    let ended = world.run(&clock, BOX_DEADLINE);
    assert!(ended.procs.iter().all(Host::is_empty), "the killed box's tree and ctl settle");
    let [Process::Binary(binary), Process::Calls(calls)] = ended.procs.as_slice() else { panic!("box and ctl") };
    CtlOutcome {
        model: None,
        exit: binary.exit_status(),
        stdout: calls.stdout().to_vec(),
        stderr: binary.stderr().to_vec(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        calls: calls
            .ends()
            .iter()
            .map(|call| super::Ended { exit: call.exit, stdout: call.stdout.clone(), stderr: call.stderr.clone() })
            .collect(),
        checked: checked.take().or_else(|| killed.get().then_some(())),
        visits: Vec::new(),
    }
}

fn chosen_fake_ports() -> (std::ffi::OsString, std::ffi::OsString) {
    let ports = super::free_addresses::<9>(&[]).map(|address| address.port().to_string()).join(",");
    ("TEMPER_INTERNAL_FAKE_BOX_PORTS".into(), ports.into())
}

fn cut_binary(directory: &Path, flags: Vec<String>) -> Binary {
    let mut environment = vec![("PATH".into(), "/usr/bin:/bin".into())];
    if flags.iter().any(|flag| flag == "--fake-backends") {
        environment.push(chosen_fake_ports());
    }
    Binary::start_with_tree(
        dying_with_test(Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: flags.into_iter().map(Into::into).collect(),
            directory: directory.into(),
            environment,
        }),
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("temper box starts")
}
