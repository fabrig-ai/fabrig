mod fake_box;

use skein_fake_machine::{Item, Machine};
use skein_io::{
    self as io,
    kernel::{Complete, Exit, Fd, Op, ServiceSignal, Submit},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, stream};
use skein_shell::Host;
use skein_sim::{Answer, Ask, Call, Config, Handle, Pid, Program, Reply, Sim};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::{Depth, ENTER, LEAVE};
use temper_client_shell_tui::{
    limits,
    opener::Opener,
    run::{Run, Settings},
};
use temper_client_tui::{Note, Scheme};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum NoteWrite {
    Done,
    Hold,
    Refuse,
}

struct World {
    sim: Sim,
    files: Machine,
    pid: Pid,
    input: Fd,
    output: Fd,
    signals: Fd,
    children: Vec<fake_box::Child>,
    bound: std::collections::BTreeSet<Token>,
    hold_link: bool,
    connects: Vec<io::Addr>,
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
    note_write: NoteWrite,
    held_note: Option<Call>,
    horizon: Time,
}
impl World {
    fn new(refuse: bool) -> Self {
        Self::configured(refuse, false, false, Outcome::Success)
    }

    fn configured(refuse: bool, approval: bool, opener: bool, outcome: Outcome) -> Self {
        Self::recording(refuse, approval, opener, outcome, Recording::None)
    }

