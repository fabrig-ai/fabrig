//! Named backend snapshots and lower events for program step tests (programming-model.md 10.2).
#![expect(clippy::disallowed_methods, reason = "ordinary Rust step fixtures, programming-model.md 10.2")]
use super::*;
use skein_lib::Wall;

fn limits() -> protocol::ComponentLimits {
    let client = skein_llm_world::limits();
    let adapter = protocol::Limits {
        client,
        tool_bytes: 4096,
        history_bytes: 4096,
        rendered_result: 4096,
        stated: protocol::Stated {
            host_part: 4096,
            path_bytes: 128,
            read_window: 128,
            tool_payload: 128,
            file_bytes: 128,
            call_input_bytes: 4096,
            list_entries: 2,
            search_hits: 2,
            search_bytes: 128,
            shell_timeout: Duration::from_secs(1),
            tool_deadline: Duration::from_secs(1),
            job_poll: Duration::from_secs(1),
            job_wait: Duration::from_secs(1),
            jobs: 1,
            shell_output: 128,
            output_ceiling: 128,
        },
    };
    protocol::ComponentLimits {
        adapter,
        connection: skein_llm_connection::Limits {
            memory: 1 << 30,
            endpoints: 1,
            connections: 2,
            calls: 2,
            per_endpoint: 2,
            idle_keep: Duration::from_secs(1),
            io: io::Limits {
                sockets: 3,
                names: 0,
                refusals: 1,
                intake: 4096,
                receive: 4096,
                output: 4096,
                sends: 2,
                accepts: 1,
                backlog: 2,
                close_timeout: Duration::from_secs(1),
                retry: Duration::from_millis(10),
            },
            tls: skein_tls::client::Limits { read: 4096, send: 4096, records: skein_tls::client::MAX_RECORD },
        },
        receiving: protocol::Receiving {
            max_completion_bytes: 16384,
            max_completion_blocks: 8,
            decoded_call_bytes: 4096,
            max_failure_bytes: 256,
        },
        contract_bytes: 4096,
        llms: 1,
        accounts: 1,
        grant_value_bytes: 128,
    }
}
pub(super) fn backend() -> Backend {
    backend_with_line_bytes(4096)
}

pub(super) fn backend_with_line_bytes(line_bytes: u32) -> Backend {
    let limits = limits();
    Backend::new(
        Token::new(10),
        Settings {
            program: Programs { claude: b"/claude".as_slice().into(), directory: b"/programs".as_slice().into() },
            directory: None,
            workspace: None,
            path_bytes: 4096,
            relay: b"/agent".as_slice().into(),
            environment: Box::default(),
            mcp: mcp::Limits { calls: 4, unauthenticated: Time::from_nanos(1000), ..mcp::fixture::SMALL },
            adapter: limits.adapter,
            receiving: limits.receiving,
            stream_tokens: Settings::stream_tokens(line_bytes),
            prompt_bytes: 4096,
            contract_bytes: 4096,
            line_bytes,
            input_bytes: 32768,
            max_turns: 8,
            grace: Duration::from_secs(1),
            startup: Duration::from_secs(30),
        },
        &Secret::new(Box::new([7; 32])),
    )
}
pub(super) fn env(seconds: u64) -> Env<protocol::ComponentLimits> {
    Env {
        now: Time::from_nanos(seconds.checked_mul(1_000_000_000).expect("simulated clock")),
        wall: Wall::EPOCH,
        limits: limits(),
    }
}

pub(super) fn request(backend: &Backend) -> FromDomain {
    FromDomain::Complete {
        owner: Token::new(80),
        grant: Some(GrantName { account: 0, generation: 1 }),
        prompt: prompt_with(Box::default()),
        deadlines: smith_domain::session::Deadlines {
            head: Duration::from_secs(1),
            idle: Duration::from_secs(1),
            whole: Time::from_nanos(30_000_000_000),
        },
        bounds: backend.settings.receiving,
        outcome: smith_domain::run::outcome::OutcomeSpec {
            change: None,
            verdicts: Box::default(),
            report: None,
            failure: None,
        },
        deliver: None,
        observation: None,
        deltas: smith_protocol_llm::Deltas { text: false, input: false },
    }
}

pub(super) fn connect_server(backend: &mut Backend) {
    let first = Token::new(100);
    let owner = Token::new(101);
    let mut io = Queue::with_capacity(64);
    let mut server = mcp::Server::new(
        first,
        0,
        &[7; 32],
        Box::new([mcp::Tool {
            name: b"foo".as_slice().into(),
            description: b"Tool".as_slice().into(),
            input_schema: br#"{"type":"object"}"#.as_slice().into(),
        }]),
        backend.settings.mcp,
        &mut io,
    )
    .expect("server");
    server.from_io(
        env(0).now,
        io::Event::Listening {
            owner: first,
            listener: Token::new(200),
            addr: io::Addr::Ip(core::net::SocketAddr::from(([127, 0, 0, 1], 47100))),
        },
        &mut io,
    );
    let Some(mcp::Up::Listening { .. }) = server.up() else { panic!("listening") };
    server.from_io(
        env(0).now,
        io::Event::Accepted {
            owner: first,
            socket: Token::new(201),
            peer: io::Addr::Ip(core::net::SocketAddr::from(([127, 0, 0, 1], 50000))),
        },
        &mut io,
    );
    let secret = mcp::secret_text(&[7; 32]);
    server.from_io(
        env(0).now,
        io::Event::Stream { owner, up: skein_lib::stream::Up::Bytes(mcp::json::join(&[&secret, b"\n"])) },
        &mut io,
    );
    let right = io
        .iter()
        .find_map(|request| match request {
            io::Request::Output { down: skein_lib::stream::OutputDown::Room { right, .. }, .. } => Some(*right),
            io::Request::Listen { .. }
            | io::Request::Connect { .. }
            | io::Request::Bind { .. }
            | io::Request::Reject { .. }
            | io::Request::Stream { .. }
            | io::Request::Output { .. }
            | io::Request::Spawn { .. }
            | io::Request::Signal { .. }
            | io::Request::Usage { .. }
            | io::Request::Window { .. }
            | io::Request::Close { .. }
            | io::Request::Abort { .. } => None,
        })
        .expect("ack room");
    server.from_io(
        env(0).now,
        io::Event::Output {
            owner,
            up: skein_lib::stream::OutputUp::Settled { right, outcome: skein_lib::stream::OutputOutcome::Granted },
        },
        &mut io,
    );
    assert_eq!(server.up(), Some(mcp::Up::Connected));
    backend.server = Some(server);
}

