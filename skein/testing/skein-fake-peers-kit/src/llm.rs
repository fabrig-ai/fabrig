//! LLM peer construction shared by plaintext worlds and the TLS handshake.
use skein_fake_llm_domain::{self as domain, api};
use skein_fake_llm_protocol::{documents, provider};
use skein_fake_peers::{Limits, Transport, llm};
use skein_lib::Duration;
use skein_llm::{Credential, Endpoint, Provider};
use std::net::SocketAddr;

#[must_use]
pub fn limits() -> Limits {
    Limits {
        io: skein_io::Limits { sockets: 3, retry: Duration::from_millis(1), ..skein_io::Limits::TEST },
        connections: 2,
        queue: 64,
        plaintext: 32_768,
        ciphertext: 32_768,
        observations: 16,
        observation_bytes: 32_768,
    }
}

pub fn make(address: SocketAddr, transport: Transport) -> Result<llm::Peer, skein_fake_peers::Error> {
    let call = skein_llm_kit::call(1);
    make_on(call.endpoint, call.credential.expect("fixture credential"), address, transport)
}

pub fn make_on(
    endpoint: Endpoint,
    credential: Credential,
    address: SocketAddr,
    transport: Transport,
) -> Result<llm::Peer, skein_fake_peers::Error> {
    make_scripted(endpoint, credential, address, transport, Box::new([]))
}

pub fn make_scripted(
    endpoint: Endpoint,
    credential: Credential,
    address: SocketAddr,
    transport: Transport,
    scripts: Box<[api::Script]>,
) -> Result<llm::Peer, skein_fake_peers::Error> {
    make_scripted_with_limits(endpoint, credential, address, transport, scripts, limits())
}

pub fn make_scripted_with_limits(
    endpoint: Endpoint,
    credential: Credential,
    address: SocketAddr,
    transport: Transport,
    scripts: Box<[api::Script]>,
    limits: Limits,
) -> Result<llm::Peer, skein_fake_peers::Error> {
    let machine_limits = skein_llm_kit::fake::config();
    let domain = domain::Domain::configured(
        &machine_limits,
        23,
        scripts,
        api::Menu { arguments: Box::new([]), invalid: Box::new([]) },
    )
    .expect("configured script domain");
    make_domain_with_limits(endpoint, credential, address, transport, domain, limits)
}

pub fn make_domain_with_limits(
    endpoint: Endpoint,
    credential: Credential,
    address: SocketAddr,
    transport: Transport,
    domain: domain::Domain,
    limits: Limits,
) -> Result<llm::Peer, skein_fake_peers::Error> {
    let machine_limits = skein_llm_kit::fake::config();
    llm::Peer::new(
        address,
        transport,
        limits,
        provider::Config {
            list: None,
            list_status: None,
            without_credential: false,
            usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
            echo: skein_llm::openai::Echo::NONE,
            provider: match endpoint.provider {
                Provider::OpenAi => documents::Provider::OpenAi,
                Provider::Anthropic => documents::Provider::Anthropic,
            },
            path: endpoint.target,
            headers: Box::new([]),
        },
        credential,
        skein_llm_kit::fake::limits(&skein_llm_kit::limits()),
        domain,
        machine_limits,
    )
}
