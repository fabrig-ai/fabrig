use crate::clients::translation;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;

const LISTENER: Token = Token::new(77);
const STARTING_LISTENER: Token = Token::new(99);
const REPLY: ReplyTo = ReplyTo::new(Token::new(9));
const PROCESS_UPTIME: skein_lib::Time = skein_lib::Time::from_nanos(600_000_000_000);
// The fixture's first configured person is the deployment's owner.
const OWNER_PARTY: u64 = 1;

fn given_secret() -> skein_lib::Secret {
    skein_lib::Secret::new(Box::new([23; crate::clients::LINK_SECRET_BYTES]))
}

fn with_given_secret() -> crate::Service {
    with_secrets(None)
}

fn with_given_secrets() -> crate::Service {
    with_secrets(Some(skein_lib::Secret::new(Box::new([37; crate::clients::LINK_SECRET_BYTES]))))
}

fn with_secrets(local_secret: Option<skein_lib::Secret>) -> crate::Service {
    let mut service = super::fixtures::started();
    let settings = super::fixtures::engine();
    let mut seeds = super::fixtures::seeds();
    seeds.link_secret = Some(given_secret());
    seeds.local_secret = local_secret;
    service.engine.as_mut().expect("engine").clients = crate::clients::Clients::new(
        &jig_protocol_config::Directories::working(),
        &settings.jig,
        &settings.clients,
        &seeds,
        &settings.limits.root.journal,
        None,
        None,
        &[],
    )
    .expect("clients with a given secret");
    super::fixtures::restart(&mut service);
    service
}

fn sign_in_with_link(engine: &mut crate::engine::Engine, link: &[u8]) -> jig_client_protocol_wire::v21::SignInAnswer {
    let body = peer::link_body(engine, link);
    let mut peer = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine, true);
    peer.sign_in_answer(engine)
}

fn given_link() -> Box<[u8]> {
    crate::clients::first_link_url(([127, 0, 0, 1], 7100).into(), None, &given_secret())
}

fn local_link(service: &mut crate::Service) -> Box<[u8]> {
    super::fixtures::turn(service);
    service.local_links().pop().expect("the file's link")
}

fn link_route(link: &[u8]) -> &[u8] {
    let query = skein_lib::bytes::find(link, b"?").expect("link query");
    link.get(..query).expect("link route")
}

#[test]
fn a_given_secret_prints_no_first_link_and_signs_in() {
    let mut service = with_given_secret();
    let local = local_link(&mut service);
    assert!(local != given_link(), "the file never holds the given link");
    assert!(service.local_links().is_empty(), "one local link");
    assert!(service.links().is_empty(), "the box already holds the first link");
    let engine = service.engine.as_mut().expect("engine");
    let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) = sign_in_with_link(engine, &given_link()) else {
        panic!("the given secret signs in")
    };
    assert_eq!(answer.party(), OWNER_PARTY, "the link signs in the owner");
    assert!(answer.token().is_some(), "the sign-in returns a credential");
    super::fixtures::turn(&mut service);
    assert!(service.links().is_empty(), "spending the first link prints nothing");
    assert!(service.local_links().is_empty(), "spending the given link draws no local link");
}

#[test]
fn a_given_local_link_signs_in_and_is_never_kept_or_printed() {
    let mut service = with_given_secrets();
    super::fixtures::turn(&mut service);
    assert!(service.local_links().is_empty(), "the given local link is never kept");
    assert!(service.links().is_empty(), "the given printed link is never printed again");
    let local = crate::clients::first_link_url(
        ([127, 0, 0, 1], 7100).into(),
        None,
        &skein_lib::Secret::new(Box::new([37; crate::clients::LINK_SECRET_BYTES])),
    );
    let engine = service.engine.as_mut().expect("engine");
    let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) = sign_in_with_link(engine, &local) else {
        panic!("the given local link signs in")
    };
    assert_eq!(answer.party(), OWNER_PARTY, "the local link signs in the owner");
    assert!(answer.token().is_some(), "the sign-in returns a credential");
    assert!(!forges_linked(engine), "a local sign-in leaves the printed link unused");
    let next = local_link(&mut service);
    assert!(next != local && next != given_link(), "the next local link differs from both given links");
    assert!(service.local_links().is_empty(), "one successor is kept");
    assert!(service.links().is_empty(), "a local sign-in prints nothing");
    let engine = service.engine.as_mut().expect("engine");
    let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) = sign_in_with_link(engine, &given_link()) else {
        panic!("the given printed link still signs in")
    };
    assert_eq!(answer.party(), OWNER_PARTY);
    assert!(forges_linked(engine), "the printed link's sign-in is in the forges document");
    super::fixtures::turn(&mut service);
    assert!(service.links().is_empty(), "neither given link is printed");
    assert!(service.local_links().is_empty(), "the printed sign-in leaves the successor alone");
}

