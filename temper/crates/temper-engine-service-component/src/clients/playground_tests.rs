use super::*;
use crate::tests::fixtures;
use temper_client_protocol_wire::{self as wire, v1 as w};

const PLAYGROUND_SOURCE: u64 = 1;

fn enqueue(clients: &mut Clients) {
    enqueue_request(clients, w::Request::StartPlayground);
}
fn enqueue_request(clients: &mut Clients, request: w::Request) {
    let requesting = w::Requesting { request };
    clients.out.push(kit::Request::Application(kit::ToApplication::Request {
        reply_to: skein_lib::ReplyTo::new(Token::new(91)),
        sign_in: 1,
        key: [9; 16],
        digest: [7; 32],
        scope: people::ApplicationScope::Deployment,
        carried: None,
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        record: wire::requesting(&requesting),
    }));
}

#[test]
fn start_playground_reads_the_key_file_then_asks_the_engine() {
    start_reads_the_key(w::Request::StartPlayground, temper_engine_domain_forge::Ask::StartPlayground);
}
fn start_reads_the_key(request: w::Request, expected: temper_engine_domain_forge::Ask) {
    let mut service = fixtures::demo_service();
    let engine = service.engine.as_mut().unwrap();
    engine.clients.starting = None;
    enqueue_request(&mut engine.clients, request.clone());
    engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
    assert!(engine.events.is_empty());
    let read = service.take_handover_read().unwrap();
    assert!(read.path.ends_with(b"demo/llm.key"));
    assert_eq!(read.max, 4097);
    assert_eq!(read.forge, 0);
    // The shell removes exactly one LF. This remaining LF belongs to the key.
    service.handover_read(read.forge, read.run, Ok(Some(skein_lib::Secret::new(b"test-key\n".as_slice().into()))));
    let engine = service.engine.as_mut().unwrap();
    let mut store = fixtures::marked_store();
    engine.clients.secrets_step(&mut store);
    store.iterate();
    while let Some(answer) = store.host.above.pop() {
        engine.clients.secret_answer(answer);
    }
    engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
    let Some(root::Event::Party(root::PartyInput::Application {
        request: root::Application::Forge(ask),
        carried: Some(carried),
        ..
    })) = engine.events.pop()
    else {
        panic!("carried start")
    };
    assert_eq!(ask, expected);
    assert_ne!(carried.number, 0);
    assert_eq!(carried.digest, skein_lib::sha256::digest(b"test-key\n"));
    // The playground adds its key through accounts, whose store adoption
    // accepts a carried Key alone (jig's protocol/store/secrets.md).
    let _later = store.secrets(
        Token::new(PLAYGROUND_SOURCE),
        jig_protocol_store::Secrets::Adopt {
            carried: carried.number,
            owner: jig_protocol_store::SecretOwner::Source,
            number: PLAYGROUND_SOURCE,
            replaces: None,
        },
    );
    store.iterate();
    let Some(jig_protocol_store::Event::Secret { value: Some(jig_protocol_store::SecretValue::Key(value)), .. }) =
        store.secrets(
            Token::new(PLAYGROUND_SOURCE),
            jig_protocol_store::Secrets::Get {
                owner: jig_protocol_store::SecretOwner::Source,
                number: PLAYGROUND_SOURCE,
            },
        )
    else {
        panic!("the demo key is adoptable as a source")
    };
    assert_eq!(value.expose(), b"test-key\n");
}

#[test]
fn unreadable_empty_and_oversize_playground_keys_carry_nothing() {
    for answer in [
        Err(crate::HandoverUnread::Kernel(2)),
        Ok(None),
        Ok(Some(skein_lib::Secret::new(Box::new([])))),
        Ok(Some(skein_lib::Secret::new(Box::new([b'x'; 4097])))),
    ] {
        let mut service = fixtures::demo_service();
        let engine = service.engine.as_mut().unwrap();
        engine.clients.starting = None;
        enqueue(&mut engine.clients);
        engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
        let read = service.take_handover_read().unwrap();
        service.handover_read(read.forge, read.run, answer);
        let engine = service.engine.as_mut().unwrap();
        engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
        let Some(root::Event::Party(root::PartyInput::Application {
            request: root::Application::Forge(temper_engine_domain_forge::Ask::StartPlayground),
            carried: None,
            ..
        })) = engine.events.pop()
        else {
            panic!("start without a carried value")
        };
        assert!(engine.clients.playground_carries.is_empty());
    }
}

#[test]
fn concurrent_playground_starts_wait_for_the_first_keys_read_and_admission() {
    let mut service = fixtures::demo_service();
    let engine = service.engine.as_mut().unwrap();
    for _ in 0..=PLAYGROUND_STARTS {
        enqueue(&mut engine.clients);
        engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
    }
    assert_eq!(engine.clients.playground_starts.len(), 1);
    assert_eq!(engine.clients.playground_reads.len(), 1);
    assert_eq!(engine.clients.out.len(), PLAYGROUND_STARTS);
}

#[test]
fn running_and_starting_playgrounds_do_not_write_or_read_another_key() {
    use temper_engine_domain_forge::{Playground, PlaygroundState};
    for state in [PlaygroundState::Running, PlaygroundState::Starting] {
        let mut service = fixtures::demo_service();
        let engine = service.engine.as_mut().unwrap();
        engine.clients.starting = None;
        engine.clients.playground = Some(Playground { state, forge: 1, project: None, account: None });
        enqueue(&mut engine.clients);
        engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
        assert!(engine.clients.playground_read().is_none(), "no key IO while {state:?}");
        assert!(!engine.events.is_empty(), "the root answers the start from its row");
    }
}

#[test]
fn the_playground_read_uses_the_restored_engine_row() {
    use temper_engine_domain_forge as forge;
    let mut previous = fixtures::ready().engine.unwrap();
    let row = forge::Playground { state: forge::PlaygroundState::Running, forge: 1, project: Some(1), account: None };
    fixtures::commit(
        &mut previous.store,
        1,
        Box::new([root::Write::Save(root::Record::Forge { row: Box::new(forge::Stored::Playground(row)) })]),
    );
    previous.store.host.above.pop().expect("playground saved");
    let reopened = fixtures::reopened_store(previous.store, &previous.env.limits);
    let mut settings = fixtures::engine();
    // Hold the reset at its project so the read observes the restored row.
    settings.jig.acts = Box::new([]);
    settings.jig.limits.core.people.act_grants = 0;
    let mut service = fixtures::start_configured(settings, &[(1, 100)], Some(reopened), false, false);
    fixtures::restart(&mut service);
    let engine = service.engine.as_ref().unwrap();
    let resetting = forge::Playground { state: forge::PlaygroundState::Resetting(forge::ResetStep::Project), ..row };
    assert_eq!(engine.domain.playground(), Some(&resetting));
    assert_eq!(engine.clients.playground.as_ref(), engine.domain.playground());
}
