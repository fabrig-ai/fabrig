use jig_client_protocol_wire::Record as _;
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

#[expect(
    clippy::struct_excessive_bools,
    reason = "the world independently refuses files, login spawning and box handoff"
)]
struct World {
    sim: Sim,
    files: Machine,
    pid: Pid,
    input: Fd,
    output: Fd,
    signals: Fd,
    children: Vec<fake_box::Child>,
    login: Option<fake_box::Login>,
    login_spawns: usize,
    login_refused: bool,
    login_name: Box<[u8]>,
    loan_signals: Vec<(io::kernel::Signal, io::kernel::Target)>,
    spawn_root: Option<Fd>,
    bound: std::collections::BTreeSet<Token>,
    hold_link: bool,
    connects: Vec<io::Addr>,
    run: Run,
    now: Time,
    wall: Wall,
    bytes: Vec<u8>,
    engine: Option<Engine>,
    outcome: Outcome,
    spawns: usize,
    draining: bool,
    refuse_save: bool,
    refuse_remove: bool,
    listed: usize,
    removed: Vec<Box<[u8]>>,
    recording_sync: Option<Handle>,
    saved: usize,
    stores: usize,
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
        Self::front_configured(
            refuse,
            approval,
            opener,
            outcome,
            record,
            scheme,
            link,
            notes,
            note,
            front,
            Config::calm(),
            None,
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::fn_params_excessive_bools,
        reason = "the simulated front varies settings and terminal buffer size independently"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "the simulated run assembles independent shell, front and terminal fixtures"
    )]
    fn front_configured(
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
        config: Config,
        files: Option<Machine>,
        deployment: Option<(u16, Option<jig_client::Record>)>,
    ) -> Self {
        let (mut files, root) = if let Some(mut files) = files {
            let root = files.reopen_root(0);
            (files, root)
        } else {
            let mut files = Machine::for_user(USER);
            let origin = if front { b"tui/at/127.0.0.1/9000".as_slice() } else { b"tui/at/127.0.0.1/8367".as_slice() };
            let root = files.lay(&[
                Item::directory(b"tui").mode(0o700),
                Item::directory(b"tui/at").mode(0o700),
                Item::directory(b"tui/at/127.0.0.1").mode(0o700),
                Item::directory(origin).mode(if refuse { 0o755 } else { 0o700 }),
            ]);
            (files, root)
        };
        let mut sim = Sim::new(SEED, config);
        let pid = sim.spawn_process();
        let recording = (record == Recording::Root).then(|| {
            let fd = files.open(root, b".", skein_fake_machine::How::Directory).expect("recording root");
            sim.root(pid, Handle::new(fd.raw()))
        });
        let note_root = notes.then(|| {
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
        let port = deployment.as_ref().map_or(if front { 9000 } else { 8367 }, |(port, _)| *port);
        let settings = Settings {
            record: recording,
            front: spawn_root.map(|directory| temper_client_shell_tui::run::Front {
                set_up: false,
                executable: b"/bin/temper".as_slice().into(),
                directories: temper_shell::directories::Directories {
                    config: b"/config".as_slice().into(),
                    data: b"/data".as_slice().into(),
                    cache: b"/cache".as_slice().into(),
                    origins: [skein_shell::Origin::Flag; 3],
                },
                arguments: vec![Box::from(b"--data".as_slice()), Box::from(b"/data".as_slice())].into(),
                environment: vec![Box::from(b"HOME=/home/person".as_slice())].into(),
                directory,
                log: b"/data/box/logs/box.log".as_slice().into(),
            }),
            engine: jig_client_shell_native::Engine::Plain(skein_io::Addr::Ip(std::net::SocketAddr::from((
                [127, 0, 0, 1],
                port,
            )))),
            directory: format!("tui/at/127.0.0.1/{port}").as_bytes().into(),
            theme: deployment.and_then(|(_, theme)| theme),
            address: temper_client_tui::Origin::Http(std::net::SocketAddr::from(([127, 0, 0, 1], port))),
            scheme,
            plain: false,
            version: Box::default(),
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
            login: None,
            login_spawns: 0,
            login_refused: false,
            login_name: Box::new([]),
            loan_signals: Vec::new(),
            spawn_root,
            bound: std::collections::BTreeSet::new(),
            hold_link: false,
            connects: Vec::new(),
            run,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            bytes: Vec::new(),
            engine,
            outcome,
            spawns: 0,
            draining: false,
            refuse_save: false,
            refuse_remove: false,
            listed: 0,
            removed: Vec::new(),
            recording_sync: None,
            saved: 0,
            stores: 0,
            leave_after: 0,
            note_write: NoteWrite::Done,
            held_note: None,
            horizon: Time::ZERO.saturating_add(limits::BACKGROUND).saturating_add(Duration::from_millis(40)),
        }
    }

    fn on_files(files: Machine, record: Recording, wall: Wall) -> Self {
        let mut world = Self::front_configured(
            false,
            false,
            false,
            Outcome::Success,
            record,
            Some(Scheme::Dark),
            None,
            false,
            None,
            false,
            Config::calm(),
            Some(files),
            None,
        );
        world.wall = wall;
        world
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
                            Some((
                                *pidfd,
                                spawn.program.as_ref() == b"/bin/temper-box",
                                spawn.args.first().is_some_and(|arg| arg.as_ref() == b"program-login"),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                for (pidfd, boxed, login) in spawns {
                    if login {
                        self.login = Some(fake_box::Login::new(&mut self.sim, self.pid, pidfd));
                        continue;
                    }
                    self.children.push(fake_box::Child::new(&mut self.sim, self.pid, pidfd, boxed, !self.hold_link));
                }
                self.run.iterate(self.now, Wall::from_nanos(self.wall.as_nanos() + self.now.as_nanos()));
                runnable = self.run.work_pending(self.now);
            }
            self.inspect_submissions();
            self.sim.submit(self.pid, self.run.submissions());
            self.serve();
            for child in &mut self.children {
                child.iterate(&mut self.sim, self.now);
            }
            if let Some(login) = &mut self.login {
                login.iterate(&mut self.sim);
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
                || self.login.as_ref().is_some_and(|login| login.pending(&self.sim))
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

    fn inspect_submissions(&mut self) {
        for submit in self.run.submissions().iter() {
            if let Op::Spawn { spawn } = &submit.kind {
                if spawn.args.first().is_some_and(|arg| arg.as_ref() == b"program-login") {
                    self.login_spawns += 1;
                    assert_eq!(spawn.program.as_ref(), b"/bin/temper-box");
                    assert_eq!(spawn.args.len(), 8);
                    assert_eq!(spawn.args.last().expect("NAME"), &self.login_name);
                    let flags: Vec<_> = spawn.args.iter().take(7).map(AsRef::as_ref).collect();
                    assert_eq!(
                        flags,
                        [
                            b"program-login".as_slice(),
                            b"--config",
                            b"/config",
                            b"--data",
                            b"/data",
                            b"--cache",
                            b"/cache"
                        ]
                    );
                    assert_eq!(spawn.env.as_ref(), &[Box::<[u8]>::from(b"HOME=/home/person".as_slice())]);
                    assert_eq!(Some(spawn.root), self.spawn_root);
                    assert_eq!(spawn.dir.as_ref(), b".");
                    assert_eq!(
                        spawn.pipes.as_ref(),
                        &(0..3)
                            .map(|child| io::kernel::Pipe { child, way: io::kernel::Way::Inherit, parent: None })
                            .collect::<Vec<_>>()
                    );
                    let leave =
                        self.bytes.windows(LEAVE.len()).rposition(|bytes| bytes == LEAVE).expect("left before spawn");
                    assert_eq!(self.bytes.get(leave + LEAVE.len()..).expect("after LEAVE"), b"\x1b[H\x1b[2J\ntemper: Claude Code signs in here. Approve it on Anthropic's page, in the tab it opens; if that page shows a code, paste it here. temper comes back when it's done, or when you press Cancel in the browser.\n\n", "clear, LF, line and two LFs immediately after LEAVE, before spawn");
                } else if spawn.program.as_ref() == b"/bin/browser" {
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
                if self.login_spawns > 0 {
                    if let Op::Signal { signal, to, .. } = &submit.kind {
                        self.loan_signals.push((*signal, *to));
                    }
                } else {
                    assert_eq!(*to, io::kernel::Target::Child, "the browser's group stays");
                }
            }
        }
        if self.login.as_ref().is_some_and(|login| !login.finished) {
            for submit in self.run.submissions().iter() {
                assert!(
                    !matches!(&submit.kind, Op::PipeRead { fd, .. } if *fd == self.input),
                    "no TUI key read while lent"
                );
                assert!(!matches!(&submit.kind, Op::PipeWrite { fd, .. } if *fd == self.output), "no frame while lent");
            }
        }
    }

    fn serve(&mut self) {
        let mut calls = Queue::with_capacity(64);
        self.sim.calls(&mut calls);
        let mut answers = Queue::with_capacity(64);
        while let Some(call) = calls.pop() {
            if matches!(&call.ask, Ask::List { .. }) {
                self.listed += 1;
            }
            if let Ask::Remove { name, .. } = &call.ask {
                self.removed.push(name.clone());
                if self.refuse_remove {
                    answers.push(Answer { ticket: call.ticket, result: Err(io::kernel::Error::Permission) });
                    continue;
                }
            }
            if let Ask::Rename { to, .. } = &call.ask
                && to.as_ref() == b"store"
            {
                self.stores += 1;
            }
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
                && to.starts_with(b"recording-")
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
            if let Ask::Spawn { program, args, reach, .. } = &call.ask {
                if reach.is_some() {
                    answers.push(Answer {
                        ticket: call.ticket,
                        result: Err(io::kernel::Error::Reach(io::kernel::Unenforced::Unsupported)),
                    });
                    continue;
                }
                if args.first().is_some_and(|arg| arg.as_ref() == b"program-login") && self.login_refused {
                    answers.push(Answer { ticket: call.ticket, result: Err(io::kernel::Error::NotFound) });
                    continue;
                }
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
    batch_bodies: std::collections::BTreeSet<Token>,
    discarding: std::collections::BTreeSet<Token>,
    link_answer: Option<jig_client_protocol_wire::SignInAnswer>,
    link_sign_ins: usize,
    approvals: usize,
    terminal_watch: Option<u64>,
    committed_chat: Option<CommittedChat>,
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
            batch_bodies: std::collections::BTreeSet::new(),
            discarding: std::collections::BTreeSet::new(),
            link_answer: None,
            link_sign_ins: 0,
            approvals: 0,
            terminal_watch: None,
            committed_chat: None,
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
        } else if self.batch_bodies.remove(&owner) {
            self.batch(owner, bytes);
        } else if self.bodies.remove(&owner) {
            self.watch(owner, bytes);
        } else if bytes
            .starts_with(format!("GET /jig/v1/stream?version={}", jig_client_protocol_wire::VERSION).as_bytes())
            && self.deny_stream
        {
            self.denied += 1;
            self.respond(owner, b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n".as_slice().into());
        } else if bytes
            .starts_with(format!("GET /jig/v1/stream?version={}", jig_client_protocol_wire::VERSION).as_bytes())
        {
            self.stream = Some(owner);
            let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n".to_vec();
            let mut me = jig_client_test_records::frame_record();
            if let Some(chat) = &self.committed_chat {
                me.projects = chat.projects();
            }
            response.extend_from_slice(&hello_event(me));
            self.respond(owner, response.into_boxed_slice());
        } else if bytes.starts_with(b"POST /jig/v1/watch ") || bytes.starts_with(b"POST /jig/v1/batch ") {
            let header = std::str::from_utf8(bytes).expect("HTTP header");
            let length = header
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .map(|(_, value)| value.trim().parse::<u32>().expect("body size"))
                })
                .expect("content length");
            if bytes.starts_with(b"POST /jig/v1/batch ") {
                self.batch_bodies.insert(owner);
            } else {
                self.bodies.insert(owner);
            }
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
                let mut writer = skein_lib::Writer::new(
                    usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                        .expect("answer size"),
                );
                answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("answer");
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
        use jig_client_protocol_wire as w;
        let open = w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("watch body");
        if let Some(delivery) = self.opened(&open) {
            self.respond(self.stream.expect("opened stream"), delivery);
        }
        self.respond(owner, {
            let answer = w::OpenAnswer::Admitted;
            let mut writer = skein_lib::Writer::new(
                usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                    .expect("short admitted"),
            );
            answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("admitted");
            http_response(&writer.finish())
        });
    }

    fn opened(&mut self, open: &jig_client_protocol_wire::Open) -> Option<Box<[u8]>> {
        use jig_client_protocol_wire as w;
        if open.subject.owner != w::Owner::Jig {
            return self
                .committed_chat
                .as_ref()?
                .forge_watch(&open.subject)
                .map(|document| owned_delivery_event(open.watch, document));
        }
        let subject = w::Subject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(open.subject.record.as_ref()))
            .expect("subject");
        if subject == w::Subject::Me {
            let projects = self.committed_chat.as_ref().map_or_else(Box::default, CommittedChat::projects);
            Some(me_event(open.watch, projects))
        } else if matches!(subject, w::Subject::Terminal(_)) {
            self.terminal_watch = Some(open.watch);
            None
        } else {
            self.committed_chat
                .as_ref()
                .and_then(|chat| chat.watched(&subject))
                .map(|value| delivery_event(open.watch, &value))
        }
    }
    fn batch(&mut self, owner: Token, bytes: &[u8]) {
        use jig_client_protocol_wire as w;
        let batch = w::Batch::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("batch body");
        let mut frames = Vec::new();
        let mut deliveries = Vec::new();
        for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
            let reply = match member {
                w::Member::Open(open) => {
                    if let Some(delivery) = self.opened(&open) {
                        deliveries.extend_from_slice(&delivery);
                    }
                    let answer = w::OpenAnswer::Admitted;
                    let mut writer = skein_lib::Writer::new(
                        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                            .expect("answer size"),
                    );
                    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("answer");
                    writer.finish()
                }
                w::Member::Close(_) => {
                    let answer = w::CloseAnswer::Closed;
                    let mut writer = skein_lib::Writer::new(
                        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                            .expect("answer size"),
                    );
                    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("answer");
                    writer.finish()
                }
                w::Member::Read(read) => {
                    let answer = self.committed_chat.as_ref().and_then(|chat| chat.read(&read)).map_or(
                        w::ReadAnswer::None(w::ReadAnswerNone { value: w::ReadNone::Unknown }),
                        |value| {
                            w::ReadAnswer::Value(w::Owned {
                                owner: w::Owner::Jig,
                                version: w::VERSION,
                                record: wire(&value),
                            })
                        },
                    );
                    wire(&answer)
                }
            };
            let part = w::BatchReply { member: u16::try_from(place).expect("member"), reply: Some(reply) };
            let mut writer = skein_lib::Writer::new(
                usize::try_from(part.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                    .expect("part size"),
            );
            part.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("part bytes");
            frames.extend_from_slice(
                &part.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture").to_be_bytes(),
            );
            frames.extend_from_slice(&writer.finish());
        }
        if !deliveries.is_empty() {
            // One stream demand carries all watch snapshots from this batch.
            self.respond(self.stream.expect("opened stream"), deliveries.into_boxed_slice());
        }
        self.respond(owner, http_response(&frames));
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

const LANDING_CHAT: u64 = 7;
const ASK_LABEL: u32 = 8;
const LANDING_REPLY: &str = "The change landed.";

struct CommittedChat {
    task: jig_client_protocol_wire::Task,
    rows: Box<[u8]>,
    activity: jig_client_protocol_wire::ActivityPage,
}

fn wire(value: &impl jig_client_protocol_wire::Record<Limits = jig_client_protocol_wire::Limits>) -> Box<[u8]> {
    use jig_client_protocol_wire::CEILINGS;
    let size = usize::try_from(value.measure(&CEILINGS).expect("bounded fixture")).expect("wire size");
    let mut writer = skein_lib::Writer::new(size);
    value.write(&CEILINGS, &mut writer).expect("measured record");
    writer.finish()
}

impl CommittedChat {
    fn landing(reply: &[u8]) -> Self {
        use jig_client_protocol_wire as w;
        use jig_client_test_records as records;
        let w::Fact::Chip(mut root) = records::chip_root_fact() else { panic!("chip root") };
        root.value.phase = w::Phase::Idle;
        root.value.title = records::wire_text(b"Update README");
        let chip = records::chip_from_rows(&[
            w::Fact::Chip(root),
            w::Fact::ChipPhase(w::ChipPhaseRow {
                parent: Some(0),
                value: w::ChipPhaseOwnerFields {
                    phase: w::ChipPhase::Active(w::ChipPhaseActive { value: w::RunStage::Idle }),
                    details: 2,
                },
            }),
            w::Fact::ChipDetails(Box::new(records::wire_owner_chip_details_row())),
        ]);
        let requester = w::Requester::Person(w::RequesterPerson { value: 7 });
        let mut task = records::task_record();
        task.chip = chip.clone();
        task.requester = requester.clone();
        task.flags.conversation = true;
        task.flags.listed = true;
        task.label = ASK_LABEL;
        task.activity = 1;
        let mut row = records::task_line_record();
        row.chip = chip;
        row.requester = requester;
        row.flags = task.flags.clone();
        row.label = ASK_LABEL;
        row.tracked = None;
        row.parts = skein_lib::List::from_boxed(Box::new([w::RowPart::Unknown]));
        let rows = piece(&row);
        let mut activity = records::activity_rows(&[records::turn_line(1, 20, 1, &committed_turn(reply))]);
        activity.cut = false;
        activity.after = None;
        Self { task, rows, activity }
    }

    fn projects(&self) -> Box<[u8]> {
        use jig_client_protocol_wire as w;
        let template = |number, label, name: &[u8]| w::Template {
            number,
            name: jig_client_test_records::wire_text(name),
            executor: w::TemplateExecutor::Charter(w::TemplateCharter { value: 7 }),
            contract: w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: 64 }),
            shallower: 1,
            budget: w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap: None }),
            deadline: w::TemplateDeadline::None,
            tracked: None,
            flags: self.task.flags.clone(),
            label,
            grace: None,
            llm: None,
            resources: skein_lib::List::with_capacity(0),
        };
        let ask = piece(&template(5, 7, b"Ask temper"));
        let chat = piece(&template(jig_client::shapes::CHAT, ASK_LABEL, b"Chat"));
        piece(&w::MeProject {
            left_out: 0,
            defaults: skein_lib::List::with_capacity(0),
            public: None,
            templates: [ask.as_ref(), chat.as_ref()].concat().into_boxed_slice(),
            creating: None,
            project: self.task.project,
            name: jig_client_test_records::wire_text(b"Widget"),
            role: w::HeldRole::Owner,
            pool: None,
            models: skein_lib::List::with_capacity(0),
        })
    }

    fn forge_watch(&self, subject: &jig_client_protocol_wire::Owned) -> Option<jig_client_protocol_wire::Owned> {
        use jig_client_protocol_wire as j;
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let owner = j::Owner::Family(j::Family { number: wire::FORGE });
        if subject.owner != owner || subject.version != wire::CLIENT_VERSION {
            return None;
        }
        let envelope = j::FamilyRecord::decode(&j::CEILINGS, &mut skein_lib::Reader::new(&subject.record))
            .expect("forge watch envelope");
        let watch = wire::decode_watching(&envelope.record, &w::CEILINGS).expect("forge watch");
        let record = if let w::Watch::Changes(changes) = &watch.watch
            && changes.project == self.task.project
        {
            let landed = skein_lib::List::from_boxed(Box::new([w::ChangesLanded {
                goal: None,
                chat: Some(w::ChangeFrom { task: LANDING_CHAT, title: b"Update README".as_slice().into() }),
                milestone: None,
                signed: skein_lib::List::with_capacity(0),
                commit: [0xab; 32],
                default_branch: true,
                task: 8,
                title: b"Update README".as_slice().into(),
                at: 21,
            }]));
            wire::changes_panel(&w::ChangesPanel {
                project: self.task.project,
                landing: b"main".as_slice().into(),
                catching_up: None,
                queues: skein_lib::List::with_capacity(0),
                more: 0,
                landed_default: landed.clone(),
                landed_default_more: 0,
                landed,
                landed_more: 0,
            })
        } else if let w::Watch::Drafts(drafts) = &watch.watch
            && drafts.project == self.task.project
        {
            wire::drafts_panel(&w::DraftsPanel {
                project: self.task.project,
                drafts: skein_lib::List::with_capacity(0),
            })
        } else {
            return None;
        };
        Some(j::Owned { owner, version: wire::CLIENT_VERSION, record })
    }

    fn watched(&self, subject: &jig_client_protocol_wire::Subject) -> Option<jig_client_protocol_wire::DeliveryValue> {
        use jig_client_protocol_wire as w;
        if let w::Subject::Tasks(tasks) = subject
            && tasks.query.project.is_none_or(|n| n == self.task.project)
        {
            let rows = if tasks.query.label == Some(ASK_LABEL) { self.rows.clone() } else { Box::default() };
            Some(w::DeliveryValue::Tasks(w::Tasks { rows, more: false, ended_mark: 0 }))
        } else if let w::Subject::Task(task) = subject
            && task.task == LANDING_CHAT
        {
            Some(w::DeliveryValue::Task(Box::new(self.task.clone())))
        } else {
            None
        }
    }

    fn read(&self, owned: &jig_client_protocol_wire::Owned) -> Option<jig_client_protocol_wire::ReadValue> {
        use jig_client_protocol_wire as w;
        if owned.owner != w::Owner::Jig {
            return None;
        }
        let read = w::Read::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&owned.record)).expect("read request");
        if let w::Read::Task(task) = &read
            && task.task == LANDING_CHAT
        {
            Some(w::ReadValue::Task(self.task.clone()))
        } else if let w::Read::Tasks(tasks) = &read
            && tasks.live.is_none()
            && tasks.ended.is_none()
        {
            let mut page = jig_client_test_records::task_page_record();
            if tasks.query.phase == Some(w::PhaseClass::Ended) {
                page.ended = w::EndedGroup::Rows(w::EndedRows { rows: Box::default(), after: None, cut: false });
            } else if tasks.query.label == Some(ASK_LABEL) {
                page.live.clone_from(&self.rows);
            }
            Some(w::ReadValue::Tasks(page))
        } else if let w::Read::Project(project) = &read
            && project.project == self.task.project
        {
            Some(w::ReadValue::Project(jig_client_test_records::owner_project_record()))
        } else if let w::Read::Activity(activity) = read
            && activity.task == LANDING_CHAT
            && activity.before.is_none()
        {
            Some(w::ReadValue::Activity(self.activity.clone()))
        } else {
            None
        }
    }
}

fn piece(value: &impl jig_client_protocol_wire::Record<Limits = jig_client_protocol_wire::Limits>) -> Box<[u8]> {
    let bytes = wire(value);
    let mut out = skein_lib::Writer::new(4 + bytes.len());
    skein_codec::piece::write(&mut out, &bytes).expect("one measured piece");
    out.finish()
}

fn committed_turn(reply: &[u8]) -> Box<[u8]> {
    use smith_transcript::{self as s, Record as _};
    let turn = s::Turn {
        kind: s::TurnKind::Ordinary,
        opening: None,
        endpoint: Box::default(),
        model: Box::default(),
        effort: None,
        dialect: 0,
        place: 1,
        usage: s::Usage { input: None, output: None, cache_read: None, cache_write: None, reasoning: None },
        spent: 0,
        jobs: 0,
        messages: skein_lib::List::from_boxed(Box::new([s::Message {
            role: s::Role::Assistant,
            blocks: skein_lib::List::from_boxed(Box::new([s::Block::Text(s::Text {
                text: reply.into(),
                replay: None,
            })])),
        }])),
        system: None,
    };
    let mut writer =
        skein_lib::Writer::new(skein_lib::length::usize_of(turn.measure(&s::CEILINGS).expect("turn size")));
    turn.write(&s::CEILINGS, &mut writer).expect("committed assistant turn");
    writer.finish()
}

#[test]
fn a_committed_landing_reply_is_visible_in_its_chat() {
    let mut reading = Reading::new(CommittedChat::landing(LANDING_REPLY.as_bytes()));
    for label in ["Work", "Chats · 1", &format!("T{LANDING_CHAT}")] {
        reading.select(label);
    }
    reading.until(|page| page.contains(LANDING_REPLY));
    assert!(reading.frame().contains("Update README"), "the reply is in its own chat");
    reading.close();
}

// The numbered terminal uses the same engine peer and simulated sockets and files as Run.
struct Reading {
    sim: Sim,
    files: Machine,
    pid: Pid,
    input: Fd,
    output: Fd,
    bytes: Vec<u8>,
    now: Time,
    terminal: temper_client_shell_terminal::terminal::Terminal,
    engine: Engine,
}
impl Reading {
    fn new(chat: CommittedChat) -> Self {
        use temper_client_shell_terminal::{
            client::{NativeClient, Settings, Start},
            limits,
            terminal::{Loop, Terminal},
        };
        let mut sim = Sim::new(SEED, Config::calm());
        let pid = sim.spawn_process();
        let input = sim.open_inherited_read(pid);
        let output = sim.open_inherited_write(pid);
        let signals = sim.open_signal_source(pid);
        let mut files = Machine::for_user(USER);
        let dir = b"private";
        let root = files.lay(&[Item::directory(dir).mode(0o700)]);
        let root = sim.root(pid, Handle::new(root.raw()));
        let bounds = limits::native();
        let view = limits::view(&bounds).expect("native view bounds").0;
        let native = NativeClient::new(
            Settings {
                engine: io::Addr::Ip("127.0.0.1:8367".parse().expect("fake engine")),
                directory: dir.as_slice().into(),
                limits: bounds,
            },
            root,
            USER,
            SEED,
            Start::Link(LOCAL_LINK.into()),
        )
        .expect("native client");
        let state = Loop::new(
            &limits::terminal(),
            &view.tree,
            dir.as_slice().into(),
            Some(skein_lib::Secret::new(LOCAL_LINK.into())),
        );
        let terminal = Terminal::new(native, state, input, output, signals).expect("terminal descriptors");
        let mut engine = Engine::new(&mut sim);
        engine.committed_chat = Some(chat);
        Self { sim, files, pid, input, output, bytes: Vec::new(), now: Time::ZERO, terminal, engine }
    }

    fn step(&mut self) {
        self.sim.reap(self.pid, self.terminal.completions());
        self.terminal.iterate(self.now, Wall::from_nanos(self.now.as_nanos()));
        self.sim.submit(self.pid, self.terminal.submissions());
        let mut calls = Queue::with_capacity(64);
        let mut answers = Queue::with_capacity(64);
        self.sim.calls(&mut calls);
        while let Some(call) = calls.pop() {
            skein_fake_machine::step(&mut self.files, call, &mut answers);
        }
        self.sim.answer(&mut answers);
        if self.terminal.is_empty() {
            self.engine.close();
        }
        self.engine.iterate(&mut self.sim, self.now);
        self.bytes.extend(self.sim.peer_drain(self.pid, self.output, ROOM));
        if !self.terminal.work_pending(self.now)
            && self.sim.ready(self.pid) == 0
            && !self.sim.deferred(self.pid)
            && !self.engine.pending(&self.sim)
            && let Some(next) =
                [self.sim.next_due(), self.terminal.next_mechanism_deadline(), self.terminal.next_policy_deadline()]
                    .into_iter()
                    .flatten()
                    .min()
        {
            self.now = next;
            self.sim.advance_to(next);
        }
    }

    fn frame(&self) -> &str {
        let output = std::str::from_utf8(&self.bytes).expect("terminal UTF-8");
        output.rsplit_once("\n== ").map_or(output, |(_, page)| page)
    }

    fn until(&mut self, seen: impl Fn(&str) -> bool) {
        for _ in 0..512 {
            self.step();
            if seen(self.frame()) {
                return;
            }
        }
        panic!("terminal did not print the expected frame:\n{}", self.frame());
    }

    fn select(&mut self, label: &str) {
        self.until(|frame| control(frame, label).is_some());
        let number = control(self.frame(), label).expect("numbered control");
        let command = format!("{number}\n");
        assert_eq!(self.sim.peer_feed(self.pid, self.input, command.as_bytes()), command.len());
    }

    fn close(mut self) {
        self.sim.peer_close(self.pid, self.input);
        for _ in 0..512 {
            self.step();
            if self.terminal.is_empty() && self.engine.io.is_empty() {
                break;
            }
        }
        assert!(self.terminal.is_empty() && self.engine.io.is_empty(), "both sides drain");
        assert_eq!(self.terminal.exit(), Some(Exit::Code(0)));
        self.sim.assert_no_open_fds(self.pid);
        self.sim.assert_quiescent(self.pid);
        self.sim.assert_no_open_fds(self.engine.pid);
        self.sim.assert_quiescent(self.engine.pid);
        assert_eq!(self.files.open_handles(), 0);
    }
}

fn control<'a>(frame: &'a str, label: &str) -> Option<&'a str> {
    frame.lines().find_map(|line| {
        let line = line.trim_start();
        let line = line.strip_prefix("- ").unwrap_or(line).strip_prefix('[')?;
        let (number, text) = line.split_once("] ")?;
        let matches = text == label
            || text.strip_prefix(label).is_some_and(|rest| rest.starts_with(" (") || rest.starts_with(" · "));
        (matches && !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())).then_some(number)
    })
}

