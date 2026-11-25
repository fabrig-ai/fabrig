//! Native endpoint resolution and bounded trust loading before channel startup.
//! Contract: smith/docs/design/shell/README.md, section 2.6; smith/docs/design/protocol/agent.md, section 4.
use super::document::TrustDocument;
use super::{
    ConfigurationRefusal,
    endpoints::{EndpointTransport, PreparedEndpoint},
};
use skein_llm_connection as connection;
#[cfg(unix)]
use skein_tls as tls;
use smith_protocol_llm as llm;
use std::io::Write;
use std::net::SocketAddr;
#[cfg(unix)]
use std::net::ToSocketAddrs;

/// Find the first executable rg file in an absolute directory on the commands' PATH.
/// This reads metadata only; listings and searches run the result with no environment.
#[cfg(unix)]
#[must_use]
pub fn find_rg(path: Option<&[u8]>) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    find_rg_with(path, |file| {
        std::fs::metadata(file)
            .ok()
            .map(|metadata| FileFacts { is_file: metadata.is_file(), mode: metadata.permissions().mode() })
    })
}

#[cfg(unix)]
pub(super) struct FileFacts {
    pub(super) is_file: bool,
    pub(super) mode: u32,
}

#[cfg(unix)]
pub(super) fn find_rg_with(
    path: Option<&[u8]>,
    mut facts: impl FnMut(&std::path::Path) -> Option<FileFacts>,
) -> Option<std::path::PathBuf> {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    std::env::split_paths(OsStr::from_bytes(path?))
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join("rg"))
        .find(|file| facts(file).is_some_and(|file| file.is_file && file.mode & 0o111 != 0))
}

#[cfg(unix)]
pub(super) fn resolve(address: &str) -> Result<SocketAddr, ConfigurationRefusal> {
    let mut resolved = address.to_socket_addrs().map_err(|error| format!("endpoint address {address:?}: {error}"))?;
    resolved.next().ok_or_else(|| ConfigurationRefusal::Input(format!("endpoint address {address:?} has no result")))
}

#[cfg(not(unix))]
pub(super) fn resolve(address: &str) -> Result<SocketAddr, ConfigurationRefusal> {
    address.parse().map_err(|error| format!("endpoint address {address:?}: {error}; no name resolution").into())
}

#[cfg(unix)]
pub(super) fn endpoints(
    endpoints: Box<[PreparedEndpoint]>,
    limits: &connection::Limits,
    errors: &mut dyn Write,
    #[cfg(unix)] mut roots: RootReader<'_>,
) -> Result<Box<[llm::ConfiguredEndpoint]>, ConfigurationRefusal> {
    endpoints_with(endpoints, limits, resolve, |der, trust| endpoint_trust(der, trust, errors, &mut roots))
}

pub(super) fn endpoints_with(
    endpoints: Box<[PreparedEndpoint]>,
    limits: &connection::Limits,
    address: impl Fn(&str) -> Result<SocketAddr, ConfigurationRefusal>,
    mut trust: impl FnMut(Option<&str>, Option<&TrustDocument>) -> Result<skein_tls::Config, ConfigurationRefusal>,
) -> Result<Box<[llm::ConfiguredEndpoint]>, ConfigurationRefusal> {
    let mut configured = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        let address = address(&endpoint.address)?;
        let transport = match endpoint.transport {
            EndpointTransport::Plaintext => {
                if !address.ip().is_loopback() {
                    return Err("plaintext endpoint address must be loopback".into());
                }
                connection::Transport::Plaintext
            }
            EndpointTransport::Tls { server_name, trust_der, trust: roots } => {
                connection::Transport::Tls { server_name, trust: trust(trust_der.as_deref(), roots.as_ref())? }
            }
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
            oversized_reasoning: endpoint.oversized_reasoning,
        });
    }
    Ok(configured.into_boxed_slice())
}

// Startup trust material and decoded configuration are bounded together.
/// Bound on startup trust material and decoded configuration, in bytes.
pub const TRUST_BYTES: u64 = 16_777_216;

