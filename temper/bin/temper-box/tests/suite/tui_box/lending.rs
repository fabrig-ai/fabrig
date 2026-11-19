//! Process wiring for a terminal lent to the found box (client/tui/box.md, Lending's tests).
use super::{Binary, Command, Delegate, Expectation, Host, Mode, Referee, Streams, Terminal, real, support};
use jig_client_protocol_wire as w;
use skein_io::{
    self as io,
    kernel::{Complete, Exit, Fd, Size, Submit},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, Writer, stream};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::OsString,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::PathBuf,
};

enum Body {
    Watch,
    Batch,
}

// The fake engine shares the binary observer's and the person's one ring.
struct Peer {
    descriptor: Fd,
    address: String,
    io: io::Io,
    env: Env<io::Limits>,
    entities: BTreeSet<Token>,
    bodies: BTreeMap<Token, Body>,
    responses: BTreeMap<Token, VecDeque<Box<[u8]>>>,
    stream: Option<Token>,
    closing: bool,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    told: Queue<io::Event>,
}
impl Peer {
    fn new() -> Self {
        let (descriptor, address) =
            skein_shell::open_listener("127.0.0.1:0".parse().expect("loopback"), 8).expect("fake listener");
        let limits = io::Limits { sockets: 16, ..io::Limits::TEST };
        let mut io = io::Io::new(&limits);
        io.adopt_listener(descriptor, address).expect("adopt fake listener");
        let mut peer = Self {
            descriptor,
            address: address.to_string(),
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            entities: BTreeSet::new(),
            bodies: BTreeMap::new(),
            responses: BTreeMap::new(),
            stream: None,
            closing: false,
            complete: Queue::with_capacity(256),
            submit: Queue::with_capacity(256),
            told: Queue::uncapped(),
        };
        peer.down(io::Request::Listen { owner: Token::new(1), addr: io::Addr::Ip(address) });
        peer
    }
    fn address(&self) -> &str {
        &self.address
    }
    fn down(&mut self, request: io::Request) {
        io::down(&mut self.io, &self.env, request, &mut self.submit);
    }
    fn close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        for entity in self.entities.iter().copied().collect::<Vec<_>>() {
            self.down(io::Request::Abort { entity });
        }
    }
    fn respond(&mut self, owner: Token, response: Vec<u8>) {
        let room = u32::try_from(response.len()).expect("response bound");
        let queued = self.responses.entry(owner).or_default();
        queued.push_back(response.into());
        if queued.len() > 1 {
            return;
        }
        self.down(io::Request::Stream {
            stream: owner,
            down: stream::Down::Demand { read: stream::Read::Nothing, room },
        });
    }
    fn receive(&mut self, owner: Token, bytes: &[u8]) {
        if let Some(body) = self.bodies.remove(&owner) {
            match body {
                Body::Watch => {
                    let open =
                        w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("watch request");
                    let answer = self.opened(&open);
                    self.respond(owner, response(&answer));
                }
                Body::Batch => self.batch(owner, bytes),
            }
        } else if bytes.starts_with(format!("GET /jig/v1/stream?version={} ", w::VERSION).as_bytes())
            || bytes.starts_with(format!("GET /jig/v1/stream?version={}&replaces=", w::VERSION).as_bytes())
        {
            self.stream = Some(owner);
            self.respond(
                owner,
                [b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n".to_vec(), hello()].concat(),
            );
        } else if bytes.starts_with(b"POST /jig/v1/sign-in/link ") {
            self.respond(owner, signed_in());
        } else if bytes.starts_with(b"POST /jig/v1/watch ") || bytes.starts_with(b"POST /jig/v1/batch ") {
            let header = std::str::from_utf8(bytes).expect("HTTP header");
            let length = header
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                        .map(|(_, value)| value.trim().parse::<u32>().expect("length"))
                })
                .expect("body length");
            let body = if bytes.starts_with(b"POST /jig/v1/batch ") { Body::Batch } else { Body::Watch };
            self.bodies.insert(owner, body);
            self.down(io::Request::Stream {
                stream: owner,
                down: stream::Down::Demand { read: stream::Read::Fill(length), room: 0 },
            });
        } else {
            // The end's account request needs no answer for this wiring story.
            self.respond(owner, response(b""));
        }
    }
    fn opened(&mut self, open: &w::Open) -> Box<[u8]> {
        if open.subject().owner() == &w::Owner::Jig {
            let subject = w::Subject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(open.subject().record()))
                .expect("subject");
            if matches!(subject, w::Subject::Terminal(_)) {
                self.respond(self.stream.expect("event stream"), terminal(open.watch()));
            }
        }
        let answer = w::OpenAnswer::Admitted;
        let mut writer = Writer::new(usize::try_from(answer.measure()).expect("answer size"));
        answer.encode(&mut writer).expect("admitted");
        writer.finish()
    }
    fn batch(&mut self, owner: Token, bytes: &[u8]) {
        let batch = w::Batch::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("batch request");
        let mut frames = Vec::new();
        for (place, member) in batch.into_parts().members.into_boxed().into_iter().enumerate() {
            let reply = match member {
                w::Member::Open(open) => self.opened(&open),
                w::Member::Read(_) | w::Member::Close(_) => Box::new([]),
            };
            let part = w::BatchReply::new(
                &w::CEILINGS,
                w::BatchReplyParts { member: u16::try_from(place).expect("at most 64 members"), reply: Some(reply) },
            )
            .expect("member reply");
            let mut writer = Writer::new(usize::try_from(part.measure()).expect("part size"));
            part.encode(&mut writer).expect("part bytes");
            let bytes = writer.finish();
            frames.extend_from_slice(&u32::try_from(bytes.len()).expect("frame size").to_be_bytes());
            frames.extend_from_slice(&bytes);
        }
        self.respond(owner, response(&frames));
    }
}
impl Host for Peer {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
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
                            read: stream::Read::Scan { until: stream::Delimiter::CRLF_CRLF, max: 8192 },
                            room: 0,
                        },
                    });
                }
                io::Event::Stream { owner, up: stream::Up::Bytes(bytes) } => self.receive(owner, &bytes),
                io::Event::Stream { owner, up: stream::Up::Room } => {
                    let queued = self.responses.get_mut(&owner).expect("response room");
                    let bytes = queued.pop_front().expect("queued response");
                    let next = queued.front().map(|bytes| u32::try_from(bytes.len()).expect("response bound"));
                    if next.is_none() {
                        self.responses.remove(&owner);
                    }
                    self.down(io::Request::Stream { stream: owner, down: stream::Down::Send(bytes) });
                    if let Some(room) = next {
                        self.down(io::Request::Stream {
                            stream: owner,
                            down: stream::Down::Demand { read: stream::Read::Nothing, room },
                        });
                    } else if self.stream != Some(owner) {
                        self.down(io::Request::Stream { stream: owner, down: stream::Down::Finish });
                        self.down(io::Request::Close { entity: owner });
                    }
                }
                io::Event::Stream { owner, up: stream::Up::End | stream::Up::Failed(_) } => {
                    self.down(io::Request::Abort { entity: owner });
                }
                io::Event::Closed { owner } => {
                    self.entities.remove(&owner);
                    self.bodies.remove(&owner);
                    self.responses.remove(&owner);
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
                | io::Event::Failed { .. } => panic!("unexpected fake engine event: {event:?}"),
            }
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.complete.is_empty() || !self.submit.is_empty() || self.io.is_ready() || self.io.is_due(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.closing && self.io.is_empty() && self.complete.is_empty() && self.submit.is_empty() && self.told.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("peer operations")
    }
}

