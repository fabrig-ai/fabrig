//! The component's owner routes io and a TLS stream to the independent fake
//! LLM byte peer in both dialects (llm-connection.md, section 8).

use core::net::Ipv4Addr;

use skein_fake_llm_domain::{self as fake, api};
use skein_fake_llm_protocol::{documents, provider};
use skein_io::kernel::Addr;
use skein_io::{Event as IoEvent, Request as IoRequest};
use skein_lib::Wall;
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Intake, List, Queue, Time, Token};
use skein_llm::{Credential, Provider};
use skein_llm_connection::{Component, Deadlines, Endpoint, Event, Limits, Request};
use skein_llm_connection_world::plaintext::Wire;
use skein_world::domain::assert_replays;
use skein_world::stream::Trace;

fn limits() -> Limits {
    Limits {
        endpoints: 1,
        connections: 1,
        calls: 1,
        per_endpoint: 1,
        memory: 16 * 1024 * 1024,
        idle_keep: Duration::from_secs(10),
        io: skein_io::Limits { sockets: 1, intake: 19_000, output: 19_000, ..skein_io::Limits::TEST },
        tls: skein_tls::client::Limits { read: 4096, send: 4096, records: skein_tls::client::MAX_RECORD },
    }
}

#[expect(clippy::too_many_lines, reason = "the protocol story drives the component, io and fake peer")]
fn run(dialect: Provider, seed: u64) -> (Trace, (u32, u32)) {
    let mut call = skein_llm_kit::call(7);
    match dialect {
        Provider::OpenAi => {}
        Provider::Anthropic => {
            call.endpoint = skein_llm::Endpoint::anthropic();
            call.credential = Some(Credential::key(b"fake-token".as_slice().into()));
            call.prompt.affinity = None;
        }
    }
    call.prompt.output_ceiling(dialect, 4096).expect("provider output ceiling");
    call.prompt.instructions = b"connection-world".as_slice().into();
    let peer_credential = Credential {
        scheme: skein_llm_kit::credential_of(&call).scheme,
        value: skein_llm_kit::credential_of(&call).value.clone(),
    };
    let peer_limits = skein_llm_kit::fake::limits(&skein_llm_kit::limits());
    let provider_kind = match dialect {
        Provider::OpenAi => documents::Provider::OpenAi,
        Provider::Anthropic => documents::Provider::Anthropic,
    };
    let mut service = provider::Service::new(
        provider::Config {
            list: None,
            list_status: None,
            without_credential: false,
            usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
            echo: skein_llm::openai::Echo::NONE,
            provider: provider_kind,
            path: call.endpoint.target.clone(),
            headers: Box::new([]),
        },
        &peer_limits,
    )
    .expect("fake service");
    let mut peer = provider::Server::new(Token::new(2), &peer_limits).expect("fake server");
    let fake_limits = skein_llm_kit::fake::config();
    let scripts = Box::new([api::Script {
        cue: b"connection-world".as_slice().into(),
        turns: Box::new([api::Turn {
            lines: Box::new([api::Line::Text { text: b"scripted answer".as_slice().into() }]),
            finish: api::Finish::Stop,
            tokens: 2,
        }]),
    }]);
    let mut domain = fake::Domain::try_scripted(&fake_limits, 3, scripts).expect("scripted domain");
    let mut endpoints = List::with_capacity(1);
    endpoints
        .push(Endpoint {
            address: Addr::from((Ipv4Addr::LOCALHOST, 443)),
            transport: skein_llm_connection::Transport::Plaintext,
            llm: call.endpoint,
            limits: skein_llm_kit::limits(),
            credential: skein_llm::client::CredentialLimits { value: 2048 },
        })
        .expect("one endpoint");
    let mut component = Component::new(endpoints, &limits()).expect("component");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let peer_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: peer_limits };
    let fake_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fake_limits };
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    let mut peer_up = Queue::uncapped();
    let mut peer_down = Queue::uncapped();
    let mut replies = Queue::uncapped();
    let mut to_peer = Intake::with_capacity(32768);
    let mut peer_demand: Option<(Read, u32)> = None;
    let mut grant: Option<u32> = None;
    let mut wire = Wire::new(seed);
    let mut received = 0_usize;
    let mut owner = None;
    let mut completed = 0_u32;
    let mut fragments = 0_u32;
    provider::start(
        &mut peer,
        &mut service,
        core::slice::from_ref(&peer_credential),
        &peer_env,
        &mut peer_up,
        &mut peer_down,
    );
    component.down(
        &env,
        Request::Start {
            drop_reasoning: false,
            call: Token::new(7),
            endpoint: 0,
            prompt: call.prompt,
            credential: call.credential,
            deadlines: Deadlines::none(),
        },
        &mut up,
        &mut io,
    );
    component.down(&env, Request::Next { call: Token::new(7) }, &mut up, &mut io);
    for _ in 0_u32..40_000 {
        if let Some(event) = up.pop() {
            wire.trace.push(format!("owner {event:?}"));
            match event {
                Event::Delta { call, .. } | Event::Block { call, .. } => {
                    fragments += 1;
                    component.down(&env, Request::Next { call }, &mut up, &mut io);
                }
                Event::Completed { call, completion, .. } => {
                    assert_eq!(call, Token::new(7));
                    assert!(!completion.content.is_empty());
                    completed += 1;
                }
                Event::Closed | Event::Refused { .. } | Event::Failed { .. } | Event::Cancelled { .. } => {
                    panic!("unexpected component event: {event:?}");
                }
            }
        }
        if let Some(request) = io.pop() {
            match request {
                IoRequest::Connect { owner: token, .. } => {
                    assert!(owner.replace(token).is_none());
                    component.up(&env, IoEvent::Connecting { owner: token, socket: Token::new(100) }, &mut up, &mut io);
                    component.up(&env, IoEvent::Connected { owner: token }, &mut up, &mut io);
                }
                IoRequest::Stream { stream, down } => {
                    assert_eq!(stream, Token::new(100));
                    wire.take(down);
                }
                IoRequest::Close { entity } | IoRequest::Abort { entity } => {
                    assert_eq!(entity, Token::new(100));
                    component.up(&env, IoEvent::Closed { owner: owner.expect("connected owner") }, &mut up, &mut io);
                }
                other @ (IoRequest::Listen { .. }
                | IoRequest::Bind { .. }
                | IoRequest::Reject { .. }
                | IoRequest::Output { .. }
                | IoRequest::Spawn { .. }
                | IoRequest::Window { .. }
                | IoRequest::Usage { .. }
                | IoRequest::Signal { .. }) => panic!("unexpected io request: {other:?}"),
            }
        }
        if wire.received.len() > received {
            to_peer.append(&wire.received[received..]).expect("bounded request intake");
            received = wire.received.len();
        }
        if let Some(answer) = wire.answer() {
            component.up(
                &env,
                IoEvent::Stream { owner: owner.expect("connected owner"), up: answer },
                &mut up,
                &mut io,
            );
        }
        if let Some(event) = peer_up.pop() {
            match event {
                provider::Event::Domain(input) => fake::step(&mut domain, &fake_env, input, &mut replies),
                provider::Event::Head { .. }
                | provider::Event::List { .. }
                | provider::Event::Close
                | provider::Event::Closed => {}
            }
        }
        if let Some(reply) = replies.pop() {
            provider::down(
                &mut peer,
                &mut service,
                core::slice::from_ref(&peer_credential),
                &peer_env,
                reply,
                &mut peer_up,
                &mut peer_down,
            );
            domain.reclaim();
            service.reclaim();
        }
        if let Some(down) = peer_down.pop() {
            match down {
                Down::Demand { read: Read::Nothing, room: 0 } => peer_demand = None,
                Down::Demand { read, room } => {
                    assert!(peer_demand.is_none(), "one demand at a time");
                    peer_demand = Some((read, room));
                }
                Down::Send(bytes) => {
                    assert!(bytes.len() <= usize::try_from(grant.take().expect("room grant")).expect("room fits"));
                    wire.write(&bytes);
                }
                Down::Finish => wire.eof = true,
            }
        }
        if let Some((read, room)) = peer_demand {
            let answer = match to_peer.meet(read) {
                Some(bytes) => Some(Up::Bytes(bytes)),
                None if room > 0 => {
                    grant = Some(room);
                    Some(Up::Room)
                }
                None => None,
            };
            if let Some(answer) = answer {
                peer_demand = None;
                provider::up(
                    &mut peer,
                    &mut service,
                    core::slice::from_ref(&peer_credential),
                    &peer_env,
                    answer,
                    &mut peer_up,
                    &mut peer_down,
                );
            }
        }
        if peer.has_work() {
            provider::resume(
                &mut peer,
                &mut service,
                core::slice::from_ref(&peer_credential),
                &peer_env,
                &mut peer_up,
                &mut peer_down,
            );
        }
        if domain.is_due(fake_env.now) {
            fake::fire(&mut domain, &fake_env, &mut replies);
        }
        if component.has_work() {
            component.fire(&env, &mut up, &mut io);
        }
        if completed == 1 && !component.has_work() && io.is_empty() && up.is_empty() && !peer.has_work() {
            break;
        }
    }
    assert_eq!(service.count(), 1, "the independent fake decoded the call");
    assert_eq!(completed, 1);
    assert!(fragments > 0);
    let later = env;
    component.down(&later, Request::Close, &mut up, &mut io);
    match io.pop() {
        Some(IoRequest::Close { entity }) => {
            assert_eq!(entity, Token::new(100));
            component.up(&later, IoEvent::Closed { owner: owner.expect("owner") }, &mut up, &mut io);
            component.reclaim();
        }
        other => panic!("expected idle close, got {other:?}"),
    }
    provider::closed(&mut peer, &mut service, &peer_env, &mut peer_up, &mut peer_down);
    service.reclaim();
    domain.reclaim();
    assert!(matches!(up.pop(), Some(Event::Closed)));
    assert!(up.is_empty() && io.is_empty() && !component.has_work());
    (wire.trace, (completed, fragments))
}

