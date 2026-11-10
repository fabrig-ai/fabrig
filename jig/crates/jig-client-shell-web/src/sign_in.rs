//! Take the printed link out of history before starting the host (protocol/browser.md, 11).
use skein_lib::base64::{self, Alphabet};
#[derive(Debug)]
pub struct Route<'a> {
    pub path: &'a [u8],
    pub home: &'a [u8],
}
#[derive(PartialEq, Eq)]
pub struct Taken {
    pub secret: Box<[u8]>,
    pub address: Box<[u8]>,
}
// Hide sign-in secret bytes to keep diagnostics safe (skein/docs/design/runtime/lib.md, 6).
impl core::fmt::Debug for Taken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Taken").field("secret_len", &self.secret.len()).field("address", &self.address).finish()
    }
}
/// Read the mapping's base64url secret only at the configured link route.
#[must_use]
pub fn take(route: &Route<'_>, path_and_query: &[u8]) -> Option<Taken> {
    let text = std::str::from_utf8(path_and_query).ok()?;
    let (path, query) = text.split_once('?')?;
    if path.as_bytes() != route.path {
        return None;
    }
    let value = query.split('&').find_map(|part| part.strip_prefix("secret="))?;
    let secret = match base64::decode(Alphabet::Url, value.as_bytes(), 32) {
        Ok(secret) if secret.len() == 32 => secret,
        Ok(_) | Err(_) => Box::new([]),
    };
    Some(Taken { secret, address: route.home.into() })
}
#[cfg(target_arch = "wasm32")]
pub mod web {
    pub use crate::web::{show_once, take_from_page};
}
