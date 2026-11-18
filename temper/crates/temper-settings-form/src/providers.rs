//! Provider JSON in jig's form (jig/docs/design/domain/engine/llm-providers.md, 6).
//! The shared client form cannot name the engine's configuration parser.
use crate::file::{Decoder, Error, Fields, error, field, optional};
use crate::view as c;
use alloc::boxed::Box;
use skein_lib::{Duration, List};
mod render;
pub use render::encode;

fn text(d: &Decoder<'_>, fields: &Fields, name: &[u8], most: u32) -> Result<Box<[u8]>, Error> {
    match d.words(field(fields, name)?, most, name) {
        Ok(value) => Ok(value.bytes().into()),
        Err(why) => Err(why.named(name)),
    }
}
fn maybe_text(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<Option<Box<[u8]>>, Error> {
    if optional(fields, name).is_some() { Ok(Some(text(d, fields, name, 4096)?)) } else { Ok(None) }
}
fn number(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<u32, Error> {
    let value = match d.number(field(fields, name)?) {
        Ok(value) => value,
        Err(why) => return Err(why.named(name)),
    };
    u32::try_from(value).or(Err(error(name, b"integer bound")))
}
fn boolean(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<bool, Error> {
    match optional(fields, name) {
        Some(at) => match d.boolean(at) {
            Ok(value) => Ok(value),
            Err(why) => Err(why.named(name)),
        },
        None => Ok(false),
    }
}
fn texts(d: &Decoder<'_>, fields: &Fields, name: &[u8], most: u32, bytes: u32) -> Result<Box<[Box<[u8]>]>, Error> {
    let items = d.array(field(fields, name)?, most, name)?;
    let mut values = List::with_capacity(items.len());
    for &at in &items {
        let value = match d.words(at, bytes, name) {
            Ok(value) => value.bytes().into(),
            Err(why) => return Err(why.named(name)),
        };
        values.push(value).expect("bounded list");
    }
    Ok(values.into_boxed())
}

pub(super) fn read(d: &Decoder<'_>, at: u32, expected: u32) -> Result<c::ProviderSetting, Error> {
    let fields = d.object(
        at,
        &[
            b"number",
            b"name",
            b"endpoint",
            b"direct",
            b"signed_in_plan",
            b"manage_usage",
            b"keep",
            b"drop",
            b"sources",
            b"endpoints",
            b"withdrawn",
        ],
    )?;
    let number = number(d, &fields, b"number")?;
    if number != expected {
        return Err(error(b"number", b"must equal setting.number"));
    }
    if number == 0 || number > 16 {
        return Err(error(b"number", b"from 1 through 16"));
    }
    let name = text(d, &fields, b"name", 256)?;
    let endpoint = maybe_text(d, &fields, b"endpoint")?;
    let direct = maybe_text(d, &fields, b"direct")?;
    let list = if optional(&fields, b"keep").is_some() {
        if endpoint.is_none() {
            return Err(error(b"keep", b"endpoint required"));
        }
        Some(c::ProviderList {
            keep: texts(d, &fields, b"keep", 16, 64)?,
            drop: if optional(&fields, b"drop").is_some() {
                texts(d, &fields, b"drop", 16, 64)?
            } else {
                Box::default()
            },
        })
    } else {
        if optional(&fields, b"drop").is_some() {
            return Err(error(b"drop", b"keep required"));
        }
        None
    };
    let sources = d.object(field(&fields, b"sources")?, &[b"sign_in", b"keys", b"programs"])?;
    let sign_in = match optional(&sources, b"sign_in") {
        Some(at) => match registration(d, at) {
            Ok(value) => Some(value),
            Err(why) => return Err(why.at(b"sources.sign_in")),
        },
        None => None,
    };
    if sign_in.is_some() && optional(&fields, b"signed_in_plan").is_none() {
        return Err(error(b"signed_in_plan", b"missing member"));
    }
    let programs = if optional(&sources, b"programs").is_some() {
        texts(d, &sources, b"programs", 64, 4096)?
    } else {
        Box::default()
    };
    let items = d.array(field(&fields, b"endpoints")?, 64, b"endpoints")?;
    let mut endpoints = List::with_capacity(items.len());
    for &at in &items {
        let value = match read_endpoint(d, at, &name) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"endpoints")),
        };
        endpoints.push(value).expect("bounded endpoints");
    }
    Ok(c::ProviderSetting {
        provider: c::Provider {
            name,
            number,
            endpoint,
            direct,
            list,
            signed_in_plan: boolean(d, &fields, b"signed_in_plan")?,
            manage_usage: maybe_text(d, &fields, b"manage_usage")?,
            sign_in,
            keys: boolean(d, &sources, b"keys")?,
            programs,
        },
        endpoints: endpoints.into_boxed(),
        withdrawn: boolean(d, &fields, b"withdrawn")?,
    })
}
fn transport(d: &Decoder<'_>, fields: &Fields) -> Result<c::Transport, Error> {
    let kind = text(d, fields, b"transport", 4096)?;
    match kind.as_ref() {
        b"plaintext" => {
            for name in [b"server_name".as_slice(), b"trust"] {
                if optional(fields, name).is_some() {
                    return Err(error(name, b"unknown member"));
                }
            }
            Ok(c::Transport::Plaintext)
        }
        b"tls" => Ok(c::Transport::Tls {
            server_name: text(d, fields, b"server_name", 4096)?,
            trust: maybe_text(d, fields, b"trust")?,
        }),
        _ => Err(error(b"transport", b"tls or plaintext required")),
    }
}
#[expect(
    clippy::disallowed_methods,
    reason = "the provider JSON boundary parses a socket address, as jig-protocol-config does"
)]
fn socket(bytes: &[u8]) -> Result<core::net::SocketAddr, Error> {
    core::str::from_utf8(bytes)
        .or(Err(error(b"socket", b"invalid address")))?
        .parse()
        .or(Err(error(b"socket", b"invalid address")))
}
fn address(d: &Decoder<'_>, at: u32) -> Result<c::EndpointAddress, Error> {
    let fields = d.object(at, &[b"socket", b"host", b"port"])?;
    match (optional(&fields, b"socket"), optional(&fields, b"host")) {
        (Some(_), None) => {
            if optional(&fields, b"port").is_some() {
                return Err(error(b"port", b"unknown member"));
            }
            Ok(c::EndpointAddress::Socket(socket(&text(d, &fields, b"socket", 128)?)?))
        }
        (None, Some(_)) => Ok(c::EndpointAddress::Host {
            name: text(d, &fields, b"host", 253)?,
            port: u16::try_from(number(d, &fields, b"port")?).or(Err(error(b"port", b"integer bound")))?,
        }),
        (None, None) | (Some(_), Some(_)) => Err(error(b"address", b"one of socket or host required")),
    }
}
fn duration(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<Duration, Error> {
    let value = match d.number(field(fields, name)?) {
        Ok(value) => value,
        Err(why) => return Err(why.named(name)),
    };
    let nanos = value.checked_mul(1_000_000).ok_or(error(name, b"duration bound"))?;
    Ok(Duration::from_nanos(nanos))
}
fn read_endpoint(d: &Decoder<'_>, at: u32, owner: &[u8]) -> Result<c::Endpoint, Error> {
    let fields = d.object(
        at,
        &[
            b"name",
            b"number",
            b"dialect",
            b"program",
            b"address",
            b"transport",
            b"server_name",
            b"trust",
            b"headers",
            b"connect_ms",
            b"handshake_ms",
        ],
    )?;
    let items = d.array(field(&fields, b"headers")?, 16, b"headers")?;
    let mut headers = List::with_capacity(items.len());
    for &at in &items {
        let fields = d.object(at, &[b"name", b"value"])?;
        let name = text(d, &fields, b"name", 4096)?;
        for secret in [b"authorization".as_slice(), b"proxy-authorization", b"x-api-key", b"api-key", b"cookie"] {
            if name.eq_ignore_ascii_case(secret) {
                return Err(error(b"headers.name", b"credential header refused"));
            }
        }
        headers.push(c::Header { name, value: text(d, &fields, b"value", 4096)? }).expect("bounded headers");
    }
    let address = match address(d, field(&fields, b"address")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"address")),
    };
    Ok(c::Endpoint {
        name: text(d, &fields, b"name", 4096)?,
        number: number(d, &fields, b"number")?,
        dialect: number(d, &fields, b"dialect")?,
        provider: owner.into(),
        program: maybe_text(d, &fields, b"program")?,
        address,
        transport: transport(d, &fields)?,
        headers: headers.into_boxed(),
        connect: duration(d, &fields, b"connect_ms")?,
        handshake: duration(d, &fields, b"handshake_ms")?,
    })
}
fn registration(d: &Decoder<'_>, at: u32) -> Result<c::Registration, Error> {
    let fields = d.object(
        at,
        &[
            b"issuer",
            b"client",
            b"scopes",
            b"name_hint",
            b"authorization",
            b"token",
            b"revocation",
            b"keys",
            b"transport",
            b"server_name",
            b"trust",
            b"must_grant",
            b"resource",
            b"redirect_path",
            b"redirect_port",
            b"secret",
        ],
    )?;
    if optional(&fields, b"secret").is_some() {
        return Err(error(b"secret", b"confidential registration refused"));
    }
    let redirect_port = match optional(&fields, b"redirect_port") {
        Some(_) => Some(
            u16::try_from(number(d, &fields, b"redirect_port")?).or(Err(error(b"redirect_port", b"integer bound")))?,
        ),
        None => None,
    };
    Ok(c::Registration {
        issuer: text(d, &fields, b"issuer", 4096)?,
        client: text(d, &fields, b"client", 4096)?,
        scopes: texts(d, &fields, b"scopes", 64, 4096)?,
        name_hint: maybe_text(d, &fields, b"name_hint")?,
        authorization: text(d, &fields, b"authorization", 4096)?,
        token: text(d, &fields, b"token", 4096)?,
        revocation: maybe_text(d, &fields, b"revocation")?,
        keys: text(d, &fields, b"keys", 4096)?,
        endpoint_transport: transport(d, &fields)?,
        must_grant: texts(d, &fields, b"must_grant", 64, 4096)?,
        resource: maybe_text(d, &fields, b"resource")?,
        redirect_path: text(d, &fields, b"redirect_path", 4096)?,
        redirect_port,
    })
}
