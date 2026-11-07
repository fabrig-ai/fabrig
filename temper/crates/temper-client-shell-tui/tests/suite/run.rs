use skein_fake_machine::{Item, Machine};
use skein_io::{
    self as io,
    kernel::{Complete, Exit, Fd, Op, ServiceSignal, Submit},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, stream};
use skein_shell::Host;
use skein_sim::{Answer, Ask, Config, Handle, Pid, Program, Reply, Sim};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::{Depth, ENTER, LEAVE};
use temper_client_shell_tui::{
    limits,
    opener::Opener,
    run::{Run, Settings},
};
use temper_client_tui::Scheme;

const USER: u32 = 1000;
const SEED: u64 = 7;
const SIZE: Size = Size { rows: 24, columns: 80 };
const ROOM: usize = 65_536;
const SIGN_IN: &[u8] = b"Sign in this terminal to 127.0.0.1:8367";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Recording {
    None,
    Root,
}

struct World {
    sim: Sim,
    files: Machine,
    pid: Pid,
    input: Fd,
    output: Fd,
    signals: Fd,
    run: Run,
    now: Time,
    bytes: Vec<u8>,
    engine: Option<Engine>,
    outcome: Outcome,
    spawns: usize,
    draining: bool,
    refuse_save: bool,
    recording_sync: Option<Handle>,
    saved: usize,
    leave_after: usize,
}
impl World {
    fn new(refuse: bool) -> Self {
        Self::configured(refuse, false, false, Outcome::Success)
    }

    fn configured(refuse: bool, approval: bool, opener: bool, outcome: Outcome) -> Self {
        Self::recording(refuse, approval, opener, outcome, Recording::None)
    }

    fn recording(refuse: bool, approval: bool, opener: bool, outcome: Outcome, record: Recording) -> Self {
        let mut files = Machine::for_user(USER);
        let root = files.lay(&[Item::directory(b"tui").mode(if refuse { 0o755 } else { 0o700 })]);
        let mut sim = Sim::new(SEED, Config::calm());
        let pid = sim.spawn_process();
        let recording = (record == Recording::Root).then(|| {
            let fd = files.open(root, b".", skein_fake_machine::How::Directory).expect("recording root");
            sim.root(pid, Handle::new(fd.raw()))
        });
        let root = sim.root(pid, Handle::new(root.raw()));
        let input = sim.open_inherited_read(pid);
        let output = sim.open_inherited_write(pid);
        let signals = sim.open_signal_source(pid);
        let engine = approval.then(|| Engine::new(&mut sim));
        let settings = Settings {
            record: recording,
            engine: skein_io::Addr::Ip("127.0.0.1:8367".parse().expect("loopback")),
            directory: b"tui".as_slice().into(),
            address: b"127.0.0.1:8367".as_slice().into(),
            scheme: Scheme::Dark,
            depth: Depth::Indexed,
            size: SIZE,
            limits: limits::tui(),
            opener: opener.then(|| Opener { program: Box::from(b"/bin/browser".as_slice()), env: Box::new([]) }),
        };
        let run = Run::new(settings, root, USER, SEED, input, output, signals).expect("adopt descriptors");
        Self {
            sim,
            files,
            pid,
            input,
            output,
            signals,
            run,
            now: Time::ZERO,
            bytes: Vec::new(),
            engine,
            outcome,
            spawns: 0,
            draining: false,
            refuse_save: false,
            recording_sync: None,
            saved: 0,
            leave_after: 0,
        }
    }

