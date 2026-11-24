//! The two halves of a held drive session (temper/docs/design/testing/qa/README.md, 6).
use std::{
    cell::RefCell,
    io::{Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
};
use temper_qa_tool::{
    CaseInput, CommandFaces, Exit, Home, Kind, Refusal, answer,
    faces::{
        ArchiveFace, DirectoryMode, DriveFace, Environment, FilesFace, Processes, RecipeWalk, RemoveMode, WalkOutput,
        WriteMode, WriteOptions,
    },
    place::PlacePaths,
    testing::{Agents, Child, Clock, Connection, Git, Listener, Lock, Tree, World},
};

const PORTS: &str = "31001,31002,31003,31004,31005,31006,31007,31008,31009";
const GOOD: &str = "web press button \"Start\"";
const PATCH: &str = "deployment patch engine landing.json";
const REPLACE: WriteOptions = WriteOptions::private(WriteMode::Replace);

struct Fixture {
    files: Tree,
    home: Home,
    place: PathBuf,
    setups: PathBuf,
    kind: Kind,
}

impl Fixture {
    fn new(kind: Kind) -> Self {
        let world = World::new();
        let mut files = world.files();
        let directory = &world.home.directory;
        let home = if kind == Kind::Deployment {
            let path = directory.join("qa.json");
            let mut config: serde_json::Value =
                serde_json::from_slice(&files.read(&path).expect("config")).expect("JSON");
            config["places"] = serde_json::json!([{"directory":"places/1","kind":"deployment","ports":[31001,31002,31003,31004,31005,31006,31007,31008,31009]}]);
            put(&files, &path, serde_json::to_vec(&config).expect("JSON"));
            Home::open(directory, &mut files).expect("deployment home")
        } else {
            world.home
        };
        home.transact(&mut files, |ledger, _| {
            ledger.record_landing("build", 10, None, "Start", 0);
            let pages = [("web".into(), vec!["work".into()]), ("ctl".into(), vec!["task".into()])].into();
            let setups = vec!["fresh".into(), "turn-landing".into()];
            let deployments = if kind == Kind::Deployment { vec!["turn-landing".into()] } else { Vec::new() };
            let triage = ledger.start_session(1, "low", pages, setups, deployments, 0).expect("triage");
            let case: &[u8] = if kind == Kind::Box {
                br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"press Start","expect":"started"}]}"#
            } else {
                br#"{"surface":"ctl","page":"task","setup":"turn-landing","steps":[{"do":"ctl task list","expect":"listed"}]}"#
            };
            ledger.add_case(triage, &CaseInput::decode(case, "case")?, 1)?;
            ledger.end_session(triage, Exit::Code(0), None, 2, "qa")?;
            ledger.keep_build("build", 3)?;
            ledger.set_build_holds("build", vec![1], 1)?;
            assert_eq!(ledger.start_pass("places/1", kind, &["web".into(), "ctl".into()], "high", 4).expect("pass").session, 2);
            Ok(())
        }).expect("pass records");
        let place = home.directory.join("places/1");
        let setups = home.directory.join("passes/1/setups");
        for directory in ["bin", "bundle", kind.name()] {
            mkdir(&files, &place.join(directory));
        }
        Self { files, home, place, setups, kind }
    }

    fn environment(&self) -> Environment {
        [
            ("PATH", b"/usr/bin:/bin".as_slice()),
            ("CAPTURED", b"drive-environment"),
            ("TEMPER_INTERNAL_QA_HOME", self.home.directory.as_os_str().as_encoded_bytes()),
            ("TEMPER_INTERNAL_QA_SESSION", b"2"),
        ]
        .map(|(name, value)| (name.as_bytes().into(), value.into()))
        .into()
    }

    fn recipe(&self, name: &str, parent: Option<&str>, scripts: bool) {
        let directory = self.setups.join(name);
        mkdir(&self.files, &directory);
        let after = parent.map_or_else(String::new, |name| format!("after {name}\n"));
        let steps = if self.kind == Kind::Deployment { "check\nctl task list" } else { "web see Start" };
        put(&self.files, &directory.join("recipe"), format!("recipe 1\n{after}{steps}\n"));
        if scripts {
            put(&self.files, &directory.join("scripts.json"), "{}");
        }
    }

    fn setup(&self, name: &str) {
        let text = if name == "fresh" { "fresh\n".into() } else { format!("{name} {}\n", "a".repeat(64)) };
        put(&self.files, &self.place.join("setup"), text);
    }

    fn chain(&self) {
        self.recipe("welcome", None, true);
        self.recipe("project-landing", Some("welcome"), false);
        self.recipe("chat-draft", Some("project-landing"), true);
        self.setup("chat-draft");
    }

    fn depth(&self) {
        for depth in 0..=9 {
            let parent = (depth > 0).then(|| format!("n{}", depth - 1));
            self.recipe(&format!("n{depth}"), parent.as_deref(), false);
        }
    }

    fn deployment(&self) {
        self.recipe("root", None, true);
        self.recipe("middle", Some("root"), false);
        self.recipe("turn-landing", Some("middle"), true);
        mkdir(&self.files, &self.setups.join("root/config"));
        self.setup("turn-landing");
    }

    fn kept_lines(&self, kind: Kind, lines: &str) {
        let session = self.place.join(kind.name()).join("session");
        mkdir(&self.files, &session);
        put(&self.files, &session.join("lines"), lines);
    }

    fn free(&self) {
        assert!(!self.files.exists(self.place.join("drive.sock")), "holder removed socket");
        let lock = self.place.join("drive.lock");
        assert!(self.files.exists(&lock), "holder left its lock file");
        self.files.hold_lock(lock).expect("holder released lock");
    }

    fn run_with(&self, arguments: &[&str], processes: Agents, clock: Clock) -> Output {
        let mut output = Output::default();
        let arguments: Vec<_> = arguments.iter().map(|word| word.as_bytes().into()).collect();
        output.status = answer(
            CommandFaces {
                git: Git::new(self.files.clone()),
                processes,
                clock,
                files: self.files.clone(),
                drive: Unused,
                archive: Unused,
            },
            "test",
            "test",
            &arguments,
            &self.environment(),
            &mut &b""[..],
            &mut output.stdout,
            &mut output.stderr,
        );
        output
    }

    fn run(&self, arguments: &[&str]) -> Output {
        self.run_on(arguments, Clock::default())
    }

    fn run_on(&self, arguments: &[&str], clock: Clock) -> Output {
        let mut processes = Agents::new(self.files.clone());
        processes.script_holder(|_, _| panic!("the command must not launch a holder"));
        self.run_with(arguments, processes, clock)
    }
}

