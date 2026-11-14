//! A real connection client and independent fake LLM process hosted by
//! Skein's world kit on simulated loopback (llm/connection.md, section 8;
//! testing/fakes/llm.md, section 3). The referee sees only owner events and peer
//! observations. Plaintext scenarios replay; TLS controls share this io
//! composition and the test trust chain without claiming replay.

use std::net::{Ipv4Addr, SocketAddr};

use skein_fake_llm_domain::{self as fake, api};
use skein_fake_llm_protocol::{documents, provider};
use skein_fake_peers::{Transport, llm};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};
use skein_llm_connection::{Component, Deadlines, Endpoint, Event, Limits, Request};
use skein_sim::Config;
use skein_world::{Delegate, Host, Memory, Outcome, Referee, World};

const ROOM: u32 = 256;
const CUE: &[u8] = b"simulated-world";

fn limits(calls: u32) -> Limits {
    Limits {
        endpoints: 1,
        connections: calls,
        calls,
        per_endpoint: calls,
        memory: 16 * 1024 * 1024,
        idle_keep: Duration::from_secs(300),
        io: io::Limits { sockets: calls, sends: 4, backlog: 2, retry: Duration::from_millis(1), ..io::Limits::TEST },
        // HTTP may demand one whole head line; TLS must admit that derived read.
        tls: skein_tls::client::Limits {
            read: skein_llm::client::largest_read(&skein_llm_kit::limits()),
            send: 4096,
            records: skein_tls::client::MAX_RECORD,
        },
    }
}

/// The client's service loop; its public face is only the events its owner received.
pub struct Client {
    io: io::Io,
    limits: Limits,
    component: Option<Component>,
    address: Option<SocketAddr>,
    transport: Transport,
    io_events: Queue<io::Event>,
    events: Queue<Event>,
    requests: Queue<io::Request>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    received: Vec<Event>,
    calls: u32,
    input: usize,
    completed: u32,
    first_delta: Option<Time>,
    completion_time: Option<Time>,
    closed: bool,
}

impl Client {
    fn new(transport: Transport, calls: u32, input: usize) -> Self {
        let limits = limits(calls);
        Self {
            io: io::Io::new(&limits.io),
            limits,
            component: None,
            address: None,
            transport,
            io_events: Queue::uncapped(),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            submissions: Queue::with_capacity(ROOM),
            completions: Queue::with_capacity(ROOM),
            received: Vec::with_capacity(32),
            calls,
            input,
            completed: 0,
            first_delta: None,
            completion_time: None,
            closed: false,
        }
    }

    fn start(&mut self, now: Time, wall: Wall) {
        let call = skein_llm_kit::call(7);
        let mut endpoints = List::with_capacity(1);
        endpoints
            .push(Endpoint {
                address: self.address.expect("referee supplied the bound address"),
                transport: match self.transport {
                    Transport::Plaintext => skein_llm_connection::Transport::Plaintext,
                    Transport::Tls => skein_llm_connection::Transport::Tls {
                        server_name: skein_tls_kit::pki::name(),
                        trust: skein_tls_kit::pki::client(&[]),
                    },
                },
                llm: call.endpoint,
                limits: skein_llm_kit::limits(),
                credential: skein_llm::client::CredentialLimits { value: 2048 },
            })
            .expect("one endpoint");
        let mut component = Component::new(endpoints, &self.limits).expect("configured component");
        let env = Env { now, wall, limits: self.limits };
        for index in 0..self.calls {
            let token = Token::new(u64::from(index).checked_add(7).expect("bounded calls"));
            let call = skein_llm_kit::call(token.raw());
            let mut prompt = call.prompt;
            prompt.instructions = CUE.into();
            if self.input > 0 {
                prompt.messages = Box::new([skein_llm::Message {
                    role: skein_llm::Role::User,
                    content: Box::new([skein_llm::Block::Text {
                        text: vec![b'x'; self.input].into_boxed_slice(),
                        replay: None,
                    }]),
                }]);
            }
            component.down(
                &env,
                Request::Start {
                    drop_reasoning: false,
                    call: token,
                    endpoint: 0,
                    prompt,
                    credential: call.credential,
                    deadlines: Deadlines { whole: Some(Duration::from_secs(10)), ..Deadlines::none() },
                },
                &mut self.events,
                &mut self.requests,
            );
            component.down(&env, Request::Next { call: token }, &mut self.events, &mut self.requests);
        }
        self.component = Some(component);
    }
}

