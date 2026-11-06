//! Pure endpoint preparation from declarations, before native resolution or trust loading.
//! It retains validated names, request settings and transport sources, never kernel handles.
//! Contract: smith/docs/design/protocol/agent.md, section 4; smith/docs/design/protocol/limits.md, sections 2–4.

use skein_http::Header;
use skein_lib::{Duration, List};
use skein_llm as shared;
use skein_llm_connection as connection;
use skein_tls as tls;
use smith_agent_service as service;
use smith_agent_service::profile::Limits as DeclaredLimits;
use smith_agent_service::profile::{ENDPOINTS, Layer, LimitName, Per, Refusal};
use smith_domain::{self as domain, run};
use smith_protocol_llm as llm;
use std::net::SocketAddr;

use super::{ConfigurationRefusal, Endpoint, document, duration};

pub(super) struct PreparedEndpoints {
    pub(super) domain: Box<[run::charter::Endpoint]>,
    pub(super) channel: smith_protocol_translate::Endpoints,
    pub(super) llm: Box<[PreparedEndpoint]>,
    pub(super) limits: Box<[domain::EndpointLimits]>,
}

pub(super) struct PreparedEndpoint {
    pub(super) address: String,
    pub(super) transport: EndpointTransport,
    pub(super) endpoint: shared::Endpoint,
    pub(super) client: shared::client::Limits,
    pub(super) credential: shared::client::CredentialLimits,
    pub(super) name: domain::llm::Endpoint,
    pub(super) connect: Duration,
    pub(super) handshake: Duration,
    pub(super) oversized_reasoning: llm::OversizedReasoning,
}

pub(super) enum EndpointTransport {
    Plaintext,
    #[cfg_attr(
        not(unix),
        expect(
            dead_code,
            reason = "only the cfg(unix) trust loading reads these; the checks that build them are portable"
        )
    )]
    Tls {
        server_name: tls::Name,
        trust_der: Option<String>,
        trust: Option<document::TrustDocument>,
    },
}

pub(super) fn build_endpoints(
    endpoints: Vec<Endpoint>,
    declarations: &service::profile::Configuration,
    limits: &service::Limits,
    profile: &service::profile::Profile,
) -> Result<PreparedEndpoints, ConfigurationRefusal> {
    let mut domain_endpoints = Vec::with_capacity(endpoints.len());
    let mut channel_endpoints = List::with_capacity(ENDPOINTS);
    let mut llm_endpoints = Vec::with_capacity(endpoints.len());
    let mut endpoint_limits = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    let mut numbers = std::collections::BTreeSet::new();
    for (mut endpoint, declared) in endpoints.into_iter().zip(&declarations.endpoints) {
        let mut client =
            service::profile::endpoint_client(profile, declared, limits.llm.translation.client.history_items)
                .map_err(ConfigurationRefusal::Profile)?;
        if endpoint.name.is_empty()
            || endpoint.name.len()
                > usize::try_from(smith_charter::CEILINGS.llm_endpoint).expect("endpoint name bound fits usize")
        {
            return Err("endpoint name is empty or too long".into());
        }
        if !names.insert(endpoint.name.clone()) || !numbers.insert(endpoint.number) {
            return Err("duplicate endpoint name or number".into());
        }
        let transport = endpoint_transport(&mut endpoint)?;
        let mut llm_endpoint = match declared.provider {
            shared::Provider::OpenAi => shared::Endpoint::openai(),
            shared::Provider::Anthropic => shared::Endpoint::anthropic(),
        };
        if let Some(authority) = endpoint.authority {
            if authority.is_empty() || authority.contains(['\r', '\n']) {
                return Err("invalid endpoint HTTP authority".into());
            }
            llm_endpoint.authority = authority.into_bytes().into();
        }
        if let Some(target) = endpoint.target {
            if !target.starts_with('/') || target.contains(['\r', '\n']) {
                return Err("invalid endpoint HTTP target".into());
            }
            llm_endpoint.target = target.into_bytes().into();
        }
        let mut headers = Vec::with_capacity(endpoint.headers.len());
        for header in endpoint.headers {
            if header.name.eq_ignore_ascii_case("session-id")
                || header.name.eq_ignore_ascii_case("thread-id")
                || header.name.is_empty()
                || header.name.contains(['\r', '\n', ':'])
                || header.value.contains(['\r', '\n'])
            {
                return Err("invalid endpoint HTTP header".into());
            }
            headers.push(Header { name: header.name.into_bytes().into(), value: header.value.into_bytes().into() });
        }
        llm_endpoint.headers = headers.into_boxed_slice();
        domain_endpoints.push(run::charter::Endpoint(endpoint.number));
        channel_endpoints
            .push(smith_protocol_translate::Endpoint {
                name: endpoint.name.into_bytes().into(),
                number: endpoint.number,
                dialect: endpoint.dialect,
            })
            .map_err(|_| "too many channel endpoints")?;
        let protocol_allowance = service::profile::endpoint_allowance(profile, declared, &limits.domain.session.tools)
            .map_err(ConfigurationRefusal::Profile)?;
        endpoint_limits.push(domain::EndpointLimits {
            endpoint: run::charter::Endpoint(declared.number),
            window: declared.limits.window,
            output: declared.limits.output,
            protocol_allowance,
            head: declared.limits.head,
            idle: declared.limits.idle,
        });
        let credential = credential_limits(&limits.llm);
        let head = shared::client::request_head(&llm_endpoint, &credential, &client).map_err(|error| {
            ConfigurationRefusal::Profile(Refusal::Connection(connection::EndpointError::Client(error)))
        })?;
        service::profile::validate::request_head(endpoint.number, head, tls::client::MAX_PLAINTEXT)
            .map_err(ConfigurationRefusal::Profile)?;
        client.http.request = head;
        llm_endpoints.push(PreparedEndpoint {
            address: endpoint.address,
            transport,
            endpoint: llm_endpoint,
            client,
            credential,
            name: domain::llm::Endpoint(endpoint.number),
            connect: declared.connect,
            handshake: declared.handshake,
            oversized_reasoning: declared.limits.oversized_reasoning,
        });
    }
    Ok(PreparedEndpoints {
        domain: domain_endpoints.into_boxed_slice(),
        channel: smith_protocol_translate::Endpoints::new(channel_endpoints),
        llm: llm_endpoints.into_boxed_slice(),
        limits: endpoint_limits.into_boxed_slice(),
    })
}