#[test]
fn a_spent_given_secret_prints_a_fresh_link() {
    let mut service = with_given_secret();
    let local = local_link(&mut service);
    let engine = service.engine.as_mut().expect("engine");
    let link = given_link();
    let _signed_in = sign_in_with_link(engine, &link);
    let answer = sign_in_with_link(engine, &link);
    assert!(answer == jig_client_protocol_wire::v21::SignInAnswer::LinkUsed, "a spent secret is used");
    super::fixtures::turn(&mut service);
    let fresh = service.links().pop().expect("a fresh link is printed");
    assert!(local != link && local != fresh, "the file never holds a printed link");
    assert!(service.local_links().is_empty(), "a fresh printed link draws no local link");
    assert!(fresh != link, "a later link has a fresh secret");
    assert!(service.links().is_empty(), "one fresh link is printed");
    let engine = service.engine.as_mut().expect("engine");
    let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) = sign_in_with_link(engine, &fresh) else {
        panic!("the fresh printed link signs in")
    };
    assert_eq!(answer.party(), OWNER_PARTY);
}

#[test]
fn the_local_link_is_kept_apart_and_signs_in_in_either_order() {
    for (row, local_first) in [("local first", true), ("given first", false)] {
        let mut service = with_given_secret();
        let local = local_link(&mut service);
        let given = given_link();
        assert!(local != given, "{row}: the file's link has its own secret");
        assert!(link_route(&local) == link_route(&given), "{row}: the address and route agree");
        assert!(service.local_links().is_empty(), "{row}: exactly one local link");
        assert!(service.links().is_empty(), "{row}: the given link is never printed again");
        assert!(!forges_linked(service.engine.as_ref().expect("engine")), "{row}: no printed sign-in at start");
        let mut linked = false;
        let order = if local_first { [(&local, true), (&given, false)] } else { [(&given, false), (&local, true)] };
        for (link, is_local) in order {
            let engine = service.engine.as_mut().expect("engine");
            let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) = sign_in_with_link(engine, link) else {
                panic!("{row}: either link signs in after the other")
            };
            assert_eq!(answer.party(), OWNER_PARTY, "{row}: the link signs in the owner");
            assert!(answer.token().is_some(), "{row}: the sign-in returns a credential");
            linked |= !is_local;
            assert_eq!(forges_linked(engine), linked, "{row}: only the printed sign-in sets linked");
            super::fixtures::turn(&mut service);
            if is_local {
                let next = service.local_links().pop().expect("the next local link");
                assert!(next != local && next != given, "{row}: spending the local link replaces only itself");
            }
            assert!(service.local_links().is_empty(), "{row}: only spending the local link draws its successor");
            assert!(service.links().is_empty(), "{row}: neither sign-in prints a link");
        }
    }
}

#[test]
fn first_link_url_is_what_serve_prints() {
    let public = crate::startup::application::Origin {
        scheme: crate::startup::application::Scheme::Https,
        host: Box::from(b"temper.example.com".as_slice()),
        port: Some(8443),
        name: Box::from(b"temper.example.com:8443".as_slice()),
    };
    let address = ([127, 0, 0, 1], 7100).into();
    let secret = given_secret();
    for origin in [None, Some(&public)] {
        let url = crate::clients::first_link_url(address, origin, &secret);
        assert!(url == crate::clients::printed::link_public(address, origin, &secret), "same URL bytes");
        assert!(!url.contains(&b'\n'), "the URL has no line feed");
    }
}

