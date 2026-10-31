//! Accounts effects routed through the reference service.
use super::*;
use crate::tests::fixtures;
fn finish_start(service: &mut crate::Service) {
    for _ in 0_u32..64 {
        fixtures::turn(service);
        if fixtures::engine(service).domain.ready() {
            assert!(fixtures::engine(service).store.component.mark_durable());
            return;
        }
    }
    panic!("account fixture start completes before serving requests");
}

fn carry_llm_key(engine: &mut crate::Engine, value: &[u8]) -> u64 {
    let _later = engine.store.secrets(
        Token::new(123),
        store::Secrets::Carry { kind: jig_protocol_store::CarriedKind::Key, value: Secret::new(value.into()) },
    );
    // The start's mark may still have a terminal here. Preserve unrelated
    // answers and take only this explicit carry's terminal.
    for _ in 0..engine.store.above.len() {
        let event = engine.store.above.pop().expect("store answer");
        if let store::Event::Carried { owner, number } = &event
            && *owner == Token::new(123)
        {
            return *number;
        }
        engine.store.above.push(event);
    }
    panic!("carried LLM key")
}
fn pump_accounts(accounts: &mut Accounts, engine: &mut crate::Engine) -> List<domain::FromComponent> {
    let mut events = Queue::with_capacity(64);
    let mut heard = List::with_capacity(64);
    for _ in 0_u32..32 {
        crate::store::answer(engine);
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        while let Some(event) = events.pop() {
            if let root::Event::Core(core::Event::Account(domain::Event::Component(event))) = event {
                heard.push(event).expect("bounded component events");
            }
        }
    }
    heard
}
#[test]
fn llm_key_work_reaches_the_store_and_the_workers_table_through_the_service() {
    let mut service = fixtures::started(&fixtures::accounts_document());
    finish_start(&mut service);
    let engine = fixtures::engine(&mut service);
    let mut accounts = engine.accounts.take().expect("accounts component");
    let carried = carry_llm_key(engine, b"first-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: false }));
    assert!(has_llm_key_grant(&pump_accounts(&mut accounts, engine), 1));
    assert_eq!(
        engine.values.accounts.value(7, 1, Time::from_nanos(1)).expect("worker value").token.expose(),
        b"first-llm-key"
    );
    let carried = carry_llm_key(engine, b"next-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: true }));
    assert!(has_llm_key_grant(&pump_accounts(&mut accounts, engine), 2));
    accounts.request(domain::Request::Signing(domain::Signing::Forget { source: 7 }));
    pump_accounts(&mut accounts, engine);
    assert!(engine.values.accounts.value(7, 1, Time::from_nanos(1)).is_none());
    assert!(engine.values.accounts.value(7, 2, Time::from_nanos(1)).is_none());
    let Some(store::Event::Secret { value: None, .. }) =
        engine.store.secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Source, number: 7 })
    else {
        panic!("removed LLM key value")
    };
}
#[test]
fn service_restart_restores_an_llm_key_before_want_and_sweeps_only_unnamed_sources() {
    let mut service = fixtures::started(&fixtures::accounts_document());
    finish_start(&mut service);
    let engine = fixtures::engine(&mut service);
    let mut accounts = engine.accounts.take().expect("accounts component");
    let carried = carry_llm_key(engine, b"kept-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: false }));
    pump_accounts(&mut accounts, engine);
    engine.values.accounts.remove(7);
    let carried_orphan = carry_llm_key(engine, b"orphaned-carried-value");
    let _later = engine.store.secrets(
        Token::new(123),
        store::Secrets::Keep {
            owner: store::SecretOwner::Source,
            number: 8,
            value: jig_protocol_store::SecretValue::Record(Secret::new(b"orphaned-source-value".as_slice().into())),
        },
    );
    while engine.store.above.pop().is_some() {}
    let mut fresh_service = fixtures::started(&fixtures::accounts_document());
    let mut restarted = fixtures::engine(&mut fresh_service).accounts.take().expect("new component");
    restarted.request(domain::Request::Signing(domain::Signing::Restore {
        source: 7,
        provider: 1,
        kind: domain::Kind::Key,
    }));
    restarted.request(domain::Request::Component(domain::ToComponent::Want { account: 7 }));
    restarted.request(domain::Request::Signing(domain::Signing::Restored));
    let heard = pump_accounts(&mut restarted, engine);
    // One restored source and one queued want yield exactly one grant.
    let mut grants = 0_u32;
    for event in &heard {
        if let domain::FromComponent::Granted { account: 7, generation: 1, .. } = event {
            grants = grants.checked_add(1).expect("bounded component events");
        }
    }
    assert_eq!(grants, 1);
    assert_eq!(
        engine.values.accounts.value(7, 1, Time::from_nanos(1)).expect("restored value").token.expose(),
        b"kept-llm-key"
    );
    for (owner, number, exists) in [
        (store::SecretOwner::Source, 1, false),
        (store::SecretOwner::Source, 7, true),
        (store::SecretOwner::Source, 8, false),
        (store::SecretOwner::Application, carried_orphan, true),
    ] {
        let Some(store::Event::Secret { value, .. }) =
            engine.store.secrets(Token::new(123), store::Secrets::Get { owner, number })
        else {
            panic!("stored value")
        };
        assert_eq!(value.is_some(), exists, "{owner:?} {number}");
    }
}

fn has_llm_key_grant(heard: &List<domain::FromComponent>, wanted: u64) -> bool {
    for event in heard {
        if let domain::FromComponent::Granted {
            account: 7,
            generation,
            form: domain::Form::Credential(domain::Scheme::Key),
            ..
        } = event
            && *generation == wanted
        {
            return true;
        }
    }
    false
}

#[test]
fn a_restored_sources_wrong_kind_stops_the_engine() {
    for (kind, value) in [
        (domain::Kind::Key, store::SecretValue::Record(Secret::new(Box::from(b"record".as_slice())))),
        (domain::Kind::SignIn, store::SecretValue::Key(Secret::new(Box::from(b"key".as_slice())))),
    ] {
        let mut service = fixtures::started(&fixtures::accounts_document());
        finish_start(&mut service);
        let engine = fixtures::engine(&mut service);
        let mut accounts = engine.accounts.take().expect("accounts runtime");
        engine
            .store
            .secrets(Token::new(123), store::Secrets::Keep { owner: store::SecretOwner::Source, number: 7, value });
        while engine.store.above.pop().is_some() {}
        accounts.request(domain::Request::Signing(domain::Signing::Restore { source: 7, provider: 1, kind }));
        pump_accounts(&mut accounts, engine);
        crate::store::answer(engine);
        assert_eq!(
            engine.end,
            Some(crate::End::Stopped(crate::Stop::StorePage {
                phase: None,
                failure: store::restore::Failure {
                    cause: store::restore::Cause::Decode(store::restore::DecodeFailure::Value),
                    row: None,
                },
            })),
            "wrong restored kind must stop: {kind:?}"
        );
    }
}
