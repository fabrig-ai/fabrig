//! Runtime forge admission (forge-connection.md, sections 3, 6.1 and 7.2).
use super::fixtures;
use crate::{Access, Answered, Event, Request, Unusable, settings::Forge, start::Start};
use skein_forge::api;
use skein_lib::{Duration, List, Token};

fn forge() -> Forge {
    fixtures::two_forges().forges.into_boxed().into_iter().last().expect("second forge")
}
fn access() -> Access {
    forge().access.expect("fixture access")
}
fn added(h: &mut fixtures::Harness, forge: Forge, refused: Option<Unusable>) {
    let number = forge.number;
    h.send(Request::Add { forge });
    match h.up.pop().expect("Add's answer") {
        Event::Added { forge, refused: heard } => assert_eq!((forge, heard), (number, refused)),
        other @ (Event::Answered(_)
        | Event::Installation { .. }
        | Event::Fact { .. }
        | Event::GitToken { .. }
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed) => panic!("Add's answer: {other:?}"),
    }
    assert!(h.up.is_empty(), "one Add answer");
}
fn register(h: &mut fixtures::Harness, forge: &Forge) {
    let mut endpoints = List::with_capacity(h.env.limits.forges);
    for endpoint in core::mem::replace(&mut h.endpoints, List::with_capacity(0)).into_boxed() {
        endpoints.push(endpoint).expect("held endpoints");
    }
    endpoints.push((forge.address, forge.number)).expect("added endpoint");
    h.endpoints = endpoints;
}
fn ready(h: &mut fixtures::Harness, number: u16) {
    h.run(Duration::ZERO);
    h.connected(number);
    h.answer(number, &fixtures::user_ok());
    h.answer(number, &fixtures::settings_ok());
    h.run(Duration::ZERO);
    assert_eq!(h.start(number), Start::Ready { cap: 2 });
}
fn branch(h: &mut fixtures::Harness, call: u64, repository: api::Repository) {
    h.call(call, repository, api::Op::Read(api::Read::Branch { branch: Box::from(fixtures::BRANCH) }));
    h.run(Duration::ZERO);
    h.connected(repository.forge);
    fixtures::branch_answers(h, repository.forge);
    assert_eq!(
        h.answered(call),
        Some(&Answered {
            call: Token::new(call),
            cost: 2,
            free: 0,
            rate: None,
            result: Ok(api::Answer::Commit(fixtures::BRANCH_HEAD)),
        })
    );
}
#[test]
fn an_added_forge_serves_calls_once_connected() {
    // The owner's initial list need only fit its initial set, not the runtime limit.
    let mut settings = fixtures::settings();
    let mut initial = List::with_capacity(1);
    initial.push(settings.forges.into_boxed().into_iter().next().expect("forge 1")).expect("one forge");
    settings.forges = initial;
    let mut h = fixtures::Harness::new(settings);
    ready(&mut h, fixtures::FORGE);
    let mut forge = forge();
    forge.access = None;
    register(&mut h, &forge);
    let opened = h.connections_opened();
    added(&mut h, forge, None);
    assert_eq!(h.start(fixtures::OTHER_FORGE), Start::Unconnected);
    h.call(1, fixtures::OTHER_REPO, api::Op::Read(api::Read::Settings));
    match h.up.pop().expect("immediate answer") {
        Event::Answered(answer) => assert_eq!(
            answer,
            Answered { call: Token::new(1), cost: 0, free: 0, rate: None, result: Err(api::Error::NotConnected) }
        ),
        other @ (Event::Added { .. }
        | Event::Installation { .. }
        | Event::Fact { .. }
        | Event::GitToken { .. }
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed) => panic!("NotConnected: {other:?}"),
    }
    assert_eq!(h.connections_opened(), opened);
    h.send(Request::Connect { forge: fixtures::OTHER_FORGE, access: access() });
    ready(&mut h, fixtures::OTHER_FORGE);
    branch(&mut h, 2, fixtures::OTHER_REPO);
    branch(&mut h, 3, fixtures::REPO);
}
#[test]
fn an_add_past_the_limit_or_of_a_held_number_is_refused() {
    let limits = crate::Limits { forges: 1, ..fixtures::limits() };
    let mut h = fixtures::Harness::with(limits, fixtures::shared(), fixtures::settings());
    ready(&mut h, fixtures::FORGE);
    added(&mut h, forge(), Some(Unusable::Forges { forges: 1 }));
    let mut duplicate = forge();
    duplicate.number = fixtures::FORGE;
    duplicate.access = None;
    added(&mut h, duplicate, Some(Unusable::Duplicate { forge: fixtures::FORGE }));
    assert_eq!(h.component.start(fixtures::OTHER_FORGE), None);
    assert_eq!(h.start(fixtures::FORGE), Start::Ready { cap: 2 });
    branch(&mut h, 1, fixtures::REPO);
}
#[test]
fn an_added_forge_with_access_starts_at_once() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    ready(&mut h, fixtures::FORGE);
    let forge = forge();
    register(&mut h, &forge);
    added(&mut h, forge, None);
    assert_eq!(h.start(fixtures::OTHER_FORGE), Start::Checking { next: h.env.limits.start_backoff });
    ready(&mut h, fixtures::OTHER_FORGE);
    assert_eq!(h.connected_count, 2);
    branch(&mut h, 1, fixtures::OTHER_REPO);
}
#[test]
fn closing_and_aborted_components_ignore_add() {
    for request in [Request::Close, Request::Abort] {
        let mut h = fixtures::Harness::new(fixtures::settings());
        h.send(request);
        let before = h.up.len();
        h.send(Request::Add { forge: forge() });
        assert_eq!(h.up.len(), before);
        assert_eq!(h.component.start(fixtures::OTHER_FORGE), None);
        h.run(Duration::ZERO);
        assert!(h.closed);
        h.send(Request::Add { forge: forge() });
        assert!(h.up.is_empty());
        assert_eq!(h.component.start(fixtures::OTHER_FORGE), None);
    }
}

