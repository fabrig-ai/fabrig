//! Workers' figures (temper/docs/design/protocol/configuration/figures.md, 2).
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, llm::EndpointAddress};
use jig_protocol_worker_wire::v17 as w;
use skein_lib::{Decimal, Duration, List, bytes};
use temper_client_protocol_wire::v1 as s;

/// The most endpoints workers' figures carry.
pub const ENDPOINTS: u32 = w::CEILINGS.agents_endpoints;

/// Compose the engine's startup limits and endpoints with its current agents setting.
/// Startup checks the endpoint count; composition takes at most the wire's bound.
#[must_use]
pub fn compose(engine: &settings::Engine, agents: &s::AgentFigures) -> w::Figures {
    Composer::new(&engine.jig, &engine.limits).compose(agents)
}

/// The agent's call bound, independent of the carriers' `tool_payload` floor.
#[must_use]
pub fn tool_payload(agents: &s::AgentFigures) -> u32 {
    for figure in &agents.profile {
        if figure.name.as_ref() == b"declared.tool_payload" {
            return u32::try_from(figure.value).expect("judged agent payload fits u32");
        }
    }
    temper_service_core::derived::STOCK.payload
}

/// Startup values retained while the stored agents setting changes.
#[derive(Debug)]
pub struct Composer {
    limits: List<w::Limit>,
    endpoints: Box<[config::llm::Endpoint]>,
}
impl Composer {
    #[must_use]
    pub fn new(engine: &config::Engine, own: &limits::Limits) -> Self {
        let mut values = jig_protocol_config::overrides::list(&engine.limits);
        for &value in &limits::list(own) {
            values.push(value).expect("uncapped startup limits");
        }
        let mut limits = List::with_capacity(values.len());
        for &(name, value) in &values {
            limits.push(limit(name.as_bytes(), value)).expect("counted startup limits");
        }
        Self { limits, endpoints: engine.llm.endpoints.clone() }
    }

    #[must_use]
    pub fn compose(&self, agents: &s::AgentFigures) -> w::Figures {
        let mut profile = List::with_capacity(agents.profile.len());
        for figure in &agents.profile {
            profile.push(limit(&figure.name, figure.value)).expect("counted profile figures");
        }
        let mut endpoints =
            List::with_capacity(u32::try_from(self.endpoints.len()).expect("configured endpoints").min(ENDPOINTS));
        for endpoint in self.endpoints.iter().take(skein_lib::usize_of(ENDPOINTS)) {
            let mut limits = &agents.fallback;
            for named in &agents.endpoints {
                if named.endpoint.as_ref() == endpoint.name.as_ref() {
                    limits = &named.limits;
                    break;
                }
            }
            endpoints.push(endpoint_figure(endpoint, limits)).expect("counted endpoints");
        }
        let agents =
            w::Agents::new(&w::CEILINGS, w::AgentsParts { profile, endpoints }).expect("configured agents fit figures");
        w::Figures::new(&w::CEILINGS, w::FiguresParts { places: 0, limits: self.limits.clone(), agents })
            .expect("configured limits fit figures")
    }
}

fn limit(name: &[u8], value: u64) -> w::Limit {
    w::Limit::new(&w::CEILINGS, w::LimitParts { name: bytes::copy_of(name), value }).expect("limit name fits figures")
}

fn endpoint_figure(endpoint: &config::llm::Endpoint, limits: &s::AgentLimits) -> w::AgentEndpoint {
    let (host, port) = match &endpoint.address {
        EndpointAddress::Host { name, port } => (name.clone(), *port),
        EndpointAddress::Socket(socket) => (ip(socket.ip()), socket.port()),
    };
    let transport = match &endpoint.transport {
        config::llm::Transport::Plaintext => w::Transport::Plaintext,
        config::llm::Transport::Tls { server_name, trust } => w::Transport::Tls(
            w::Tls::new(&w::CEILINGS, w::TlsParts { server_name: server_name.clone(), trust: trust.clone() })
                .expect("configured TLS fits figures"),
        ),
    };
    let mut headers = List::with_capacity(u32::try_from(endpoint.headers.len()).expect("configured headers"));
    for header in &endpoint.headers {
        headers
            .push(
                w::Header::new(&w::CEILINGS, w::HeaderParts { name: header.name.clone(), value: header.value.clone() })
                    .expect("configured header fits figures"),
            )
            .expect("counted headers");
    }
    let limits = w::EndpointLimits::new(
        &w::CEILINGS,
        w::EndpointLimitsParts {
            window: limits.window,
            output: limits.output,
            reasoning_item: limits.reasoning_item,
            head: Duration::from_millis(limits.head_ms),
            idle: Duration::from_millis(limits.idle_ms),
            oversized: match &limits.oversized {
                s::Oversized::Fail => w::Oversized::Fail,
                s::Oversized::Drop => w::Oversized::Drop,
            },
        },
    )
    .expect("endpoint limits fit figures");
    w::AgentEndpoint::new(
        &w::CEILINGS,
        w::AgentEndpointParts {
            name: endpoint.name.clone(),
            number: endpoint.number,
            dialect: endpoint.dialect,
            provider: endpoint.provider.clone(),
            program: endpoint.program.clone(),
            host,
            port,
            transport,
            headers,
            connect: endpoint.connect,
            handshake: endpoint.handshake,
            limits,
        },
    )
    .expect("configured endpoint fits figures")
}

fn ip(address: core::net::IpAddr) -> Box<[u8]> {
    let mut out = List::with_capacity(39);
    match address {
        core::net::IpAddr::V4(ip) => {
            for (at, octet) in ip.octets().iter().enumerate() {
                if at != 0 {
                    out.push(b'.').expect("IPv4 fits 39 bytes");
                }
                for byte in Decimal::of(u64::from(*octet)).as_bytes() {
                    out.push(*byte).expect("IPv4 fits 39 bytes");
                }
            }
        }
        core::net::IpAddr::V6(ip) => {
            for (at, segment) in ip.segments().iter().enumerate() {
                if at != 0 {
                    out.push(b':').expect("IPv6 fits 39 bytes");
                }
                let mut begun = false;
                for shift in [12_u32, 8, 4, 0] {
                    let digit = u8::try_from((segment >> shift) & 15).expect("hex digit");
                    if begun || digit != 0 || shift == 0 {
                        begun = true;
                        out.push(bytes::hex_digit(digit)).expect("IPv6 fits 39 bytes");
                    }
                }
            }
        }
    }
    out.into_boxed()
}