    fn settle(&mut self) {
        for _ in 0..512 {
            self.sim.reap(self.pid, self.run.completions());
            self.run.iterate(self.now, Wall::from_nanos(self.now.as_nanos()));
            for submit in self.run.submissions().iter() {
                if let Op::Spawn { spawn } = &submit.kind {
                    self.spawns += 1;
                    assert_eq!(spawn.program.as_ref(), b"/bin/browser");
                    assert_eq!(spawn.args.as_ref(), &[Box::<[u8]>::from(APPROVAL)]);
                }
                if let Op::Signal { to, .. } = &submit.kind {
                    assert_eq!(*to, io::kernel::Target::Child, "the browser's group stays");
                }
            }
            self.sim.submit(self.pid, self.run.submissions());
            self.serve();
            if let Some(engine) = &mut self.engine {
                if self.run.is_empty() {
                    engine.close();
                }
                engine.iterate(&mut self.sim, self.now);
            }
            let bytes = self.sim.peer_drain(self.pid, self.output, ROOM);
            if bytes.windows(LEAVE.len()).any(|part| part == LEAVE) {
                assert!(self.saved >= self.leave_after, "final recording synced before LEAVE");
                assert!(self.recording_sync.is_none(), "no recording sync in flight at LEAVE");
            }
            self.bytes.extend(bytes);
            if self.sim.ready(self.pid) > 0
                || self.sim.deferred(self.pid)
                || self.run.work_pending(self.now)
                || self.engine.as_ref().is_some_and(|engine| engine.pending(&self.sim))
            {
                continue;
            }
            let next = [self.sim.next_due(), self.run.next_mechanism_deadline(), self.run.next_policy_deadline()]
                .into_iter()
                .flatten()
                .min();
            if let Some(next) = next {
                assert!(next > self.now, "idle deadline is in the future");
                if !self.draining && next > Time::ZERO.saturating_add(Duration::from_millis(40)) {
                    return;
                }
                self.now = next;
                self.sim.advance_to(next);
            } else {
                return;
            }
        }
        panic!(
            "the run did not settle: run={} ready={} deferred={} engine={} policy={:?} mech={:?}: {}",
            self.run.work_pending(self.now),
            self.sim.ready(self.pid),
            self.sim.deferred(self.pid),
            self.engine.as_ref().is_some_and(|engine| engine.pending(&self.sim)),
            self.run.next_policy_deadline(),
            self.run.next_mechanism_deadline(),
            self.sim.render_trace()
        );
    }

    fn serve(&mut self) {
        let mut calls = Queue::with_capacity(64);
        self.sim.calls(&mut calls);
        let mut answers = Queue::with_capacity(64);
        while let Some(call) = calls.pop() {
            if let Ask::Rename { to_dir, to, .. } = &call.ask
                && to.as_ref() == b"recording"
            {
                if std::mem::take(&mut self.refuse_save) {
                    answers.push(Answer { ticket: call.ticket, result: Err(io::kernel::Error::Permission) });
                    continue;
                }
                self.recording_sync = Some(*to_dir);
            }
            if let Ask::Sync { file } = &call.ask
                && self.recording_sync == Some(*file)
            {
                self.recording_sync = None;
                self.saved += 1;
            }
            if let Ask::Spawn { args, .. } = &call.ask {
                assert_eq!(args.as_ref(), &[Box::<[u8]>::from(APPROVAL)]);
                let result = match self.outcome {
                    Outcome::Success => Ok(Reply::Program(Program::Exit(0))),
                    Outcome::Failure => Ok(Reply::Program(Program::Exit(1))),
                    Outcome::Refused => Err(io::kernel::Error::NotFound),
                    Outcome::Running => Ok(Reply::Program(Program::Never)),
                };
                answers.push(Answer { ticket: call.ticket, result });
            } else {
                skein_fake_machine::step(&mut self.files, call, &mut answers);
            }
        }
        self.sim.answer(&mut answers);
    }

    fn drained(&mut self, refused: bool) {
        assert!(self.bytes.starts_with(ENTER), "ENTER first");
        assert!(self.bytes.ends_with(LEAVE), "LEAVE last");
        assert_eq!(self.bytes.windows(LEAVE.len()).filter(|part| *part == LEAVE).count(), 1);
        assert_eq!(self.run.refused(), refused);
        assert_eq!(self.run.exit(), Some(Exit::Code(u8::from(refused))));
        assert!(self.run.is_empty());
        assert!(self.run.signed_in().is_none());
        self.sim.assert_no_open_fds(self.pid);
        self.sim.assert_quiescent(self.pid);
        assert_eq!(self.files.open_handles(), 0);
        if let Some(engine) = &self.engine {
            assert!(engine.io.is_empty());
            self.sim.assert_no_open_fds(engine.pid);
            self.sim.assert_quiescent(engine.pid);
        }
        let length = self.bytes.len();
        self.settle();
        assert_eq!(self.bytes.len(), length, "nothing follows LEAVE");
    }
}

#[derive(Clone, Copy, Debug)]
enum WayOut {
    End,
    ControlC,
    Termination,
    Refused,
}