pub(super) fn server_reply(backend: &mut Backend) -> serde_json::Value {
    for _ in 0_u8..16 {
        for _ in 0..backend.io.capacity() {
            let Some(request) = backend.io.pop() else { break };
            match request {
                io::Request::Output { stream, down: skein_lib::stream::OutputDown::Room { right, .. } }
                    if stream == Token::new(201) =>
                {
                    backend.server.as_mut().expect("server").from_io(
                        env(0).now,
                        io::Event::Output {
                            owner: Token::new(101),
                            up: skein_lib::stream::OutputUp::Settled {
                                right,
                                outcome: skein_lib::stream::OutputOutcome::Granted,
                            },
                        },
                        &mut backend.io,
                    );
                }
                io::Request::Output { stream, down: skein_lib::stream::OutputDown::Send { bytes, .. } }
                    if stream == Token::new(201) =>
                {
                    return serde_json::from_slice(&bytes).expect("MCP reply");
                }
                io::Request::Listen { .. }
                | io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }
                | io::Request::Close { .. }
                | io::Request::Abort { .. } => {}
            }
        }
    }
    panic!("MCP response pending");
}

/// A completion with its child and all three program pipes open.
pub(super) fn active() -> Backend {
    running(backend())
}

/// Keep caller-selected limits and attach the standard running completion.
pub(super) fn running(mut backend: Backend) -> Backend {
    backend.phase = Phase::Running;
    backend.child = Some(Token::new(30));
    backend.pipes = Pipes::from_spawn(&[Token::new(31), Token::new(32), Token::new(33)]);
    backend.owner = Some(Token::new(80));
    backend.request = Some(request(&backend));
    backend
}

/// A running completion whose output is supplied as parsed records.
pub(super) fn records() -> Backend {
    records_with(backend())
}

pub(super) fn records_with(backend: Backend) -> Backend {
    let mut backend = running(backend);
    backend.pipes = Pipes { input: Some(Token::new(31)), ..Pipes::CLOSED };
    backend
}

/// A running completion with the deterministic window identity installed.
pub(super) fn window() -> Backend {
    let mut backend = active();
    let FromDomain::Complete { prompt, .. } = backend.request.as_ref().expect("completion") else {
        panic!("completion");
    };
    let session::Opening::New { id } = session::opening(prompt, None, None) else {
        panic!("new window");
    };
    backend.running = Some(id);
    backend
}

/// A window whose program pipes have settled before the relay terminal.
pub(super) fn drained_window() -> Backend {
    let mut backend = window();
    backend.child = Some(backend.token(Owner::Program));
    backend.pipes = Pipes::CLOSED;
    backend
}

/// A backend and the queues supplied at its step boundary.
pub(super) struct Harness {
    pub backend: Backend,
    pub up: Queue<ToDomain>,
    pub below: Queue<Below>,
}

impl Harness {
    pub fn new(backend: Backend) -> Self {
        Self { backend, up: Queue::with_capacity(32), below: Queue::with_capacity(64) }
    }
}

/// An ordinary main prompt with the scenario's message history.
pub(super) fn prompt_with(messages: Box<[llm::Message]>) -> llm::Prompt {
    llm::Prompt {
        purpose: llm::Purpose::Ordinary { guard: None },
        asker: llm::Asker::Main,
        endpoint: llm::Endpoint(0),
        affinity: llm::Affinity { key: [0; 16], thread: 0 },
        model: b"fake".as_slice().into(),
        effort: None,
        system: Box::default(),
        tools: smith_domain::tools::Grants { inspect: false, modify: false, shell: false },
        namespace: smith_domain::tools::Namespace::NoWorkspace,
        served: Box::default(),
        choice: llm::ToolChoice::Auto,
        messages,
        max_tokens: 128,
    }
}

/// A lazy startup whose program spawn has not answered yet.
pub(super) fn spawning() -> Backend {
    let mut backend = backend();
    backend.phase = Phase::Starting;
    backend.spawning = true;
    backend
}

/// A spawned program waiting for its init record.
pub(super) fn starting() -> Backend {
    let mut backend = active();
    backend.phase = Phase::Starting;
    backend.deadline = Some(env(30).now);
    backend.child = Some(backend.token(Owner::Program));
    backend
}

/// One child with no remaining output streams.
pub(super) fn running_child() -> Backend {
    let mut backend = backend();
    backend.phase = Phase::Running;
    backend.child = Some(Token::new(30));
    backend
}

/// A program input line waiting for output room.
pub(super) fn input(bytes: Box<[u8]>) -> Backend {
    let mut backend = backend();
    backend.phase = Phase::Running;
    backend.pipes.input = Some(Token::new(31));
    backend.outgoing = Some(bytes);
    backend.send_input();
    backend
}
