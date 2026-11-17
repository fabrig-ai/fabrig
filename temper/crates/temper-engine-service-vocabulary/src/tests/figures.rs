//! Composing workers' figures (temper/docs/design/protocol/configuration/figures.md, 9).
use super::fixtures;
use crate::{figures, limits};
use jig_protocol_config as config;
use jig_protocol_worker_wire::v16 as w;
use skein_lib::{Duration, List};
use temper_client_protocol_wire::v1 as s;

#[test]
#[expect(clippy::too_many_lines, reason = "one case checks every endpoint field and both limit lists")]
fn compose_keeps_startup_limits_profile_and_endpoint_order_with_named_or_fallback_limits() {
    let mut engine = fixtures::engine();
    let mut openai = engine.jig.llm.endpoints[0].clone();
    openai.name = b"openai".as_slice().into();
    openai.number = 9;
    openai.dialect = 2;
    openai.provider = b"remote-provider".as_slice().into();
    openai.program = Some(b"remote-program".as_slice().into());
    openai.address = config::Address::Host { name: b"endpoint.example".as_slice().into(), port: 7443 };
    openai.transport = config::llm::Transport::Tls {
        server_name: b"server.example".as_slice().into(),
        trust: Some(b"trust.pem".as_slice().into()),
    };
    openai.headers =
        Box::new([config::llm::Header { name: b"x-test".as_slice().into(), value: b"test-value".as_slice().into() }]);
    openai.connect = Duration::from_millis(17);
    openai.handshake = Duration::from_millis(23);
    let mut local = engine.jig.llm.endpoints[0].clone();
    local.name = b"local".as_slice().into();
    local.number = 2;
    local.program = None;
    local.address = config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 8080).into()));
    engine.jig.llm.endpoints = Box::new([openai, local]);
    let named = s::AgentLimits::new(
        &s::CEILINGS,
        s::AgentLimitsParts {
            window: 2_000_000,
            output: 1234,
            reasoning_item: 99,
            head_ms: 345,
            idle_ms: 678,
            oversized: s::Oversized::Drop,
        },
    )
    .unwrap();
    let fallback = s::AgentLimits::new(
        &s::CEILINGS,
        s::AgentLimitsParts {
            window: 500,
            output: 100,
            reasoning_item: 50,
            head_ms: 10,
            idle_ms: u64::MAX,
            oversized: s::Oversized::Fail,
        },
    )
    .unwrap();
    let agents = s::AgentFigures::new(
        &s::CEILINGS,
        s::AgentFiguresParts {
            profile: List::from_boxed(Box::new([s::AgentFigure::new(
                &s::CEILINGS,
                s::AgentFigureParts { name: b"policy.max_turns".as_slice().into(), value: 19 },
            )
            .unwrap()])),
            endpoints: List::from_boxed(Box::new([s::AgentEndpointLimits::new(
                &s::CEILINGS,
                s::AgentEndpointLimitsParts { endpoint: b"openai".as_slice().into(), limits: named.clone() },
            )
            .unwrap()])),
            fallback: fallback.clone(),
        },
    )
    .unwrap();
    let figures = figures::compose(&engine, &agents);
    assert_eq!(figures.places(), 0);
    let profile = figures.agents().profile();
    assert_eq!(profile.len(), 1);
    assert_eq!((profile.as_slice()[0].name(), profile.as_slice()[0].value()), (b"policy.max_turns".as_slice(), 19));
    assert_eq!(figures.agents().endpoints().len(), 2);
    for (index, expected_limits) in [&named, &fallback].iter().enumerate() {
        let source = &engine.jig.llm.endpoints[index];
        let endpoint = &figures.agents().endpoints().as_slice()[index];
        assert_eq!(
            (endpoint.name(), endpoint.number(), endpoint.dialect()),
            (source.name.as_ref(), source.number, source.dialect)
        );
        assert_eq!(
            (endpoint.provider(), endpoint.program().as_deref()),
            (source.provider.as_ref(), source.program.as_deref())
        );
        assert_eq!((endpoint.connect(), endpoint.handshake()), (source.connect, source.handshake));
        assert_eq!(endpoint.headers().len(), u32::try_from(source.headers.len()).unwrap());
        for (header, expected) in endpoint.headers().iter().zip(source.headers.iter()) {
            assert_eq!((header.name(), header.value()), (expected.name.as_ref(), expected.value.as_ref()));
        }
        let limits = endpoint.limits();
        assert_eq!(
            (limits.window(), limits.output(), limits.reasoning_item()),
            (expected_limits.window(), expected_limits.output(), expected_limits.reasoning_item())
        );
        assert_eq!(
            (limits.head(), limits.idle()),
            (Duration::from_millis(expected_limits.head_ms()), Duration::from_millis(expected_limits.idle_ms()))
        );
    }
    let openai = &figures.agents().endpoints().as_slice()[0];
    assert_eq!((openai.host(), openai.port()), (b"endpoint.example".as_slice(), 7443));
    let w::Transport::Tls(tls) = openai.transport() else {
        panic!("TLS kept");
    };
    assert_eq!(
        (tls.server_name(), tls.trust().as_deref()),
        (b"server.example".as_slice(), Some(b"trust.pem".as_slice()))
    );
    assert_eq!(openai.limits().oversized(), &w::Oversized::Drop);
    let local = &figures.agents().endpoints().as_slice()[1];
    assert_eq!((local.host(), local.port()), (b"127.0.0.1".as_slice(), 8080));
    assert_eq!(local.transport(), &w::Transport::Plaintext);
    assert_eq!(local.limits().oversized(), &w::Oversized::Fail);
    let mut unique = skein_lib::Set::with_capacity(figures.limits().len());
    for limit in figures.limits() {
        assert!(unique.insert(limit.name()).unwrap());
    }
    let mut jig = config::stock();
    let mut own = limits::stock();
    for limit in figures.limits() {
        if limits::find(limit.name()).is_some() {
            limits::apply(&mut own, limit.name(), limit.value()).unwrap();
        } else {
            config::overrides::apply(&mut jig, limit.name(), limit.value()).unwrap();
        }
    }
    assert_eq!(config::overrides::list(&jig), config::overrides::list(&engine.jig.limits));
    assert_eq!(own, engine.limits);
}

#[test]
fn compose_takes_only_the_first_64_endpoints_of_an_unchecked_value() {
    let mut engine = fixtures::engine();
    let mut endpoints = List::with_capacity(65);
    for number in 1..=65 {
        let mut endpoint = engine.jig.llm.endpoints[0].clone();
        endpoint.number = number;
        endpoints.push(endpoint).unwrap();
    }
    engine.jig.llm.endpoints = endpoints.into_boxed();
    let figures = figures::compose(&engine, &crate::stock::agents());
    assert_eq!(figures.agents().endpoints().len(), 64);
    assert_eq!(figures.agents().endpoints().as_slice().last().unwrap().number(), 64);
}