fn approval_response() -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    let answer = w::ApprovalAskAnswer::Asked(w::ApprovalAsked {
        secret: skein_lib::Secret::new(Box::new([7; 32])),
        code: *b"WDJBMJHT",
        expires: 600_000_000_000,
    });
    let mut writer = skein_lib::Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("short answer"),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("answer");
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
    fn loan() -> Self {
        Self::loan_with_buffer(u32::try_from(ROOM).expect("terminal buffer size"))
    }

    fn loan_with_buffer(buffer: u32) -> Self {
        let mut world = Self::front_configured(
            false,
            true,
            false,
            Outcome::Success,
            Recording::Root,
            Some(Scheme::Dark),
            None,
            false,
            None,
            true,
            Config { buffer, ..Config::calm() },
            None,
            None,
        );
        world.settle();
        world
    }

    fn lend(&mut self, name: &[u8], cancelled: bool) {
        use jig_client_protocol_wire as w;
        self.login_name = name.into();
        let engine = self.engine.as_mut().expect("engine");
        let login = w::TerminalLogin { number: 1, provider: 7, name: name.into(), cancelled };
        let terminal = w::Terminal { login: Some(login) };
        engine.respond(
            engine.stream.expect("stream"),
            delivery_event(engine.terminal_watch.expect("terminal watch"), &w::DeliveryValue::Terminal(terminal)),
        );
        self.settle();
    }

    fn finish_login(&mut self, exit: Exit) {
        self.login.as_mut().expect("spawned login").finish(&mut self.sim, exit);
        self.settle();
    }

    fn loan_record(&mut self) -> Vec<temper_client_shell_tui::journey::Line> {
        if self.run.terminated() {
            return replay(&self.recording_file().expect("closing recording").0);
        }
        self.keys(b"\x1bS");
        replay(&self.recording_file().expect("loan recording").0)
    }

    fn end_loan_world(&mut self) {
        self.draining = true;
        self.keys(b"\x03");
        self.sim.assert_no_open_fds(self.pid);
        self.sim.assert_quiescent(self.pid);
        assert_eq!(self.files.open_handles(), 0);
    }

    fn keys(&mut self, bytes: &[u8]) {
        assert_eq!(self.sim.peer_feed(self.pid, self.input, bytes), bytes.len());
        self.settle();
    }

    fn recording_file(&mut self) -> Option<(Vec<u8>, u32)> {
        let mut name = skein_lib::Writer::new(30);
        name.put(b"tui/recording-").expect("recording prefix");
        self.wall.write_basic_utc(&mut name).expect("recording name");
        self.file(&name.finish())
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
        world.wall = Wall::from_nanos(1_794_733_232_000_000_000);
        world.settle();
        world.keys(&[mark.as_slice(), save.as_slice()].concat());
        let (first, mode) = world
            .file(b"tui/recording-20261115T090032Z")
            .unwrap_or_else(|| panic!("{mark:?}: save key writes recording"));
        assert_eq!(mode, 0o600, "{mark:?}: recording is private");
        let lines = replay(&first);
        assert!(lines.contains(&Line::Seed(SEED)), "{mark:?}");
        // Host construction uses zero clocks; the first report follows the background wait.
        assert!(lines.contains(&Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH }), "{mark:?}");
        assert!(
            lines.contains(&Line::Turn {
                now: Time::ZERO.saturating_add(limits::BACKGROUND),
                wall: Wall::from_nanos(world.wall.as_nanos() + limits::BACKGROUND.as_nanos()),
            }),
            "{mark:?}: the start turn keeps the live clocks"
        );
        assert!(lines.contains(&Line::Screen { size: SIZE, scheme: Some(Scheme::Dark) }), "{mark:?}");
        assert!(lines.contains(&Line::Address(b"http://127.0.0.1:8367".as_slice().into())), "{mark:?}");
        assert!(lines.iter().any(|line| matches!(line, Line::Mark(Some(_)))), "{mark:?}");
        assert!(matches!(lines.last(), Some(Line::End(Some(_)))), "{mark:?}");
        assert!(
            !lines.iter().any(|line| matches!(line, Line::Key { key: skein_ui_screen_terminal::Key::Alt(_), .. })),
            "{mark:?}: recording keys never reach the host"
        );
        world.wall = Wall::from_nanos(1_794_819_632_000_000_000);
        world.keys(&[b"\t".as_slice(), save.as_slice()].concat());
        let (second, mode) =
            world.file(b"tui/recording-20261115T090032Z").unwrap_or_else(|| panic!("{mark:?}: second save"));
        assert_eq!(mode, 0o600, "{mark:?}");
        assert!(second.len() > first.len(), "{mark:?}: second save includes more reports");
        replay(&second);
        world.draining = true;
        world.leave_after = world.saved + 1;
        world.keys(b"\x03");
        let (closed, mode) =
            world.file(b"tui/recording-20261115T090032Z").unwrap_or_else(|| panic!("{mark:?}: closing save"));
        assert_eq!(mode, 0o600, "{mark:?}");
        assert!(closed.len() > second.len(), "{mark:?}: closing saves the final reports");
        assert!(replay(&closed).contains(&Line::Quit), "{mark:?}");
        world.drained(false);
    }
}