impl Host for Client {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.component.is_none() && self.address.is_some() {
            self.start(now, wall);
        }
        let io_env = Env { now, wall, limits: self.limits.io };
        let env = Env { now, wall, limits: self.limits };
        io::stage::up(&mut self.io, &io_env, &mut self.completions, &mut self.io_events, &mut self.submissions, now);
        for _ in 0..ROOM {
            let Some(event) = self.io_events.pop() else { break };
            self.component.as_mut().expect("io follows Start").up(&env, event, &mut self.events, &mut self.requests);
        }
        if let Some(component) = &mut self.component
            && (component.has_work() || component.next_deadline().is_some_and(|due| due <= now))
        {
            component.fire(&env, &mut self.events, &mut self.requests);
        }
        for _ in 0..ROOM {
            let Some(event) = self.events.pop() else { break };
            match &event {
                Event::Delta { call, .. } | Event::Block { call, .. } => {
                    if let Event::Delta { .. } = &event {
                        self.first_delta.get_or_insert(now);
                    }
                    self.component.as_mut().expect("started").down(
                        &env,
                        Request::Next { call: *call },
                        &mut self.events,
                        &mut self.requests,
                    );
                }
                Event::Completed { .. } => {
                    self.completed += 1;
                    self.completion_time = Some(now);
                    if self.completed == self.calls {
                        self.component.as_mut().expect("started").down(
                            &env,
                            Request::Close,
                            &mut self.events,
                            &mut self.requests,
                        );
                    }
                }
                Event::Closed => {
                    assert!(!self.closed, "one component Closed");
                    assert_eq!(self.completed, self.calls);
                    self.closed = true;
                }
                Event::Refused { .. } | Event::Failed { .. } | Event::Cancelled { .. } => {
                    panic!("positive call failed: {event:?}")
                }
            }
            assert!(self.received.len() < 32, "bounded owner observations");
            self.received.push(event);
        }
        io::stage::down(&mut self.io, &io_env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
        if let Some(component) = &mut self.component {
            component.reclaim();
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || !self.io_events.is_empty()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.completions.is_empty()
            || (self.component.is_none() && self.address.is_some())
            || self.component.as_ref().is_some_and(|component| {
                component.has_work() || component.next_deadline().is_some_and(|due| due <= now)
            })
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.component.as_ref().and_then(Component::next_deadline)
    }

    fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.io_events.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
            && self.closed
            && !self.component.as_ref().is_some_and(Component::has_work)
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits.io).expect("bounded io")
    }
}

/// Processes joined only by the world's simulated network.
pub enum Process {
    /// The connection service sends owner records to the referee and closes its sockets.
    Client(Box<Client>),
    /// The independent fake sends outside observations and settles on shutdown.
    Peer(Box<Held>),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Client(client) => client.as_ref(),
            Self::Peer(held) => held.as_ref(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Client(client) => client.as_mut(),
            Self::Peer(held) => held.as_mut(),
        }
    }
}

/// The independent fake, whose host releases its held reply once, at a set time.
pub struct Held {
    peer: llm::Peer,
    release: Option<(Box<[u8]>, Time)>,
}