#[test]
fn a_link_printed_on_the_first_clock_tick_is_live() {
    let mut service = super::fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.clients.iterate(
        PROCESS_UPTIME,
        skein_lib::Wall::EPOCH,
        &mut engine.events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut engine.accounts,
    );
    let Some(skein_io::Request::Listen { owner, addr }) = engine.clients.io.pop() else { panic!("listen") };
    assert!(engine.clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
    engine.clients.iterate(
        PROCESS_UPTIME,
        skein_lib::Wall::EPOCH,
        &mut engine.events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut engine.accounts,
    );
    let link = engine.clients.links.pop().expect("printed link");
    let body = peer::link_body(engine, &link);
    let mut peer = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump_at(engine, false, PROCESS_UPTIME, skein_lib::Wall::EPOCH);
    assert_eq!(engine.clients.early.len(), 1, "the printed link reaches the root");
}
#[test]
fn a_chat_ask_reaches_the_root_as_a_party_ask() {
    let ask = core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: REPLY,
        sign_in: 3,
        key: [5; 16],
        ask: super::fixtures::chat(1, b"question"),
    });
    let Ok(root::PartyInput::Ask { models: _, reply_to, sign_in, key, ask }) = translation::to_root(ask) else {
        panic!("party ask")
    };
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
    assert_eq!(key, [5; 16]);
    assert_eq!(ask, super::fixtures::chat(1, b"question"));
}
#[test]
fn whose_preserves_its_reply_right_and_sign_in() {
    let event = core::Event::People(people::Event::Whose { reply_to: REPLY, sign_in: 3 });
    let Ok(root::PartyInput::Whose { reply_to, sign_in }) = translation::to_root(event) else { panic!("party whose") };
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
}
#[test]
fn sign_out_preserves_its_reply_right_and_sign_in() {
    let event = core::Event::People(people::Event::SignOut { reply_to: REPLY, sign_in: 3 });
    let Ok(root::PartyInput::SignOut { reply_to, sign_in }) = translation::to_root(event) else {
        panic!("party sign out")
    };
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
}
#[test]
fn not_ready_uses_the_clients_reply_vocabulary() {
    for (row, input, expected) in [
        (
            "whose",
            root::PartyInput::Whose { reply_to: REPLY, sign_in: 3 },
            people::Reply::Again(people::Again::NotReady),
        ),
        (
            "sign out",
            root::PartyInput::SignOut { reply_to: REPLY, sign_in: 3 },
            people::Reply::Again(people::Again::NotReady),
        ),
        (
            "ask",
            root::PartyInput::Ask {
                models: Box::new([]),
                reply_to: REPLY,
                sign_in: 3,
                key: [5; 16],
                ask: super::fixtures::chat(1, b"question"),
            },
            people::Reply::Again(people::Again::NotReady),
        ),
    ] {
        let Some(kit::Event::Reply { to, reply }) = translation::not_ready(input) else {
            panic!("not-ready reply {row}")
        };
        assert_eq!(to, REPLY, "{row}");
        assert_eq!(reply, expected, "{row}");
    }
}
#[test]
fn a_released_reply_keeps_its_terminal_and_destination() {
    let request = root::PartyRequest::Deliver(root::Delivery::WebReply { to: REPLY, reply: people::Reply::SignedOut });
    let Some(kit::Event::Reply { to, reply }) = translation::to_component(request) else { panic!("clients reply") };
    assert_eq!(to, REPLY);
    assert_eq!(reply, people::Reply::SignedOut);
}