#[test]
fn every_way_out_leaves_the_terminal_as_it_was_found() {
    let clock = skein_shell::Clock::new();
    let recording = if cfg!(feature = "record") {
        [Recording::None, Recording::Root].as_slice()
    } else {
        [Recording::None].as_slice()
    };
    for (way, record) in [WayOut::End, WayOut::ControlC, WayOut::Termination, WayOut::Refused]
        .into_iter()
        .flat_map(|way| recording.iter().map(move |record| (way, *record)))
    {
        let began = clock.now().now;
        let mut world = World::recording(matches!(way, WayOut::Refused), false, false, Outcome::Success, record);
        world.settle();
        if matches!(way, WayOut::Refused) {
            assert!(!world.bytes.windows(SIGN_IN.len()).any(|part| part == SIGN_IN), "no Start on refusal");
            world.drained(true);
            println!("{way:?}: {} ns", clock.now().now.as_nanos().saturating_sub(began.as_nanos()));
            continue;
        }
        assert!(world.bytes.starts_with(ENTER), "{way:?}: ENTER");
        assert!(world.bytes.windows(SIGN_IN.len()).any(|part| part == SIGN_IN), "{way:?}: Start drawn");
        assert_eq!(world.run.exit(), None);
        world.leave_after = usize::from(record == Recording::Root);
        match way {
            WayOut::End => world.sim.peer_close(world.pid, world.input),
            WayOut::ControlC => {
                assert_eq!(world.sim.peer_feed(world.pid, world.input, b"\x03\rignored"), 9);
            }
            WayOut::Termination => {
                world.sim.deliver_service_signal(world.pid, world.signals, ServiceSignal::Terminate, None);
            }
            WayOut::Refused => unreachable!("refusal checked above"),
        }
        world.settle();
        #[cfg(feature = "record")]
        if record == Recording::Root {
            replay(&world.recording_file().expect("closing records every way out").0);
        }
        world.drained(false);
        println!("{way:?}: {} ns", clock.now().now.as_nanos().saturating_sub(began.as_nanos()));
    }
}

const APPROVAL: &[u8] = b"http://127.0.0.1:8367/terminal?code=WDJB-MJHT";

#[derive(Clone, Copy, Debug)]
enum Outcome {
    Success,
    Failure,
    Refused,
    Running,
}

struct Engine {
    pid: Pid,
    io: io::Io,
    env: Env<io::Limits>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    told: Queue<io::Event>,
    entities: std::collections::BTreeSet<Token>,
    responses: std::collections::BTreeMap<Token, Box<[u8]>>,
    closing: bool,
}
impl Engine {
    fn new(sim: &mut Sim) -> Self {
        let limits = io::Limits { sockets: 4, ..io::Limits::TEST };
        let mut engine = Self {
            pid: sim.spawn_process(),
            io: io::Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            complete: Queue::with_capacity(256),
            submit: Queue::with_capacity(256),
            told: Queue::uncapped(),
            entities: std::collections::BTreeSet::new(),
            responses: std::collections::BTreeMap::new(),
            closing: false,
        };
        engine.down(io::Request::Listen {
            owner: Token::new(1),
            addr: io::Addr::Ip("127.0.0.1:8367".parse().expect("engine")),
        });
        for _ in 0..8 {
            engine.iterate(sim, Time::ZERO);
        }
        assert_eq!(engine.entities.len(), 1, "engine listens before the client starts");
        engine
    }
    fn down(&mut self, request: io::Request) {
        assert!(self.io.takes());
        io::down(&mut self.io, &self.env, request, &mut self.submit);
    }
    fn iterate(&mut self, sim: &mut Sim, now: Time) {
        self.env.now = now;
        sim.reap(self.pid, &mut self.complete);
        io::stage::up(&mut self.io, &self.env, &mut self.complete, &mut self.told, &mut self.submit, now);
        while let Some(event) = self.told.pop() {
            match event {
                io::Event::Listening { listener, .. } => {
                    self.entities.insert(listener);
                }
                io::Event::Accepted { socket, .. } => {
                    self.entities.insert(socket);
                    self.down(io::Request::Bind { socket, owner: socket });
                    self.down(io::Request::Stream {
                        stream: socket,
                        down: stream::Down::Demand {
                            read: stream::Read::Scan { until: stream::Delimiter::CRLF_CRLF, max: 1024 },
                            room: 0,
                        },
                    });
                }
                io::Event::Stream { owner, up: stream::Up::Bytes(bytes) } => {
                    if bytes.starts_with(b"POST /jig/v1/approval/ask ") {
                        let response = approval_response();
                        let room = u32::try_from(response.len()).expect("short response");
                        self.responses.insert(owner, response);
                        self.down(io::Request::Stream {
                            stream: owner,
                            down: stream::Down::Demand { read: stream::Read::Nothing, room },
                        });
                    } else {
                        assert!(
                            bytes.starts_with(b"POST /jig/v1/sign-in/approval "),
                            "only the ask and take reach the engine"
                        );
                    }
                }
                io::Event::Stream { owner, up: stream::Up::Room } => {
                    let response = self.responses.remove(&owner).expect("response room");
                    self.down(io::Request::Stream { stream: owner, down: stream::Down::Send(response) });
                    self.down(io::Request::Stream { stream: owner, down: stream::Down::Finish });
                    self.down(io::Request::Close { entity: owner });
                }
                io::Event::Stream { owner, up: stream::Up::End | stream::Up::Failed(_) } => {
                    self.down(io::Request::Abort { entity: owner });
                }
                io::Event::Closed { owner } => {
                    self.entities.remove(&owner);
                }
                io::Event::Output { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Window { .. }
                | io::Event::Resized
                | io::Event::Shutdown { .. }
                | io::Event::Failed { .. } => panic!("unexpected engine event: {event:?}"),
            }
        }
        self.io.reclaim();
        sim.submit(self.pid, &mut self.submit);
    }
    fn pending(&self, sim: &Sim) -> bool {
        self.io.is_ready()
            || !self.complete.is_empty()
            || !self.submit.is_empty()
            || sim.ready(self.pid) > 0
            || sim.deferred(self.pid)
    }
    fn close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        let entities = self.entities.iter().copied().collect::<Vec<_>>();
        for entity in entities {
            self.down(io::Request::Abort { entity });
        }
    }
}
fn approval_response() -> Box<[u8]> {
    use jig_client_protocol_wire::v19 as w;
    let answer = w::ApprovalAskAnswer::Asked(
        w::ApprovalAsked::new(
            &w::CEILINGS,
            w::ApprovalAskedParts {
                secret: skein_lib::Secret::new(Box::new([7; 32])),
                code: *b"WDJBMJHT",
                expires: 600_000_000_000,
            },
        )
        .expect("approval"),
    );
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("short answer"));
    answer.encode(&mut writer).expect("answer");
    let body = writer.finish();
    let mut response =
        format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    response.extend_from_slice(&body);
    response.into_boxed_slice()
}

