//! Ordering at the composition's owner and store boundaries.
use super::*;
use crate::tests::fixtures;
use skein_lib::bytes;

fn machine() -> Component {
    let limits = fixtures::driver_limits();
    let seed = Secret::new(Box::new([7; 32]));
    Component::new(
        bytes::copy_of(b"accounts-test"),
        Map::with_capacity(1),
        limits,
        CryptoRng::new(&seed, 1).expect("seed"),
        2,
    )
}
fn env() -> Env<driver::Limits> {
    Env { now: Time::from_nanos(1), wall: Wall::from_nanos(1), limits: fixtures::driver_limits() }
}
fn open(component: &mut Component, out: &mut Queue<Effect>) {
    component.step(&env(), Event::Opened { installation: [5; 16] }, out);
    component.fire(&env(), out);
}
fn request(component: &mut Component, signing: domain::Signing, out: &mut Queue<Effect>) {
    component.step(&env(), Event::Request(domain::Request::Signing(signing)), out);
}
fn template() -> driver::Account {
    driver::Account::SignIn {
        registration: oauth::Registration {
            authorization_url: bytes::copy_of(b"http://127.0.0.1:31000/authorize"),
            token_endpoint: bytes::copy_of(b"http://127.0.0.1:31000/token"),
            client_id: bytes::copy_of(b"client"),
            redirect_uri: bytes::copy_of(b"http://localhost:31234/callback"),
            scope: bytes::copy_of(b"read"),
            wire: oauth::WireFormat::Form,
            client_secret: None,
            pkce_for_confidential: false,
            metadata_claim: None,
            terms: None,
        },
        endpoint: driver::Endpoint { address: ([127, 0, 0, 1], 31000).into(), transport: driver::Transport::Plaintext },
        keeper: driver::Keeper::Owner { kept: None },
        shared: false,
        origins: List::with_capacity(0),
    }
}
#[test]
fn a_key_is_kept_before_its_value_is_lent() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    open(&mut component, &mut out);
    request(&mut component, domain::Signing::Keep { source: 7, carried: 11, replaces: false }, &mut out);
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, request: store::Secrets::Get { number: 11, .. } }) = out.pop() else {
        panic!("read carried key")
    };
    component.step(
        &env(),
        Event::Store(store::Event::Secret {
            owner: exchange,
            value: Some(store::SecretValue::Carried {
                kind: store::CarriedKind::Key,
                value: Secret::new(bytes::copy_of(b"key")),
            }),
        }),
        &mut out,
    );
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, request: store::Secrets::Adopt { .. } }) = out.pop() else {
        panic!("adopt before lending")
    };
    assert!(component.value(7, 1, env().now).is_none());
    assert!(out.is_empty());
    component.step(&env(), Event::Store(store::Event::Kept { owner: exchange }), &mut out);
    component.fire(&env(), &mut out);
    assert_eq!(component.value(7, 1, env().now).expect("durable key").token.expose(), b"key");
    let Some(Effect::Granted { source: 7, generation: 1, .. }) = out.pop() else {
        panic!("unexpected component effect");
    };
    let Some(Effect::Child(domain::Event::Component(domain::FromComponent::Granted { account: 7, .. }))) = out.pop()
    else {
        panic!("unexpected component effect");
    };
}
#[test]
fn only_the_component_refreshes_after_the_durable_start_and_restore() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    component.providers.insert(1, template()).ok().expect("provider");
    request(&mut component, domain::Signing::Restore { source: 0, provider: 1, kind: domain::Kind::SignIn }, &mut out);
    component.step(
        &env(),
        Event::Request(domain::Request::Component(domain::ToComponent::Want { account: 0 })),
        &mut out,
    );
    component.fire(&env(), &mut out);
    assert!(out.is_empty());
    open(&mut component, &mut out);
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, request: store::Secrets::Get { .. } }) = out.pop() else {
        panic!("restore first")
    };
    for _ in 0_u32..3 {
        component.fire(&env(), &mut out);
        assert!(out.is_empty());
    }
    assert!(!component.has_work(env().now), "pending restore waits on the store");
    let mut record = fixtures::record(0, b"expired", 1);
    record.expires_at = Wall::EPOCH;
    let value = oauth::encode_record(&record, &env().limits.client.document).expect("record");
    component.step(
        &env(),
        Event::Store(store::Event::Secret {
            owner: exchange,
            value: Some(store::SecretValue::Record(Secret::new(value))),
        }),
        &mut out,
    );
    component.fire(&env(), &mut out);
    let next = out.pop();
    let Some(Effect::Io(io::Request::Connect { .. })) = next else {
        panic!("the component asks the refresh transport: {next:?}, rest: {out:?}");
    };
    assert!(component.value(0, 1, env().now).is_none(), "expired value cannot be lent");
}
#[test]
fn naming_and_restore_precede_the_source_sweep() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    open(&mut component, &mut out);
    request(&mut component, domain::Signing::Restore { source: 7, provider: 1, kind: domain::Kind::Key }, &mut out);
    request(&mut component, domain::Signing::Restored, &mut out);
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, request: store::Secrets::Get { .. } }) = out.pop() else {
        panic!("restore before sweep")
    };
    assert!(out.is_empty());
    component.step(
        &env(),
        Event::Store(store::Event::Secret {
            owner: exchange,
            value: Some(store::SecretValue::Key(Secret::new(bytes::copy_of(b"key")))),
        }),
        &mut out,
    );
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange: SWEEP, request: store::Secrets::Load { .. } }) = out.pop() else {
        panic!("unexpected component effect");
    };
    component.inventory.row(7);
    component.inventory.row(8);
    component.inventory.loaded(None);
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange: SWEEP, request: store::Secrets::Erase { number: 8, .. } }) = out.pop() else {
        panic!("unexpected component effect");
    };
    assert!(out.is_empty());
}
#[test]
fn revocation_coalesces_sign_out_and_waits_for_durable_erasure() {
    for confirmed in [None, Some(true), Some(false)] {
        let mut component = machine();
        let mut out = Queue::uncapped();
        open(&mut component, &mut out);
        let record = fixtures::record(0, b"token", 1);
        let records::Next::Store(_) = component.records.keep(0, record.clone(), &env().limits.client.document) else {
            panic!("keep")
        };
        assert!(component.records.kept(0).is_some());
        component.required.insert(0, Box::default()).unwrap();
        component.above.push(driver::Event::Failed {
            account: 0,
            ends: driver::Ends::Refresh,
            failure: driver::Failure::Exchange(oauth::Failure::Refused),
        });
        component.fire(&env(), &mut out);
        let Some(Effect::Removed { source: 0 }) = out.pop() else {
            panic!("unexpected component effect");
        };
        let Some(Effect::Child(domain::Event::Component(domain::FromComponent::Failed { .. }))) = out.pop() else {
            panic!("unexpected component effect");
        };
        let Some(Effect::Store { request: store::Secrets::Erase { .. }, .. }) = out.pop() else {
            panic!("unexpected component effect");
        };
        if let Some(revoked) = confirmed {
            component.above.push(driver::Event::SignedOut { account: 0, revoked });
            component.above.push(driver::Event::SignedOut { account: 0, revoked });
            component.fire(&env(), &mut out);
            assert!(out.is_empty(), "one erase, no premature terminal");
        }
        let records::Next::Store(before) = component.records.keep(0, record.clone(), &env().limits.client.document)
        else {
            panic!("slot remains")
        };
        assert!(before.replaces.is_some());
        assert!(component.records.stale(0).is_some());
        component.step(&env(), Event::Store(store::Event::Erased { owner: Token::new(0) }), &mut out);
        component.fire(&env(), &mut out);
        let signed_out = match out.pop() {
            Some(Effect::Child(domain::Event::SignedOut { confirmed, .. })) => Some(confirmed),
            None => None,
            other => panic!("terminal: {other:?}"),
        };
        assert_eq!(signed_out, confirmed);
        let records::Next::Store(after) = component.records.keep(0, record, &env().limits.client.document) else {
            panic!("retained slot")
        };
        assert_eq!(after.replaces, None);
    }
}
#[test]
fn a_move_finishes_its_remove_read_and_notification_before_forget() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    open(&mut component, &mut out);
    component.providers.insert(1, template()).ok().expect("provider");
    component.source_providers.insert(0, 1).unwrap();
    component.moves.stage(0, template());
    request(&mut component, domain::Signing::Withdraw { source: 0, display: bytes::copy_of(b"Moved") }, &mut out);
    request(&mut component, domain::Signing::Forget { source: 0 }, &mut out);
    component.fire(&env(), &mut out);
    assert_eq!(component.early.len(), 1);
    let Some(Effect::SourceRemoved { source: 0 }) = out.pop() else {
        panic!("unexpected component effect");
    };
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, request: store::Secrets::Get { owner: store::SecretOwner::Source, .. } }) =
        out.pop()
    else {
        panic!("remove must precede the source read");
    };
    for _ in 0_u32..3 {
        component.fire(&env(), &mut out);
        assert_eq!(component.early.len(), 1);
        assert!(out.is_empty());
    }
    component.step(&env(), Event::Store(store::Event::Secret { owner: exchange, value: None }), &mut out);
    component.fire(&env(), &mut out);
    let Some(Effect::Child(domain::Event::Withdrawn { source: 0, .. })) = out.pop() else {
        panic!("unexpected component effect");
    };
    assert_eq!(component.early.len(), 1);
    component.fire(&env(), &mut out);
    assert!(component.early.is_empty());
}
#[test]
fn an_abort_discards_an_unsubmitted_key_adoption() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    open(&mut component, &mut out);
    request(&mut component, domain::Signing::Keep { source: 7, carried: 11, replaces: false }, &mut out);
    component.abort();
    component.fire(&env(), &mut out);
    assert!(component.value(7, 1, env().now).is_none());
    for effect in &out {
        if let Effect::Store { .. } = effect {
            panic!("abort submitted a secret request");
        }
    }
}

