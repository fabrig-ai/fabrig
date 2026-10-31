//! Native endpoint resolution and bounded trust loading before channel startup.
//! Contract: smith/docs/design/shell.md, section 2; smith/docs/design/protocol/agent.md, section 4.
use super::{ConfigurationRefusal, Endpoint};
use skein_tls as tls;
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

// Startup trust material and decoded configuration are bounded together.
/// Bound on startup trust material and decoded configuration, in bytes.
pub const TRUST_BYTES: u64 = 16_777_216;

/// Load the bounded startup trust roots used by agent and local endpoints.
#[cfg(unix)]
pub fn trust(der: Option<&str>) -> Result<tls::Config, String> {
    let source = match der {
        Some(path) => skein_shell::Trust::Der(path.into()),
        None => skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(Into::into) },
    };
    trust_source(&source)
}

/// Load account trust arrays through the existing bounded root owner.
#[cfg(unix)]
pub fn configured_trust(sources: &[skein_shell::Trust]) -> Result<tls::Config, String> {
    trust_sources(sources).map_err(ConfigurationRefusal::detail)
}

#[cfg(unix)]
pub(super) fn trust_source(source: &skein_shell::Trust) -> Result<tls::Config, String> {
    trust_sources(std::slice::from_ref(source)).map_err(ConfigurationRefusal::detail)
}

#[cfg(unix)]
pub(super) fn endpoint_trust(endpoint: &Endpoint) -> Result<tls::Config, ConfigurationRefusal> {
    let Some(document) = &endpoint.trust else {
        return trust(endpoint.trust_der.as_deref()).map_err(ConfigurationRefusal::Input);
    };
    if endpoint.trust_der.is_some() {
        return Err("endpoint cannot name both trust_der and trust".into());
    }
    let mut sources = Vec::new();
    if document.system {
        sources.push(skein_shell::Trust::Machine { ssl_cert_file: std::env::var_os("SSL_CERT_FILE").map(Into::into) });
    }
    sources.extend(document.der.iter().map(|path| skein_shell::Trust::Der(path.into())));
    sources.extend(document.pem.iter().map(|path| skein_shell::Trust::Machine { ssl_cert_file: Some(path.into()) }));
    trust_sources(&sources)
}

#[cfg(unix)]
fn trust_sources(sources: &[skein_shell::Trust]) -> Result<tls::Config, ConfigurationRefusal> {
    let mut certificates = Vec::new();
    let mut bytes = 0_u64;
    let mut malformed = 0_u32;
    let mut other_blocks = 0_u32;
    let mut paths = Vec::new();
    for source in sources {
        let bounds = skein_shell::RootBounds {
            bundle_bytes: u32::try_from(TRUST_BYTES).expect("trust bound fits u32"),
            certificates: 512,
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
                .checked_add(u64::try_from(certificate.len()).expect("bounded certificate"))
                .ok_or("trust roots exceed their byte bound")?;
            if bytes > TRUST_BYTES || certificates.len() >= 512 {
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
    eprintln!(
        "smith: trust roots: {} taken, {} skipped, {} malformed blocks, {} other blocks",
        parsed.taken, parsed.skipped, malformed, other_blocks
    );
    Ok(config)
}