impl Host for Held {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.release.as_ref().is_some_and(|(_, at)| now >= *at) {
            let (name, _) = self.release.take().expect("due release");
            self.peer.release(&name).expect("one bounded release");
        }
        self.peer.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        self.peer.completions()
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        self.peer.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.release.as_ref().is_some_and(|(_, at)| now >= *at) || self.peer.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        [self.peer.next_mechanism_deadline(), self.release.as_ref().map(|(_, at)| *at)].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.peer.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.peer.is_empty()
    }
    fn operations(&self) -> u32 {
        self.peer.operations()
    }
}

struct Judge {
    activated: bool,
    calls: u32,
    answer: Vec<u8>,
    passed: bool,
    teardown_started: bool,
    query_times: std::collections::BTreeMap<Token, Time>,
    input: usize,
    release_at: Option<Time>,
}

impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, processes: &mut [Process]) {
        let address = processes.iter().find_map(|process| match process {
            Process::Peer(held) => held.peer.address(),
            Process::Client(_) => None,
        });
        if address.is_some() {
            self.activated = true;
        }
        for process in processes {
            match process {
                Process::Client(client) => client.address = address,
                Process::Peer(held) => {
                    if self.passed {
                        held.peer.shutdown();
                        self.teardown_started = true;
                    }
                }
            }
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the referee returns each owner and peer finding before advancing the scenario"
    )]
    fn watch(&mut self, now: Time, processes: &[Process]) -> Result<(), String> {
        let mut completed = 0;
        let mut client_closed = false;
        let mut queries = 0;
        let mut answers = 0;
        for process in processes {
            match process {
                Process::Client(client) => {
                    client_closed = client.closed;
                    if client.closed
                        && let Some(at) = self.release_at
                    {
                        if client.first_delta.ok_or("first piece reached the owner")? >= at {
                            return Err(String::from(
                                "assertion failed: client.first_delta.expect(\"first piece reached the owner\") < at",
                            ));
                        }
                        if client.completion_time.ok_or("released call completed")? <= at {
                            return Err(String::from(
                                "assertion failed: client.completion_time.expect(\"released call completed\") > at",
                            ));
                        }
                    }
                    for event in &client.received {
                        if let Event::Completed { completion, .. } = event {
                            if completion.content.len() != 1 {
                                return Err(format!(
                                    "assertion `left == right` failed: one literal scripted block\n  left: {:?}\n right: {:?}",
                                    completion.content.len(),
                                    1
                                ));
                            }
                            match &completion.content[0] {
                                skein_llm::Block::Text { text, .. } => {
                                    if text.as_ref() != self.answer {
                                        return Err(format!(
                                            "assertion `left == right` failed: the client's actual terminal is the literal script\n  left: {:?}\n right: {:?}",
                                            text.as_ref(),
                                            self.answer
                                        ));
                                    }
                                }
                                block @ (skein_llm::Block::Refusal { .. }
                                | skein_llm::Block::ToolCall { .. }
                                | skein_llm::Block::ToolResult { .. }
                                | skein_llm::Block::Reasoning { .. }
                                | skein_llm::Block::Oversize { .. }
                                | skein_llm::Block::Cut { .. }
                                | skein_llm::Block::Dropped { .. }) => {
                                    return Err(format!("expected scripted text, got {block:?}"));
                                }
                            }
                            completed += 1;
                        }
                    }
                }
                Process::Peer(held) => {
                    for observation in held.peer.observations() {
                        match observation {
                            llm::Observation::Query { connection, query } => {
                                if query.system.as_ref() != CUE {
                                    return Err(format!(
                                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                        query.system.as_ref(),
                                        CUE
                                    ));
                                }
                                self.query_times.entry(*connection).or_insert(now);
                                if self.input > 0 {
                                    if query.messages.len() != 1 {
                                        return Err(format!(
                                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                            query.messages.len(),
                                            1
                                        ));
                                    }
                                    if query.messages[0].parts.len() != 1 {
                                        return Err(format!(
                                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                            query.messages[0].parts.len(),
                                            1
                                        ));
                                    }
                                    match &query.messages[0].parts[0] {
                                        api::Part::Text { text } => {
                                            if text.len() != self.input {
                                                return Err(format!(
                                                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                                    text.len(),
                                                    self.input
                                                ));
                                            }
                                            if !text.iter().all(|byte| *byte == b'x') {
                                                return Err(String::from(
                                                    "assertion failed: text.iter().all(|byte| *byte == b'x')",
                                                ));
                                            }
                                        }
                                        part @ (api::Part::Opaque { .. }
                                        | api::Part::ToolCall { .. }
                                        | api::Part::ToolOutput { .. }) => {
                                            return Err(format!("expected literal user text, got {part:?}"));
                                        }
                                    }
                                }
                                queries += 1;
                            }
                            llm::Observation::Answered { connection, result } => {
                                if *result != Ok(()) {
                                    return Err(format!(
                                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                        *result,
                                        Ok::<(), api::Error>(())
                                    ));
                                }
                                let connection = connection.ok_or("live call route")?;
                                let started = self.query_times[&connection];
                                if now < started.checked_add(Duration::from_millis(5)).expect("bounded script delay") {
                                    return Err(String::from("batching must not shortcut the actual script latency"));
                                }
                                answers += 1;
                            }
                            llm::Observation::Head { headers, .. } => {
                                if !(headers.iter().any(|field| {
                                    field.is(b"content-type") && field.value.as_ref() == b"application/json"
                                })) {
                                    return Err(String::from(
                                        "assertion failed: headers.iter().any(|field| field.is(b\"content-type\") && field.value.as_ref() == b\"application/json\")",
                                    ));
                                }
                                if headers.iter().any(|field| {
                                    field.is(b"authorization")
                                        || field.is(b"x-api-key")
                                        || field.is(b"chatgpt-account-id")
                                }) {
                                    return Err(String::from(
                                        "assertion failed: !headers.iter().any(|field| field.is(b\"authorization\") || field.is(b\"x-api-key\") || field.is(b\"chatgpt-account-id\"))",
                                    ));
                                }
                            }
                            llm::Observation::Credential { route, scheme, index, .. } => {
                                if (*route, *scheme, *index)
                                    != (skein_llm::Provider::OpenAi, Some(skein_llm::Scheme::Bearer), Some(0))
                                {
                                    return Err(format!(
                                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                        (*route, *scheme, *index),
                                        (skein_llm::Provider::OpenAi, Some(skein_llm::Scheme::Bearer), Some(0))
                                    ));
                                }
                            }
                            llm::Observation::Accepted { .. }
                            | llm::Observation::List { .. }
                            | llm::Observation::Closed { .. } => {}
                        }
                    }
                }
            }
        }
        if completed > self.calls || queries > self.calls || answers > self.calls {
            return Err(String::from("unique calls and terminals"));
        }
        self.passed = client_closed && completed == self.calls && queries == self.calls && answers == self.calls;
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if !self.activated || (self.passed && !self.teardown_started) {
            Some(Time::ZERO)
        } else if self.passed {
            None
        } else {
            Some(Time::from_nanos(20_000_000_000))
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= Time::from_nanos(20_000_000_000) && !self.passed)
            .then(|| "scripted calls have not completed".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed
    }
}

