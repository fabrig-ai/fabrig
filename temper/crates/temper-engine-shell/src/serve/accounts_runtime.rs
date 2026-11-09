//! Resolve configured sign-in transports before starting the accounts component.
use super::{config, server_name, service, trust};
use skein_lib::List;
use skein_oauth_accounts as driver;

/// A sign-in host that did not resolve at start (temper/docs/design/protocol/configuration.md,
/// section 2): loopback port 0, which no listener can hold, so its exchanges fail at their connect.
const HELD: std::net::SocketAddr =
    std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 0));

pub(super) fn resolve(
    engine: &config::Engine,
    lookup: fn(&str, u16) -> Option<std::net::SocketAddr>,
) -> Result<service::AccountsRuntime, String> {
    let mut runtime = service::AccountsRuntime {
        trust: None,
        providers: List::with_capacity(u32::try_from(engine.llm.providers.len()).expect("provider count")),
    };
    for provider in &engine.llm.providers {
        let Some(registration) = &provider.sign_in else { continue };
        let endpoint = endpoint(&registration.token, &registration.endpoint_transport, lookup)?;
        let mut origins = List::<driver::Origin>::with_capacity(3);
        for uri in [Some(&*registration.issuer), Some(&*registration.keys), registration.revocation.as_deref()]
            .into_iter()
            .flatten()
        {
            let (origin, _) = origin(uri)?;
            if origins.iter().any(|item| &*item.uri == origin) {
                continue;
            }
            origins
                .push(driver::Origin {
                    uri: origin.into(),
                    endpoint: endpoint_for_origin(origin, &registration.endpoint_transport, lookup)?,
                })
                .expect("three configured origins");
        }
        runtime.providers.push((provider.number, endpoint, origins)).expect("provider count");
    }
    Ok(runtime)
}
fn transport(configured: &config::Transport) -> Result<driver::Transport, String> {
    Ok(match configured {
        config::Transport::Plaintext => driver::Transport::Plaintext,
        config::Transport::Tls { server_name: name, trust: file } => {
            let trust = trust(file.as_deref())?;
            driver::Transport::Tls { server_name: server_name(name)?, trust }
        }
    })
}
fn origin(uri: &[u8]) -> Result<(&[u8], u16), String> {
    let (start, port) = if uri.starts_with(b"https://") {
        (8, 443)
    } else if uri.starts_with(b"http://") {
        (7, 80)
    } else {
        return Err("sign-in endpoint requires HTTP or HTTPS".into());
    };
    let end =
        uri[start..].iter().position(|byte| matches!(byte, b'/' | b'?' | b'#')).map_or(uri.len(), |at| start + at);
    Ok((&uri[..end], port))
}
fn endpoint(
    uri: &[u8],
    configured: &config::Transport,
    lookup: fn(&str, u16) -> Option<std::net::SocketAddr>,
) -> Result<driver::Endpoint, String> {
    let (origin, _) = origin(uri)?;
    endpoint_for_origin(origin, configured, lookup)
}
fn endpoint_for_origin(
    uri: &[u8],
    configured: &config::Transport,
    lookup: fn(&str, u16) -> Option<std::net::SocketAddr>,
) -> Result<driver::Endpoint, String> {
    let (_, port) = origin(uri)?;
    let authority =
        std::str::from_utf8(uri).map_err(|why| why.to_string())?.split_once("://").expect("checked scheme").1;
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest.split_once(']').ok_or("invalid endpoint host")?;
        let port = match suffix.strip_prefix(':') {
            Some(value) => value.parse::<u16>().map_err(|why| why.to_string())?,
            None if suffix.is_empty() => port,
            None => return Err("invalid endpoint port".into()),
        };
        (host, port)
    } else if let Some((host, value)) = authority.rsplit_once(':') {
        (host, value.parse::<u16>().map_err(|why| why.to_string())?)
    } else {
        (authority, port)
    };
    let address = lookup(host, port).unwrap_or(HELD);
    Ok(driver::Endpoint { address, transport: transport(configured)? })
}
