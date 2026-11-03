//! Pure endpoint preparation from declarations, before native resolution or trust loading.
//! It retains validated names, request settings and transport sources, never kernel handles.
//! Contract: smith/docs/design/protocol/agent.md, section 4; smith/docs/design/protocol/limits.md, sections 2–4.

use skein_http::Header;
use skein_lib::{Duration, List};
use skein_llm as shared;
use skein_llm_connection as connection;
use skein_tls as tls;
use smith_agent_service as service;
use smith_agent_service::profile::{ENDPOINTS, Layer, LimitName, Per, Refusal};
use smith_domain::{self as domain, run};
use smith_protocol_llm as llm;
use std::net::SocketAddr;

use super::{ConfigurationRefusal, Endpoint, ModelDocument, document, duration};

pub(super) struct PreparedEndpoints {
    pub(super) domain: Box<[run::charter::Endpoint]>,
    pub(super) channel: smith_protocol_translate::Endpoints,
    pub(super) llm: Box<[PreparedEndpoint]>,
    pub(super) models: Box<[domain::ConfiguredModel]>,
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
    pub(super) identity: llm::IdentityProfile,
    pub(super) models: Box<[llm::ConfiguredModel]>,
}

pub(super) enum EndpointTransport {
    Plaintext,
    Tls { server_name: tls::Name, trust_der: Option<String>, trust: Option<document::TrustDocument> },
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
    let mut models = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    let mut numbers = std::collections::BTreeSet::new();
    for (mut endpoint, declared) in endpoints.into_iter().zip(&declarations.endpoints) {
        let mut client = service::profile::endpoint_client(profile, declared, limits.llm.adapter.client.history_items)
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
            shared::Provider::OpenAiCodex => return Err("endpoint provider must be openai or anthropic".into()),
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
        let identity = llm::IdentityProfile::Plain;
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
        configure_models(declared, &endpoint.models, protocol_allowance, &mut models);
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
            identity,
            models: endpoint_models(&endpoint.models)?,
        });
    }
    Ok(PreparedEndpoints {
        domain: domain_endpoints.into_boxed_slice(),
        channel: smith_protocol_translate::Endpoints::new(channel_endpoints),
        llm: llm_endpoints.into_boxed_slice(),
        models: models.into_boxed_slice(),
    })
}

fn configure_models(
    declared: &service::profile::Endpoint,
    inputs: &[ModelDocument],
    protocol_allowance: u64,
    models: &mut Vec<domain::ConfiguredModel>,
) {
    for (model, input) in declared.models.iter().zip(inputs) {
        models.push(domain::ConfiguredModel {
            prices: input.prices.rates(),
            endpoint: run::charter::Endpoint(declared.number),
            model: model.name.clone(),
            efforts: model.efforts.clone(),
            window: model.window,
            output: model.output,
            protocol_allowance,
            head: model.head,
            idle: model.idle,
        });
    }
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

fn endpoint_models(models: &[ModelDocument]) -> Result<Box<[llm::ConfiguredModel]>, ConfigurationRefusal> {
    let mut configured = Vec::with_capacity(models.len());
    for model in models {
        configured.push(llm::ConfiguredModel {
            name: model.name.as_bytes().into(),
            oversized_reasoning: reasoning_policy(model.oversized_reasoning.as_deref())?,
        });
    }
    Ok(configured.into_boxed_slice())
}

fn endpoint_duration(millis: u64, endpoint: u32, field: &'static str) -> Result<Duration, ConfigurationRefusal> {
    duration(millis, LimitName { layer: Layer::Policy, field, per: Per::Endpoint(endpoint) })
}

fn reasoning_policy(value: Option<&str>) -> Result<llm::OversizedReasoning, ConfigurationRefusal> {
    match value {
        None | Some("fail") => Ok(llm::OversizedReasoning::Fail),
        Some("drop") => Ok(llm::OversizedReasoning::Drop),
        Some(_) => Err("model oversized_reasoning must be fail or drop".into()),
    }
}

pub(super) fn build_models(endpoint: &Endpoint) -> Result<service::profile::Endpoint, ConfigurationRefusal> {
    let number = endpoint.number;
    let models = &endpoint.models;
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
    let identity = llm::IdentityProfile::Plain;
    let option_bytes = llm::endpoint_own(identity).map_err(|_| {
        ConfigurationRefusal::Profile(Refusal::Derived {
            limit: LimitName { layer: Layer::Service, field: "endpoint_own", per: Per::Endpoint(number) },
        })
    })?;
    if models.is_empty() {
        return Err(ConfigurationRefusal::Profile(Refusal::Field {
            limit: LimitName { layer: Layer::Declared, field: "models", per: Per::Endpoint(number) },
        }));
    }
    let mut names = std::collections::BTreeSet::new();
    let mut prepared = Vec::with_capacity(models.len());
    for (index, model) in models.iter().enumerate() {
        if model.efforts.iter().any(|name| {
            name.is_empty()
                || name.len()
                    > usize::try_from(smith_charter::CEILINGS.llm_effort_some).expect("effort bound fits usize")
        }) {
            return Err("model efforts must be names within 32 bytes".into());
        }
        let index = u32::try_from(index).map_err(|_| {
            ConfigurationRefusal::Profile(Refusal::Derived {
                limit: LimitName::deployment(Layer::Service, "configured_models"),
            })
        })?;
        let per = Per::Model { endpoint: number, model: index };
        if model.name.is_empty()
            || model.name.len()
                > usize::try_from(smith_charter::CEILINGS.llm_model).expect("model name bound fits usize")
            || model.window == 0
            || model.output == 0
            || model.reasoning_item == 0
        {
            return Err("model declaration contains an empty or zero quantity".into());
        }
        if !names.insert(model.name.clone()) {
            return Err("duplicate declared model".into());
        }
        model.prices.rates().check().map_err(|_| "model prices must be positive")?;
        prepared.push(service::profile::Model {
            name: model.name.as_bytes().into(),
            efforts: model.efforts.iter().map(|name| name.as_bytes().into()).collect(),
            window: model.window,
            output: model.output,
            reasoning_item: model.reasoning_item,
            oversized_reasoning: reasoning_policy(model.oversized_reasoning.as_deref())?,
            head: duration(model.head, LimitName { layer: Layer::Declared, field: "head", per })?,
            idle: duration(model.idle, LimitName { layer: Layer::Declared, field: "idle", per })?,
        });
    }
    Ok(service::profile::Endpoint {
        number,
        provider,
        option_bytes,
        connect: endpoint_duration(endpoint.connect, number, "connect")?,
        handshake: endpoint_duration(endpoint.handshake, number, "handshake")?,
        models: prepared.into_boxed_slice(),
    })
}

fn credential_limits(limits: &llm::ComponentLimits) -> shared::client::CredentialLimits {
    shared::client::CredentialLimits {
        value: limits.grant_value_bytes,
        account_id: limits.grant_value_bytes.min(smith_channel::CEILINGS.withdrawn_value_id),
    }
}
