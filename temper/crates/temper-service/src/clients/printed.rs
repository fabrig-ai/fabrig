//! Print the sign-in URL after the listener address is known.
use alloc::boxed::Box;
pub(crate) use jig_protocol_engine_clients_http::address_text;
pub(crate) fn link(address: ::core::net::SocketAddr, secret: &skein_lib::Secret) -> Box<[u8]> {
    jig_protocol_engine_clients_http::printed_link(
        address,
        &jig_protocol_engine_clients_http::link_path(&super::http_settings(None), secret),
    )
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