#[test]
fn recording_runs_keep_distinct_files() {
    let mut first = World::recording(false, true, false, Outcome::Success, Recording::Root);
    first.wall = Wall::from_nanos(1_794_733_232_000_000_000);
    first.settle();
    first.keys(b"\x03");
    first.drained(false);
    let saved = first.recording_file().expect("first run saved");
    let mut second = World::on_files(first.files, Recording::Root, Wall::from_nanos(1_794_819_632_000_000_000));
    second.settle();
    second.keys(b"\x1bs");
    assert!(second.file(b"tui/recording-20261115T090032Z") == Some(saved.clone()), "second run leaves first alone");
    replay(&second.recording_file().expect("second run's own file").0);
    second.keys(b"\x03");
    assert!(second.file(b"tui/recording-20261115T090032Z") == Some(saved), "closing also leaves the first run alone");
    second.drained(false);
}

const OTHER_RECORDING_FILES: &[&[u8]] = &[
    b"recording",
    b"recording-20261101T000000Z.skein-1a2b",
    b"recording-notatime",
    b"recording-20260229T000000Z",
    b"recording-20261101T240000Z",
    b"recording-20261101T006000Z",
    b"recording-20261101T000060Z",
    b"recording-20261101T000000Z-extra",
    b"token",
    b"store",
    b"address",
    b"sign-in",
];

