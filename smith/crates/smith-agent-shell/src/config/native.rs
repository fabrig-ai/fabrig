//! Native endpoint resolution and bounded trust loading before channel startup.
//! Contract: smith/docs/design/shell.md, section 2; smith/docs/design/protocol/agent.md, section 4.
#[cfg(unix)]
use super::document::TrustDocument;
use super::{
    ConfigurationRefusal,
    endpoints::{EndpointTransport, PreparedEndpoint},
};
use skein_llm_connection as connection;
use skein_tls as tls;
use smith_protocol_llm as llm;
use std::io::Write;
use std::net::SocketAddr;
#[cfg(unix)]
use std::net::ToSocketAddrs;

#[cfg(unix)]
pub(super) fn resolve(address: &str) -> Result<SocketAddr, ConfigurationRefusal> {
    let mut resolved = address.to_socket_addrs().map_err(|error| format!("endpoint address {address:?}: {error}"))?;
    resolved.next().ok_or_else(|| ConfigurationRefusal::Input(format!("endpoint address {address:?} has no result")))
}

#[cfg(not(unix))]
pub(super) fn resolve(address: &str) -> Result<SocketAddr, ConfigurationRefusal> {
    address.parse().map_err(|error| format!("endpoint address {address:?}: {error}; no name resolution").into())
}

pub(super) fn endpoints(
    endpoints: Box<[PreparedEndpoint]>,
    limits: &connection::Limits,
    errors: &mut dyn Write,
) -> Result<Box<[llm::ConfiguredEndpoint]>, ConfigurationRefusal> {
    let mut configured = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        let address = resolve(&endpoint.address)?;
        let transport = match endpoint.transport {
            EndpointTransport::Plaintext => {
                if !address.ip().is_loopback() {
                    return Err("plaintext endpoint address must be loopback".into());
                }
                connection::Transport::Plaintext
            }
            #[cfg(unix)]
            EndpointTransport::Tls { server_name, trust_der, trust } => connection::Transport::Tls {
                server_name,
                trust: endpoint_trust(trust_der.as_deref(), trust.as_ref(), errors)?,
            },
            #[cfg(not(unix))]
            EndpointTransport::Tls { .. } => return Err("TLS requires a native shell".into()),
        };
        let mut destination = connection::Endpoint {
            address,
            transport,
            llm: endpoint.endpoint,
            limits: endpoint.client,
            credential: endpoint.credential,
        };
        destination.pieces(limits);
        configured.push(llm::ConfiguredEndpoint {
            name: endpoint.name,
            destination,
            connect: endpoint.connect,
            handshake: endpoint.handshake,
            identity: endpoint.identity,
            models: endpoint.models,
        });
    }
    Ok(configured.into_boxed_slice())
}

// Startup trust material and decoded configuration are bounded together.
/// Bound on startup trust material and decoded configuration, in bytes.
pub const TRUST_BYTES: u64 = 16_777_216;

const CERTIFICATES: u32 = 512;

/// Load the bounded startup trust roots used by agent and local endpoints.
#[cfg(unix)]
pub fn trust(der: Option<&str>) -> Result<tls::Config, String> {
    trust_with(der, &mut std::io::stderr())
}

#[cfg(unix)]
fn trust_with(der: Option<&str>, errors: &mut dyn Write) -> Result<tls::Config, String> {
    let source = match der {
        Some(path) => skein_shell::Trust::Der(path.into()),
        None => skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(Into::into) },
    };
    trust_sources(std::slice::from_ref(&source), errors).map_err(ConfigurationRefusal::detail)
}

/// Load account trust arrays through the existing bounded root owner.
#[cfg(unix)]
pub fn configured_trust(sources: &[skein_shell::Trust]) -> Result<tls::Config, String> {
    trust_sources(sources, &mut std::io::stderr()).map_err(ConfigurationRefusal::detail)
}

#[cfg(all(test, unix))]
pub(super) fn trust_source(source: &skein_shell::Trust) -> Result<tls::Config, String> {
    trust_sources(std::slice::from_ref(source), &mut std::io::stderr()).map_err(ConfigurationRefusal::detail)
}

#[cfg(unix)]
fn endpoint_trust(
    der: Option<&str>,
    document: Option<&TrustDocument>,
    errors: &mut dyn Write,
) -> Result<tls::Config, ConfigurationRefusal> {
    let Some(document) = document else {
        return trust_with(der, errors).map_err(ConfigurationRefusal::Input);
    };
    let mut sources = Vec::new();
    if document.system {
        sources.push(skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(Into::into) });
    }
    sources.extend(document.der.iter().map(|path| skein_shell::Trust::Der(path.into())));
    sources.extend(document.pem.iter().map(|path| skein_shell::Trust::Machine { ssl_cert_file: Some(path.into()) }));
    trust_sources(&sources, errors)
}

#[cfg(unix)]
fn trust_sources(sources: &[skein_shell::Trust], errors: &mut dyn Write) -> Result<tls::Config, ConfigurationRefusal> {
    let mut certificates = Vec::new();
    let mut bytes = 0_u64;
    let mut malformed = 0_u32;
    let mut other_blocks = 0_u32;
    let mut paths = Vec::new();
    for source in sources {
        let bounds = skein_shell::RootBounds {
            bundle_bytes: u32::try_from(TRUST_BYTES).expect("trust bound fits u32"),
            certificates: CERTIFICATES,
            certificate_bytes: match source {
                skein_shell::Trust::Der(_) => 65_536,
                skein_shell::Trust::Machine { .. } => 8192,
            },
        };
        let roots = skein_shell::read_trust_roots(source, &bounds).map_err(|why| {
            ConfigurationRefusal::Input(match why {
                skein_shell::RootsRefusal::NoBundle => "machine trust roots: no certificate bundle found".into(),
                skein_shell::RootsRefusal::Unread { path, why } => {
                    format!("trust certificate {}: {why:?}", path.display())
                }
                skein_shell::RootsRefusal::TooMany { bound } => format!("trust roots exceed {bound} certificates"),
                skein_shell::RootsRefusal::TooLong { index, bound } => {
                    format!("trust certificate {index} exceeds {bound} bytes")
                }
            })
        })?;
        for certificate in roots.certificates {
            bytes = bytes
                .checked_add(skein_lib::len_u64(certificate.len()))
                .ok_or("trust roots exceed their byte bound")?;
            if bytes > TRUST_BYTES
                || certificates.len() >= usize::try_from(CERTIFICATES).expect("certificate bound fits usize")
            {
                return Err("trust roots exceed their combined bound".into());
            }
            certificates.push(certificate);
        }
        malformed = malformed.checked_add(roots.malformed).ok_or("trust block count overflow")?;
        other_blocks = other_blocks.checked_add(roots.other_blocks).ok_or("trust block count overflow")?;
        paths.push(roots.path.display().to_string());
    }
    let (config, parsed) =
        tls::Config::from_der(&certificates, &[]).map_err(|why| format!("TLS trust {}: {why:?}", paths.join(", ")))?;
    drop(writeln!(
        errors,
        "smith: trust roots: {} taken, {} skipped, {} malformed blocks, {} other blocks",
        parsed.taken, parsed.skipped, malformed, other_blocks
    ));
    Ok(config)
}