#[cfg(unix)]
const CERTIFICATES: u32 = 512;

#[cfg(unix)]
type ReadRoots<'a> = dyn FnMut(&skein_shell::Trust, &skein_shell::RootBounds) -> Result<skein_shell::Roots, skein_shell::RootsRefusal>
    + 'a;

#[cfg(unix)]
pub(super) enum RootReader<'a> {
    Native,
    Given(&'a mut ReadRoots<'a>),
}

#[cfg(unix)]
impl RootReader<'_> {
    fn machine(&self) -> skein_shell::Trust {
        let ssl_cert_file = match self {
            Self::Native => std::env::var_os("SSL_CERT_FILE").map(Into::into),
            Self::Given(_) => None,
        };
        skein_shell::Trust::Machine { ssl_cert_file }
    }

    fn read(
        &mut self,
        source: &skein_shell::Trust,
        bounds: &skein_shell::RootBounds,
    ) -> Result<skein_shell::Roots, skein_shell::RootsRefusal> {
        match self {
            Self::Native => skein_shell::read_trust_roots(source, bounds),
            Self::Given(read) => read(source, bounds),
        }
    }
}

/// Load the bounded startup trust roots used by agent and local endpoints.
#[cfg(unix)]
pub fn trust(der: Option<&str>) -> Result<tls::Config, String> {
    trust_with(der, &mut std::io::stderr(), &mut RootReader::Native)
}

#[cfg(unix)]
fn trust_with(der: Option<&str>, errors: &mut dyn Write, roots: &mut RootReader<'_>) -> Result<tls::Config, String> {
    let source = match der {
        Some(path) => skein_shell::Trust::Der(path.into()),
        None => roots.machine(),
    };
    trust_sources(std::slice::from_ref(&source), errors, roots).map_err(ConfigurationRefusal::detail)
}

/// Load account trust arrays through the existing bounded root owner.
#[cfg(unix)]
pub fn configured_trust(sources: &[skein_shell::Trust]) -> Result<tls::Config, String> {
    trust_sources(sources, &mut std::io::stderr(), &mut RootReader::Native).map_err(ConfigurationRefusal::detail)
}

#[cfg(all(test, unix))]
pub(super) fn trust_source(source: &skein_shell::Trust) -> Result<tls::Config, String> {
    configured_trust(std::slice::from_ref(source))
}

#[cfg(unix)]
pub(super) fn endpoint_trust(
    der: Option<&str>,
    document: Option<&TrustDocument>,
    errors: &mut dyn Write,
    roots: &mut RootReader<'_>,
) -> Result<tls::Config, ConfigurationRefusal> {
    let Some(document) = document else {
        return trust_with(der, errors, roots).map_err(ConfigurationRefusal::Input);
    };
    let mut sources = Vec::new();
    if document.system {
        sources.push(roots.machine());
    }
    sources.extend(document.der.iter().map(|path| skein_shell::Trust::Der(path.into())));
    sources.extend(document.pem.iter().map(|path| skein_shell::Trust::Machine { ssl_cert_file: Some(path.into()) }));
    trust_sources(&sources, errors, roots)
}

#[cfg(not(unix))]
pub(super) fn endpoint_trust(
    _der: Option<&str>,
    _document: Option<&TrustDocument>,
    _errors: &mut dyn Write,
) -> Result<skein_tls::Config, ConfigurationRefusal> {
    Err("TLS requires a native shell".into())
}

#[cfg(unix)]
fn trust_sources(
    sources: &[skein_shell::Trust],
    errors: &mut dyn Write,
    reader: &mut RootReader<'_>,
) -> Result<tls::Config, ConfigurationRefusal> {
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
        let roots = reader.read(source, &bounds).map_err(|why| {
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
    if malformed > 0 {
        drop(writeln!(
            errors,
            "smith: trust roots: {} taken, {} skipped, {} malformed blocks, {} other blocks",
            parsed.taken, parsed.skipped, malformed, other_blocks
        ));
    }
    Ok(config)
}