fn other_recording_bytes(name: &[u8]) -> &'static [u8] {
    match name {
        b"token" => &[7; 40],
        b"store" => &[0; 4],
        _ => b"untouched",
    }
}

fn earlier_recordings(count: u32) -> Machine {
    let mut items = vec![
        Item::directory(b"tui").mode(0o700),
        Item::directory(b"tui/at").mode(0o700),
        Item::directory(b"tui/at/127.0.0.1").mode(0o700),
        Item::directory(b"tui/at/127.0.0.1/8367").mode(0o700),
    ];
    for day in 1..=count {
        items.push(Item::file(format!("tui/recording-202611{day:02}T000000Z").as_bytes(), b"earlier").mode(0o600));
    }
    for name in OTHER_RECORDING_FILES {
        let path = [b"tui/".as_slice(), name].concat();
        items.push(Item::file(&path, other_recording_bytes(name)).mode(0o600));
    }
    items.push(Item::directory(b"tui/recording-20000101T000000Z").mode(0o700));
    items.push(Item::link(b"tui/recording-20000102T000000Z", b"recording"));
    let mut files = Machine::for_user(USER);
    let root = files.lay(&items);
    files.close(root);
    files
}

#[test]
fn recording_retains_nine_earlier_files_and_ignores_other_entries() {
    for (record, count) in [(Recording::Root, 12), (Recording::Root, 9), (Recording::Root, 2), (Recording::None, 12)] {
        let mut world = World::on_files(earlier_recordings(count), record, Wall::from_nanos(1_794_733_232_000_000_000));
        world.settle();
        let removed = if record == Recording::Root { count.saturating_sub(9) } else { 0 };
        assert_eq!(world.removed.len(), usize::try_from(removed).unwrap(), "pruned before any save");
        for save in [false, true] {
            if save {
                world.keys(b"\x1bs");
            }
            for day in 1..=count {
                let path = format!("tui/recording-202611{day:02}T000000Z");
                let expected = (day > removed).then(|| (b"earlier".to_vec(), 0o600));
                assert_eq!(world.file(path.as_bytes()), expected, "{path}");
            }
            for name in OTHER_RECORDING_FILES {
                assert_eq!(
                    world.file(&[b"tui/".as_slice(), name].concat()),
                    Some((other_recording_bytes(name).to_vec(), 0o600)),
                    "{name:?}"
                );
            }
        }
        assert_eq!(world.listed > 0, record == Recording::Root, "only recording lists the directory");
        assert_eq!(world.recording_file().is_some(), record == Recording::Root);
        let listed = world.listed;
        world.keys(b"\x03");
        assert_eq!(world.listed, listed, "retention is once per run, not per save");
        world.drained(false);
    }
}