#[test]
fn a_wrong_restored_kind_aborts_the_composition() {
    for (kind, value) in [
        (domain::Kind::Key, store::SecretValue::Record(Secret::new(bytes::copy_of(b"record")))),
        (domain::Kind::SignIn, store::SecretValue::Key(Secret::new(bytes::copy_of(b"key")))),
    ] {
        let mut component = machine();
        let mut out = Queue::uncapped();
        open(&mut component, &mut out);
        request(&mut component, domain::Signing::Restore { source: 7, provider: 1, kind }, &mut out);
        component.fire(&env(), &mut out);
        let Some(Effect::Store { exchange, .. }) = out.pop() else { panic!("restore read") };
        component.step(&env(), Event::Store(store::Event::Secret { owner: exchange, value: Some(value) }), &mut out);
        component.fire(&env(), &mut out);
        let Some(Effect::InvalidSecret) = out.pop() else {
            panic!("unexpected component effect");
        };
        let Some(Effect::Child(domain::Event::Component(domain::FromComponent::Closed))) = out.pop() else {
            panic!("unexpected component effect");
        };
        assert!(component.is_empty());
    }
}
#[test]
fn abort_consumes_a_restore_answer_without_restoring_the_source() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    open(&mut component, &mut out);
    request(&mut component, domain::Signing::Restore { source: 7, provider: 1, kind: domain::Kind::Key }, &mut out);
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, .. }) = out.pop() else { panic!("restore read") };
    component.step(
        &env(),
        Event::Store(store::Event::Secret {
            owner: exchange,
            value: Some(store::SecretValue::Key(Secret::new(bytes::copy_of(b"key")))),
        }),
        &mut out,
    );
    component.step(&env(), Event::Abort, &mut out);
    component.fire(&env(), &mut out);
    assert!(component.value(7, 1, env().now).is_none());
    let Some(Effect::Child(domain::Event::Component(domain::FromComponent::Closed))) = out.pop() else {
        panic!("unexpected component effect");
    };
    assert!(component.is_empty());
}