#[test]
fn responses_fake_peer_over_plaintext_replays() {
    assert_replays(7, 8, |seed| run(Provider::OpenAi, seed));
}

#[test]
fn anthropic_fake_peer_over_plaintext_replays() {
    assert_replays(7, 8, |seed| run(Provider::Anthropic, seed));
}

const LIST_BODY: &[u8] = br#"{"data":[{"id":"world-model","extra":true}]}"#;
// Two of the kit peer's 128-byte send grants, plus one byte.
const MULTI_PIECE_LIST: &[u8] = &[b'x'; 257];
const LIST_CUE: &[u8] = b"list-script";
const FIRST_CALL: &[u8] = b"first scripted answer";

fn list_world(route: Provider, scheme: Option<skein_llm::Scheme>) -> skein_llm_kit::fake::Exchange {
    let mut call = match route {
        Provider::OpenAi => skein_llm_kit::public_call(7),
        Provider::Anthropic => skein_llm_kit::anthropic_call(7),
    };
    call.credential = scheme.map(|scheme| Credential { scheme, value: b"list-fixture".as_slice().into() });
    call.prompt.instructions = LIST_CUE.into();
    let scripts = Box::new([api::Script {
        cue: LIST_CUE.into(),
        turns: Box::new([api::Turn {
            lines: Box::new([api::Line::Text { text: FIRST_CALL.into() }]),
            finish: api::Finish::Stop,
            tokens: 3,
        }]),
    }]);
    skein_llm_kit::fake::Exchange::new(call, skein_llm_kit::limits_for(route), scripts)
}

