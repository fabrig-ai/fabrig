//! Print the sign-in URL after the listener address is known.
use alloc::boxed::Box;
pub(crate) fn link(address: ::core::net::SocketAddr, secret: &skein_lib::Secret) -> Box<[u8]> {
    let host = address_text(address).expect("bound listener address");
    let path = jig_protocol_engine_clients_http::link_path(&super::http_settings(None), secret);
    let parts: [&[u8]; 3] = [b"http://", &host, &path];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("short printed URL");
    }
    let mut out = skein_lib::Writer::new(length);
    for part in parts {
        out.put(part).expect("measured URL");
    }
    out.finish()
}

pub(crate) fn address_text(address: ::core::net::SocketAddr) -> Option<Box<[u8]>> {
    use ::core;
    let mut text = skein_lib::Intake::with_capacity(64);
    match address.ip() {
        core::net::IpAddr::V4(ip) => {
            for (at, octet) in ip.octets().iter().enumerate() {
                if at != 0 {
                    text.append(b".").ok()?;
                }
                text.append(skein_lib::Decimal::of(u64::from(*octet)).as_bytes()).ok()?;
            }
        }
        core::net::IpAddr::V6(ip) => {
            if !ip.is_loopback() {
                return None;
            }
            text.append(b"[::1]").ok()?;
        }
    }
    text.append(b":").ok()?;
    text.append(skein_lib::Decimal::of(u64::from(address.port())).as_bytes()).ok()?;
    text.meet(skein_lib::stream::Read::Fill(text.len()))
}
pub(crate) fn link_public(
    address: ::core::net::SocketAddr,
    public: Option<&crate::startup::application::Origin>,
    secret: &skein_lib::Secret,
) -> Box<[u8]> {
    let Some(public) = public else { return link(address, secret) };
    let base = public.base();
    let base = base.strip_suffix(b"/").expect("origin separator");
    let path = jig_protocol_engine_clients_http::link_path(&super::http_settings(None), secret);
    let mut out = skein_lib::Writer::new(base.len().checked_add(path.len()).expect("printed URL"));
    out.put(base).expect("origin");
    out.put(&path).expect("link");
    out.finish()
}