    fn recording(refuse: bool, approval: bool, opener: bool, outcome: Outcome, record: Recording) -> Self {
        Self::options(refuse, approval, opener, outcome, record, Some(Scheme::Dark), None, false, None)
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::fn_params_excessive_bools,
        reason = "the simulated run settings vary independently"
    )]
    fn options(
        refuse: bool,
        approval: bool,
        opener: bool,
        outcome: Outcome,
        record: Recording,
        scheme: Option<Scheme>,
        link: Option<&[u8]>,
        notes: bool,
        note: Option<Note>,
    ) -> Self {
        Self::front_options(refuse, approval, opener, outcome, record, scheme, link, notes, note, false)
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::fn_params_excessive_bools,
        reason = "the simulated front adds an independent inherited pipe"
    )]
    fn front_options(
        refuse: bool,
        approval: bool,
        opener: bool,
        outcome: Outcome,
        record: Recording,
        scheme: Option<Scheme>,
        link: Option<&[u8]>,
        notes: bool,
        note: Option<Note>,
        front: bool,
    ) -> Self {
        let mut files = Machine::for_user(USER);
        let root = files.lay(&[Item::directory(b"tui").mode(if refuse { 0o755 } else { 0o700 })]);
        let mut sim = Sim::new(SEED, Config::calm());
        let pid = sim.spawn_process();
        let recording = (record == Recording::Root).then(|| {
            let fd = files.open(root, b".", skein_fake_machine::How::Directory).expect("recording root");
            sim.root(pid, Handle::new(fd.raw()))
        });
        let note_root = notes.then(|| {
            if let Some(root) = recording {
                return root;
            }
            let fd = files.open(root, b".", skein_fake_machine::How::Directory).expect("note root");
            sim.root(pid, Handle::new(fd.raw()))
        });
        let spawn_root = front.then(|| {
            let fd = files.open(root, b".", skein_fake_machine::How::Directory).expect("captured cwd");
            sim.root(pid, Handle::new(fd.raw()))
        });
        let root = sim.root(pid, Handle::new(root.raw()));
        let input = sim.open_inherited_read(pid);
        let output = sim.open_inherited_write(pid);
        let signals = sim.open_signal_source(pid);
        let engine = approval.then(|| Engine::new(&mut sim));
        let settings = Settings {
            record: recording,
            front: spawn_root.map(|directory| temper_client_shell_tui::run::Front {
                executable: b"/bin/temper".as_slice().into(),
                cache: b"/cache".as_slice().into(),
                arguments: vec![Box::from(b"--data".as_slice()), Box::from(b"/data".as_slice())].into(),
                environment: vec![Box::from(b"HOME=/home/person".as_slice())].into(),
                directory,
                log: b"/data/box/logs/box.log".as_slice().into(),
            }),
            engine: skein_io::Addr::Ip(
                if front { "127.0.0.1:9000" } else { "127.0.0.1:8367" }.parse().expect("loopback"),
            ),
            directory: b"tui".as_slice().into(),
            address: if front { b"127.0.0.1:9000" } else { b"127.0.0.1:8367" }.as_slice().into(),
            scheme,
            boxed: true,
            link_file: link.map(Into::into),
            note,
            files: note_root,
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
            children: Vec::new(),
            bound: std::collections::BTreeSet::new(),
            hold_link: false,
            connects: Vec::new(),
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
            note_write: NoteWrite::Done,
            held_note: None,
            horizon: Time::ZERO.saturating_add(limits::BACKGROUND).saturating_add(Duration::from_millis(40)),
        }
    }

    fn settle(&mut self) {
        // The front wakes for its own work, completions and deadlines; the
        // engine's turns must not supply a wake the real front would miss.
        let mut runnable = true;
        for _ in 0..512 {
            if runnable
                || self.sim.ready(self.pid) > 0
                || self.sim.deferred(self.pid)
                || self.run.next_deadline().is_some_and(|at| at <= self.now)
            {
                self.sim.reap(self.pid, self.run.completions());
                let spawns: Vec<_> = self
                    .run
                    .completions()
                    .iter()
                    .filter_map(|complete| {
                        if let (Op::Spawn { spawn }, Ok(io::kernel::Done::Spawned { pidfd, .. })) =
                            (&complete.kind, &complete.result)
                            && [b"/bin/temper".as_slice(), b"/bin/temper-box"].contains(&spawn.program.as_ref())
                            && self.bound.insert(complete.op)
                        {
                            Some((*pidfd, spawn.program.as_ref() == b"/bin/temper-box"))
                        } else {
                            None
                        }
                    })
                    .collect();
                for (pidfd, boxed) in spawns {
                    self.children.push(fake_box::Child::new(&mut self.sim, self.pid, pidfd, boxed, !self.hold_link));
                }
                self.run.iterate(self.now, Wall::from_nanos(self.now.as_nanos()));
                runnable = self.run.work_pending(self.now);
            }
            for submit in self.run.submissions().iter() {
                if let Op::Spawn { spawn } = &submit.kind {
                    if spawn.program.as_ref() == b"/bin/browser" {
                        self.spawns += 1;
                        assert_eq!(spawn.args.as_ref(), &[Box::<[u8]>::from(APPROVAL)]);
                    } else {
                        assert!([b"/bin/temper".as_slice(), b"/bin/temper-box"].contains(&spawn.program.as_ref()));
                    }
                }
                if let Op::Connect { addr, .. } = &submit.kind {
                    self.connects.push(io::Addr::Ip(*addr));
                }
                if let Op::Signal { to, .. } = &submit.kind {
                    assert_eq!(*to, io::kernel::Target::Child, "the browser's group stays");
                }
            }
            self.sim.submit(self.pid, self.run.submissions());
            self.serve();
            for child in &mut self.children {
                child.iterate(&mut self.sim, self.now);
            }
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
            if runnable
                || self.sim.ready(self.pid) > 0
                || self.sim.deferred(self.pid)
                || self.run.work_pending(self.now)
                || self.engine.as_ref().is_some_and(|engine| engine.pending(&self.sim))
                || self.children.iter().any(|child| child.pending(&self.sim))
            {
                continue;
            }
            let next = [self.sim.next_due(), self.run.next_mechanism_deadline(), self.run.next_policy_deadline()]
                .into_iter()
                .flatten()
                .min();
            if let Some(next) = next {
                assert!(next > self.now, "idle deadline is in the future");
                if !self.draining && next > self.horizon {
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
            if let Ask::Rename { to, .. } = &call.ask
                && to.as_ref() == b"sign-in"
            {
                match std::mem::replace(&mut self.note_write, NoteWrite::Done) {
                    NoteWrite::Hold => {
                        self.held_note = Some(call);
                        continue;
                    }
                    NoteWrite::Refuse => {
                        answers.push(Answer { ticket: call.ticket, result: Err(io::kernel::Error::Permission) });
                        continue;
                    }
                    NoteWrite::Done => {}
                }
            }
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
            if let Ask::Spawn { program, args, .. } = &call.ask {
                if [b"/bin/temper".as_slice(), b"/bin/temper-box"].contains(&program.as_ref()) {
                    answers.push(Answer { ticket: call.ticket, result: Ok(Reply::Program(Program::Service)) });
                    continue;
                }
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

    fn box_child(&mut self) -> &mut fake_box::Child {
        self.children.iter_mut().find(|child| child.boxed).expect("box spawned")
    }

    fn drained(&mut self, refused: bool) {
        assert!(self.bytes.starts_with(ENTER), "ENTER first");
        assert!(self.bytes.ends_with(LEAVE), "LEAVE last");
        assert_eq!(self.bytes.windows(LEAVE.len()).filter(|part| *part == LEAVE).count(), 1);
        assert_eq!(self.run.refused(), refused);
        assert_eq!(self.run.exit(), Some(Exit::Code(u8::from(refused || self.run.terminated()))));
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
    let recording = [Recording::None, Recording::Root];
    for (way, front, record) in [
        (WayOut::End, false),
        (WayOut::ControlC, false),
        (WayOut::Termination, false),
        (WayOut::Refused, false),
        (WayOut::ControlC, true),
    ]
    .into_iter()
    .flat_map(|(way, front)| recording.iter().map(move |record| (way, front, *record)))
    {
        let began = clock.now().now;
        let mut world = World::front_options(
            matches!(way, WayOut::Refused),
            false,
            false,
            Outcome::Success,
            record,
            Some(Scheme::Dark),
            None,
            false,
            None,
            front,
        );
        world.settle();
        if matches!(way, WayOut::Refused) {
            assert!(!world.bytes.windows(SIGN_IN.len()).any(|part| part == SIGN_IN), "no Start on refusal");
            world.drained(true);
            println!("{way:?}: {} ns", clock.now().now.as_nanos().saturating_sub(began.as_nanos()));
            continue;
        }
        assert!(world.bytes.starts_with(ENTER), "{way:?}: ENTER");
        if !front {
            assert!(world.bytes.windows(SIGN_IN.len()).any(|part| part == SIGN_IN), "{way:?}: Start drawn");
        }
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
            WayOut::Refused => unreachable!("early closing checked above"),
        }
        world.settle();
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
    signed_in: bool,
    stream: Option<Token>,
    deny_stream: bool,
    denied: usize,
    bodies: std::collections::BTreeSet<Token>,
    discarding: std::collections::BTreeSet<Token>,
    link_answer: Option<jig_client_protocol_wire::v20::SignInAnswer>,
    link_sign_ins: usize,
    approvals: usize,
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
            signed_in: false,
            stream: None,
            deny_stream: false,
            denied: 0,
            bodies: std::collections::BTreeSet::new(),
            discarding: std::collections::BTreeSet::new(),
            link_answer: None,
            link_sign_ins: 0,
            approvals: 0,
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
                    self.receive(owner, &bytes);
                }
                io::Event::Stream { owner, up: stream::Up::Room } => {
                    let response = self.responses.remove(&owner).expect("response room");
                    self.down(io::Request::Stream { stream: owner, down: stream::Down::Send(response) });
                    if self.stream != Some(owner) {
                        self.down(io::Request::Stream { stream: owner, down: stream::Down::Finish });
                        self.down(io::Request::Close { entity: owner });
                    }
                }
                io::Event::Stream { owner, up: stream::Up::End | stream::Up::Failed(_) } => {
                    self.down(io::Request::Abort { entity: owner });
                }
                io::Event::Closed { owner } => {
                    self.entities.remove(&owner);
                    self.discarding.remove(&owner);
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
    fn receive(&mut self, owner: Token, bytes: &[u8]) {
        if self.discarding.contains(&owner) {
            self.down(io::Request::Stream {
                stream: owner,
                down: stream::Down::Demand { read: stream::Read::Arrived { max: 1024 }, room: 0 },
            });
        } else if self.bodies.remove(&owner) {
            self.watch(owner, bytes);
        } else if bytes.starts_with(b"GET /jig/v1/stream ") && self.deny_stream {
            self.denied += 1;
            self.respond(owner, b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n".as_slice().into());
        } else if bytes.starts_with(b"GET /jig/v1/stream ") {
            self.stream = Some(owner);
            let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n".to_vec();
            response.extend_from_slice(&hello_event());
            self.respond(owner, response.into_boxed_slice());
        } else if bytes.starts_with(b"POST /jig/v1/watch ") {
            let header = std::str::from_utf8(bytes).expect("HTTP header");
            let length = header
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .map(|(_, value)| value.trim().parse::<u32>().expect("body size"))
                })
                .expect("content length");
            self.bodies.insert(owner);
            self.down(io::Request::Stream {
                stream: owner,
                down: stream::Down::Demand { read: stream::Read::Fill(length), room: 0 },
            });
        } else if bytes.starts_with(b"POST /jig/v1/approval/ask ") {
            self.approvals += 1;
            self.respond(owner, approval_response());
        } else if bytes.starts_with(b"POST /jig/v1/sign-in/link ") {
            self.link_sign_ins += 1;
            if let Some(answer) = self.link_answer.take() {
                let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer size"));
                answer.encode(&mut writer).expect("answer");
                self.respond(owner, http_response(&writer.finish()));
            } else {
                self.signed_in = true;
                self.respond(owner, sign_in_response());
            }
        } else {
            assert!(
                self.signed_in || bytes.starts_with(b"POST /jig/v1/sign-in/approval "),
                "only the ask and take reach the engine"
            );
            self.discarding.insert(owner);
            self.down(io::Request::Stream {
                stream: owner,
                down: stream::Down::Demand { read: stream::Read::Arrived { max: 1024 }, room: 0 },
            });
        }
    }

    fn watch(&mut self, owner: Token, bytes: &[u8]) {
        use jig_client_protocol_wire::v20 as w;
        let open = w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("watch body");
        if open.subject().owner() == &w::Owner::Jig
            && w::Subject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(open.subject().record())).expect("subject")
                == w::Subject::Me
        {
            self.respond(self.stream.expect("opened stream"), me_event(open.watch()));
        }
        self.respond(owner, {
            let answer = w::OpenAnswer::Admitted;
            let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("short admitted"));
            answer.encode(&mut writer).expect("admitted");
            http_response(&writer.finish())
        });
    }

    fn respond(&mut self, owner: Token, response: Box<[u8]>) {
        let room = u32::try_from(response.len()).expect("short response");
        self.responses.insert(owner, response);
        self.down(io::Request::Stream {
            stream: owner,
            down: stream::Down::Demand { read: stream::Read::Nothing, room },
        });
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
    use jig_client_protocol_wire::v20 as w;
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
    http_response(&body)
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
        self.file(b"tui/recording")
    }

    fn file(&mut self, path: &[u8]) -> Option<(Vec<u8>, u32)> {
        let root = self.files.reopen_root(0);
        let file = self.files.open(root, path, skein_fake_machine::How::ReadNoFollow);
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

fn replay(recording: &[u8]) -> Vec<temper_client_shell_tui::journey::Line> {
    use temper_client_shell_tui::journey;
    let lines = journey::lines(recording).expect("read recording");
    assert!(
        !lines.iter().any(|line| matches!(
            line,
            journey::Line::Key { key: skein_ui_screen_terminal::Key::Alt('M' | 'S' | 'm' | 's'), .. }
        )),
        "recording controls never reach the host"
    );
    let mut replay = journey::Replay::new(recording).expect("replay envelope");
    while let Some(mark) = replay.next().expect("same host duties") {
        assert_eq!(mark.live.as_deref(), Some(mark.frame.print().as_ref()), "live frame replays");
    }
    lines
}

#[test]
fn recording_keys_save_whole_and_closing_saves_again() {
    use temper_client_shell_tui::journey::Line;
    for (mark, save) in [(b"\x1bM", b"\x1bS"), (b"\x1bm", b"\x1bs")] {
        let mut world = World::recording(false, true, false, Outcome::Success, Recording::Root);
        world.settle();
        world.keys(&[mark.as_slice(), save.as_slice()].concat());
        let (first, mode) = world.recording_file().unwrap_or_else(|| panic!("{mark:?}: save key writes recording"));
        assert_eq!(mode, 0o600, "{mark:?}: recording is private");
        let lines = replay(&first);
        assert!(lines.contains(&Line::Seed(SEED)), "{mark:?}");
        assert!(lines.contains(&Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH }), "{mark:?}");
        assert!(lines.contains(&Line::Screen { size: SIZE, scheme: Some(Scheme::Dark) }), "{mark:?}");
        assert!(lines.contains(&Line::Address(b"127.0.0.1:8367".as_slice().into())), "{mark:?}");
        assert!(lines.iter().any(|line| matches!(line, Line::Mark(Some(_)))), "{mark:?}");
        assert!(matches!(lines.last(), Some(Line::End(Some(_)))), "{mark:?}");
        assert!(
            !lines.iter().any(|line| matches!(line, Line::Key { key: skein_ui_screen_terminal::Key::Alt(_), .. })),
            "{mark:?}: recording keys never reach the host"
        );
        world.keys(&[b"\t".as_slice(), save.as_slice()].concat());
        let (second, mode) = world.recording_file().unwrap_or_else(|| panic!("{mark:?}: second save"));
        assert_eq!(mode, 0o600, "{mark:?}");
        assert!(second.len() > first.len(), "{mark:?}: second save includes more reports");
        replay(&second);
        world.draining = true;
        world.leave_after = world.saved + 1;
        world.keys(b"\x03");
        let (closed, mode) = world.recording_file().unwrap_or_else(|| panic!("{mark:?}: closing save"));
        assert_eq!(mode, 0o600, "{mark:?}");
        assert!(closed.len() > second.len(), "{mark:?}: closing saves the final reports");
        assert!(replay(&closed).contains(&Line::Quit), "{mark:?}");
        world.drained(false);
    }
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
    let note = Note::Until(Wall::from_nanos(600_000_000_000));
    let mut world = World::options(
        false,
        false,
        false,
        Outcome::Success,
        Recording::Root,
        Some(Scheme::Dark),
        None,
        false,
        Some(note),
    );
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
    assert!(lines.contains(&Line::Noted(note)));
    world.leave_after = world.saved + 1;
    world.keys(b"\x03");
    world.drained(false);
}

#[test]
fn background_selects_the_recorded_start_scheme() {
    use skein_ui_screen::Rgb;
    use skein_ui_screen_terminal::ASK_BACKGROUND;
    use temper_client_shell_tui::journey::Line;
    let light = Rgb { red: 253, green: 246, blue: 227 };
    let dark = Rgb { red: 0, green: 43, blue: 54 };
    for (scheme, answer, expected, background) in [
        (None, Some(b"\x1b]11;rgb:fdfd/f6f6/e3e3\x1b\\".as_slice()), Some(Scheme::Light), Some(light)),
        (None, Some(b"\x1b]11;rgb:0/2b/36\x07".as_slice()), Some(Scheme::Dark), Some(dark)),
        (None, None, None, None),
        (Some(Scheme::Light), None, Some(Scheme::Light), None),
        (Some(Scheme::Light), Some(b"\x1b]11;rgb:0/2b/36\x07".as_slice()), Some(Scheme::Light), Some(dark)),
        (Some(Scheme::Dark), Some(b"\x1b]11;rgb:fdfd/f6f6/e3e3\x1b\\".as_slice()), Some(Scheme::Dark), Some(light)),
    ] {
        let mut world =
            World::options(false, false, false, Outcome::Success, Recording::Root, scheme, None, false, None);
        world.horizon = Time::ZERO.saturating_add(Duration::from_millis(40));
        world.settle();
        assert_eq!(world.bytes.as_slice(), [ENTER, ASK_BACKGROUND].concat(), "every start asks before drawing");
        assert!(
            world.run.next_mechanism_deadline().is_some_and(|at| at <= Time::ZERO.saturating_add(limits::BACKGROUND))
        );
        if let Some(answer) = answer {
            world.keys(answer);
        } else {
            world.now = Time::ZERO.saturating_add(limits::BACKGROUND);
            world.sim.advance_to(world.now);
            world.settle();
        }
        world.keys(b"\x1bS");
        let records = replay(&world.recording_file().expect("start recording").0);
        assert!(records.contains(&Line::Screen { size: SIZE, scheme: expected }));
        let recorded = records.iter().find_map(|line| if let Line::Background(rgb) = line { Some(*rgb) } else { None });
        assert_eq!(recorded, background, "background is independent of the scheme: {scheme:?}");
        assert!(records.contains(&Line::Box));
        world.draining = true;
        world.keys(b"\x03");
        world.drained(false);
    }
}

const LOCAL_SECRET: &[u8] = b"AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM";
const LOCAL_LINK: &[u8] =
    b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM";

#[test]
fn a_fresh_tui_reads_the_runtime_link_outside_its_state_directory() {
    for state in ["state-auto", "state-retry", "state-bothenv"] {
        let mut world = World::options(
            false,
            true,
            false,
            Outcome::Success,
            Recording::None,
            Some(Scheme::Dark),
            Some(b"/runtime/temper/printed-link-deployment"),
            false,
            None,
        );
        let runtime =
            world.files.lay(&[Item::file(b"printed-link-deployment", &[LOCAL_LINK, b"\n"].concat()).mode(0o600)]);
        let root = world.sim.root(world.pid, Handle::new(runtime.raw()));
        world.run = world.run.with_local_link(root, b"/runtime/temper/printed-link-deployment");
        world.settle();
        let engine = world.engine.as_ref().expect("fake engine");
        assert_eq!(engine.link_sign_ins, 1, "{state}: automatic link sign-in");
        assert_eq!(engine.approvals, 0, "{state}: no browser approval");
        let screen = terminal_text(&world.bytes);
        assert!(screen.contains("open http://127.0.0.1:8367 in a"), "{state}: {screen}");
        assert!(screen.contains("No project yet"), "{state}: {screen}");
        assert!(screen.contains("● Live"), "{state}: {screen}");
        assert!(!screen.contains("Signed out"), "{state}: {screen}");
        world.draining = true;
        world.sim.peer_close(world.pid, world.input);
        world.settle();
        assert_eq!(world.run.exit(), Some(Exit::Code(0)));
        world.sim.assert_no_open_fds(world.pid);
        world.sim.assert_quiescent(world.pid);
        assert_eq!(world.files.open_handles(), 0);
    }
}

fn terminal_text(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(ENTER).expect("terminal enters full screen");
    let bytes = bytes.strip_prefix(skein_ui_screen_terminal::ASK_BACKGROUND).expect("terminal asks its background");
    let mut cells = std::collections::BTreeMap::new();
    let mut at = (1, 1);
    let mut chars = std::str::from_utf8(bytes).expect("terminal UTF-8").chars();
    while let Some(glyph) = chars.next() {
        if glyph != '\x1b' {
            cells.insert(at, glyph);
            at.1 += 1;
            continue;
        }
        assert_eq!(chars.next(), Some('['), "terminal emits CSI commands");
        let mut parameters = String::new();
        let command = loop {
            let byte = chars.next().expect("complete CSI command");
            if ('@'..='~').contains(&byte) {
                break byte;
            }
            parameters.push(byte);
        };
        match command {
            'H' => {
                at = parameters.split_once(';').map_or((1, 1), |(row, column)| {
                    (row.parse().expect("cursor row"), column.parse().expect("cursor column"))
                });
            }
            'J' => {
                assert_eq!(parameters, "2", "whole-screen clear");
                cells.clear();
            }
            'm' | 'h' | 'l' => {}
            other => panic!("unexpected terminal command: {other}"),
        }
    }
    cells.values().collect()
}

#[test]
fn local_file_is_read_and_reported_before_sign_in() {
    use temper_client_shell_tui::journey::{Line, Sent};
    for (name, bytes, mode, found) in [
        ("private", Some([LOCAL_LINK, b"\n"].concat()), 0o600, Some(LOCAL_LINK)),
        ("missing", None, 0o600, None),
        ("public", Some(LOCAL_LINK.to_vec()), 0o644, None),
        ("past bound", Some(vec![b'x'; 4097]), 0o600, None),
        ("no final newline", Some(LOCAL_LINK.to_vec()), 0o600, Some(LOCAL_LINK)),
    ] {
        let mut world = World::options(
            false,
            false,
            false,
            Outcome::Success,
            Recording::Root,
            Some(Scheme::Dark),
            Some(b"printed-link"),
            false,
            None,
        );
        if let Some(bytes) = bytes {
            let root = world.files.reopen_root(0);
            let file = world
                .files
                .open(root, b"printed-link", skein_fake_machine::How::Create { mode })
                .expect("fixture link");
            world.files.write(file, 0, &bytes).expect("fixture bytes");
            world.files.close(file);
            world.files.close(root);
        }
        world.settle();
        world.keys(b"\x1bS");
        let lines = replay(&world.recording_file().expect("local recording").0);
        let read = lines.iter().position(|line| *line == Line::Read).expect("local duty");
        let found_at = lines
            .iter()
            .position(|line| *line == Line::Found(found.map(Into::into)))
            .expect("local result, final newline dropped");
        assert!(read < found_at, "{name}");
        if found.is_some() {
            let sent = lines
                .iter()
                .position(|line| matches!(line, Line::Send { message: Sent::SignInLink(_), .. }))
                .expect("sign-in-link send");
            assert!(found_at < sent, "{name}");
        }
        assert!(
            !world.bytes.windows(LOCAL_SECRET.len()).any(|bytes| bytes == LOCAL_SECRET),
            "link absent from terminal"
        );
        world.draining = true;
        world.keys(b"\x03");
        world.drained(false);
    }
}

#[test]
fn sign_in_expiry_writes_a_private_note_only_with_a_root() {
    for (notes, recording, fault) in [
        (true, Recording::None, NoteWrite::Done),
        (false, Recording::None, NoteWrite::Done),
        (true, Recording::None, NoteWrite::Hold),
        (true, Recording::None, NoteWrite::Refuse),
        (true, Recording::Root, NoteWrite::Hold),
    ] {
        let mut world =
            World::options(false, true, false, Outcome::Success, recording, Some(Scheme::Dark), None, notes, None);
        world.settle();
        world.note_write = fault;
        world.keys(&[b"\x1b[200~", LOCAL_LINK, b"\x1b[201~\r"].concat());
        if fault != NoteWrite::Done {
            assert!(world.file(b"tui/sign-in").is_none(), "first note has not replaced the file");
            assert_eq!(world.held_note.is_some(), fault == NoteWrite::Hold);
            let engine = world.engine.as_mut().expect("engine");
            engine.deny_stream = true;
            let entity = engine.stream.take().expect("live stream");
            engine.down(io::Request::Abort { entity });
            world.horizon = world.now.saturating_add(Duration::from_secs(2));
            world.settle();
            assert!(world.run.signed_in().is_some());
            assert!(world.engine.as_ref().expect("engine").denied > 0, "reconnected stream refused");
            if fault == NoteWrite::Hold {
                world.keys(b"\x03");
                assert!(!world.run.is_empty(), "closing drains the note");
                assert!(!world.bytes.ends_with(LEAVE), "note drains before LEAVE");
            }
            if let Some(call) = world.held_note.take() {
                let mut answers = Queue::with_capacity(4);
                skein_fake_machine::step(&mut world.files, call, &mut answers);
                world.sim.answer(&mut answers);
                world.settle();
            }
        }
        let expected = if fault == NoteWrite::Done {
            b"127.0.0.1:8367 until 600000000000\n".as_slice()
        } else {
            b"127.0.0.1:8367 signed-out\n".as_slice()
        };
        assert_eq!(
            world.file(b"tui/sign-in"),
            notes.then(|| (expected.to_vec(), 0o600)),
            "expiry note written whole and private"
        );
        assert!(!world.bytes.windows(LOCAL_SECRET.len()).any(|bytes| bytes == LOCAL_SECRET));
        world.draining = true;
        if !world.run.is_empty() {
            world.keys(b"\x03");
        }
        world.settle();
        // A successful sign-in reports its address to the binary.
        assert!(world.run.is_empty());
    }
}

fn sign_in_response() -> Box<[u8]> {
    use jig_client_protocol_wire::v20 as w;
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts {
                token: Some(skein_lib::Secret::new(Box::new([7; 40]))),
                party: 17,
                expires: 600_000_000_000,
            },
        )
        .expect("signed in"),
    );
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("short answer"));
    answer.encode(&mut writer).expect("answer");
    let body = writer.finish();
    http_response(&body)
}

