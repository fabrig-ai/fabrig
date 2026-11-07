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
}
impl World {
    fn new(refuse: bool) -> Self {
        Self::configured(refuse, false, false, Outcome::Success)
    }

    fn configured(refuse: bool, approval: bool, opener: bool, outcome: Outcome) -> Self {
        let mut files = Machine::for_user(USER);
        let root = files.lay(&[Item::directory(b"tui").mode(if refuse { 0o755 } else { 0o700 })]);
        let mut sim = Sim::new(SEED, Config::calm());
        let pid = sim.spawn_process();
        let root = sim.root(pid, Handle::new(root.raw()));
        let input = sim.open_inherited_read(pid);
        let output = sim.open_inherited_write(pid);
        let signals = sim.open_signal_source(pid);
        let engine = approval.then(|| Engine::new(&mut sim));
        let settings = Settings {
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
            self.bytes.extend(self.sim.peer_drain(self.pid, self.output, ROOM));
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
                if self.engine.is_some()
                    && !self.draining
                    && next > Time::ZERO.saturating_add(Duration::from_millis(40))
                {
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

#[derive(Debug)]
enum WayOut {
    End,
    ControlC,
    Termination,
    Refused,
}

#[test]
fn every_way_out_leaves_the_terminal_as_it_was_found() {
    let clock = skein_shell::Clock::new();
    for way in [WayOut::End, WayOut::ControlC, WayOut::Termination, WayOut::Refused] {
        let began = clock.now().now;
        let mut world = World::new(matches!(way, WayOut::Refused));
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