fn response(body: &[u8]) -> Vec<u8> {
    [
        format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes(),
        body.to_vec(),
    ]
    .concat()
}
fn signed_in() -> Vec<u8> {
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts { token: Some(skein_lib::Secret::new(Box::new([7; 40]))), party: 17, expires: u64::MAX },
        )
        .expect("signed in"),
    );
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("size"));
    answer.encode(&mut writer).expect("sign-in answer");
    response(&writer.finish())
}
fn hello() -> Vec<u8> {
    let mut served = skein_lib::List::with_capacity(1);
    served
        .push(
            w::Served::new(
                &w::CEILINGS,
                w::ServedParts {
                    owner: w::Owner::Jig,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
            )
            .expect("served version"),
        )
        .expect("one version");
    let hello =
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello");
    sse(&w::StreamEvent::Opened(
        w::Opened::new(&w::CEILINGS, w::OpenedParts { hello, me: me() }).expect("opened stream"),
    ))
}
fn me() -> w::Owned {
    let text =
        || w::DocumentText::new(&w::CEILINGS, w::DocumentTextParts { bytes: Box::new([]), cut: false }).expect("text");
    let me = w::Me::new(
        &w::CEILINGS,
        w::MeParts {
            party: w::MeParty::new(
                &w::CEILINGS,
                w::MePartyParts { number: 17, kind: w::PartyKind::Person, login: text(), name: text() },
            )
            .expect("party"),
            projects: Box::new([]),
            needs: w::Needs::new(
                &w::CEILINGS,
                w::NeedsParts { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            )
            .expect("needs"),
            needs_total: 0,
            updates: w::Count::new(&w::CEILINGS, w::CountParts { value: 0, limited: false }).expect("count"),
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
    .expect("me without projects");
    let value = w::DeliveryValue::Me(me);
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("size"));
    value.encode(&mut writer).expect("me document");
    w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: writer.finish() })
        .expect("envelope")
}
fn terminal(watch: u64) -> Vec<u8> {
    let login = w::TerminalLogin::new(
        &w::CEILINGS,
        w::TerminalLoginParts { number: 1, provider: 7, name: b"work".as_slice().into(), cancelled: false },
    )
    .expect("login");
    let value = w::DeliveryValue::Terminal(
        w::Terminal::new(&w::CEILINGS, w::TerminalParts { login: Some(login) }).expect("terminal"),
    );
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("size"));
    value.encode(&mut writer).expect("terminal document");
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
    )
    .expect("envelope");
    sse(&w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch, document }).expect("delivery"),
    ))
}
fn sse(event: &w::StreamEvent) -> Vec<u8> {
    use skein_lib::base64::{self, Alphabet};
    let mut writer = Writer::new(usize::try_from(event.measure()).expect("size"));
    event.encode(&mut writer).expect("event");
    let bytes = writer.finish();
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("size")).expect("base64 size");
    let mut writer = Writer::new(usize::try_from(size).expect("size"));
    base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("base64");
    [b"data: ".as_slice(), writer.finish().as_ref(), b"\n\n"].concat()
}