type Listening = (
    crate::Component,
    skein_lib::Env<crate::Limits>,
    skein_lib::Queue<Event>,
    skein_lib::Queue<skein_io::Request>,
    Token,
);
fn listening() -> Listening {
    use skein_lib::{Env, Queue, Time, Wall};
    let mut settings = fixtures::hook_settings();
    settings.forges.get_mut(0).expect("forge 1").access = None;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::limits() };
    let mut component = crate::Component::new(settings, &env.limits, &fixtures::shared()).expect("hook settings");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    component.begin(&env, &mut up, &mut io);
    let (listener, address) = match io.pop().expect("listener") {
        skein_io::Request::Listen { owner, addr } => (owner, addr),
        other @ (skein_io::Request::Connect { .. }
        | skein_io::Request::Bind { .. }
        | skein_io::Request::Reject { .. }
        | skein_io::Request::Stream { .. }
        | skein_io::Request::Output { .. }
        | skein_io::Request::Spawn { .. }
        | skein_io::Request::Signal { .. }
        | skein_io::Request::Usage { .. }
        | skein_io::Request::Window { .. }
        | skein_io::Request::Close { .. }
        | skein_io::Request::Abort { .. }) => panic!("listener: {other:?}"),
    };
    component.up(
        &env,
        skein_io::Event::Listening { owner: listener, listener: Token::new(90), addr: address },
        &mut up,
        &mut io,
    );
    match up.pop() {
        Some(Event::Listening { .. }) => {}
        other => panic!("listener bound: {other:?}"),
    }
    (component, env, up, io, listener)
}
#[test]
fn an_added_forges_hooks_are_heard() {
    use skein_lib::Secret;
    let (mut component, env, mut up, mut io, listener) = listening();
    let address = skein_io::Addr::Ip(([127, 0, 0, 1], 7200).into());
    let mut forge = forge();
    forge.access = None;
    component.down(&env, Request::Add { forge }, &mut up, &mut io);
    match up.pop() {
        Some(Event::Added { forge, refused }) => assert_eq!((forge, refused), (2, None)),
        other => panic!("forge added: {other:?}"),
    }
    let mut access = access();
    access.webhook_secret = Some(Secret::new(Box::from(fixtures::HOOK_SECRET)));
    component.down(&env, Request::Connect { forge: fixtures::OTHER_FORGE, access }, &mut up, &mut io);
    match io.pop() {
        Some(skein_io::Request::Connect { .. }) => {}
        other => panic!("added forge starts at Connect: {other:?}"),
    }
    let socket = Token::new(91);
    component.up(&env, skein_io::Event::Accepted { owner: listener, socket, peer: address }, &mut up, &mut io);
    let hook = match io.pop().expect("bind hook") {
        skein_io::Request::Bind { socket: bound, owner } => {
            assert_eq!(bound, socket);
            owner
        }
        other @ (skein_io::Request::Listen { .. }
        | skein_io::Request::Connect { .. }
        | skein_io::Request::Reject { .. }
        | skein_io::Request::Stream { .. }
        | skein_io::Request::Output { .. }
        | skein_io::Request::Spawn { .. }
        | skein_io::Request::Signal { .. }
        | skein_io::Request::Usage { .. }
        | skein_io::Request::Window { .. }
        | skein_io::Request::Close { .. }
        | skein_io::Request::Abort { .. }) => panic!("bind hook: {other:?}"),
    };
    let mut wire = skein_world::stream::Wire::new(1);
    let mut head = fixtures::signed_head(fixtures::PULL_PAYLOAD, Some(b"pull_request"));
    assert!(head.starts_with(b"POST /hooks/1 "));
    *head.get_mut(b"POST /hooks/".len()).expect("forge number") = b'2';
    wire.write(&head);
    wire.write(fixtures::PULL_PAYLOAD);
    let mut hints = 0_u32;
    for _ in 0_u32..4096 {
        for _ in 0..io.len() {
            match io.pop().expect("hook stream") {
                skein_io::Request::Stream { stream, down } => {
                    assert_eq!(stream, socket);
                    wire.take(down);
                }
                other @ (skein_io::Request::Listen { .. }
                | skein_io::Request::Connect { .. }
                | skein_io::Request::Bind { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. }
                | skein_io::Request::Close { .. }
                | skein_io::Request::Abort { .. }) => panic!("hook stream: {other:?}"),
            }
        }
        for _ in 0..up.len() {
            match up.pop().expect("hook hint") {
                Event::Hint(hinted) => {
                    assert_eq!(hinted.hint.repository, api::Repository { forge: 2, repository: 2 });
                    assert_eq!(hinted.hint.change, api::Change::Item(7));
                    hints += 1;
                }
                other @ (Event::Added { .. }
                | Event::Answered(_)
                | Event::Installation { .. }
                | Event::Fact { .. }
                | Event::GitToken { .. }
                | Event::Listening { .. }
                | Event::ListenerLost
                | Event::Connected { .. }
                | Event::Stopped { .. }
                | Event::Closed) => panic!("hook hint: {other:?}"),
            }
        }
        if let Some(event) = wire.answer() {
            component.up(&env, skein_io::Event::Stream { owner: hook, up: event }, &mut up, &mut io);
        }
    }
    assert!(wire.received.starts_with(b"HTTP/1.1 204 "));
    assert_eq!(hints, 1, "verified delivery to the added path");
}

