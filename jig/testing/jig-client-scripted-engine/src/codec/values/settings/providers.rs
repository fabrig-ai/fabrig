//! Whole provider values at the codec edge (jig/docs/design/domain/engine/llm-providers.md, 6).
use jig_client_domain::contract as p;
use jig_client_protocol_wire as wire;
use skein_lib::List;
fn problem(reason: skein_codec::Reason) -> wire::Problem {
    wire::Problem { path: wire::paths::PROVIDER_SETTING_ENDPOINTS, reason }
}
fn encode_provider_list(value: &p::ProviderList, limits: &wire::Limits) -> Result<wire::ProviderList, wire::Problem> {
    let mut keep = List::with_capacity(limits.provider_list_keep);
    for item in &value.keep {
        let item = wire::ProviderText { bytes: item.clone() };
        if keep.push(item).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    let mut drop = List::with_capacity(limits.provider_list_drop);
    for item in &value.drop {
        let item = wire::ProviderText { bytes: item.clone() };
        if drop.push(item).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    Ok(wire::ProviderList { keep, drop })
}

fn encode_provider_registration(
    value: &p::Registration,
    limits: &wire::Limits,
) -> Result<wire::ProviderRegistration, wire::Problem> {
    let issuer = value.issuer.clone();
    let client = value.client.clone();
    let mut scopes = List::with_capacity(limits.provider_registration_scopes);
    for item in &value.scopes {
        let item = wire::ProviderText { bytes: item.clone() };
        if scopes.push(item).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    let name_hint = value.name_hint.clone();
    let authorization = value.authorization.clone();
    let token = value.token.clone();
    let revocation = value.revocation.clone();
    let keys = value.keys.clone();
    let endpoint_transport = encode_provider_transport(&value.endpoint_transport, limits)?;
    let mut must_grant = List::with_capacity(limits.provider_registration_must_grant);
    for item in &value.must_grant {
        let item = wire::ProviderText { bytes: item.clone() };
        if must_grant.push(item).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    let resource = value.resource.clone();
    let redirect_path = value.redirect_path.clone();
    let redirect_port = value.redirect_port;
    Ok(wire::ProviderRegistration {
        issuer,
        client,
        scopes,
        name_hint,
        authorization,
        token,
        revocation,
        keys,
        endpoint_transport,
        must_grant,
        resource,
        redirect_path,
        redirect_port,
    })
}

fn encode_provider(value: &p::Provider, limits: &wire::Limits) -> Result<wire::Provider, wire::Problem> {
    let name = value.name.clone();
    let number = value.number;
    let endpoint = value.endpoint.clone();
    let direct = value.direct.clone();
    let list = match &value.list {
        Some(value) => Some(encode_provider_list(value, limits)?),
        None => None,
    };
    let signed_in_plan = value.signed_in_plan;
    let manage_usage = value.manage_usage.clone();
    let sign_in = match &value.sign_in {
        Some(value) => Some(encode_provider_registration(value, limits)?),
        None => None,
    };
    let keys = value.keys;
    let mut programs = List::with_capacity(limits.provider_programs);
    for item in &value.programs {
        let item = wire::ProviderText { bytes: item.clone() };
        if programs.push(item).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    Ok(wire::Provider { name, number, endpoint, direct, list, signed_in_plan, manage_usage, sign_in, keys, programs })
}

fn encode_provider_header(value: &p::Header, _limits: &wire::Limits) -> Result<wire::ProviderHeader, wire::Problem> {
    let name = value.name.clone();
    let value = value.value.clone();
    Ok(wire::ProviderHeader { name, value })
}

fn encode_provider_endpoint(
    value: &p::Endpoint,
    limits: &wire::Limits,
) -> Result<wire::ProviderEndpoint, wire::Problem> {
    let name = value.name.clone();
    let number = value.number;
    let dialect = value.dialect;
    let provider = value.provider.clone();
    let program = value.program.clone();
    let address = encode_provider_address(&value.address, limits)?;
    let transport = encode_provider_transport(&value.transport, limits)?;
    let mut headers = List::with_capacity(limits.provider_endpoint_headers);
    for item in &value.headers {
        let item = encode_provider_header(item, limits)?;
        if headers.push(item).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    let connect = value.connect;
    let handshake = value.handshake;
    Ok(wire::ProviderEndpoint {
        name,
        number,
        dialect,
        provider,
        program,
        address,
        transport,
        headers,
        connect,
        handshake,
    })
}

fn encode_provider_transport(
    value: &p::Transport,
    _limits: &wire::Limits,
) -> Result<wire::ProviderTransport, wire::Problem> {
    Ok(match value {
        p::Transport::Plaintext => wire::ProviderTransport::Plaintext,
        p::Transport::Tls { server_name, trust } => {
            wire::ProviderTransport::Tls(wire::ProviderTls { server_name: server_name.clone(), trust: trust.clone() })
        }
    })
}

fn encode_provider_address(
    value: &p::EndpointAddress,
    _limits: &wire::Limits,
) -> Result<wire::ProviderAddress, wire::Problem> {
    Ok(match value {
        p::EndpointAddress::Host { name, port } => {
            wire::ProviderAddress::Host(wire::ProviderHost { name: name.clone(), port: *port })
        }
        p::EndpointAddress::Socket(core::net::SocketAddr::V4(value)) => {
            wire::ProviderAddress::Ipv4(wire::ProviderIpv4 { ip: value.ip().octets(), port: value.port() })
        }
        p::EndpointAddress::Socket(core::net::SocketAddr::V6(value)) => {
            wire::ProviderAddress::Ipv6(wire::ProviderIpv6 {
                ip: value.ip().octets(),
                port: value.port(),
                flowinfo: value.flowinfo(),
                scope_id: value.scope_id(),
            })
        }
    })
}

pub(super) fn encode(
    value: &p::ProviderSetting,
    limits: &wire::Limits,
) -> Result<wire::ProviderSetting, wire::Problem> {
    let provider = encode_provider(&value.provider, limits)?;
    let mut endpoints = List::with_capacity(limits.provider_setting_endpoints);
    for endpoint in &value.endpoints {
        let endpoint = encode_provider_endpoint(endpoint, limits)?;
        if endpoints.push(endpoint).is_err() {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    Ok(wire::ProviderSetting { provider, endpoints, withdrawn: value.withdrawn })
}