fn hello_event() -> Box<[u8]> {
    use jig_client_protocol_wire::v20 as w;
    use skein_lib::List;
    let limits = &w::CEILINGS;
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(
                limits,
                w::ServedParts {
                    owner: w::Owner::Jig,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
            )
            .expect("Jig version"),
        )
        .expect("one owner");
    let hello = w::StreamEvent::Hello(
        w::Hello::new(limits, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
    );
    sse(&hello)
}
fn me_event(watch: u64) -> Box<[u8]> {
    use jig_client_protocol_wire::v20 as w;
    use skein_lib::Writer;
    let limits = &w::CEILINGS;
    let text = |bytes: &[u8]| {
        w::DocumentText::new(limits, w::DocumentTextParts { bytes: bytes.into(), cut: false }).expect("text")
    };
    let me = w::Me::new(
        limits,
        w::MeParts {
            party: w::MeParty::new(
                limits,
                w::MePartyParts { number: 7, kind: w::PartyKind::Person, login: text(b"owner"), name: text(b"Owner") },
            )
            .expect("party"),
            projects: Box::new([]),
            needs: w::Needs::new(limits, w::NeedsParts { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 })
                .expect("needs"),
            needs_total: 0,
            updates: w::Count::new(limits, w::CountParts { value: 0, limited: false }).expect("count"),
            results_seen: 0,
            cut: false,
            retention: Duration::from_secs(1000),
            deployment_role: None,
            providers: Box::new([]),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 256,
            words_bytes: 8192,
        },
    )
    .expect("Me");
    let value = w::DeliveryValue::Me(me);
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("Me size"));
    value.encode(&mut writer).expect("Me fits");
    let document = w::Owned::new(
        limits,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
    )
    .expect("Me document");
    let delivery =
        w::StreamEvent::Delivery(w::Delivery::new(limits, w::DeliveryParts { watch, document }).expect("delivery"));
    sse(&delivery)
}