fn endpoint_transport(endpoint: &mut Endpoint) -> Result<EndpointTransport, ConfigurationRefusal> {
    match endpoint.transport.as_str() {
        "tls" => {
            if endpoint.trust_der.is_some() && endpoint.trust.is_some() {
                return Err("endpoint cannot name both trust_der and trust".into());
            }
            Ok(EndpointTransport::Tls {
                server_name: tls::Name::new(
                    endpoint.server_name.as_deref().ok_or("TLS endpoint requires server_name")?,
                )
                .ok_or("invalid TLS server name")?,
                trust_der: endpoint.trust_der.take(),
                trust: endpoint.trust.take(),
            })
        }
        "plaintext" => {
            if let Ok(address) = endpoint.address.parse::<SocketAddr>()
                && !address.ip().is_loopback()
            {
                return Err("plaintext endpoint address must be loopback".into());
            }
            if endpoint.server_name.is_some() || endpoint.trust_der.is_some() || endpoint.trust.is_some() {
                return Err("plaintext endpoint cannot name TLS server_name or trust_der".into());
            }
            Ok(EndpointTransport::Plaintext)
        }
        _ => Err("endpoint transport must be tls or plaintext".into()),
    }
}

fn endpoint_duration(millis: u64, endpoint: u32, field: &'static str) -> Result<Duration, ConfigurationRefusal> {
    duration(millis, LimitName { layer: Layer::Policy, field, per: Per::Endpoint(endpoint) })
}

fn reasoning_policy(value: Option<&str>) -> Result<llm::OversizedReasoning, ConfigurationRefusal> {
    match value {
        None | Some("fail") => Ok(llm::OversizedReasoning::Fail),
        Some("drop") => Ok(llm::OversizedReasoning::Drop),
        Some(_) => Err("endpoint oversized_reasoning must be fail or drop".into()),
    }
}

pub(super) fn build_limits(endpoint: &Endpoint) -> Result<service::profile::Endpoint, ConfigurationRefusal> {
    let number = endpoint.number;
    let declared = &endpoint.limits;
    let (provider, dialect) = match endpoint.provider.as_str() {
        "openai" => (shared::Provider::OpenAi, 3),
        "anthropic" => (shared::Provider::Anthropic, 2),
        _ => return Err("endpoint provider must be openai or anthropic".into()),
    };
    if endpoint.dialect == 1 {
        return Err("endpoint provider must be openai or anthropic".into());
    }
    if endpoint.dialect != dialect {
        return Err("endpoint dialect does not match provider".into());
    }
    let option_bytes = llm::endpoint_own().map_err(|_| {
        ConfigurationRefusal::Profile(Refusal::Derived {
            limit: LimitName { layer: Layer::Service, field: "endpoint_own", per: Per::Endpoint(number) },
        })
    })?;
    if declared.window == 0 || declared.output == 0 || declared.reasoning_item == 0 {
        return Err("endpoint limits contain a zero quantity".into());
    }
    let per = Per::Endpoint(number);
    let limits = DeclaredLimits {
        window: declared.window,
        output: declared.output,
        reasoning_item: declared.reasoning_item,
        oversized_reasoning: reasoning_policy(declared.oversized_reasoning.as_deref())?,
        head: duration(declared.head, LimitName { layer: Layer::Declared, field: "head", per })?,
        idle: duration(declared.idle, LimitName { layer: Layer::Declared, field: "idle", per })?,
    };
    Ok(service::profile::Endpoint {
        number,
        provider,
        option_bytes,
        connect: endpoint_duration(endpoint.connect, number, "connect")?,
        handshake: endpoint_duration(endpoint.handshake, number, "handshake")?,
        limits,
    })
}

fn credential_limits(limits: &llm::ComponentLimits) -> shared::client::CredentialLimits {
    shared::client::CredentialLimits { value: limits.grant_value_bytes }
}