fn mkdir(files: &Tree, path: &Path) {
    files.clone().directory(path, DirectoryMode::Parents).expect("directory");
}

fn put(files: &Tree, path: &Path, bytes: impl AsRef<[u8]>) {
    files.clone().write(path, bytes.as_ref(), REPLACE, false).expect("write fixture");
}

#[derive(Default)]
struct Output {
    status: u8,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Output {
    fn answer(&self, code: u8, text: &str) {
        assert_eq!(
            self.status,
            code,
            "stdout: {}; stderr: {}",
            String::from_utf8_lossy(&self.stdout),
            String::from_utf8_lossy(&self.stderr)
        );
        assert_eq!(self.stdout, text.as_bytes());
        assert!(self.stderr.is_empty(), "{}", String::from_utf8_lossy(&self.stderr));
    }

    fn refusal(&self, text: &str) {
        assert_eq!(self.status, 1);
        assert_eq!(self.stdout, b"");
        assert_eq!(self.stderr, format!("temper-qa: {text}\n").as_bytes());
    }
}

struct Unused;

impl DriveFace for Unused {
    fn list(&mut self, _: &PlacePaths, _: &Path, _: &mut impl Processes) -> Result<String, Refusal> {
        panic!("holder does not list recipes")
    }
    fn hash(&mut self, _: &PlacePaths, _: &Path, _: &str, _: &mut impl Processes) -> Result<String, Refusal> {
        panic!("holder does not hash recipes")
    }
    fn walk(
        &mut self,
        _: &RecipeWalk<'_>,
        _: &mut impl FilesFace,
        _: &mut impl Processes,
    ) -> Result<WalkOutput, Refusal> {
        panic!("holder does not walk recipes")
    }
}

impl ArchiveFace for Unused {
    fn create(
        &mut self,
        _: &Path,
        _: &Path,
        _: &[&str],
        _: &mut impl FilesFace,
        _: &mut impl Processes,
    ) -> Result<(), Refusal> {
        panic!("holder does not create copies")
    }
    fn extract(
        &mut self,
        _: &Path,
        _: &Path,
        _: &[&str],
        _: &mut impl FilesFace,
        _: &mut impl Processes,
    ) -> Result<(), Refusal> {
        panic!("holder does not extract copies")
    }
}

// Client half: the entry runs Drive against the holder's scripted wire replies.
mod client {
    use super::*;