fn sse(event: &jig_client_protocol_wire::v20::StreamEvent) -> Box<[u8]> {
    use skein_lib::{
        Writer,
        base64::{self, Alphabet},
    };
    let mut writer = Writer::new(usize::try_from(event.measure()).expect("event size"));
    event.encode(&mut writer).expect("event");
    let bytes = writer.finish();
    let mut writer = Writer::new(
        usize::try_from(
            base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("short event"))
                .expect("base64 size"),
        )
        .expect("base64 size"),
    );
    base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("base64");
    [b"data: ", writer.finish().as_ref(), b"\n\n"].concat().into_boxed_slice()
}

fn http_response(body: &[u8]) -> Box<[u8]> {
    let mut response =
        format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    response.extend_from_slice(body);
    response.into_boxed_slice()
}

const HANDED_LINK: &[u8] =
    b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[test]
fn a_front_connects_after_early_readiness_or_a_refused_connect() {
    // Native monotonic time is already past the origin when the front starts.
    const STARTED_AT: Time = Time::from_nanos(60_000_000_000);
    // Fault probabilities are in thousandths.
    const EVERY_COMPLETION: u32 = 1000;
    for (listening, latency) in [(true, 0), (false, 0), (true, EVERY_COMPLETION)] {
        let mut world = World::front_options(
            false,
            listening,
            false,
            Outcome::Success,
            Recording::None,
            Some(Scheme::Dark),
            None,
            false,
            None,
            true,
        );
        world.now = STARTED_AT;
        world.sim.advance_to(world.now);
        world.sim.set_faults(skein_sim::Faults { latency, latency_max: limits::BACKGROUND, ..skein_sim::Faults::NONE });
        world.horizon = world.now.saturating_add(limits::BACKGROUND);
        world.settle();
        if world.engine.is_none() {
            world.engine = Some(Engine::new(&mut world.sim));
        }
        world.horizon = world.now.saturating_add(limits::tui().native.connections.deadline);
        world.settle();
        assert!(world.run.signed_in().is_some(), "front signs in: listening={listening}, latency={latency}");
        assert!(terminal_text(&world.bytes).contains("● Live"));
        world.draining = true;
        world.keys(b"\x03");
        assert_eq!(world.run.exit(), Some(Exit::Code(0)));
        world.sim.assert_no_open_fds(world.pid);
        world.sim.assert_quiescent(world.pid);
        assert_eq!(world.files.open_handles(), 0);
    }
}