fn list_request(route: Provider, method: &str, target: &str, credential: Option<&str>) -> Vec<u8> {
    let field = match route {
        Provider::OpenAi => "Authorization: Bearer ",
        Provider::Anthropic => "x-api-key: ",
    };
    let auth = credential.map_or_else(String::new, |value| format!("{field}{value}\r\n"));
    format!("{method} {target} HTTP/1.1\r\nHost: skein.test\r\n{auth}Connection: close\r\n\r\n").into_bytes()
}

fn list_reply(response: &[u8]) -> (u16, &[u8]) {
    let boundary = response.windows(b"\r\n\r\n".len()).position(|bytes| bytes == b"\r\n\r\n").expect("HTTP head");
    let head = core::str::from_utf8(&response[..boundary]).expect("HTTP response head");
    let status = head.split_ascii_whitespace().nth(1).expect("status").parse().expect("numeric status");
    (status, &response[boundary + b"\r\n\r\n".len()..])
}

struct ListRow {
    name: &'static str,
    method: &'static str,
    target: &'static str,
    body: Option<&'static [u8]>,
    status: Option<u16>,
    credential: Option<&'static str>,
    accepted: bool,
    list: bool,
    want: [u16; 2],
    want_body: Option<&'static [u8]>,
}

const LIST_ROWS: &[ListRow] = &[
    ListRow {
        name: "list",
        method: "GET",
        target: "/v1/models",
        body: Some(LIST_BODY),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [200, 200],
        want_body: Some(LIST_BODY),
    },
    ListRow {
        name: "query",
        method: "GET",
        target: "/v1/models?limit=1000",
        body: Some(LIST_BODY),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [200, 200],
        want_body: Some(LIST_BODY),
    },
    ListRow {
        name: "opaque bytes",
        method: "GET",
        target: "/v1/models",
        body: Some(b"\0not json\xff"),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [200, 200],
        want_body: Some(b"\0not json\xff"),
    },
    ListRow {
        name: "multiple pieces",
        method: "GET",
        target: "/v1/models",
        body: Some(MULTI_PIECE_LIST),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [200, 200],
        want_body: Some(MULTI_PIECE_LIST),
    },
    ListRow {
        name: "empty body",
        method: "GET",
        target: "/v1/models",
        body: Some(b""),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [200, 200],
        want_body: Some(b""),
    },
    ListRow {
        name: "absent",
        method: "GET",
        target: "/v1/models",
        body: None,
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [404, 404],
        want_body: Some(b""),
    },
    ListRow {
        name: "status",
        method: "GET",
        target: "/v1/models",
        body: Some(LIST_BODY),
        status: Some(503),
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [503, 503],
        want_body: Some(b""),
    },
    ListRow {
        name: "status without body",
        method: "GET",
        target: "/v1/models",
        body: None,
        status: Some(429),
        credential: Some("list-fixture"),
        accepted: true,
        list: true,
        want: [429, 429],
        want_body: Some(b""),
    },
    ListRow {
        name: "wrong credential",
        method: "GET",
        target: "/v1/models",
        body: Some(LIST_BODY),
        status: None,
        credential: Some("unaccepted"),
        accepted: true,
        list: true,
        want: [401, 401],
        want_body: None,
    },
    ListRow {
        name: "no credential",
        method: "GET",
        target: "/v1/models",
        body: Some(LIST_BODY),
        status: Some(503),
        credential: None,
        accepted: true,
        list: true,
        want: [401, 401],
        want_body: None,
    },
    ListRow {
        name: "local",
        method: "GET",
        target: "/v1/models",
        body: Some(LIST_BODY),
        status: None,
        credential: None,
        accepted: false,
        list: true,
        want: [200, 200],
        want_body: Some(LIST_BODY),
    },
    ListRow {
        name: "wrong method",
        method: "POST",
        target: "/v1/models",
        body: Some(LIST_BODY),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: false,
        want: [403, 400],
        want_body: None,
    },
    ListRow {
        name: "other path",
        method: "GET",
        target: "/v1/models-extra",
        body: Some(LIST_BODY),
        status: None,
        credential: Some("list-fixture"),
        accepted: true,
        list: false,
        want: [403, 400],
        want_body: None,
    },
];