#[test]
fn a_new_dialect_gets_its_tables_and_github_its_own_connection() {
    use super::github_start;
    let limits = crate::Limits { connections: 1, ..fixtures::limits() };
    let shared = crate::Shared { calls: 1, ..fixtures::shared() };
    let mut h = fixtures::Harness::with(limits, shared, fixtures::settings());
    ready(&mut h, fixtures::FORGE);
    let mut forge = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
    forge.number = fixtures::OTHER_FORGE;
    forge.address = self::forge().address;
    register(&mut h, &forge);
    h.git_tokens = Some(List::with_capacity(8));
    h.installation_events = Some(List::with_capacity(8));
    h.facts = Some(List::with_capacity(8));
    added(&mut h, forge, None);
    h.send(Request::Installations { forge: fixtures::OTHER_FORGE, accepted: Box::new([1]) });
    h.run(Duration::ZERO);
    assert_eq!(h.connections_opened(), 2, "GitHub start has its own slot while Forgejo's is held");
    h.connected(fixtures::OTHER_FORGE);
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(github_start::LIVE));
    h.answer(fixtures::OTHER_FORGE, &github_start::mint(github_start::FULL, github_start::EXPIRES));
    h.answer(fixtures::OTHER_FORGE, &github_start::mint(github_start::CONTENTS, github_start::EXPIRES));
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(github_start::VIEWER));
    h.run(Duration::ZERO);
    assert_eq!(h.start(fixtures::OTHER_FORGE), Start::Ready { cap: 100 });
    assert_eq!(h.connected_count, 2);
    assert_eq!(h.git_tokens.as_ref().expect("git tokens").len(), 1);
    // An accepted unsuspended installation is unchanged in the initial baseline.
    assert!(h.installation_events.as_ref().expect("installations").is_empty());
    h.run(limits.idle);
    fixtures::github_repository(&mut h, fixtures::OTHER_REPO, 22);
    h.call(1, fixtures::OTHER_REPO, api::Op::Read(api::Read::Branch { branch: Box::from(fixtures::BRANCH) }));
    h.run(Duration::ZERO);
    h.connected(fixtures::OTHER_FORGE);
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(fixtures::GITHUB_BRANCH));
    h.run(Duration::ZERO);
    assert_eq!(
        (h.answered(1).expect("GitHub call").cost, &h.answered(1).expect("GitHub call").result),
        (1, &Ok(api::Answer::Commit(fixtures::BRANCH_HEAD)))
    );

    let mut settings = fixtures::github_settings();
    settings.forges.get_mut(0).expect("GitHub forge").access = None;
    let mut h = fixtures::Harness::new(settings);
    let forge = self::forge();
    register(&mut h, &forge);
    added(&mut h, forge, None);
    ready(&mut h, fixtures::OTHER_FORGE);
    branch(&mut h, 1, fixtures::OTHER_REPO);
}