#[test]
fn failed_recording_removals_are_silent_and_do_not_prevent_saving() {
    let mut world =
        World::on_files(earlier_recordings(12), Recording::Root, Wall::from_nanos(1_794_733_232_000_000_000));
    world.refuse_remove = true;
    world.settle();
    assert_eq!(world.removed.len(), 3, "each old recording's removal attempted despite failures");
    let drawn = world.bytes.len();
    world.keys(b"\x1bs");
    assert_eq!(world.bytes.len(), drawn, "no removal failure drawn");
    replay(&world.recording_file().expect("save goes on").0);
    for day in 1..=12 {
        assert_eq!(
            world.file(format!("tui/recording-202611{day:02}T000000Z").as_bytes()),
            Some((b"earlier".to_vec(), 0o600))
        );
    }
    world.keys(b"\x03");
    world.drained(false);
}

#[test]
fn closing_save_reports_its_own_outcome() {
    for refuse in [false, true] {
        let mut world = World::recording(false, true, false, Outcome::Success, Recording::Root);
        world.settle();
        world.keys(b"\x1bs");
        assert_eq!(world.run.recording_save(), Some((b"recording-19700101T000000Z".as_slice(), true)));
        world.refuse_save = refuse;
        world.keys(b"\x03");
        world.drained(false);
        assert_eq!(
            world.run.recording_save(),
            Some((b"recording-19700101T000000Z".as_slice(), !refuse)),
            "closing result after LEAVE, not the earlier save's"
        );
    }
    let mut world = World::new(false);
    world.settle();
    world.keys(b"\x03");
    world.drained(false);
    assert_eq!(world.run.recording_save(), None);
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
    for (path, bytes) in [
        (b"tui/at/127.0.0.1/8367/store".as_slice(), store.as_slice()),
        (b"tui/at/127.0.0.1/8367/token".as_slice(), CREDENTIAL),
    ] {
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

fn deployment_files() -> Machine {
    let mut files = Machine::for_user(USER);
    let root = files.lay(&[
        Item::directory(b"tui").mode(0o700),
        Item::directory(b"tui/at").mode(0o700),
        Item::directory(b"tui/at/127.0.0.1").mode(0o700),
        Item::directory(b"tui/at/127.0.0.1/8367").mode(0o700),
        Item::directory(b"tui/at/127.0.0.1/9000").mode(0o700),
        Item::file(b"tui/at/127.0.0.1/8367/token", &[7; 40]).mode(0o600),
        Item::file(b"tui/address", b"http://127.0.0.1:8367").mode(0o600),
    ]);
    files.close(root);
    files
}

fn at_deployment(files: Machine, port: u16, theme: Option<jig_client::Record>) -> World {
    World::front_configured(
        false,
        false,
        false,
        Outcome::Success,
        Recording::Root,
        Some(Scheme::Light),
        None,
        true,
        None,
        false,
        Config::calm(),
        Some(files),
        Some((port, theme)),
    )
}

#[test]
fn switching_origins_never_loads_another_origins_credential() {
    use temper_client_shell_tui::journey::Line;
    let mut files = deployment_files();
    for (port, held) in [(8367, true), (9000, false), (8367, true)] {
        let mut world = at_deployment(files, port, None);
        world.settle();
        world.keys(b"\x1bS");
        let lines = replay(&world.recording_file().expect("start facts").0);
        assert_eq!(lines.iter().any(|line| matches!(line, Line::Credential(_))), held, "port {port}");
        world.draining = true;
        world.keys(b"\x03");
        world.drained(false);
        assert_eq!(world.file(b"tui/at/127.0.0.1/8367/token"), Some((vec![7; 40], 0o600)));
        assert!(world.file(b"tui/token").is_none());
        files = world.files;
    }
}

#[test]
fn a_carried_theme_is_saved_once_and_loaded_on_return() {
    use temper_client_shell_tui::journey::Line;
    use temper_client_tui::theme::THEME_KEY;
    let theme = jig_client::Record { key: THEME_KEY.into(), bytes: b"dark".as_slice().into() };
    let mut first = at_deployment(deployment_files(), 9000, Some(theme));
    first.settle();
    first.keys(b"\x1bS");
    let lines = replay(&first.recording_file().expect("first facts").0);
    assert!(lines.contains(&Line::Stored { key: THEME_KEY.into(), bytes: b"dark".as_slice().into() }));
    let (store, mode) = first.file(b"tui/at/127.0.0.1/9000/store").expect("theme saved in new store");
    assert_eq!(mode, 0o600);
    assert_eq!(jig_client_shell_native::store_record(&store, THEME_KEY).expect("kept theme").bytes.as_ref(), b"dark");
    assert!(first.file(b"tui/store").is_none());
    assert_eq!(first.stores, 1, "one Save of the carried theme alone");
    first.draining = true;
    first.keys(b"\x03");
    first.drained(false);
    let root = first.files.reopen_root(0);
    let address = first.files.open(root, b"tui/address", skein_fake_machine::How::Write).expect("address");
    first.files.write(address, 0, b"http://127.0.0.1:9000").expect("last address changed");
    first.files.close(address);
    first.files.close(root);
    let mut second = at_deployment(first.files, 9000, None);
    second.settle();
    second.keys(b"\x1bS");
    let lines = replay(&second.recording_file().expect("second facts").0);
    assert!(lines.contains(&Line::Stored { key: THEME_KEY.into(), bytes: b"dark".as_slice().into() }));
    assert_eq!(second.stores, 0, "a return reads its own theme without carrying again");
    second.draining = true;
    second.keys(b"\x03");
    second.drained(false);
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
            assert!(world.file(b"tui/at/127.0.0.1/8367/sign-in").is_none(), "first note has not replaced the file");
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
            world.file(b"tui/at/127.0.0.1/8367/sign-in"),
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
    use jig_client_protocol_wire as w;
    let answer = w::SignInAnswer::SignedIn(w::SignedIn {
        token: Some(skein_lib::Secret::new(Box::new([7; 40]))),
        party: 17,
        expires: 600_000_000_000,
    });
    let mut writer = skein_lib::Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("short answer"),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("answer");
    let body = writer.finish();
    http_response(&body)
}

fn hello_event(me: jig_client_protocol_wire::Me) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    use skein_lib::List;
    let owners = limits::tui().native.client.client.domain.owners;
    let mut served = List::with_capacity(u32::try_from(owners.len() + 1).expect("owner count"));
    served
        .push(w::Served {
            owner: w::Owner::Jig,
            first: jig_client_protocol_wire::VERSION,
            last: jig_client_protocol_wire::VERSION,
        })
        .expect("Jig owner");
    for owner in owners {
        let family = w::Family { number: owner.family };
        served
            .push(w::Served { owner: w::Owner::Family(family), first: owner.version, last: owner.version })
            .expect("family owner");
    }
    let hello =
        jig_client_test_records::opened(w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) }, me);
    sse(&hello)
}
fn me_event(watch: u64, projects: Box<[u8]>) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    let text = |bytes: &[u8]| w::DocumentText { bytes: bytes.into(), cut: false };
    let me = w::Me {
        party: w::MeParty { number: 7, kind: w::PartyKind::Person, login: text(b"owner"), name: text(b"Owner") },
        projects,
        needs: w::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: w::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: None,
        providers: Box::new([]),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        words_bytes: 8192,
        sign_in_expires: 86_400_000_000_000,
    };
    delivery_event(watch, &w::DeliveryValue::Me(me))
}