enum Root {
    Client(Box<Binary>),
    Terminal(Box<Terminal>),
    Peer(Box<Peer>),
}
impl Delegate for Root {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Client(x) => x.as_ref(),
            Self::Terminal(x) => x.as_ref(),
            Self::Peer(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Client(x) => x.as_mut(),
            Self::Terminal(x) => x.as_mut(),
            Self::Peer(x) => x.as_mut(),
        }
    }
}
#[derive(Debug)]
enum Phase {
    Lending,
    Reading,
    Back,
    Closing,
}
struct Judge {
    failed: Option<String>,
    phase: Phase,
    patience: support::patience::Patience,
    settled: bool,
}
impl Referee<Root> for Judge {
    fn act(&mut self, _now: Time, roots: &mut [Root]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let [Root::Client(binary), Root::Terminal(terminal), Root::Peer(peer)] = roots else {
                return Err(String::from("binary, terminal and peer"));
            };
            terminal.client_exit = binary.exit_status();
            match self.phase {
                Phase::Lending if terminal.has(b"LOGIN-READY") && !terminal.writing => {
                    if !terminal.has(b"\x1b[?1049l") {
                        return Err(String::from("LEAVE before login"));
                    }
                    if !terminal.has(b"temper: Claude Code signs in here.") {
                        return Err(String::from(
                            "assertion failed: terminal.has(b\"temper: Claude Code signs in here.\")",
                        ));
                    }
                    terminal.bytes.clear();
                    terminal.keys(b"work input\n");
                    self.phase = Phase::Reading;
                }
                Phase::Reading if terminal.has(b"LOGIN-READ") => {
                    self.phase = Phase::Back;
                }
                Phase::Back
                    if terminal.has(b"\x1b[?1049h") && terminal.has(b"Claude Code signed in") && !terminal.writing =>
                {
                    let modes = skein_shell::terminal_modes(terminal.stream.ok_or(String::from("master"))?)
                        .map_err(|error| format!("terminal modes: {error:?}"))?;
                    if modes.echo || modes.lines {
                        return Err(String::from("the TUI's raw modes are back"));
                    }
                    terminal.keys(b"\x03");
                    self.phase = Phase::Closing;
                }
                Phase::Closing if binary.exit_status().is_some() => peer.close(),
                Phase::Lending | Phase::Reading | Phase::Back | Phase::Closing => {}
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
        self.settled = matches!(self.phase, Phase::Closing) && roots.iter().all(Host::is_empty);
        if !self.settled
            && let Some(figures) = self.patience.failed()
        {
            return Err(format!("terminal loan did not settle in {:?}: {figures}", self.phase));
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

#[test]
fn temper_box_lends_its_pty_to_the_found_program_login_and_draws_again() {
    let scratch = support::Scratch::startup("tui-lends-terminal");
    let root = scratch.path();
    let client = root.join("temper");
    fs::copy(skein_scratch::paths::binary("temper-box").with_file_name("temper"), &client)
        .expect("client beside fake box");
    let peer = Peer::new();
    let script = format!(
        r"#!/usr/bin/env python3
import os, signal, sys, termios
if sys.argv[1:] == ['--version']:
    print({identity:?})
elif sys.argv[1] == 'program-login':
    old = signal.pthread_sigmask(signal.SIG_BLOCK, {{signal.SIGTTOU}})
    os.tcsetpgrp(0, os.getpgrp())
    signal.pthread_sigmask(signal.SIG_SETMASK, old)
    modes = termios.tcgetattr(0)
    assert modes[3] & termios.ECHO and modes[3] & termios.ICANON
    assert sys.argv[2:] == ['--config', os.environ['TEMPER_CONFIG'], '--data', os.environ['TEMPER_DATA'], '--cache', os.environ['TEMPER_CACHE'], 'work']
    print('LOGIN-READY', flush=True)
    assert sys.stdin.readline() == 'work input\n'
    print('LOGIN-READ', flush=True)
else:
    assert sys.argv[1] == '--front'
    os.write(3, b'http://{address}/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA')
    os.close(3)
    sys.stdin.buffer.read()
",
        identity = temper_version::line("temper-box"),
        address = peer.address()
    );
    let fake = root.join("temper-box");
    fs::write(&fake, script).expect("fake box and its line-reading program-login");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).expect("executable fake box");
    let mut environment: Vec<(OsString, OsString)> = vec![
        ("PATH".into(), "/usr/bin:/bin".into()),
        ("HOME".into(), root.into()),
        ("XDG_STATE_HOME".into(), root.join("state").into()),
        ("TEMPER_CONFIG".into(), root.join("config").into()),
        ("TEMPER_DATA".into(), root.join("data").into()),
        ("TEMPER_CACHE".into(), root.join("cache").into()),
        ("TERM".into(), "xterm-256color".into()),
        ("COLORFGBG".into(), "15;0".into()),
    ];
    environment.extend(
        temper_test_env::environment()
            .iter()
            .filter(|(key, _)| key.as_ref() == b"TMPDIR")
            .map(|(key, value)| (std::ffi::OsStr::from_bytes(key).into(), std::ffi::OsStr::from_bytes(value).into())),
    );
    let mut binary = Binary::start_with_tree(
        Command {
            program: PathBuf::from("/bin/sh"),
            arguments: vec![
                "-c".into(),
                "read -r ready; exec \"$@\"".into(),
                "tui-loan".into(),
                client.into(),
                "box".into(),
            ],
            environment,
            directory: root.into(),
        },
        Mode::Terminal,
        8192,
        Expectation::Supervise,
        false,
    )
    .expect("temper box on a pseudo-terminal");
    let Streams::Terminal { stream } = binary.take_streams().expect("terminal") else { panic!("terminal master") };
    skein_shell::resize_terminal(stream, Size { columns: 80, rows: 24 }).expect("80 by 24");
    let mut world = real::World::new(Judge {
        failed: None,
        phase: Phase::Lending,
        patience: support::patience::Patience::watching("TUI's terminal loan", support::WITHIN, &binary),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Root::Client(Box::new(binary)));
    world.spawn_with_fds(vec![stream], || Root::Terminal(Box::new(Terminal::new(stream))));
    world.spawn_with_fds(vec![peer.descriptor], || Root::Peer(Box::new(peer)));
    let ended = world.run(&skein_shell::Clock::new(), support::WITHIN);
    let Root::Client(binary) = &ended.procs[0] else { panic!("client") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)));
    assert!(binary.stderr().is_empty(), "{}", String::from_utf8_lossy(binary.stderr()));
    assert_eq!(binary.tree_status().cleanup(), super::Cleanup::None);
    assert!(ended.procs.iter().all(Host::is_empty));
    scratch.remove();
}