fn origin() -> crate::settings::LogOrigin {
    crate::settings::LogOrigin {
        origin: skein_forge::github::redirect::Origin {
            tls: false,
            host: Box::from(b"logs.test".as_slice()),
            port: 3002,
        },
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3002).into()),
    }
}
fn origins(count: u32) -> List<crate::settings::LogOrigin> {
    let mut origins = List::with_capacity(count);
    for _ in 0..count {
        origins.push(origin()).expect("origin room");
    }
    origins
}
#[test]
fn added_forges_obey_starts_address_access_and_origin_checks() {
    use crate::settings::Transport;
    use skein_lib::Secret;
    let number = fixtures::OTHER_FORGE;
    let invalid_token = Access {
        credential: skein_forge::credential::Credential::Token(Secret::new(Box::from(b"bad\rtoken".as_slice()))),
        ..access()
    };
    let mut h = fixtures::Harness::new(fixtures::settings());
    ready(&mut h, fixtures::FORGE);
    let opened = h.connections_opened();
    for (forge, expected) in [
        (
            Forge { address: skein_io::Addr::InProcess(skein_io::Name::EMPTY), ..forge() },
            Unusable::Address { forge: number },
        ),
        (
            Forge { address: skein_io::Addr::Ip(([192, 0, 2, 1], 3001).into()), ..forge() },
            Unusable::Address { forge: number },
        ),
        (
            Forge { transport: Transport::Plaintext { name: Some(Box::from(b"bad/name".as_slice())) }, ..forge() },
            Unusable::PlaintextName { forge: number },
        ),
        (Forge { access: Some(invalid_token), ..forge() }, Unusable::Token { forge: number }),
        (Forge { access: Some(Access { user: 0, ..access() }), ..forge() }, Unusable::User { forge: number }),
        (
            Forge {
                access: Some(Access {
                    webhook_secret: Some(Secret::new(Box::from(fixtures::HOOK_SECRET))),
                    ..access()
                }),
                ..forge()
            },
            Unusable::Secret { forge: number },
        ),
        (Forge { log_origins: origins(1), ..forge() }, Unusable::Origin { forge: number, origin: origin().origin }),
        (Forge { log_origins: origins(3), ..forge() }, Unusable::LogOrigins { forge: number, origins: 3, limit: 2 }),
        (
            Forge { transport: Transport::Plaintext { name: Some(fixtures::filled(40_000, b'a')) }, ..forge() },
            Unusable::Request { forge: number },
        ),
    ] {
        // Start and Add name the same refused value; neither may retain this forge.
        let mut start_settings = fixtures::settings();
        start_settings.forges.push(forge).expect("two forge settings");
        assert_eq!(crate::limits::check(&h.env.limits, &fixtures::shared(), &start_settings), Err(expected.clone()));
        let forge = start_settings.forges.into_boxed().into_iter().last().expect("candidate");
        added(&mut h, forge, Some(expected));
        assert_eq!(h.component.start(number), None);
        assert_eq!(h.start(fixtures::FORGE), Start::Ready { cap: 2 });
        assert!(h.io.is_empty(), "refused Add requests no io");
    }
    assert_eq!(h.connections_opened(), opened);
    // Refusals consume no capacity or forge number.
    let forge = forge();
    register(&mut h, &forge);
    added(&mut h, forge, None);
    ready(&mut h, number);
    branch(&mut h, 1, fixtures::REPO);
}
#[test]
fn adding_the_first_github_checks_its_limits_and_origins() {
    let limits = crate::Limits { key_bytes: 1, ..fixtures::limits() };
    let mut h = fixtures::Harness::with(limits, fixtures::shared(), fixtures::settings());
    let mut github = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
    github.number = fixtures::OTHER_FORGE;
    github.access = None;
    added(&mut h, github, Some(Unusable::KeyBytes { key_bytes: 1 }));
    assert_eq!(h.component.start(fixtures::OTHER_FORGE), None);
    let mut h = fixtures::Harness::new(fixtures::settings());
    ready(&mut h, fixtures::FORGE);
    for invalid in [
        crate::settings::LogOrigin {
            origin: skein_forge::github::redirect::Origin { tls: true, ..origin().origin },
            ..origin()
        },
        crate::settings::LogOrigin { address: skein_io::Addr::InProcess(skein_io::Name::EMPTY), ..origin() },
        crate::settings::LogOrigin { address: skein_io::Addr::Ip(([192, 0, 2, 1], 3002).into()), ..origin() },
        crate::settings::LogOrigin {
            origin: skein_forge::github::redirect::Origin {
                host: Box::from(b"127.0.0.1".as_slice()),
                ..origin().origin
            },
            ..origin()
        },
    ] {
        let expected = Unusable::Origin { forge: fixtures::OTHER_FORGE, origin: invalid.origin.clone() };
        let mut github = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
        github.number = fixtures::OTHER_FORGE;
        github.access = None;
        github.log_origins = List::with_capacity(1);
        github.log_origins.push(invalid).expect("one origin");
        added(&mut h, github, Some(expected));
        assert_eq!(h.component.start(fixtures::OTHER_FORGE), None);
        assert!(h.io.is_empty());
    }
}