#[test]
fn model_lists_answer_the_route_and_observe_only_credential_identity() {
    use skein_llm::Scheme::{Bearer, Key};
    for (route, scheme, status_index) in
        [(Provider::OpenAi, Bearer, 0), (Provider::OpenAi, Key, 0), (Provider::Anthropic, Key, 1)]
    {
        for row in LIST_ROWS {
            let mut world = list_world(route, row.accepted.then_some(scheme));
            world.service.set_list(row.body.map(Into::into), row.status);
            let (response, lists) = world.raw(&list_request(route, row.method, row.target, row.credential));
            let (status, body) = list_reply(&response);
            assert_eq!(status, row.want[status_index], "{route:?} {scheme:?} {}", row.name);
            if let Some(want) = row.want_body {
                assert_eq!(body, want, "{} body is the world's bytes", row.name);
            }
            let matched = row.accepted && row.credential == Some("list-fixture");
            let presented = provider::Presented {
                route,
                scheme: if matched {
                    Some(scheme)
                } else {
                    row.credential.map(|_| match route {
                        Provider::OpenAi => Bearer,
                        Provider::Anthropic => Key,
                    })
                },
                index: matched.then_some(0),
            };
            let want = if row.list { vec![presented] } else { vec![] };
            assert_eq!(lists, want, "{} list observes identity without credential bytes", row.name);
            assert_eq!(world.service.count(), u64::from(!row.list), "{} counts no list call", row.name);
            assert_eq!(world.service.calls(), 0, "a list holds no domain reply right");
        }
    }
}