#[test]
fn the_component_prints_one_sign_in_link_at_start() {
    let mut clients = super::fixtures::clients();
    let mut events = skein_lib::Queue::uncapped();
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    let Some(skein_io::Request::Listen { owner, addr }) = clients.io.pop() else { panic!("listen") };
    assert!(clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    let link = clients.links.pop().unwrap();
    assert!(link.starts_with(b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret="));
    assert!(clients.links.is_empty());
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    assert!(clients.links.is_empty());
}
#[test]
fn closing_a_listener_during_start_closes_the_component() {
    let mut clients = super::fixtures::clients();
    let mut events = skein_lib::Queue::uncapped();
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    let Some(skein_io::Request::Listen { owner, addr }) = clients.io.pop() else { panic!("listen") };
    let Some(kit::SecretsRequest::Get { owner: kit::SecretOwner::PublicNames }) = clients.secrets.pop() else {
        panic!("startup public names lookup")
    };
    clients.down(kit::Event::Secrets(kit::SecretsAnswer::Got {
        owner: kit::SecretOwner::PublicNames,
        value: Some(kit::Kept {
            digest: skein_lib::Secret::new(Box::new([5; 32])),
            expires: skein_lib::Wall::from_nanos(u64::MAX),
        }),
    }));
    clients.down(kit::Event::Close);
    assert!(clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    let Some(skein_io::Request::Close { entity }) = clients.io.pop() else { panic!("late listener close") };
    assert_eq!(entity, LISTENER);
    assert!(clients.up(skein_io::Event::Closed { owner }));
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    assert!(clients.closed);
}

#[test]
fn a_sign_in_digest_and_expiry_are_kept_in_secrets() {
    let mut clients = super::fixtures::clients();
    let mut store = super::fixtures::marked_store();
    clients.secrets.push(kit::SecretsRequest::Keep {
        owner: kit::SecretOwner::SignIn(3),
        value: kit::Kept {
            digest: skein_lib::Secret::new(Box::new([5; 32])),
            expires: skein_lib::Wall::from_nanos(12),
        },
    });
    clients.secrets_step(&mut store);
    store.iterate();
    let Some(jig_protocol_store::Event::Kept { owner }) = store.host.above.pop() else { panic!("digest kept") };
    assert_eq!(owner, Token::new(crate::tokens::CLIENTS + 1));
    let Some(jig_protocol_store::Event::Secret {
        value: Some(jig_protocol_store::SecretValue::Digest { expiry, digest }),
        ..
    }) = store.secrets(
        Token::new(9),
        jig_protocol_store::Secrets::Get { owner: jig_protocol_store::SecretOwner::SignIn, number: 3 },
    )
    else {
        panic!("secret read")
    };
    assert_eq!(expiry, 12);
    assert_eq!(digest, [5; 32]);
}
#[test]
fn a_sign_in_ending_waits_for_its_erase_to_commit() {
    let config = super::fixtures::engine().jig;
    let mut journal = super::fixtures::engine().limits.root.journal;
    journal.commits = 1;
    let mut clients = crate::clients::Clients::new(
        &jig_protocol_config::Directories::working(),
        &config,
        &super::fixtures::engine().clients,
        &super::fixtures::seeds(),
        &journal,
        None,
        None,
        &[],
    )
    .unwrap();
    clients.commit(3, &[root::Write::Erase(root::Key::Core(core::Key::People(people::Key::SignIn(7))))]);
    assert!(!clients.commit_room());
    let mut events = skein_lib::Queue::uncapped();
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    assert!(!clients.commit_room());
    clients.committed(3);
    clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut None,
    );
    assert!(clients.commit_room());
}

use super::fixtures::http as peer;
#[test]
fn the_next_clients_stream_opens_once_a_peer_ends_its_own() {
    // The small profile admits two streams; a third needs a predecessor freed.
    const CLIENTS: u32 = 3;
    let mut service = super::fixtures::ready();
    let link = service.links().pop().unwrap();
    let body = peer::link_body(service.engine.as_ref().expect("engine"), &link);
    let mut sign_in =
        peer::Peer::new(service.engine.as_mut().expect("engine"), b"POST", b"/jig/v1/sign-in/link", &body, None);
    sign_in.pump(service.engine.as_mut().expect("engine"), true);
    let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) =
        sign_in.sign_in_answer(service.engine.as_ref().expect("engine"))
    else {
        panic!("signed in")
    };
    let bearer = peer::bearer(answer.token().as_ref().expect("native credential").expose());
    let mut third_opened = false;
    for _ in 0..CLIENTS {
        let mut stream =
            peer::Peer::new(service.engine.as_mut().expect("engine"), b"GET", b"/jig/v1/stream", &[], Some(&bearer));
        third_opened = stream.read_hello(service.engine.as_mut().expect("engine"));
        stream.end(service.engine.as_mut().expect("engine"));
    }
    assert!(third_opened, "the third client receives its hello before a heartbeat");
}
#[test]
fn the_engine_prints_one_sign_in_link_at_start() {
    let mut service = super::fixtures::ready();
    let link = service.links().pop().expect("sign-in link");
    assert!(link.starts_with(b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret="));
    assert!(service.links().is_empty());
    super::fixtures::turn(&mut service);
    assert!(service.links().is_empty());
}
#[test]
fn a_link_sign_in_reaches_the_root_and_answers_a_token() {
    let mut service = super::fixtures::ready();
    let link = service.links().pop().unwrap();
    let body = peer::link_body(service.engine.as_ref().expect("engine"), &link);
    let mut peer =
        peer::Peer::new(service.engine.as_mut().expect("engine"), b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(service.engine.as_mut().expect("engine"), true);
    let jig_client_protocol_wire::v21::SignInAnswer::SignedIn(answer) =
        peer.sign_in_answer(service.engine.as_ref().expect("engine"))
    else {
        panic!("signed in")
    };
    assert_eq!(answer.party(), 1);
    assert!(super::fixtures::get(&mut service, core::Key::People(people::Key::SignIn(1))).is_some());
    let engine = service.engine.as_mut().expect("engine");
    let Some(jig_protocol_store::Event::Secret { value: Some(secret), .. }) = engine.store.secrets(
        Token::new(1),
        jig_protocol_store::Secrets::Get { owner: jig_protocol_store::SecretOwner::SignIn, number: 1 },
    ) else {
        panic!("digest durable in real secrets store")
    };
    let jig_protocol_store::SecretValue::Digest { digest, .. } = secret else { panic!("credential digest") };
    assert_eq!(digest, skein_lib::sha256::digest(answer.token().as_ref().expect("credential").expose()));
    let bearer = peer::bearer(answer.token().as_ref().expect("native credential").expose());
    let mut stream = peer::Peer::new(engine, b"GET", b"/jig/v1/stream", &[], Some(&bearer));
    stream.pump(engine, true);
    assert!(stream.streaming(), "the issued bearer authenticates through the stored digest");
    assert_eq!(engine.end, None);
}
#[test]
fn a_request_before_decisions_open_is_answered_not_ready() {
    let mut service = super::fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut engine.events,
        &engine.workers.forges,
        engine.demo.as_ref(),
        &mut engine.accounts,
    );
    let Some(skein_io::Request::Listen { owner, addr }) = engine.clients.io.pop() else { panic!("listener") };
    assert!(engine.clients.up(skein_io::Event::Listening { owner, listener: STARTING_LISTENER, addr }));
    engine.clients.iterate(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        &mut engine.events,
        &engine.workers.forges,
        engine.demo.as_ref(),
        &mut engine.accounts,
    );
    let link = engine.clients.links.pop().unwrap();
    let body = peer::link_body(engine, &link);
    let mut peer = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine, false);
    assert_eq!(engine.clients.early.len(), 1, "request staged during restoration");
    crate::engine::iterate(engine, skein_lib::Time::ZERO, skein_lib::Wall::EPOCH);
    peer.pump(engine, false);
    assert_eq!(peer.sign_in_answer(engine), jig_client_protocol_wire::v21::SignInAnswer::NotReady);
    assert!(!engine.domain.ready());
}
#[test]
fn a_clients_listener_failure_stops_the_engine() {
    let mut service = super::fixtures::started();
    service.process.told().push(skein_io::Event::Closed { owner: Token::new(crate::tokens::SERVER) });
    super::fixtures::turn(&mut service);
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::Listener {
            listener: crate::Listening::Clients,
            address: Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
            error: None
        }))
    );
}