#[test]
fn adding_github_refuses_unrepresentable_reserved_socket_slots() {
    let limits = crate::Limits { connections: u32::MAX, ..fixtures::limits() };
    let shared = crate::Shared { calls: u32::MAX, ..fixtures::shared() };
    let mut settings = fixtures::settings();
    settings.forges.get_mut(0).expect("initial forge").access = None;
    let mut h = fixtures::Harness::with(limits, shared, settings);
    let mut github = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
    github.number = fixtures::OTHER_FORGE;
    github.access = None;
    added(&mut h, github, Some(Unusable::Zero));
    assert_eq!(h.component.start(fixtures::OTHER_FORGE), None);
    assert!(h.io.is_empty());
    let mut forge = forge();
    forge.access = None;
    added(&mut h, forge, None);
    assert_eq!(h.start(fixtures::OTHER_FORGE), Start::Unconnected);
    assert!(h.io.is_empty());
}

#[test]
fn github_reservations_include_forges_already_held() {
    for (connections, refused) in [(u32::MAX - 2, None), (u32::MAX - 1, Some(Unusable::Zero))] {
        let limits = crate::Limits { connections, ..fixtures::limits() };
        let shared = crate::Shared { calls: connections, ..fixtures::shared() };
        let mut settings = fixtures::github_settings();
        settings.forges.get_mut(0).expect("initial GitHub forge").access = None;
        let mut h = fixtures::Harness::with(limits, shared, settings);
        let mut github = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge");
        github.number = fixtures::OTHER_FORGE;
        github.access = None;
        let state = if refused.is_none() { Some(Start::Unconnected) } else { None };
        added(&mut h, github, refused);
        assert_eq!(h.component.start(fixtures::OTHER_FORGE), state);
        assert!(h.io.is_empty());
    }
}