fn delivery_event(watch: u64, value: &jig_client_protocol_wire::DeliveryValue) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    use skein_lib::Writer;
    let mut writer = Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("document size"),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("document fits");
    let document =
        w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() };
    owned_delivery_event(watch, document)
}

fn owned_delivery_event(watch: u64, document: jig_client_protocol_wire::Owned) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    sse(&w::StreamEvent::Delivery(w::Delivery { watch, document }))
}

fn sse(event: &jig_client_protocol_wire::StreamEvent) -> Box<[u8]> {
    use skein_lib::{
        Writer,
        base64::{self, Alphabet},
    };
    let mut writer = Writer::new(
        usize::try_from(event.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("event size"),
    );
    event.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("event");
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

#[test]
fn a_loan_runs_only_the_found_box_with_inherited_descriptors_and_resumes() {
    use jig_client::contract::LoginExit;
    use temper_client_shell_tui::journey::Line;
    // The 64-byte terminal buffer forces Lending::written to finish a short write before the spawn.
    for (name, end, exit, buffer) in [
        (b"work".as_slice(), Exit::Code(0), LoginExit::Exited { code: 0 }, 65_536),
        (b"-", Exit::Code(23), LoginExit::Exited { code: 23 }, 64),
        (b"abcdefghijklmnopqrstuvwxyz01234-", Exit::Signal(2), LoginExit::Killed { signal: 2 }, 65_536),
    ] {
        let mut world = World::loan_with_buffer(buffer);
        world.lend(name, false);
        assert_eq!(world.login_spawns, 1);
        let lent = world.bytes.len();
        assert_eq!(world.bytes.windows(ENTER.len()).filter(|part| *part == ENTER).count(), 1);
        world.box_child().write(4, b"log heard while lent\n");
        world.settle();
        assert_eq!(world.bytes.len(), lent, "box reports draw nothing during the loan");
        world.finish_login(end);
        assert_eq!(
            world.loan_signals,
            [(io::kernel::Signal::Kill, io::kernel::Target::Group)],
            "Spawn reaps the login's own group, not SpawnAlone's child"
        );
        assert!(world.bytes.get(lent..).expect("new bytes").starts_with(ENTER), "ENTER before the new frame");
        let lines = world.loan_record();
        assert!(lines.contains(&Line::Lent { number: 1, exit }));
        assert!(lines.contains(&Line::BoxLine(b"log heard while lent".as_slice().into())));
        assert!(!world.run.is_empty());
        world.end_loan_world();
        assert!(!world.bytes.windows(b"\x1b[3J".len()).any(|part| part == b"\x1b[3J"), "scrollback is never erased");
    }
}

#[test]
fn a_failed_login_spawn_reports_not_started_and_takes_back_the_terminal() {
    use jig_client::contract::LoginExit;
    use temper_client_shell_tui::journey::Line;
    let mut world = World::loan();
    world.login_refused = true;
    world.lend(b"work", false);
    assert_eq!(world.login_spawns, 1);
    assert_eq!(world.bytes.windows(ENTER.len()).filter(|part| *part == ENTER).count(), 2, "takes back at once");
    assert!(world.loan_record().contains(&Line::Lent { number: 1, exit: LoginExit::NotStarted }));
    world.end_loan_world();
}

#[test]
fn unlend_or_the_runs_termination_signals_the_group_and_waits_for_the_login() {
    use jig_client::contract::LoginExit;
    use temper_client_shell_tui::journey::Line;
    for termination in [false, true] {
        let mut world = World::loan();
        world.lend(b"work", false);
        let lent = world.bytes.len();
        if termination {
            world.sim.deliver_service_signal(world.pid, world.signals, ServiceSignal::Terminate, None);
            world.settle();
        } else {
            world.lend(b"work", true);
            world.lend(b"work", true);
        }
        assert_eq!(
            world.loan_signals,
            [(io::kernel::Signal::Terminate, io::kernel::Target::Group)],
            "one SIGTERM to the group"
        );
        assert_eq!(world.bytes.len(), lent, "waits before ENTER");
        assert!(!world.run.terminated(), "termination cannot leave while login owns the terminal");
        assert!(!world.run.is_empty());
        world.finish_login(Exit::Signal(15));
        assert!(world.bytes.get(lent..).expect("back").starts_with(ENTER));
        let lines = world.loan_record();
        assert!(lines.contains(&Line::Lent { number: 1, exit: LoginExit::Killed { signal: 15 } }));
        if termination {
            assert!(world.run.terminated());
            world.box_child().close();
            world.draining = true;
            world.settle();
            world.sim.assert_no_open_fds(world.pid);
            world.sim.assert_quiescent(world.pid);
            assert_eq!(world.files.open_handles(), 0);
        } else {
            world.end_loan_world();
        }
    }
}

#[test]
fn the_box_ending_while_lent_leaves_the_login_to_its_end() {
    use jig_client::contract::LoginExit;
    use temper_client_shell_tui::journey::Line;
    let mut world = World::loan();
    world.lend(b"work", false);
    let lent = world.bytes.len();
    world.box_child().close();
    world.settle();
    assert!(!world.login.as_ref().expect("login").finished);
    assert_eq!(world.bytes.len(), lent);
    assert_eq!(world.loan_signals, [] as [(io::kernel::Signal, io::kernel::Target); 0]);
    world.finish_login(Exit::Code(0));
    assert!(world.bytes.get(lent..).expect("back").starts_with(ENTER));
    assert!(world.loan_record().contains(&Line::Lent { number: 1, exit: LoginExit::Exited { code: 0 } }));
    world.end_loan_world();
}

const HANDED_LINK: &[u8] = b"http://127.0.0.1:8367/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

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
    assert_eq!(world.connects.len(), 0);
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
    assert_ne!(world.connects.len(), 0);
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
    world.engine.as_mut().expect("engine").link_answer = Some(jig_client_protocol_wire::SignInAnswer::LinkUsed);
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