#[test]
fn a_fronts_own_termination_leaves_without_waiting_for_the_box() {
    let mut world = World::front_options(
        false,
        true,
        false,
        Outcome::Success,
        Recording::None,
        Some(Scheme::Dark),
        None,
        false,
        None,
        true,
    );
    world.settle();
    world.box_child().hold_stop = true;
    world.sim.deliver_service_signal(world.pid, world.signals, ServiceSignal::Terminate, None);
    world.settle();
    assert!(world.run.terminated(), "LEAVE is written even while the box has not ended");
    assert!(!world.box_child().finished, "the own-signal path has not waited on the box");
    assert!(world.bytes.ends_with(LEAVE));
    world.box_child().close();
    world.draining = true;
    world.settle();
    assert_eq!(world.run.exit(), Some(Exit::Code(1)));
    world.sim.assert_no_open_fds(world.pid);
    world.sim.assert_quiescent(world.pid);
    assert_eq!(world.files.open_handles(), 0);
}

#[test]
fn a_front_makes_no_connection_until_the_whole_link_arrives_after_start() {
    let mut world = World::front_options(
        false,
        true,
        false,
        Outcome::Success,
        Recording::None,
        Some(Scheme::Dark),
        None,
        false,
        None,
        true,
    );
    world.hold_link = true;
    world.settle();
    assert!(world.connects.is_empty());
    assert!(
        terminal_text(&world.bytes).contains("Starting the box"),
        "the TUI draws before the link: {} (children={})",
        terminal_text(&world.bytes),
        world.children.len()
    );
    let middle = HANDED_LINK.len() / 2;
    world.box_child().write(3, &HANDED_LINK[..middle]);
    world.settle();
    assert!(world.connects.is_empty(), "a fragment has no connection authority");
    world.box_child().write(3, &HANDED_LINK[middle..]);
    world.settle();
    assert!(world.connects.is_empty(), "even all bytes wait for EOF");
    world.box_child().end(3);
    world.settle();
    assert!(world.run.signed_in().is_some());
    assert!(!world.connects.is_empty());
    assert!(
        world
            .connects
            .iter()
            .all(|address| *address == io::Addr::Ip("127.0.0.1:8367".parse().expect("handed address"))),
        "not the initial engine at port 9000"
    );
    world.draining = true;
    world.keys(b"\x03");
    assert_eq!(world.run.exit(), Some(Exit::Code(0)));
    world.sim.assert_no_open_fds(world.pid);
    world.sim.assert_quiescent(world.pid);
    assert!(world.children.iter().all(|child| child.finished));
    assert_eq!(world.files.open_handles(), 0);
}