    struct ScriptedHolder {
        listener: Listener,
        lock: Rc<RefCell<Option<Lock>>>,
        child: Child,
        files: Tree,
        socket: PathBuf,
    }

    impl ScriptedHolder {
        fn reply(&self, status: u8, text: &str) -> Connection {
            let mut peer = self.listener.script_connection();
            write!(peer, "{status}\n{text}").expect("scripted holder reply");
            peer.close_write();
            peer
        }

        fn exit(&self, status: u8) -> impl FnOnce() + 'static {
            let mut files = self.files.clone();
            let socket = self.socket.clone();
            let lock = self.lock.clone();
            let child = self.child.clone();
            move || {
                files.remove(&socket, RemoveMode::File).expect("scripted holder removes socket");
                drop(lock.borrow_mut().take().expect("scripted holder owns lock"));
                child.exit(Exit::Code(status.into()));
            }
        }

        fn open(fixture: &Fixture, ctl: bool, status: u8, text: &str) -> Self {
            let mut processes = Agents::new(fixture.files.clone());
            let started = Rc::new(RefCell::new(None::<(Self, Connection)>));
            let save = started.clone();
            let place = fixture.place.clone();
            let environment = fixture.environment();
            let reply = text.to_owned();
            let mut clock = Clock::default();
            if status != 0 {
                let started = started.clone();
                clock.after_sleep(move || {
                    let started = started.borrow();
                    let (holder, _) = started.as_ref().expect("holder received a start request");
                    holder.exit(status)();
                });
            }
            processes.script_holder(move |command, files| {
                let args: &[_] = if ctl { &["pass", "hold", "--ctl"] } else { &["pass", "hold"] };
                assert_eq!(command.arguments(), args);
                assert_eq!(command.log, place.join("drive.log"));
                assert_eq!(command.environment, environment);
                let socket = place.join("drive.sock");
                let holder = Self {
                    lock: Rc::new(RefCell::new(Some(
                        files.hold_lock(place.join("drive.lock")).expect("holder takes lock"),
                    ))),
                    listener: files.socket(&socket).expect("holder binds"),
                    child: Child::default(),
                    files: files.clone(),
                    socket,
                };
                let child = holder.child.clone();
                let peer = holder.reply(status, &reply);
                *save.borrow_mut() = Some((holder, peer));
                Ok(child)
            });
            let args: &[_] = if ctl { &["pass", "open", "--ctl"] } else { &["pass", "open"] };
            fixture.run_with(args, processes, clock).answer(status, text);
            // A fresh Agents face numbers the first scripted child 1.
            let (holder, peer) = started.borrow_mut().take().expect("scripted holder started");
            request(peer, "start 1\n");
            if status != 0 {
                fixture.free();
            }
            holder
        }

        fn line(&self, fixture: &Fixture, line: &str, status: u8, text: &str) {
            let peer = self.reply(status, text);
            fixture.run(&["pass", "line", line]).answer(status, text);
            request(peer, &format!("line {line}\n"));
        }

        fn close(&self, fixture: &Fixture, status: u8, text: &str, kept: &str) {
            let peer = self.reply(status, text);
            // The socket reply arrives before the holder cleans up. Close must wait for that cleanup.
            let mut clock = Clock::default();
            clock.after_sleep(self.exit(status));
            fixture.run_on(&["pass", "close"], clock).answer(status, &format!("{text}kept lines:\n{kept}"));
            request(peer, "end\n");
            fixture.free();
        }
    }

    fn request(mut peer: Connection, expected: &str) {
        let mut actual = Vec::new();
        peer.read_to_end(&mut actual).expect("drain client request");
        assert_eq!(actual, expected.as_bytes());
    }

