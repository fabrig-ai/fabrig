//! The printed link takes the route's or the application's page's form (protocol/client/default-http.md, 1 and 4).
use crate::Mapping;
use alloc::boxed::Box;
use skein_lib::{
    Secret,
    base64::{self, Alphabet},
    bytes,
};
#[derive(Debug, PartialEq, Eq)]
pub struct Printed {
    pub host: Box<[u8]>,
    pub secret: Secret,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unlinked {
    NotHttp,
    NoSecret,
    Address,
}
/// Makes an asked link's URL in the printed link's form (protocol/client/default-http.md, 1).
#[must_use]
pub fn link_url(mapping: &Mapping, host: &[u8], secret: &Secret) -> Box<[u8]> {
    let encoded = base64::encoded_len(Alphabet::Url, u32::try_from(secret.len()).expect("bounded secret"))
        .expect("bounded base64");
    let (path, query): (&[u8], &[u8]) = match &mapping.page {
        Some(page) => (page, b"?secret="),
        None => (&mapping.prefix, b"/v1/sign-in/link?secret="),
    };
    let mut url = skein_lib::Writer::new(
        b"http://"
            .len()
            .checked_add(host.len())
            .expect("host")
            .checked_add(path.len())
            .expect("path")
            .checked_add(query.len())
            .expect("query")
            .checked_add(skein_lib::length::usize_of(encoded))
            .expect("secret"),
    );
    url.put(b"http://").expect("measured scheme");
    url.put(host).expect("measured host");
    url.put(path).expect("measured path");
    url.put(query).expect("measured query");
    base64::encode(Alphabet::Url, secret.expose(), &mut url).expect("measured secret");
    url.finish()
}

#[expect(clippy::disallowed_methods, reason = "the HTTP mapping parses its printed URL")]
pub fn printed(mapping: &Mapping, url: &[u8]) -> Result<Printed, Unlinked> {
    let text = core::str::from_utf8(url).or(Err(Unlinked::Address))?;
    let rest = text.strip_prefix("http://").ok_or(Unlinked::NotHttp)?;
    let (host, path) = rest.split_once('/').ok_or(Unlinked::NoSecret)?;
    let _: core::net::SocketAddr = host.parse().or(Err(Unlinked::Address))?;
    let prefix = mapping.prefix.get(1..).expect("validated prefix");
    let path = path.as_bytes();
    let route_secret = match path.strip_prefix(prefix) {
        Some(rest) => rest.strip_prefix(b"/v1/sign-in/link?secret="),
        None => None,
    };
    let secret = match route_secret {
        Some(secret) => secret,
        None => {
            let page = mapping.page.as_ref().ok_or(Unlinked::NoSecret)?;
            let page = page.get(1..).expect("validated page");
            let rest = path.strip_prefix(page).ok_or(Unlinked::NoSecret)?;
            rest.strip_prefix(b"?secret=").ok_or(Unlinked::NoSecret)?
        }
    };
    let secret = base64::decode(Alphabet::Url, secret, 32).or(Err(Unlinked::NoSecret))?;
    if secret.len() != 32 {
        return Err(Unlinked::NoSecret);
    }
    Ok(Printed { host: bytes::copy_of(host.as_bytes()), secret: Secret::new(secret) })
}