// The front and bare start use the same captured-environment recording choice.
fn front_from_environment(environment: &[temper_shell::directories::EnvironmentEntry]) -> World {
    let record = if temper_client_shell_tui::start::recording(environment) { Recording::Root } else { Recording::None };
    World::front_options(false, true, false, Outcome::Success, record, Some(Scheme::Dark), None, false, None, true)
}

#[test]
fn a_front_records_when_requested_in_the_captured_environment() {
    let mut world = front_from_environment(&[(b"TEMPER_INTERNAL_RECORD".as_slice().into(), b"1".as_slice().into())]);
    world.settle();
    world.keys(b"\x1bS");
    let lines = replay(&world.recording_file().expect("requested front recording").0);
    assert!(lines.contains(&temper_client_shell_tui::journey::Line::Front));
    world.draining = true;
    world.keys(b"\x03");
    replay(&world.recording_file().expect("front saves on closing").0);
    world.sim.assert_no_open_fds(world.pid);
    world.sim.assert_quiescent(world.pid);
    assert_eq!(world.files.open_handles(), 0);
}

#[test]
fn a_front_without_the_captured_record_variable_writes_no_recording() {
    let mut world = front_from_environment(&[]);
    world.settle();
    world.keys(b"\x1bS");
    assert!(world.recording_file().is_none(), "unrequested front writes no recording");
    world.draining = true;
    world.keys(b"\x03");
    assert!(world.recording_file().is_none(), "closing still writes none");
    world.sim.assert_no_open_fds(world.pid);
    world.sim.assert_quiescent(world.pid);
    assert_eq!(world.files.open_handles(), 0);
}