#[test]
fn approval_open_is_carried_and_closing_waits_for_no_opener() {
    const FALLBACK: &[u8] = b"Open this address in a browser";
    for (opener, outcome, fallback) in [
        (false, Outcome::Success, true),
        (true, Outcome::Success, false),
        (true, Outcome::Failure, true),
        (true, Outcome::Refused, true),
        (true, Outcome::Running, false),
    ] {
        let mut world = World::configured(false, true, opener, outcome);
        world.settle();
        assert_eq!(world.spawns, usize::from(opener), "{opener} {outcome:?}");
        assert_eq!(
            world.bytes.windows(FALLBACK.len()).any(|part| part == FALLBACK),
            fallback,
            "{opener} {outcome:?}: {}",
            String::from_utf8_lossy(&world.bytes)
        );
        if !opener {
            assert!(
                !world.bytes.windows(b"A tab opened".len()).any(|part| part == b"A tab opened"),
                "no transient opened frame before the same-iteration refusal"
            );
        }
        world.draining = true;
        world.sim.peer_feed(world.pid, world.input, b"\x03");
        world.settle();
        world.drained(false);
    }
}

impl World {
    fn keys(&mut self, bytes: &[u8]) {
        assert_eq!(self.sim.peer_feed(self.pid, self.input, bytes), bytes.len());
        self.settle();
    }

    fn recording_file(&mut self) -> Option<(Vec<u8>, u32)> {
        let root = self.files.reopen_root(0);
        let file = self.files.open(root, b"tui/recording", skein_fake_machine::How::ReadNoFollow);
        let result = file.ok().map(|file| {
            let mode = self.files.stat(file).mode;
            let bytes =
                self.files.read(file, 0, u32::try_from(ROOM).expect("test recording bound")).expect("recording");
            self.files.close(file);
            (bytes, mode)
        });
        self.files.close(root);
        result
    }
}

#[cfg(feature = "record")]
fn replay(recording: &[u8]) -> Vec<temper_client_shell_tui::journey::Line> {
    use temper_client_shell_tui::journey;
    let lines = journey::lines(recording).expect("read recording");
    assert!(
        !lines
            .iter()
            .any(|line| matches!(line, journey::Line::Key { key: skein_ui_screen_terminal::Key::Alt('M' | 'S'), .. })),
        "recording controls never reach the host"
    );
    let mut replay = journey::Replay::new(recording).expect("replay envelope");
    while let Some(mark) = replay.next().expect("same host duties") {
        assert_eq!(mark.live.as_deref(), Some(mark.frame.print().as_ref()), "live frame replays");
    }
    lines
}

