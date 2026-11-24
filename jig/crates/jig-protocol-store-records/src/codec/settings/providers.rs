//! Socket and host addresses at the settings boundary (jig/docs/design/domain/engine/llm-providers.md, 6).
use crate::settings_v3 as wire;
use jig_domain_core_vocabulary::settings::providers as p;
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