    #[test]
    fn one_holder_keeps_chain_scripts_lines_and_answers_until_close() {
        let fixture = Fixture::new(Kind::Box);
        fixture.chain();
        let holder = ScriptedHolder::open(&fixture, false, 0, "ready\n");
        let busy =
            format!("the drive session is open at {}; close it first: temper-qa pass close", fixture.place.display());
        fixture.run(&["pass", "open"]).refusal(&busy);
        holder.line(&fixture, GOOD, 0, "| web press button \"Start\"\nok\n");
        holder.line(&fixture, "fail", 5, "| fail\nfailed: fake line\n");
        holder.line(&fixture, "web look", 0, "| web look\nok\n");
        // The drive keeps the good and failed lines, not the look. Close prints that file.
        fixture.kept_lines(Kind::Box, "web press button \"Start\"\nfail\n");
        holder.close(&fixture, 0, "ended\n", "web press button \"Start\"\nfail\n");
        fixture
            .run(&["pass", "line", "web look"])
            .refusal(&format!("no drive session is open at {}: temper-qa pass open first", fixture.place.display()));
    }

    #[test]
    fn box_failures_cross_line_and_close_with_status_four() {
        let fixture = Fixture::new(Kind::Box);
        fixture.setup("fresh");
        let holder = ScriptedHolder::open(&fixture, false, 0, "ready\n");
        holder.line(&fixture, "crash", 4, "box failed: fake crash\n| last crash line\n");
        holder.exit(4)();
        fixture.free();
        let reopened = ScriptedHolder::open(&fixture, false, 0, "ready\n");
        fixture.kept_lines(Kind::Box, "");
        reopened.close(&fixture, 4, "box failed: fake stop\n", "");
    }

    #[test]
    fn fresh_starts_without_scripts_and_preserves_start_failure_statuses() {
        let fixture = Fixture::new(Kind::Box);
        fixture.setup("fresh");
        for status in [3, 4] {
            ScriptedHolder::open(
                &fixture,
                false,
                status,
                &format!("box failed: fake start {status}\n| last start line\n"),
            );
        }
        ScriptedHolder::open(&fixture, false, 0, "ready\n").close(&fixture, 0, "ended\n", "");
    }

    #[test]
    fn open_refuses_missing_setup_unreadable_chains_and_ended_sessions() {
        let fixture = Fixture::new(Kind::Box);
        fixture
            .run(&["pass", "open"])
            .refusal(&format!("no setup is restored at {}: temper-qa pass setup NAME first", fixture.place.display()));
        fixture.setup("leaf");
        fixture.recipe("leaf", Some("missing"), false);
        let output = fixture.run(&["pass", "open"]);
        assert_eq!(output.status, 1);
        assert!(
            output.stderr.starts_with(b"temper-qa: recipe missing: "),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fixture.recipe("missing", Some("leaf"), false);
        fixture.run(&["pass", "open"]).refusal("recipe missing: cycle in after chain");
        assert!(!fixture.files.exists(fixture.place.join("drive.log")), "refusals must not start the holder or drive");
        fixture.recipe("leaf", None, false);
        put(&fixture.files, &fixture.setups.join("leaf/recipe"), "recipe 1\nafter\nweb see Start\n");
        fixture.run(&["pass", "open"]).refusal("recipe leaf: invalid after record");
        fixture.depth();
        fixture.setup("n9");
        fixture.run(&["pass", "open"]).refusal("recipe n1: more than eight parents");
        fixture.setup("n8");
        ScriptedHolder::open(&fixture, false, 0, "ready\n").close(&fixture, 0, "ended\n", "");
        fixture
            .home
            .transact(&mut fixture.files.clone(), |ledger, _| {
                ledger.end_pass_session(2, Exit::Code(0), None, 5, "qa", &[])
            })
            .expect("ended session");
        for args in [&["pass", "open"][..], &["pass", "line", "web look"], &["pass", "close"]] {
            fixture.run(args).refusal("session 2 is not running");
        }
    }

    #[test]
    fn ctl_open_passes_the_flag_through_the_holder_to_drive() {
        let fixture = Fixture::new(Kind::Box);
        fixture.setup("fresh");
        ScriptedHolder::open(&fixture, true, 0, "ready\n").close(&fixture, 0, "ended\n", "");
    }

    #[test]
    fn deployment_holders_take_the_root_config_and_close_keeps_deployment_lines() {
        for ctl in [false, true] {
            let fixture = Fixture::new(Kind::Deployment);
            fixture.deployment();
            let holder = ScriptedHolder::open(&fixture, ctl, 0, "ready\n");
            holder.line(&fixture, PATCH, 0, "| deployment patch engine landing.json\nok\n");
            fixture.kept_lines(Kind::Deployment, "deployment patch engine landing.json\n");
            fixture.kept_lines(Kind::Box, "wrong scratch\n");
            holder.close(&fixture, 0, "ended\n", "deployment patch engine landing.json\n");
        }
    }
}

// Holder half: the clients queue their requests and the drive replies to each forwarded line.
mod holder {
    use super::*;
    use std::{collections::VecDeque, ffi::OsString, io::ErrorKind};
    use temper_qa_tool::{
        faces::PathKind,
        hold::{Drive, Holder, WATCH},
        testing::DriveStreams,
    };