/// Runs the shared harness with separate process heaps; large fills every delayed-call slot.
#[must_use]
pub fn run(seed: u64, faulted: bool, transport: Transport, large: bool, memory: Memory) -> Outcome<Process> {
    run_with_input(seed, faulted, transport, large, memory, 0)
}

/// Runs an exact caller-owned text upload through the independently hosted peer.
#[must_use]
pub fn run_with_input(
    seed: u64,
    faulted: bool,
    transport: Transport,
    large: bool,
    memory: Memory,
    input: usize,
) -> Outcome<Process> {
    run_with_shutdown(seed, faulted, transport, large, memory, input, false)
}

/// A live peer that ignores half-close waits for io to settle the client.
#[must_use]
pub fn run_with_shutdown(
    seed: u64,
    faulted: bool,
    transport: Transport,
    large: bool,
    memory: Memory,
    input: usize,
    ignore_half_close: bool,
) -> Outcome<Process> {
    run_configured(seed, faulted, transport, large, memory, Setup { input, ignore_half_close, held: false })
}

#[derive(Clone, Copy)]
struct Setup {
    input: usize,
    ignore_half_close: bool,
    held: bool,
}

/// Runs two text pieces separated by a host release at 50 ms.
#[must_use]
pub fn run_held(seed: u64) -> Outcome<Process> {
    run_configured(
        seed,
        false,
        Transport::Plaintext,
        false,
        Memory::Checked,
        Setup { input: 0, ignore_half_close: false, held: true },
    )
}

