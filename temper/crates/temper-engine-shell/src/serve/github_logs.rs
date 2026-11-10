//! GitHub job-log origins resolved before the loop (temper/docs/design/protocol/forge/README.md, 2.6).
use jig_protocol_config::Address;
use skein_forge::github::redirect::Origin;
use skein_forge_connection::settings::LogOrigin;
use skein_lib::List;

/// Resolve explicit origins strictly, or GitHub's twenty default results hosts best effort.
/// An explicit empty list disables job logs. Unresolved defaults are named once at startup.
pub fn resolve(
    configured: Option<&[Origin]>,
    lookup: fn(&Address) -> Result<std::net::SocketAddr, String>,
) -> Result<List<LogOrigin>, String> {
    resolve_for("temper", configured, lookup)
}
/// Resolve origins while naming the binary that serves them.
pub fn resolve_for(
    binary: &str,
    configured: Option<&[Origin]>,
    lookup: fn(&Address) -> Result<std::net::SocketAddr, String>,
) -> Result<List<LogOrigin>, String> {
    let defaults: Vec<_> = (0_u32..20)
        .map(|number| Origin {
            tls: true,
            host: format!("productionresultssa{number}.blob.core.windows.net").into_bytes().into_boxed_slice(),
            port: 443,
        })
        .collect();
    let origins = configured.unwrap_or(&defaults);
    let mut resolved = List::with_capacity(u32::try_from(origins.len()).expect("configured origins"));
    for origin in origins {
        let host = Address::Host { name: origin.host.clone(), port: origin.port };
        match lookup(&host) {
            Ok(address) => resolved
                .push(LogOrigin { origin: origin.clone(), address: skein_io::Addr::Ip(address) })
                .expect("one address per origin"),
            Err(why) if configured.is_some() => {
                return Err(format!("GitHub log origin {}: {why}", String::from_utf8_lossy(&origin.host)));
            }
            Err(_) => eprintln!(
                "{binary} serve: GitHub log origin https://{}:443: unresolved; skipped",
                String::from_utf8_lossy(&origin.host)
            ),
        }
    }
    if configured.is_none() && resolved.is_empty() {
        eprintln!("{binary} serve: GitHub log origins: none resolved; no job log will be fetched");
    }
    Ok(resolved)
}