    const PID: u32 = 41;
    const READY: (u8, &str) = (0, "ready\n");
    const END: Turn = Turn { line: "end", text: "ended\n", status: 0, exit: Some(0) };

    #[derive(Clone, Copy)]
    struct Turn {
        line: &'static str,
        text: &'static str,
        status: u8,
        exit: Option<u8>,
    }

    impl Turn {
        fn line(line: &'static str, text: &'static str, status: u8) -> Self {
            Self { line, text, status, exit: None }
        }
    }

    fn drive_arguments(fixture: &Fixture, scripts: &[&str], ctl: bool) -> Vec<OsString> {
        let mut expected = vec!["drive".into()];
        if fixture.kind == Kind::Deployment {
            expected.extend(["--deployment".into(), fixture.setups.join("root/config").into()]);
        }
        expected.extend([
            "--bin".into(),
            fixture.place.join("bin").into(),
            "--bundle".into(),
            fixture.place.join("bundle").into(),
            "--scratch".into(),
            fixture.place.join(fixture.kind.name()).into(),
        ]);
        for name in scripts {
            expected.extend(["--scripts".into(), fixture.setups.join(name).join("scripts.json").into()]);
        }
        expected.extend(["--strict".into(), "--files".into(), fixture.home.directory.join("passes/1/files").into()]);
        if fixture.kind == Kind::Deployment {
            expected.extend(["--ports".into(), PORTS.into()]);
        }
        if ctl {
            expected.push("--ctl".into());
        }
        expected
    }

    fn run(fixture: &Fixture, ctl: bool, scripts: &[&str], start: (u8, &str), turns: &[Turn]) {
        let mut files = fixture.files.clone();
        let mut processes = Agents::new(files.clone());
        let mut clock = Clock::default();
        let environment = fixture.environment();
        let expected_environment = environment.clone();
        let expected_arguments = drive_arguments(fixture, scripts, ctl);
        let program = fixture.place.join("bin/temper-client-journeys");
        let socket = fixture.place.join("drive.sock");
        let lock = fixture.place.join("drive.lock");
        let launch_socket = socket.clone();

        let expected_input = Rc::new(RefCell::new(VecDeque::from(turns.to_vec())));
        let input_script = expected_input.clone();
        let (input, drive_input) = Connection::pair();
        let (output, mut drive_output) = Connection::pair();
        let child = Child::default();
        drive_output.write_all(start.1.as_bytes()).expect("drive's start lines");
        if start.0 != 0 {
            drive_output.close_write();
            child.exit(Exit::Code(start.0.into()));
        }
        let drive_child = child.clone();
        drive_input.script_receive(move |bytes| {
            let turn = input_script.borrow_mut().pop_front().expect("unexpected line to the drive");
            assert_eq!(bytes, format!("{}\n", turn.line).as_bytes(), "line forwarded to the drive");
            drive_output.write_all(turn.text.as_bytes()).expect("drive's answer lines");
            if let Some(exit) = turn.exit {
                drive_output.close_write();
                drive_child.exit(Exit::Code(exit.into()));
            }
        });
        processes.script_drive(move |command, files| {
            assert_eq!(command.program, program.as_os_str());
            assert_eq!(command.arguments, expected_arguments);
            assert_eq!(command.environment, expected_environment);
            assert_eq!(
                files.hold_lock(lock).expect_err("holder takes the lock before launch").kind(),
                ErrorKind::WouldBlock
            );
            assert_eq!(files.kind(launch_socket), Some(PathKind::Socket), "holder binds even over a stale path");
            Ok(DriveStreams { child, input, output })
        });
        let drive = Drive::for_session(&fixture.home, 2, &mut files, &mut clock).expect("drive").with_ctl(ctl);
        let holder = Holder::start(drive, &environment, WATCH, PID, &mut files, &mut processes, &mut clock)
            .expect("start holder");

        let mut clients = vec![(ask(&fixture.files, &socket, &format!("start {PID}\n")), start.0, start.1)];
        for turn in turns {
            let request = if turn.line == "end" { "end\n".into() } else { format!("line {}\n", turn.line) };
            clients.push((ask(&fixture.files, &socket, &request), turn.status, turn.text));
        }
        holder.serve().expect("serve scripted clients");
        assert!(expected_input.borrow().is_empty(), "holder forwarded every line");
        for (mut client, status, text) in clients {
            let mut reply = String::new();
            client.read_to_string(&mut reply).expect("holder closed the connection after its answer");
            assert_eq!(reply, format!("{status}\n{text}"));
        }
        fixture.free();
    }