fn fill_credentials(peer: &mut llm::Peer, protocol_limits: &provider::Limits) {
    let first = skein_llm_kit::call(7).credential.expect("fixture credential");
    let available = usize::try_from(protocol_limits.http.head).expect("u32 fits") - first.value.len();
    let count = usize::try_from(provider::MAX_CREDENTIALS - 1).expect("u32 fits");
    for index in 0..count {
        let bytes = available / count + usize::from(index < available % count);
        let credential = skein_llm::Credential::bearer(vec![b'x'; bytes].into_boxed_slice());
        assert_eq!(peer.accept(credential), Ok(u32::try_from(index + 1).expect("bounded index")));
    }
}

fn run_configured(
    seed: u64,
    faulted: bool,
    transport: Transport,
    large: bool,
    memory: Memory,
    setup: Setup,
) -> Outcome<Process> {
    let Setup { input, ignore_half_close, held } = setup;
    let release_at = held.then_some(Time::from_nanos(50_000_000));
    let calls = if large { 2 } else { 1 };
    let answer = if large { vec![b'x'; 2048] } else { b"simulated answer".to_vec() };
    let judge = Judge {
        activated: false,
        calls,
        answer: answer.clone(),
        passed: false,
        teardown_started: false,
        query_times: std::collections::BTreeMap::new(),
        input,
        release_at,
    };
    let mut config = Config::calm();
    config.wall = skein_tls_kit::pki::VALID;
    config.buffer = if faulted { 256 } else { 1024 };
    if faulted {
        config.faults.short_recv = 300;
        config.faults.short_send = 300;
    }
    let mut world = World::new(seed, config, judge, memory);
    world.spawn(|| {
        let client_limits = limits(calls);
        let protocol_limits = skein_llm_kit::fake::limits(&skein_llm_kit::limits());
        let mut domain_limits = skein_llm_kit::fake::config();
        domain_limits.calls = calls;
        domain_limits.answer_bytes =
            u32::try_from(answer.len() + std::mem::size_of::<api::Part>()).expect("exact answer cap");
        domain_limits.script_bytes = u32::try_from(
            answer.len()
                + CUE.len()
                + std::mem::size_of::<api::Script>()
                + std::mem::size_of::<api::Turn>()
                + std::mem::size_of::<api::Line>()
                + if held { 2 * std::mem::size_of::<api::Piece>() + b"ready".len() } else { 0 },
        )
        .expect("exact script cap");
        domain_limits.latency_min = Duration::from_millis(5);
        domain_limits.latency_max = Duration::from_millis(5);
        let scripts = Box::new([api::Script {
            cue: CUE.into(),
            turns: Box::new([api::Turn {
                lines: Box::new([if held {
                    api::Line::Pieces {
                        pieces: Box::new([
                            api::Piece { text: b"simulated ".as_slice().into(), hold: None },
                            api::Piece { text: b"answer".as_slice().into(), hold: Some(b"ready".as_slice().into()) },
                        ]),
                    }
                } else {
                    api::Line::Text { text: answer.clone().into_boxed_slice() }
                }]),
                finish: api::Finish::Stop,
                tokens: 2,
            }]),
        }]);
        let domain = fake::Domain::try_scripted(&domain_limits, seed, scripts).expect("script admitted at exact cap");
        let mut io_limits = client_limits.io;
        io_limits.sockets = calls + 1;
        let limits = skein_fake_peers::Limits {
            io: io_limits,
            connections: calls,
            queue: 64,
            plaintext: 32_768,
            ciphertext: 32_768,
            observations: calls * 6,
            observation_bytes: calls * 32_768,
        };
        let mut peer = llm::Peer::new(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            transport,
            limits,
            provider::Config {
                list: None,
                list_status: None,
                without_credential: false,
                usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
                echo: skein_llm::openai::Echo::NONE,
                provider: documents::Provider::OpenAi,
                path: skein_llm::Endpoint::openai().target,
                headers: Box::new([]),
            },
            skein_llm_kit::call(7).credential.expect("fixture credential"),
            protocol_limits,
            domain,
            domain_limits,
        )
        .expect("bounded independent process");
        if large {
            fill_credentials(&mut peer, &protocol_limits);
        }
        if ignore_half_close {
            peer.ignore_half_close();
        }
        Process::Peer(Box::new(Held { peer, release: release_at.map(|at| (b"ready".as_slice().into(), at)) }))
    });
    world.spawn(|| Process::Client(Box::new(Client::new(transport, calls, input))));
    world.run()
}

