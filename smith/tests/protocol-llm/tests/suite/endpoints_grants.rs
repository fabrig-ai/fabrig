//! Startup endpoint mapping and two-generation credential table.

use core::net::SocketAddr;

use skein_lib::Time;
use skein_llm::{self as shared, Credential};
use smith_domain::{GrantName, llm};
use smith_protocol_llm::{ConfiguredEndpoint, EndpointError, Endpoints, GrantError, Grants};

fn destination(name: u32, provider: shared::Provider) -> ConfiguredEndpoint {
    let mut roots = skein_tls::RootCertStore::empty();
    roots
        .add(skein_tls::CertificateDer::from(include_bytes!("../../fixtures/root.der").to_vec()))
        .expect("fixture TLS root");
    let llm = match provider {
        shared::Provider::OpenAi => shared::Endpoint::openai(),
        shared::Provider::Anthropic => shared::Endpoint::anthropic(),
    };
    ConfiguredEndpoint {
        connect: skein_lib::Duration::from_secs(1),
        handshake: skein_lib::Duration::from_secs(1),
        name: llm::Endpoint(name),
        destination: skein_llm_connection::Endpoint {
            limits: skein_llm_world::limits(),
            credential: skein_llm::client::CredentialLimits { value: 64 },
            address: SocketAddr::from(([127, 0, 0, 1], 443)),
            transport: skein_llm_connection::Transport::Tls {
                server_name: skein_tls::Name::new("example.test").expect("server name"),
                trust: skein_tls::Config::new(roots, &[]).expect("test trust"),
            },
            llm,
        },
        models: Box::new([]),
    }
}

#[test]
fn numeric_names_resolve_the_complete_destination_and_options() {
    let endpoints = Endpoints::new(
        Box::new([destination(42, shared::Provider::OpenAi), destination(7, shared::Provider::Anthropic)]),
        2,
    )
    .expect("two named endpoints");
    let openai = endpoints.resolve(llm::Endpoint(42)).expect("configured name");
    assert_eq!(openai.index, 0);
    assert_eq!(openai.provider, shared::Provider::OpenAi);
    assert_eq!(endpoints.resolve(llm::Endpoint(7)).expect("second name").index, 1);
    assert!(endpoints.resolve(llm::Endpoint(3)).is_none());
    let (destinations, options) = endpoints.into_parts();
    assert_eq!(destinations.len(), 2);
    assert_eq!(options.get(&42).expect("named options").index, 0);
    assert_eq!(destinations.get(1).expect("second destination").llm.provider, shared::Provider::Anthropic);
}

#[test]
fn duplicate_names_fail_at_startup() {
    assert!(matches!(
        Endpoints::new(
            Box::new([destination(9, shared::Provider::Anthropic), destination(9, shared::Provider::OpenAi),]),
            2
        ),
        Err(EndpointError::Duplicate)
    ));
    assert!(matches!(
        Endpoints::new(Box::new([destination(9, shared::Provider::Anthropic)]), 0),
        Err(EndpointError::TooMany)
    ));
}

fn credential(value: &[u8]) -> Credential {
    Credential::bearer(value.into())
}

#[test]
fn a_refresh_preserves_the_first_call_then_evicts_the_oldest_generation() {
    let mut grants = Grants::new(1, 64).expect("bounded table");
    let first = GrantName { account: 0, generation: 1 };
    let second = GrantName { account: 0, generation: 2 };
    let third = GrantName { account: 0, generation: 3 };
    grants.grant(first, credential(b"first"), Time::from_nanos(100)).expect("first grant");
    let started = grants.read(first, Time::from_nanos(10)).expect("call reads its starting value");
    grants.grant(second, credential(b"second"), Time::from_nanos(200)).expect("refresh");
    assert!(started.value.as_ref() == b"first");
    assert!(grants.read(second, Time::from_nanos(20)).expect("next call").value.as_ref() == b"second");
    assert!(grants.read(first, Time::from_nanos(20)).expect("overlapping first").value.as_ref() == b"first");
    grants.grant(third, credential(b"third"), Time::from_nanos(300)).expect("next refresh");
    assert!(matches!(grants.read(first, Time::from_nanos(20)), Err(GrantError::Missing)));
    assert!(matches!(grants.read(second, Time::from_nanos(201)), Err(GrantError::Lapsed)));
    assert!(grants.read(third, Time::from_nanos(201)).expect("new value").value.as_ref() == b"third");
    assert!(matches!(grants.grant(second, credential(b"stale"), Time::from_nanos(400)), Err(GrantError::Stale)));
    assert!(!format!("{grants:?}").contains("third"), "debug hides bearer values");
}

#[test]
fn reasoning_drop_is_selected_per_declared_model() {
    use smith_protocol_llm::{ConfiguredModel, OversizedReasoning};
    let mut endpoint = destination(1, shared::Provider::OpenAi);
    endpoint.models = Box::new([
        ConfiguredModel { name: b"strict".as_slice().into(), oversized_reasoning: OversizedReasoning::Fail },
        ConfiguredModel { name: b"optional".as_slice().into(), oversized_reasoning: OversizedReasoning::Drop },
    ]);
    let endpoints = Endpoints::new(Box::new([endpoint]), 1).expect("named models");
    let options = endpoints.resolve(llm::Endpoint(1)).expect("endpoint");
    assert!(!options.drop_reasoning(b"strict"));
    assert!(options.drop_reasoning(b"optional"));
    assert!(!options.drop_reasoning(b"undeclared"));
    assert!(!options.drop_reasoning(b"strict"));
}

// profile::ACCOUNTS admits four distinct names, independent of their numeric value.
const ACCOUNT_COUNT: u32 = 4;
const SPARSE_ACCOUNT: u32 = 0x9f3a_0001;
const GRANT_BYTES: u32 = 64;

#[test]
fn a_sparse_account_number_is_granted_and_used() {
    let mut grants = Grants::new(ACCOUNT_COUNT, GRANT_BYTES).expect("bounded table");
    let name = GrantName { account: SPARSE_ACCOUNT, generation: 1 };
    grants.grant(name, Credential::key(b"key".as_slice().into()), Time::from_nanos(100)).expect("sparse grant");
    let installed = grants.read(name, Time::ZERO).expect("sparse name resolves");
    assert_eq!(installed.scheme, shared::Scheme::Key);
    shared::client::check_credential(&installed, &shared::client::CredentialLimits { value: GRANT_BYTES })
        .expect("installed credential fits its value bound");
}

#[test]
fn a_fifth_distinct_account_is_refused() {
    let mut grants = Grants::new(ACCOUNT_COUNT, GRANT_BYTES).expect("bounded table");
    for account in 0..ACCOUNT_COUNT {
        grants
            .grant(GrantName { account, generation: 1 }, credential(b"value"), Time::from_nanos(100))
            .expect("four accounts fit");
    }
    assert_eq!(
        grants.grant(GrantName { account: SPARSE_ACCOUNT, generation: 1 }, credential(b"value"), Time::from_nanos(100)),
        Err(GrantError::Full)
    );
}