    fn ask(files: &Tree, socket: &Path, request: &str) -> Connection {
        let mut client = files.clone().connect(socket).expect("connect").expect("holder listening");
        client.write_all(request.as_bytes()).expect("send client request");
        client.close_write();
        client
    }

    #[test]
    fn one_holder_keeps_chain_scripts_lines_and_answers_until_close() {
        let fixture = Fixture::new(Kind::Box);
        fixture.chain();
        let lock = fixture.files.hold_lock(fixture.place.join("drive.lock")).expect("another holder owns the lock");
        fixture.run(&["pass", "hold"]).refusal(&format!(
            "the drive session is open at {}; close it first: temper-qa pass close",
            fixture.place.display()
        ));
        assert_eq!(
            fixture
                .files
                .hold_lock(fixture.place.join("drive.lock"))
                .expect_err("the refused holder did not free the lock")
                .kind(),
            ErrorKind::WouldBlock
        );
        drop(lock);

        run(
            &fixture,
            false,
            &["welcome", "chat-draft"],
            READY,
            &[
                Turn::line(GOOD, "| web press button \"Start\"\nok\n", 0),
                Turn::line("fail", "| fail\nfailed: fake line\n", 5),
                Turn::line("web look", "| web look\nok\n", 0),
                END,
            ],
        );
    }

    #[test]
    fn box_failures_cross_line_and_close_with_status_four() {
        let fixture = Fixture::new(Kind::Box);
        fixture.setup("fresh");
        let crash =
            Turn { line: "crash", text: "box failed: fake crash\n| last crash line\n", status: 4, exit: Some(4) };
        run(&fixture, false, &[], READY, &[crash]);
        let failed_end = Turn { line: "end", text: "box failed: fake stop\n", status: 4, exit: Some(4) };
        run(&fixture, false, &[], READY, &[failed_end]);
    }

    #[test]
    fn fresh_starts_without_scripts_and_preserves_start_failure_statuses() {
        let fixture = Fixture::new(Kind::Box);
        fixture.setup("fresh");
        // A stale path is replaced by the holder that owns the lock.
        put(&fixture.files, &fixture.place.join("drive.sock"), "stale");
        for status in [3, 4] {
            let text = format!("box failed: fake start {status}\n| last start line\n");
            run(&fixture, false, &[], (status, &text), &[]);
        }
        run(&fixture, false, &[], READY, &[END]);
    }

    #[test]
    fn open_refuses_missing_setup_unreadable_chains_and_ended_sessions() {
        // The other rows are refused by the client before launch. Eight parents reach the drive.
        let fixture = Fixture::new(Kind::Box);
        fixture.depth();
        fixture.setup("n8");
        run(&fixture, false, &[], READY, &[END]);
    }

    #[test]
    fn ctl_open_passes_the_flag_through_the_holder_to_drive() {
        let fixture = Fixture::new(Kind::Box);
        fixture.setup("fresh");
        run(&fixture, true, &[], READY, &[END]);
    }

    #[test]
    fn deployment_holders_take_the_root_config_and_close_keeps_deployment_lines() {
        for ctl in [false, true] {
            let fixture = Fixture::new(Kind::Deployment);
            fixture.deployment();
            run(
                &fixture,
                ctl,
                &["root", "turn-landing"],
                READY,
                &[Turn::line(PATCH, "| deployment patch engine landing.json\nok\n", 0), END],
            );
        }
    }
}