#[test]
fn a_wanted_source_refreshes_at_its_deadline_without_an_owner_request() {
    let mut component = machine();
    let mut out = Queue::uncapped();
    component.providers.insert(1, template()).ok().expect("provider");
    open(&mut component, &mut out);
    request(&mut component, domain::Signing::Restore { source: 0, provider: 1, kind: domain::Kind::SignIn }, &mut out);
    component.step(
        &env(),
        Event::Request(domain::Request::Component(domain::ToComponent::Want { account: 0 })),
        &mut out,
    );
    component.fire(&env(), &mut out);
    let Some(Effect::Store { exchange, .. }) = out.pop() else { panic!("source read") };
    let record = fixtures::record(0, b"kept-token", 1);
    let value = oauth::encode_record(&record, &env().limits.client.document).expect("saved source");
    component.step(
        &env(),
        Event::Store(store::Event::Secret {
            owner: exchange,
            value: Some(store::SecretValue::Record(Secret::new(value))),
        }),
        &mut out,
    );
    component.fire(&env(), &mut out);
    let Some(Effect::Granted { source: 0, generation: 1, .. }) = out.pop() else { panic!("initial value") };
    let Some(Effect::Child(domain::Event::Component(domain::FromComponent::Granted { account: 0, .. }))) = out.pop()
    else {
        panic!("initial grant")
    };
    let due = component.next_deadline().expect("refresh armed");
    assert_eq!(due, Time::from_nanos(skein_lib::Duration::from_secs(50).as_nanos()));
    let mut later = env();
    later.now = due;
    later.wall = Wall::from_nanos(due.as_nanos());
    component.fire(&later, &mut out);
    assert!(component.has_work(later.now), "the timed refresh has a driver turn ready");
    component.fire(&later, &mut out);
    let Some(Effect::Io(io::Request::Connect { .. })) = out.pop() else {
        panic!("the component alone asks the timed refresh")
    };
}