#[test]
fn changing_a_list_between_requests_preserves_the_first_scripted_call() {
    for (route, scheme) in
        [(Provider::OpenAi, skein_llm::Scheme::Bearer), (Provider::Anthropic, skein_llm::Scheme::Key)]
    {
        let mut world = list_world(route, Some(scheme));
        let request = list_request(route, "GET", "/v1/models", Some("list-fixture"));
        world.service.set_list(Some(LIST_BODY.into()), Some(503));
        let (response, _) = world.raw(&request);
        assert_eq!(list_reply(&response), (503, b"".as_slice()));
        world.service.set_list(Some(LIST_BODY.into()), None);
        let (response, _) = world.raw(&request);
        assert_eq!(list_reply(&response), (200, LIST_BODY));
        let replacement = b"second world's bytes".as_slice();
        world.service.set_list(Some(replacement.into()), None);
        let (response, _) = world.raw(&request);
        assert_eq!(list_reply(&response), (200, replacement));
        world.service.set_list(None, None);
        let (response, _) = world.raw(&request);
        assert_eq!(list_reply(&response), (404, b"".as_slice()));
        assert_eq!(world.service.count(), 0, "list reads spend no calls");
        assert!(
            world.accept(Credential::key(b"late".as_slice().into())).is_err(),
            "credentials close at the first request, including a list"
        );
        world.start();
        world.run();
        let completions: Vec<_> = world
            .seen
            .iter()
            .filter_map(|event| match event {
                skein_llm::client::Event::Completed { completion, .. } => Some(completion),
                _ => None,
            })
            .collect();
        assert_eq!(completions.len(), 1, "the first call completes once");
        let content = &completions[0].content;
        assert!(
            matches!(content.as_ref(), [skein_llm::Block::Text { text, .. }] if text.as_ref() == FIRST_CALL),
            "the list leaves the first script answer intact"
        );
        assert_eq!(world.service.count(), 1, "only inference counts a call");
        world.request(skein_llm::client::Request::Close);
        world.settle();
    }
}
