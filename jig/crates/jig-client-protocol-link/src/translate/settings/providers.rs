//! Whole provider values at the codec edge (jig/docs/design/domain/engine/llm-providers.md, 6).
use crate::generated::client;
use jig_client_domain::contract as p;
use jig_client_protocol_wire as wire;
use skein_lib::length::usize_of;
fn problem(reason: skein_codec::Reason) -> wire::Problem {
    wire::Problem { path: wire::paths::PROVIDER_SETTING_ENDPOINTS, reason }
}
// These counts are admitted here, before copying the fields; all of them
// report at the enclosing provider setting's endpoints path.
fn list_fits(value: &p::ProviderList, limits: &wire::Limits) -> bool {
    value.keep.len() <= usize_of(limits.provider_list_keep) && value.drop.len() <= usize_of(limits.provider_list_drop)
}

fn registration_fits(value: &p::Registration, limits: &wire::Limits) -> bool {
    value.scopes.len() <= usize_of(limits.provider_registration_scopes)
        && value.must_grant.len() <= usize_of(limits.provider_registration_must_grant)
}

pub(super) fn encode(
    value: &p::ProviderSetting,
    limits: &wire::Limits,
) -> Result<wire::ProviderSetting, wire::Problem> {
    if let Some(list) = &value.provider.list
        && !list_fits(list, limits)
    {
        return Err(problem(skein_codec::Reason::Bound));
    }
    if let Some(registration) = &value.provider.sign_in
        && !registration_fits(registration, limits)
    {
        return Err(problem(skein_codec::Reason::Bound));
    }
    if value.provider.programs.len() > usize_of(limits.provider_programs)
        || value.endpoints.len() > usize_of(limits.provider_setting_endpoints)
    {
        return Err(problem(skein_codec::Reason::Bound));
    }
    for endpoint in &value.endpoints {
        if endpoint.headers.len() > usize_of(limits.provider_endpoint_headers) {
            return Err(problem(skein_codec::Reason::Bound));
        }
    }
    client::provider_setting_to_record(value)
}

pub(crate) fn provider_address_to_record(value: &p::EndpointAddress) -> wire::ProviderAddress {
    match value {
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
    }
}
pub(crate) fn provider_address_to_domain(value: wire::ProviderAddress) -> p::EndpointAddress {
    match value {
        wire::ProviderAddress::Host(value) => p::EndpointAddress::Host { name: value.name, port: value.port },
        wire::ProviderAddress::Ipv4(value) => p::EndpointAddress::Socket(core::net::SocketAddr::V4(
            core::net::SocketAddrV4::new(core::net::Ipv4Addr::from(value.ip), value.port),
        )),
        wire::ProviderAddress::Ipv6(value) => {
            p::EndpointAddress::Socket(core::net::SocketAddr::V6(core::net::SocketAddrV6::new(
                core::net::Ipv6Addr::from(value.ip),
                value.port,
                value.flowinfo,
                value.scope_id,
            )))
        }
    }
}