#[test]
fn a_failed_clients_bind_leaves_the_service_quiet() {
    failed_clients_bind(true, false);
}

#[test]
fn a_failed_clients_bind_before_store_open_leaves_the_service_quiet() {
    failed_clients_bind(false, false);
}

#[test]
fn a_closed_clients_bind_leaves_the_service_quiet() {
    failed_clients_bind(true, true);
}

fn failed_clients_bind(opened: bool, closed: bool) {
    use skein_lib::{Time, Wall};
    let mut service = super::fixtures::started();
    let engine = service.engine.as_mut().expect("engine");
    engine.store.opened = opened;
    engine.clients.iterate(
        Time::ZERO,
        Wall::EPOCH,
        &mut engine.events,
        &crate::tests::fixtures::forge_table(&[]),
        None,
        &mut engine.accounts,
    );
    let Some(skein_io::Request::Listen { owner, .. }) = engine.clients.io.pop() else { panic!("clients listen") };
    service.process.told().push(if closed {
        skein_io::Event::Closed { owner }
    } else {
        skein_io::Event::Failed { owner, error: skein_io::Error::Refused }
    });
    for _ in 0_u32..64 {
        super::fixtures::turn(&mut service);
        if service.is_empty() {
            break;
        }
    }
    assert_eq!(
        service.end(),
        Some(&crate::End::Stopped(crate::Stop::Listener {
            listener: crate::Listening::Clients,
            address: Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
            error: if closed { None } else { Some(skein_io::Error::Refused) }
        }))
    );
    assert!(service.is_empty(), "every child and kernel route closes");
    assert!(!service.work_pending(Time::from_nanos(1)), "the loop needs no further turn");
    assert_eq!(service.next_deadline(), None, "no timer remains armed");
}