#[test]
fn a_front_answers_its_first_read_with_the_handed_link_and_reports_the_boxs_lines() {
    use temper_client_shell_tui::journey::Line;
    use temper_client_tui::BoxLine;
    let mut world = World::front_options(
        false,
        true,
        false,
        Outcome::Success,
        Recording::Root,
        Some(Scheme::Dark),
        Some(b"printed-link"),
        false,
        Some(Note::SignedOut),
        true,
    );
    let root = world.files.reopen_root(0);
    let file =
        world.files.open(root, b"printed-link", skein_fake_machine::How::Create { mode: 0o600 }).expect("local file");
    world.files.write(file, 0, LOCAL_LINK).expect("file's link");
    world.files.close(file);
    world.files.close(root);
    world.engine.as_mut().expect("engine").link_answer = Some(jig_client_protocol_wire::v20::SignInAnswer::LinkUsed);
    world.horizon = Time::ZERO;
    world.settle();
    world.box_child().write(4, b"restarting forge\nlog split");
    world.settle();
    world.horizon = Time::ZERO.saturating_add(limits::BACKGROUND).saturating_add(Duration::from_millis(40));
    world.box_child().write(4, b" across writes\nrunning forge\n");
    world.settle();
    let mut past = b"log ".to_vec();
    past.extend(vec![b'x'; BoxLine::MOST - b"log ".len()]);
    past.extend_from_slice(b"log forged\nunknown word\nlog after dropped line\n");
    world.box_child().write(4, &past);
    world.settle();
    world.horizon = world.now.saturating_add(Duration::from_secs(2));
    world.settle();
    world.keys(b"\x1bS");
    let lines = replay(&world.recording_file().expect("front recording").0);
    assert!(lines.contains(&Line::Front), "front fact kept");
    let found: Vec<_> = lines
        .iter()
        .filter_map(|line| if let Line::Found(link) = line { Some(link.as_deref()) } else { None })
        .collect();
    assert!(found == vec![Some(HANDED_LINK), Some(LOCAL_LINK)], "handed link first, file on the next duty");
    // Read box-line records directly; their parser is also exercised by replay above.
    let boxed: Vec<_> = lines
        .iter()
        .filter_map(|line| if let Line::BoxLine(bytes) = line { Some(bytes.as_ref()) } else { None })
        .collect();
    assert!(
        boxed
            == vec![
                b"restarting forge".as_slice(),
                b"log split across writes",
                b"running forge",
                b"log after dropped line"
            ],
        "only readable whole lines, in order"
    );
    assert_eq!(world.engine.as_ref().expect("engine").approvals, 0);
    // An unfinished final piece is discarded when the writer closes.
    world.box_child().write(4, b"log unfinished");
    world.box_child().end(4);
    world.settle();
    assert!(!world.run.is_empty(), "lines EOF alone is not the box's end");
    world.box_child().close();
    world.settle();
    assert!(!world.run.is_empty(), "an unasked box end stays open");
    world.draining = true;
    world.keys(b"\x03");
    let lines = replay(&world.recording_file().expect("closing front recording").0);
    assert_eq!(lines.iter().filter(|line| matches!(line, Line::BoxLine(_))).count(), 4);
    assert!(world.bytes.ends_with(LEAVE));
    assert_eq!(world.run.exit(), Some(Exit::Code(0)));
    world.sim.assert_no_open_fds(world.pid);
    world.sim.assert_quiescent(world.pid);
    assert_eq!(world.files.open_handles(), 0);
}