/// Runs and replays the actual component against an independent plaintext process.
pub fn scenario(seed: u64, faulted: bool) {
    let first = run(seed, faulted, Transport::Plaintext, false, Memory::Unchecked);
    let second = run(seed, faulted, Transport::Plaintext, false, Memory::Unchecked);
    assert_replay(&first, &second);
    assert_eq!(first.end, second.end, "injected latency and shutdown replay");
}

/// Compares wire order, outside peer observations and every actual owner event.
pub fn assert_replay(first: &Outcome<Process>, second: &Outcome<Process>) {
    assert_eq!(first.trace, second.trace, "the actual two-process io trace replays");
    for (first, second) in first.procs.iter().zip(&second.procs) {
        match (first, second) {
            (Process::Peer(first), Process::Peer(second)) => {
                assert_eq!(first.release, second.release);
                assert_eq!(
                    first.peer.observations(),
                    second.peer.observations(),
                    "the independent peer's observations replay"
                );
            }
            (Process::Client(first), Process::Client(second)) => assert_eq!(
                format!("{:?}", first.received),
                format!("{:?}", second.received),
                "actual owner payloads and terminals replay"
            ),
            (Process::Peer(_), Process::Client(_)) | (Process::Client(_), Process::Peer(_)) => {
                panic!("process order replays")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_returns_a_completion_before_its_release() {
        let mut judge = Judge {
            activated: true,
            calls: 1,
            answer: Vec::new(),
            passed: false,
            teardown_started: false,
            query_times: std::collections::BTreeMap::new(),
            input: 0,
            release_at: Some(Time::from_nanos(3)),
        };
        let mut client = Client::new(Transport::Plaintext, 1, 0);
        client.closed = true;
        client.first_delta = Some(Time::from_nanos(1));
        client.completion_time = Some(Time::from_nanos(2));
        let mut procs = [Process::Client(Box::new(client))];
        assert_eq!(
            judge.watch(Time::from_nanos(4), &procs),
            Err(String::from("assertion failed: client.completion_time.expect(\"released call completed\") > at"))
        );
        let Process::Client(client) = &mut procs[0] else { unreachable!("the client fixture") };
        client.completion_time = Some(Time::from_nanos(4));
        assert_eq!(judge.watch(Time::from_nanos(4), &procs), Ok(()));
    }
}