#[test]
fn the_owners_return_location_reaches_the_sign_in_listener() {
    use skein_lib::{
        Intake, Writer,
        stream::{Down, Up},
    };
    let mut component = machine();
    component.providers.insert(1, template()).ok().expect("provider");
    let mut out = Queue::uncapped();
    open(&mut component, &mut out);
    request(&mut component, domain::Signing::SignIn { source: 7, provider: 1 }, &mut out);
    let listener = Token::new(100);
    let socket = Token::new(101);
    let address = io::Addr::Ip(([127, 0, 0, 1], 31234).into());
    let mut listener_owner = None;
    let mut callback_owner = None;
    let mut input = Intake::with_capacity(2048);
    let mut answer = List::with_capacity(4096);
    // A bounded callback head and response settle within these driver turns.
    for _ in 0_u32..128 {
        component.fire(&env(), &mut out);
        for _ in 0..out.len() {
            match out.pop().expect("component effect") {
                Effect::Io(io::Request::Listen { owner, .. }) => {
                    listener_owner = Some(owner);
                    component.step(
                        &env(),
                        Event::Io(io::Event::Listening { owner, listener, addr: address }),
                        &mut out,
                    );
                }
                Effect::Child(domain::Event::Visited { source: 7, address: visit }) => {
                    let state = visit
                        .split(|byte| *byte == b'&')
                        .find_map(|part| part.strip_prefix(b"state="))
                        .expect("authorization state");
                    let prefix = b"GET /callback?state=";
                    let suffix = b"&error=access_denied HTTP/1.1\r\nHost: localhost:31234\r\n\r\n";
                    let mut head =
                        Writer::new(prefix.len().checked_add(state.len()).unwrap().checked_add(suffix.len()).unwrap());
                    head.put(prefix).unwrap();
                    head.put(state).unwrap();
                    head.put(suffix).unwrap();
                    input.append(&head.finish()).unwrap();
                    component.step(
                        &env(),
                        Event::Return { source: 7, location: bytes::copy_of(b"https://app.example/accounts") },
                        &mut out,
                    );
                    component.step(
                        &env(),
                        Event::Io(io::Event::Accepted {
                            owner: listener_owner.expect("listener"),
                            socket,
                            peer: address,
                        }),
                        &mut out,
                    );
                }
                Effect::Io(io::Request::Bind { owner, .. }) => callback_owner = Some(owner),
                Effect::Io(io::Request::Stream { down: Down::Demand { read, room }, .. }) => {
                    let up = match input.meet(read) {
                        Some(value) => Some(Up::Bytes(value)),
                        None if room > 0 => Some(Up::Room),
                        None => None,
                    };
                    if let Some(up) = up {
                        component.step(
                            &env(),
                            Event::Io(io::Event::Stream { owner: callback_owner.expect("callback"), up }),
                            &mut out,
                        );
                    }
                }
                Effect::Io(io::Request::Stream { down: Down::Send(value), .. }) => {
                    for byte in value {
                        answer.push(byte).unwrap();
                    }
                }
                Effect::Io(io::Request::Close { entity } | io::Request::Abort { entity }) => {
                    let owner = if entity == listener { listener_owner } else { callback_owner }.expect("io owner");
                    component.step(&env(), Event::Io(io::Event::Closed { owner }), &mut out);
                }
                Effect::Store { exchange, request: store::Secrets::Erase { .. } } => {
                    component.step(&env(), Event::Store(store::Event::Erased { owner: exchange }), &mut out);
                }
                Effect::Removed { .. }
                | Effect::SourceRemoved { .. }
                | Effect::Child(_)
                | Effect::Io(io::Request::Stream { down: Down::Finish, .. }) => (),
                other => panic!("unexpected effect: {other:?}"),
            }
        }
    }
    assert!(answer.as_slice().starts_with(b"HTTP/1.1 303"));
    assert!(bytes::find(answer.as_slice(), b"location: https://app.example/accounts\r\n").is_some());
    assert!(bytes::find(answer.as_slice(), b"cache-control: no-store\r\n").is_some());
    assert!(bytes::find(answer.as_slice(), b"referrer-policy: no-referrer\r\n").is_some());
}
