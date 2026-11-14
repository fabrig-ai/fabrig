//! The client's TUI and its real box on a pseudo-terminal (client/tui/box.md, Its tests).
#[path = "tui_box/lending.rs"]
mod lending;
use crate::{box_link, support};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Signal, Size, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Cleanup, Command, Expectation, Mode, Streams},
    real,
};
use std::{
    collections::{BTreeMap, VecDeque},
    ffi::OsString,
    fs,
    os::unix::{ffi::OsStrExt, fs::DirBuilderExt},
    path::{Path, PathBuf},
};

const READ: Token = Token::new(1);
const WRITE: Token = Token::new(2);
const CLOSE: Token = Token::new(3);
struct Terminal {
    stream: Option<Fd>,
    reading: bool,
    writing: bool,
    closing: bool,
    client_exit: Option<Exit>,
    bytes: Vec<u8>,
    keys: VecDeque<Box<[u8]>>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
}
impl Terminal {
    fn new(stream: Fd) -> Self {
        Self {
            stream: Some(stream),
            reading: false,
            writing: false,
            closing: false,
            client_exit: None,
            bytes: Vec::new(),
            keys: VecDeque::from([Box::from(b"\n".as_slice())]),
            complete: Queue::with_capacity(8),
            submit: Queue::with_capacity(8),
        }
    }
    fn keys(&mut self, bytes: &[u8]) {
        self.keys.push_back(bytes.into());
    }
    fn has(&self, bytes: &[u8]) -> bool {
        skein_lib::bytes::find(&self.bytes, bytes).is_some()
    }
}
impl Host for Terminal {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(done) = self.complete.pop() {
            if done.op == READ {
                self.reading = false;
                let Op::PipeRead { buf, .. } = done.kind else { panic!("terminal read") };
                match done.result {
                    Ok(Done::Count(count)) if count > 0 => {
                        self.bytes.extend_from_slice(&buf[..usize::try_from(count).expect("read count")]);
                        assert!(self.bytes.len() <= 262_144, "bounded terminal output");
                    }
                    Ok(Done::Count(_)) | Err(_) => {
                        self.closing = true;
                        self.keys.clear();
                    }
                    other => panic!("terminal read count: {other:?}"),
                }
            } else if done.op == WRITE {
                let Op::PipeWrite { fd, bytes, from } = done.kind else { panic!("terminal write") };
                let Done::Count(count) = done.result.expect("terminal keys") else { panic!("write count") };
                let from = from.checked_add(count).expect("bounded keys");
                if usize::try_from(from).expect("key offset") < bytes.len() {
                    self.submit.push(Submit { op: WRITE, kind: Op::PipeWrite { fd, bytes, from } });
                } else {
                    self.writing = false;
                }
            } else {
                assert_eq!(done.op, CLOSE);
                done.result.expect("terminal master closed");
                self.closing = false;
            }
        }
        if let Some(fd) = self.stream {
            if self.closing {
                // The slave can close before process exit. Keep the master until then:
                // closing it sends SIGHUP to a client still finishing its exit.
                if self.client_exit.is_some()
                    && !self.writing
                    && !self.reading
                    && !self.submit.iter().any(|submit| submit.op == CLOSE)
                {
                    self.stream = None;
                    self.submit.push(Submit { op: CLOSE, kind: Op::Close { fd } });
                }
            } else {
                if !self.reading {
                    self.reading = true;
                    self.submit.push(Submit { op: READ, kind: Op::PipeRead { fd, buf: vec![0; 32768].into() } });
                }
                if !self.writing
                    && let Some(bytes) = self.keys.pop_front()
                {
                    self.writing = true;
                    self.submit.push(Submit { op: WRITE, kind: Op::PipeWrite { fd, bytes, from: 0 } });
                }
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.complete.is_empty() || !self.submit.is_empty() || !self.keys.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.stream.is_none() && !self.closing && self.complete.is_empty() && self.submit.is_empty()
    }
    fn operations(&self) -> u32 {
        8
    }
}

enum Root {
    Client(Box<Binary>),
    Terminal(Box<Terminal>),
    Calls(Box<support::ctl::Calls>),
}
impl Delegate for Root {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Client(x) => x.as_ref(),
            Self::Terminal(x) => x.as_ref(),
            Self::Calls(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Client(x) => x.as_mut(),
            Self::Terminal(x) => x.as_mut(),
            Self::Calls(x) => x.as_mut(),
        }
    }
}
#[derive(Clone, Copy)]
enum Case {
    Idle,
    Working,
    Killed,
}
#[derive(Debug)]
enum Phase {
    Connecting,
    SettingUp,
    Project,
    Chat,
    Question,
    Stopping,
    Done,
}
struct Judge {
    failed: Option<String>,
    case: Case,
    phase: Phase,
    root: PathBuf,
    client: PathBuf,
    patience: support::patience::Patience,
    settled: bool,
}
impl Referee<Root> for Judge {
    fn act(&mut self, _now: Time, roots: &mut [Root]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let [Root::Client(client), Root::Terminal(terminal), Root::Calls(calls)] = roots else {
                return Err(String::from("client, person and ctl"));
            };
            terminal.client_exit = client.exit_status();
            match self.phase {
                Phase::Connecting if terminal.has(b"Live") && !terminal.writing => {
                    if !box_link::link_path(&self.root).is_file() {
                        return Err(String::from("the handed link signed in and serve renewed it"));
                    }
                    match self.case {
                        Case::Idle => {
                            terminal.keys(b"\x03");
                            calls.release();
                            self.phase = Phase::Stopping;
                        }
                        Case::Killed => {
                            client.signal(Signal::Kill);
                            calls.release();
                            self.phase = Phase::Stopping;
                        }
                        Case::Working => {
                            let token = self.root.join("state/temper/tui/token");
                            if !token.is_file() {
                                return Ok(());
                            }
                            // ctl has its own directory and acts as the TUI's person.
                            fs::copy(token, self.root.join("ctl/token"))
                                .map_err(|error| format!("copy fixture credential without printing it: {error:?}"))?;
                            calls.push(support::ctl::Step::Run(
                                support::ctl::Call::new(&["playground", "start", "--key", "tui-playground"])
                                    .by(&self.client),
                            ));
                            self.phase = Phase::SettingUp;
                        }
                    }
                }
                Phase::SettingUp if calls.foreground_idle() => {
                    if !calls.ends().iter().all(|call| call.exit == Some(Exit::Code(0))) {
                        return Err(format!(
                            "playground set up: {:?}",
                            calls
                                .ends()
                                .iter()
                                .map(|call| (call.exit, support::without_secrets(&call.stderr)))
                                .collect::<Vec<_>>()
                        ));
                    }
                    self.phase = Phase::Project;
                }
                Phase::Project if terminal.has(b"playground") && !terminal.writing => {
                    terminal.bytes.clear();
                    terminal.keys(b"Add CSV export to widget\r");
                    self.phase = Phase::Chat;
                }
                // The echoed input can precede the running chat read by the TUI.
                Phase::Chat
                    if terminal.has(b"Add CSV export to widget") && terminal.has(b"Working") && !terminal.writing =>
                {
                    terminal.keys(b"\x03");
                    self.phase = Phase::Question;
                }
                Phase::Question if terminal.has(b"Stop the box?") && !terminal.writing => {
                    terminal.keys(b"\r");
                    calls.release();
                    self.phase = Phase::Stopping;
                }
                Phase::Stopping
                    if client.exit_status().is_some()
                        && match fs::read_to_string(self.root.join("data/box/logs/box.log")) {
                            Ok(log) => log,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                            Err(error) => return Err(format!("box log: {error}")),
                        }
                        .lines()
                        .next_back()
                        .is_some_and(|line| line.ends_with("Stopped")) =>
                {
                    self.phase = Phase::Done;
                }
                Phase::Connecting
                | Phase::SettingUp
                | Phase::Project
                | Phase::Chat
                | Phase::Question
                | Phase::Stopping
                | Phase::Done => {}
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, roots: &[Root]) -> Result<(), String> {
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        self.settled = matches!(self.phase, Phase::Done) && roots.iter().all(Host::is_empty);
        if !self.settled
            && let Some(figures) = self.patience.failed()
        {
            return Err(format!("TUI's box did not settle in {:?}: {figures}", self.phase));
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.settled { None } else { self.patience.until() }
    }
    fn passed(&self) -> bool {
        if self.failed.is_some() {
            return false;
        }
        self.settled
    }
}

fn run(case: Case, name: &str) {
    let scratch = support::Scratch::new(name);
    let root = scratch.path();
    let initialized = box_link::command(root, "init").arg("--box").output().expect("initialize box");
    assert!(initialized.status.success());
    support::own_listener_files(&root.join("config"));
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("config/temper.json")).expect("configuration")).expect("document");
    let address = document
        .pointer("/application/clients/listener/socket")
        .and_then(serde_json::Value::as_str)
        .expect("engine address");
    let ctl = root.join("ctl");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(&ctl).expect("ctl directory");
    support::write_private(&ctl.join("address"), format!("http://{address}\n").as_bytes());
    let command = box_link::command(root, "");
    let client = skein_scratch::paths::binary("temper-box").with_file_name("temper");
    assert!(client.is_file(), "temper is built beside temper-box");
    let mut environment: BTreeMap<OsString, OsString> = temper_test_env::environment()
        .iter()
        .map(|(key, value)| (std::ffi::OsStr::from_bytes(key).into(), std::ffi::OsStr::from_bytes(value).into()))
        .collect();
    for (key, value) in command.get_envs() {
        if let Some(value) = value {
            environment.insert(key.into(), value.into());
        } else {
            environment.remove(key);
        }
    }
    environment.insert("XDG_STATE_HOME".into(), root.join("state").into());
    environment.insert("TERM".into(), "xterm-256color".into());
    let arguments = [
        OsString::from("-c"),
        OsString::from("read -r ready; exec \"$@\""),
        OsString::from("tui-box"),
        client.as_os_str().into(),
        OsString::from("box"),
    ]
    .into_iter()
    .chain(command.get_args().map(ToOwned::to_owned))
    .collect();
    let mut binary = Binary::start_with_tree(
        Command {
            program: Path::new("/bin/sh").into(),
            arguments,
            environment: environment.into_iter().collect(),
            directory: root.into(),
        },
        Mode::Terminal,
        8192,
        Expectation::Supervise,
        false,
    )
    .expect("temper box on a pseudo-terminal");
    let streams = binary.take_streams().expect("person's terminal");
    let Streams::Terminal { stream } = streams else { panic!("pseudo-terminal") };
    skein_shell::resize_terminal(stream, Size { columns: 80, rows: 24 }).expect("80 by 24");
    let mut calls =
        support::ctl::Calls::linked(&format!("http://{address}/jig/v1/sign-in/link?secret={}", "A".repeat(43)), &ctl);
    calls.hold();
    let mut world = real::World::new(Judge {
        failed: None,
        case,
        phase: Phase::Connecting,
        root: root.into(),
        client,
        patience: support::patience::Patience::watching("TUI in front of a box", support::WITHIN, &binary),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Root::Client(Box::new(binary)));
    world.spawn_with_fds(vec![stream], || Root::Terminal(Box::new(Terminal::new(stream))));
    world.spawn_with_fds(calls.descriptors(), || Root::Calls(Box::new(calls)));
    let ended = world.run(&skein_shell::Clock::new(), support::WITHIN);
    let Root::Client(client) = &ended.procs[0] else { panic!("client observer") };
    assert_eq!(
        client.exit_status(),
        Some(match case {
            Case::Idle | Case::Working => Exit::Code(0),
            Case::Killed => Exit::Signal(9),
        })
    );
    assert!(client.stderr().is_empty(), "a stopped front prints nothing");
    assert_eq!(client.tree_status().cleanup(), Cleanup::None, "no child needed the test's cleanup");
    assert!(client.tree_status().live_pids.is_empty(), "no surviving box or step");
    assert!(ended.procs.iter().all(Host::is_empty));
    scratch.remove();
}

struct LoginJudge {
    settled: bool,
}
impl Referee<Root> for LoginJudge {
    fn watch(&mut self, _now: Time, roots: &[Root]) -> Result<(), String> {
        self.settled = roots.iter().all(Host::is_empty);

        Ok(())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}

#[test]
fn program_login_takes_its_background_groups_terminal_before_claude_reads() {
    use std::os::unix::fs::PermissionsExt;
    use temper_fetch::claude_code::{self, Libc};
    let scratch = support::Scratch::startup("login-background-terminal");
    let directory = scratch.path();
    let build = claude_code::build(
        if cfg!(target_os = "macos") {
            skein_forge::forgejo::pin::Platform::Macos
        } else {
            skein_forge::forgejo::pin::Platform::Linux
        },
        std::env::consts::ARCH,
        if cfg!(target_env = "musl") { Libc::Musl } else { Libc::Gnu },
    )
    .expect("pinned host build");
    let claude =
        Path::new(std::ffi::OsStr::from_bytes(&claude_code::file(directory.as_os_str().as_bytes()))).to_path_buf();
    fs::create_dir_all(claude.parent().expect("cached build directory")).expect("cache");
    fs::write(
        &claude,
        b"#!/bin/sh\n[ \"$DISABLE_AUTOUPDATER\" = 1 ] && [ \"$DISABLE_UPDATES\" = 1 ] || exit 10\n[ \"$#\" -eq 2 ] && [ \"$1\" = auth ] && [ \"$2\" = login ] || exit 9\nIFS= read -r answer || exit 8\n[ -z \"$answer\" ] || exit 7\nprintf 'login read its terminal\\n'\nexit 0\n",
    )
    .expect("fake claude reads its inherited terminal");
    fs::OpenOptions::new().write(true).open(&claude).expect("cached fake").set_len(build.bytes).expect("pinned size");
    fs::set_permissions(&claude, fs::Permissions::from_mode(0o700)).expect("executable claude");
    support::program_login::configuration(directory, Some(&directory.join("programs")), Path::new("fetched"));
    // Job control gives the async command a background group, but not the foreground.
    // Explicit /dev/tty keeps the shell from replacing its background stdin with /dev/null.
    let arguments = [
        OsString::from("-c"),
        OsString::from("set -m || exit 6; \"$@\" </dev/tty & child=$!; wait \"$child\""),
        OsString::from("background-login"),
        skein_scratch::paths::binary("temper-box").into_os_string(),
        OsString::from("program-login"),
    ]
    .into_iter()
    .chain(support::configuration::flags(directory))
    .chain([OsString::from("source")])
    .collect();
    let mut binary = Binary::start(
        Command { program: "/bin/sh".into(), arguments, environment: vec![], directory: directory.into() },
        Mode::Terminal,
        8192,
    )
    .expect("background login on a pseudo-terminal");
    let Streams::Terminal { stream } = binary.take_streams().expect("terminal master") else {
        panic!("pseudo-terminal");
    };
    let mut world = real::World::new(LoginJudge { settled: false });
    world.spawn_with_fds(binary.descriptors(), || Root::Client(Box::new(binary)));
    // Terminal supplies one empty line; the fake must read it from the foreground.
    world.spawn_with_fds(vec![stream], || Root::Terminal(Box::new(Terminal::new(stream))));
    let ended = world.run(&skein_shell::Clock::new(), support::WITHIN);
    let [Root::Client(binary), Root::Terminal(terminal)] = ended.procs.as_slice() else {
        panic!("login and terminal settled");
    };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "{}", String::from_utf8_lossy(binary.stderr()));
    assert!(terminal.has(b"login read its terminal"), "{}", String::from_utf8_lossy(&terminal.bytes));
    assert!(ended.procs.iter().all(Host::is_empty), "login and terminal drained");
    assert_eq!(binary.tree_status().cleanup(), Cleanup::None, "no stopped login left to clean up");
    scratch.remove();
}

#[test]
fn temper_box_signs_in_on_its_pty_and_quitting_stops_its_box_and_steps() {
    run(Case::Idle, "tui-box-idle");
    run(Case::Working, "tui-box-working");
}
#[test]
fn killing_temper_closes_its_life_pipe_and_the_box_stops() {
    run(Case::Killed, "tui-box-killed");
}
