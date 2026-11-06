//! The host's ordered pass and the owner's outputs (service.md, 11.1).
use super::*;
use crate::tests::fixtures;
use jig_domain_link::{Form, Scheme};
use skein_lib::bytes;
fn machine() -> Accounts {
    let limits = fixtures::driver_limits();
    let seed = Secret::new(Box::new([7; 32]));
    let component = Component::new(
        bytes::copy_of(b"accounts-test"),
        Map::with_capacity(0),
        limits,
        CryptoRng::new(&seed, 1).expect("seed"),
        2,
    );
    Accounts::made(component, limits)
}
fn begin(host: &mut Accounts, marked: bool, answers: &mut Queue<store::Event>) -> u32 {
    host.begin(Time::ZERO, Wall::EPOCH, marked, [5; 16], answers)
}
fn keep(host: &mut Accounts) {
    host.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried: 11, replaces: false }));
}
fn get(host: &mut Accounts, table: &mut Table) -> Token {
    assert_eq!(begin(host, true, &mut Queue::uncapped()), 1);
    let Some(Output::Store { exchange, request: store::Secrets::Get { .. } }) = host.output(table) else {
        panic!("read carried key")
    };
    exchange
}
#[test]
fn an_answer_taken_at_begin_is_stepped_before_the_fire_that_counts_its_work() {
    let mut host = machine();
    let mut answers = Queue::uncapped();
    let mut table = Table::new(1, 32);
    assert_eq!(begin(&mut host, true, &mut answers), 0);
    keep(&mut host);
    let exchange = get(&mut host, &mut table);
    answers.push(store::Event::Secret {
        owner: Token::tagged(3, exchange.raw()).expect("tagged exchange"),
        value: Some(store::SecretValue::Carried {
            kind: store::CarriedKind::Key,
            value: Secret::new(bytes::copy_of(b"key")),
        }),
    });
    assert_eq!(begin(&mut host, true, &mut answers), 1);
    assert!(answers.is_empty());
    let Some(Output::Store { request, .. }) = host.output(&mut table) else { panic!("adoption request") };
    let store::Secrets::Adopt { .. } = request else { panic!("adoption after answer") };
}
#[test]
fn opened_reaches_the_component_only_when_marked() {
    let mut host = machine();
    keep(&mut host);
    assert_eq!(begin(&mut host, false, &mut Queue::uncapped()), 0);
    assert!(!host.opened());
    assert_eq!(begin(&mut host, true, &mut Queue::uncapped()), 0);
    assert!(host.opened());
    get(&mut host, &mut Table::new(1, 32));
}
#[test]
fn a_held_event_leaves_first_and_is_work_until_it_does() {
    let mut host = machine();
    host.hold(domain::Event::Component(domain::FromComponent::Closed));
    assert!(host.held() && !host.idle() && host.work_pending(Time::ZERO, false));
    host.effects.push(Effect::InvalidSecret);
    assert_eq!(begin(&mut host, false, &mut Queue::uncapped()), 2);
    let mut table = Table::new(1, 32);
    let Some(Output::Account(domain::Event::Component(domain::FromComponent::Closed))) = host.output(&mut table) else {
        panic!("held event first")
    };
    assert!(!host.held());
    let Some(Output::InvalidSecret) = host.output(&mut table) else { panic!("waiting effect second") };
    assert_eq!(host.effects(), 0);
    assert!(!host.work_pending(Time::ZERO, false));
}
#[test]
fn a_grant_reaches_the_table_before_its_event_leaves() {
    let mut host = machine();
    host.effects.push(Effect::Granted {
        source: 7,
        generation: 1,
        value: fixtures::value(b"key"),
        expires: Time::from_nanos(1),
    });
    host.effects.push(Effect::Child(domain::Event::Component(domain::FromComponent::Granted {
        account: 7,
        generation: 1,
        form: Form::Credential(Scheme::Key),
        valid: skein_lib::Duration::from_secs(1),
    })));
    let mut table = Table::new(1, 32);
    assert!(host.output(&mut table).is_none());
    assert_eq!(table.value(7, 1, Time::ZERO).expect("grant before event").token.expose(), b"key");
    let Some(Output::Account(domain::Event::Component(event))) = host.output(&mut table) else { panic!("grant event") };
    let domain::FromComponent::Granted { account: 7, .. } = event else { panic!("granted account") };
}
#[test]
fn an_aborted_host_opens_nothing_and_drops_its_store_requests() {
    let mut host = machine();
    assert_eq!(begin(&mut host, true, &mut Queue::uncapped()), 0);
    keep(&mut host);
    assert_eq!(begin(&mut host, true, &mut Queue::uncapped()), 1);
    host.abort();
    let mut table = Table::new(1, 32);
    assert!(host.output(&mut table).is_none(), "abort drops the waiting store request");
    let mut host = machine();
    host.abort();
    for _ in 0..begin(&mut host, true, &mut Queue::uncapped()) {
        let _output = host.output(&mut table);
    }
    assert!(!host.opened(), "abort never opens the component");
    assert!(!host.work_pending(Time::ZERO, true));
}
#[test]
fn a_restored_secret_of_the_wrong_kind_is_reported() {
    let mut host = machine();
    assert_eq!(begin(&mut host, true, &mut Queue::uncapped()), 0);
    host.request(domain::Request::Signing(domain::Signing::Restore {
        source: 7,
        provider: 1,
        kind: domain::Kind::Key,
    }));
    let mut table = Table::new(1, 32);
    let exchange = get(&mut host, &mut table);
    host.answer(store::Event::Secret {
        owner: exchange,
        value: Some(store::SecretValue::Record(Secret::new(bytes::copy_of(b"record")))),
    });
    assert_eq!(begin(&mut host, true, &mut Queue::uncapped()), 2);
    let Some(Output::InvalidSecret) = host.output(&mut table) else { panic!("invalid restored secret") };
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the test embeds a public TLS root fixture")]
fn a_tls_provider_with_empty_trust_bounds_is_refused() {
    let config = fixtures::sign_in_engine();
    let roots = [bytes::copy_of(include_bytes!("../../../../skein/tests/tls/fixtures/root.der"))];
    let (trust, _) = skein_tls::Config::from_der(&roots, &[]).expect("test root");
    let server_name = skein_tls::Name::new("issuer.test").unwrap();
    for (bounds, expected) in [
        (Some(TrustBounds { roots: 0, bytes: 1 }), Err(What::Bound)),
        (Some(TrustBounds { roots: 1, bytes: 1 }), Ok(1)),
        (None, Ok(1)),
    ] {
        let endpoint = driver::Endpoint {
            address: ([127, 0, 0, 1], 443).into(),
            transport: driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() },
        };
        let mut runtime = Runtime { providers: List::with_capacity(1), trust: bounds };
        runtime.providers.push((1, endpoint, List::with_capacity(0))).unwrap();
        let result = match provider_templates(&config, Some(runtime), &config.limits.driver, &[5; 16]) {
            Ok(providers) => Ok(providers.len()),
            Err(problems) => {
                assert_eq!((problems.problems.len(), problems.past), (1, 0));
                let problem = problems.problems.get(0).unwrap();
                assert_eq!(problem.path, Path::root().member(b"engine").member(b"llm").member(b"providers"));
                Err(problem.what)
            }
        };
        assert_eq!(result, expected);
    }
}
#[test]
fn trust_bounds_refuse_tls_alone_and_none_checks_nothing() {
    for (bounds, refused) in [
        (None, false),
        (Some(TrustBounds { roots: 0, bytes: 1 }), true),
        (Some(TrustBounds { roots: 1, bytes: 0 }), true),
        (Some(TrustBounds { roots: 1, bytes: 1 }), false),
    ] {
        assert!(!untrusted(bounds, false));
        assert_eq!(untrusted(bounds, true), refused);
    }
}