#[test]
fn original_model_names_reach_the_root() {
    let mut ask = super::fixtures::chat(1, b"question");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("fixture creates a chat") };
    batch[0].authority.models = Some(people::Models::Named(1));
    let event = core::Event::People(people::Event::Ask {
        reply_to: REPLY,
        sign_in: 3,
        key: [5; 16],
        ask,
        models: Box::new([Box::from(b"small".as_slice())]),
    });
    let Ok(root::PartyInput::Ask { models, .. }) = translation::to_root(event) else { panic!("party ask") };
    assert_eq!(models.as_ref(), &[Box::<[u8]>::from(b"small".as_slice())]);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary test formats an HTTP request")]
fn clients_public_admits_the_public_and_listener_hosts_and_refuses_an_unknown_host() {
    for (host, status) in [
        ("temper.example.com:8443", "HTTP/1.1 404"),
        ("127.0.0.1:7100", "HTTP/1.1 404"),
        ("unknown.example.com", "HTTP/1.1 421"),
    ] {
        let mut config = super::fixtures::engine();
        config.clients.public = Some(crate::startup::application::Origin {
            scheme: crate::startup::application::Scheme::Https,
            host: Box::from(b"temper.example.com".as_slice()),
            port: Some(8443),
            name: Box::from(b"temper.example.com:8443".as_slice()),
        });
        let mut service = super::fixtures::started();
        let engine = service.engine.as_mut().unwrap();
        engine.clients = crate::clients::Clients::new(
            &jig_protocol_config::Directories::working(),
            &config.jig,
            &config.clients,
            &super::fixtures::seeds(),
            &config.limits.root.journal,
            None,
            None,
            &[],
        )
        .unwrap();
        engine.clients.iterate(
            skein_lib::Time::ZERO,
            skein_lib::Wall::EPOCH,
            &mut engine.events,
            &crate::tests::fixtures::forge_table(&[]),
            None,
            &mut engine.accounts,
        );
        let Some(skein_io::Request::Listen { owner, addr }) = engine.clients.io.pop() else { panic!("listen") };
        assert!(engine.clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
        let request = format!("GET /jig/unknown HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        let mut peer = peer::Peer::raw(engine, request.as_bytes());
        peer.pump(engine, false);
        assert!(peer.received().starts_with(status.as_bytes()), "{host}: {:?}", peer.received());
    }
}

#[test]
fn a_moved_file_at_its_forms_bound_passes_and_one_byte_past_is_refused() {
    use jig_client_protocol_wire::v21 as w;
    use skein_lib::{Reader, Secret, Writer, bytes};

    let mut service = super::fixtures::ready();
    let link = service.links().pop().expect("sign-in link");
    let engine = service.engine.as_mut().expect("engine");
    let body = peer::link_body(engine, &link);
    let mut login = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    login.pump(engine, true);
    let w::SignInAnswer::SignedIn(answer) = login.sign_in_answer(engine) else { panic!("signed in") };
    let bearer = peer::bearer(answer.token().as_ref().expect("credential").expose());
    let settings = super::fixtures::engine();
    let bound = jig_protocol_accounts::moved::bound(&settings.jig.limits.driver.client.document);
    assert_eq!(bound, 68_252);
    assert_eq!(u64::from(engine.clients.env.limits.wire.keyed_value_some), bound);
    for extra in 0_u8..2 {
        let value = Secret::new(bytes::zeroed(usize::try_from(bound + u64::from(extra)).unwrap()));
        let digest = skein_lib::sha256::digest(value.expose());
        // Encode under the published ceiling to put the extra byte on the wire;
        // the service decodes under temper's configured, smaller value cap.
        let limits = w::CEILINGS;
        let ask = w::Ask::AccountChange(
            w::AskAccountChange::new(
                &limits,
                w::AskAccountChangeParts {
                    holder: w::AskAccountHolder::Own,
                    change: w::AccountChange::MoveIn(
                        w::AccountChangeMoveIn::new(&limits, w::AccountChangeMoveInParts { digest }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        );
        let mut bytes = Writer::new(usize::try_from(ask.measure()).unwrap());
        ask.encode(&mut bytes).unwrap();
        let keyed = w::Keyed::new(
            &limits,
            w::KeyedParts {
                key: [extra; 16],
                owner: w::Owner::Jig,
                version: 21,
                request: bytes.finish(),
                value: Some(value),
            },
        )
        .unwrap();
        let mut bytes = Writer::new(usize::try_from(keyed.measure()).unwrap());
        keyed.encode(&mut bytes).unwrap();
        let mut call = peer::Peer::new(engine, b"POST", b"/jig/v1/request", &bytes.finish(), Some(&bearer));
        for _ in 0_u32..128 {
            call.pump(engine, true);
            if call.is_closed() {
                break;
            }
        }
        if extra == 0 {
            assert!(call.received().starts_with(b"HTTP/1.1 200"), "response: {:?}", call.received());
            let at = bytes::find(call.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
            let w::KeyedAnswer::FinalAnswer(answer) =
                w::KeyedAnswer::decode(&engine.clients.env.limits.wire, &mut Reader::new(&call.received()[at..]))
                    .unwrap()
            else {
                panic!("terminal move-in answer")
            };
            let w::Final::Refused(refused) =
                w::Final::decode(&engine.clients.env.limits.wire, &mut Reader::new(answer.record())).unwrap()
            else {
                panic!("the accounts refuse the opaque, invalid file")
            };
            let w::Refusal::AccountMove(why) = refused.refusal() else {
                panic!("the value passes both caps and reaches the accounts: {:?}", refused.refusal())
            };
            assert_eq!(why.why(), &w::AccountMoveWhy::NotOurs);
        } else {
            assert!(call.received().starts_with(b"HTTP/1.1 400"), "the wire refuses the extra byte");
            assert!(engine.events.is_empty(), "refused before reaching the root");
        }
    }
}

#[test]
fn secret_asks_in_flight_each_get_their_own_terminal() {
    use skein_lib::{List, Queue};
    // Every request leaves the clients before any terminal is delivered.
    const ASKS: u32 = 67;
    let mut clients = super::fixtures::clients();
    let mut store = super::fixtures::marked_store();
    clients.secret_terminals = Some(Queue::uncapped());
    let mut delayed = List::with_capacity(ASKS);
    for number in 0..ASKS {
        let owner = match number % 3 {
            0 => kit::SecretOwner::SignIn(u64::from(number)),
            1 => kit::SecretOwner::Service(u64::from(number)),
            _ => kit::SecretOwner::Carried(u64::from(number)),
        };
        clients.secrets.push(kit::SecretsRequest::Erase { owner });
        clients.secrets_step(&mut store);
        assert!(clients.secrets.is_empty(), "each ask issues while all earlier terminals wait");
        store.iterate();
        let terminal = store.host.above.pop().expect("one store terminal per erase");
        let jig_protocol_store::Event::Erased { owner: routed } = &terminal else { panic!("erase terminal") };
        assert_eq!(*routed, Token::new(crate::tokens::CLIENTS + 1 + u64::from(number)), "distinct routing tokens");
        assert!(store.host.above.is_empty(), "exactly one terminal");
        delayed.push((owner, terminal)).unwrap();
    }
    assert_eq!(store.secret_requests, ASKS, "all requests are in flight together");
    assert!(clients.secret_terminals.as_ref().unwrap().is_empty(), "no terminal delivered early");
    for (owner, terminal) in delayed.into_boxed().into_iter().rev() {
        store.host.secrets.push(terminal);
        clients.secrets_step(&mut store);
        assert!(store.host.secrets.is_empty(), "the reversed terminal is consumed");
        let trace = clients.secret_terminals.as_mut().unwrap();
        assert_eq!(trace.pop(), Some(owner), "each terminal keeps its original owner");
        assert!(trace.is_empty(), "one terminal per owner");
    }
    assert!(clients.secrets.is_empty());
}

#[test]
fn a_stalled_link_consumer_drops_nothing_and_keeps_order() {
    const STALLED_PASSES: u32 = 2;
    let mut service = super::fixtures::ready();
    let admitted = super::fixtures::engine().limits.sign_in.pending;
    let bound = service.links().capacity();
    let produced = admitted.max(bound).checked_add(3).unwrap();
    let _initial_link = service.links().pop().expect("startup link");
    assert!(service.links().is_empty());
    for number in 0..produced {
        service.engine.as_mut().unwrap().clients.links.push(ordered_printed_link(number));
    }
    // The binary leaves its channel untouched across two service passes.
    for _ in 0..STALLED_PASSES {
        super::fixtures::turn(&mut service);
        assert_eq!(service.links().len(), bound, "the binary-facing channel fills");
        let upstream = &service.engine.as_ref().unwrap().clients.links;
        assert_eq!(upstream.len(), produced - bound, "the producer waits with every excess link");
        assert_eq!(upstream.iter().next().unwrap().as_ref(), ordered_printed_link(bound).as_ref(), "head waits");
    }
    for number in 0..bound {
        assert_eq!(service.links().pop().unwrap(), ordered_printed_link(number), "FIFO before the stall");
    }
    assert!(service.links().is_empty());
    super::fixtures::turn(&mut service);
    assert!(service.engine.as_ref().unwrap().clients.links.is_empty(), "the waiting producer resumes");
    for number in bound..produced {
        assert_eq!(service.links().pop().unwrap(), ordered_printed_link(number), "FIFO across the stall");
    }
    assert!(service.links().is_empty(), "no duplicate links");
    assert_eq!(service.end(), None);
    assert_eq!(bound, admitted, "printed-link credit belongs to the admitted sign-in population");
}

fn ordered_printed_link(number: u32) -> Box<[u8]> {
    let secret = skein_lib::Secret::new(Box::new([u8::try_from(number).unwrap(); 32]));
    crate::clients::printed::link(([127, 0, 0, 1], 7100).into(), &secret)
}

fn settings_landing_value() -> temper_engine_domain_forge::settings::Value {
    use temper_engine_domain_forge::{GateLast, GatePattern, settings};
    settings::Value::Rule(settings::LandingRule {
        sets: Box::new([]),
        pattern: GatePattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: true,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    })
}
fn settings_application_items(bytes: &[u8]) -> Box<[core::settings::Item]> {
    use core::settings as s;
    Box::new([s::Item {
        setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
        edit: s::Edit::Set(s::Value::Application(bytes.into())),
    }])
}
fn settings_entrances(bytes: &[u8]) -> [core::Event; 2] {
    use core::settings as s;
    [
        core::Event::ChangeSettings {
            reply_to: REPLY,
            sign_in: 3,
            key: [5; 16],
            digest: [6; 32],
            change: Box::new(s::Change {
                against: 1,
                items: settings_application_items(bytes),
                running: s::Running::Stays,
            }),
        },
        core::Event::ProposeSettings {
            reply_to: REPLY,
            sign_in: 3,
            key: [5; 16],
            digest: [6; 32],
            change: Box::new(s::Proposed {
                against: 1,
                items: settings_application_items(bytes),
                reason: b"landing".as_slice().into(),
            }),
        },
    ]
}
#[test]
fn settings_entrances_carry_typed_landing_values() {
    let value = settings_landing_value();
    let bytes = temper_engine_smith::kinds::encode(&value);
    let size = u32::try_from(bytes.len()).expect("fixture encoding");
    let expected = [root::KindValue {
        kind: 1,
        scope: 0,
        number: 1,
        value: temper_engine_domain_forge::settings::SizedValue { value, size },
    }];
    for event in settings_entrances(&bytes) {
        let input = translation::to_root(event).expect("valid settings");
        let (root::PartyInput::ChangeSettings { kinds, .. } | root::PartyInput::ProposeSettings { kinds, .. }) = input
        else {
            panic!("settings input")
        };
        assert_eq!(kinds.as_ref(), &expected);
    }
}
#[test]
fn settings_entrances_refuse_cut_or_trailing_values_naming_item_one() {
    let bytes = temper_engine_smith::kinds::encode(&settings_landing_value());
    let cut = &bytes[..bytes.len() - 1];
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).expect("one trailing byte"));
    trailing.put(&bytes).expect("encoding room");
    trailing.put(&[0]).expect("trailing byte room");
    let trailing = trailing.finish();
    for malformed in [cut, trailing.as_ref()] {
        for event in settings_entrances(malformed) {
            let reply = translation::to_root(event).expect_err("malformed settings");
            let kit::Event::Reply { to, reply: people::Reply::Refused(why) } = *reply else { panic!("shape refusal") };
            assert_eq!(to, REPLY);
            assert_eq!(
                why,
                people::Refusal::NotWellFormed {
                    at: 1,
                    why: people::Malformed::Settings {
                        setting: people::Setting::Application { kind: 1, scope: 0, number: 1 },
                        fault: people::SettingFault::Shape,
                        detail: None
                    }
                }
            );
        }
    }
}

fn forges_linked(engine: &crate::engine::Engine) -> bool {
    let bytes = crate::route::forge_rows(engine, engine.domain.forges()).expect("forges document");
    temper_client_protocol_wire::decode_forges(&bytes, &temper_client_protocol_wire::v1::CEILINGS)
        .expect("whole forges document")
        .linked()
}