#[test]
#[cfg(feature = "record")]
fn recording_keys_save_whole_and_closing_saves_again() {
    use temper_client_shell_tui::journey::Line;
    let mut world = World::recording(false, true, false, Outcome::Success, Recording::Root);
    world.settle();
    world.keys(b"\x1bM\x1bS");
    let (first, mode) = world.recording_file().expect("save key writes recording");
    assert_eq!(mode, 0o600, "recording is private");
    let lines = replay(&first);
    assert!(lines.contains(&Line::Seed(SEED)));
    assert!(lines.contains(&Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH }));
    assert!(lines.contains(&Line::Screen { size: SIZE, scheme: Scheme::Dark }));
    assert!(lines.contains(&Line::Address(b"127.0.0.1:8367".as_slice().into())));
    assert!(lines.iter().any(|line| matches!(line, Line::Mark(Some(_)))));
    assert!(matches!(lines.last(), Some(Line::End(Some(_)))));
    assert!(
        !lines.iter().any(|line| matches!(line, Line::Key { key: skein_ui_screen_terminal::Key::Alt(_), .. })),
        "recording keys never reach the host"
    );
    world.keys(b"\t\x1bS");
    let (second, mode) = world.recording_file().expect("second save");
    assert_eq!(mode, 0o600);
    assert!(second.len() > first.len(), "second save includes more reports");
    replay(&second);
    world.draining = true;
    world.leave_after = world.saved + 1;
    world.keys(b"\x03");
    let (closed, mode) = world.recording_file().expect("closing save");
    assert_eq!(mode, 0o600);
    assert!(closed.len() > second.len(), "closing saves the final reports");
    assert!(replay(&closed).contains(&Line::Quit));
    world.drained(false);
}

#[test]
fn without_recording_root_keys_write_no_file() {
    let mut world = World::new(false);
    world.settle();
    world.keys(b"\x1bM\x1bS");
    assert!(world.recording_file().is_none());
    world.keys(b"\x03");
    assert!(world.recording_file().is_none());
    world.drained(false);
}

#[test]
#[cfg(feature = "record")]
fn a_failed_recording_save_is_silent_and_retried() {
    let mut world = World::recording(false, true, false, Outcome::Success, Recording::Root);
    world.settle();
    let drawn = world.bytes.len();
    world.refuse_save = true;
    world.keys(b"\x1bS");
    assert_eq!(world.bytes.len(), drawn, "a failed save draws nothing");
    assert!(!world.refuse_save, "the store reached the injected failure");
    assert!(world.recording_file().is_none(), "failed rename left no recording");
    world.keys(b"\x1bS");
    replay(&world.recording_file().expect("next save retries").0);
    world.draining = true;
    world.leave_after = world.saved + 1;
    world.keys(b"\x03");
    world.drained(false);
}

#[test]
#[cfg(feature = "record")]
fn recording_facts_include_the_loaded_records_and_credential() {
    use skein_fake_machine::How;
    use temper_client_shell_tui::journey::Line;
    const KEY: &[u8] = b"test-store";
    const VALUE: &[u8] = b"test-value";
    // The native credential's sign-in number and 32 fake secret bytes.
    const CREDENTIAL: &[u8] = &[7; 40];
    let mut store = 1_u32.to_be_bytes().to_vec();
    for field in [KEY, VALUE] {
        store.extend_from_slice(&u32::try_from(field.len()).expect("short record").to_be_bytes());
        store.extend_from_slice(field);
    }
    let mut world = World::recording(false, false, false, Outcome::Success, Recording::Root);
    let root = world.files.reopen_root(0);
    for (path, bytes) in [(b"tui/store".as_slice(), store.as_slice()), (b"tui/token".as_slice(), CREDENTIAL)] {
        let file = world.files.open(root, path, How::Create { mode: 0o600 }).expect("seed private file");
        world.files.write(file, 0, bytes).expect("seed bytes");
        world.files.close(file);
    }
    world.files.close(root);
    world.settle();
    world.keys(b"\x1bS");
    let lines = replay(&world.recording_file().expect("saved loaded facts").0);
    assert!(lines.contains(&Line::Stored { key: KEY.into(), bytes: VALUE.into() }));
    assert!(lines.contains(&Line::Credential(CREDENTIAL.into())));
    world.leave_after = world.saved + 1;
    world.keys(b"\x03");
    world.drained(false);
}
