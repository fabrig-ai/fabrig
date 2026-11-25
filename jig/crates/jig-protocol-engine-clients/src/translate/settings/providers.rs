//! Whole provider values at the codec edge (jig/docs/design/domain/engine/llm-providers.md, 6).
use jig_client_protocol_wire as wire;
use jig_domain_core_vocabulary::settings::providers as p;
use skein_codec::Reason;
use skein_lib::length::usize_of;

// These lists refuse before measuring the settings value, with the provider
// setting's path. Check borrowed counts before the map copies their contents.
pub(super) fn check(value: &p::ProviderSetting, limits: &wire::Limits) -> Result<(), wire::Problem> {
    let problem = wire::paths::PROVIDER_SETTING_ENDPOINTS.problem(Reason::Bound);
    if let Some(list) = &value.provider.list
        && (list.keep.len() > usize_of(limits.provider_list_keep)
            || list.drop.len() > usize_of(limits.provider_list_drop))
    {
        return Err(problem);
    }
    if let Some(registration) = &value.provider.sign_in
        && (registration.scopes.len() > usize_of(limits.provider_registration_scopes)
            || registration.must_grant.len() > usize_of(limits.provider_registration_must_grant))
    {
        return Err(problem);
    }
    if value.provider.programs.len() > usize_of(limits.provider_programs)
        || value.endpoints.len() > usize_of(limits.provider_setting_endpoints)
    {
        return Err(problem);
    }
    for endpoint in &value.endpoints {
        if endpoint.headers.len() > usize_of(limits.provider_endpoint_headers) {
            return Err(problem);
        }
    }
    Ok(())
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
