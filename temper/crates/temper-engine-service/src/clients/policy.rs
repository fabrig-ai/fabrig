//! The page policy for the clients' listener and static playground
//! (temper/docs/design/client/web/README.md, 2.2; temper/docs/design/playground.md, 7).
use alloc::boxed::Box;
use skein_lib::Writer;

pub(crate) use jig_protocol_engine_clients_http::server::policy::FIELDS;

const PLAYGROUND_POLICY: &[u8] = b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; worker-src 'self'";

/// The policy's fields; the playground's CSP adds `worker-src 'self'`.
pub const fn playground_fields() -> [(&'static [u8], &'static [u8]); 5] {
    let [(name, _), nosniff, opener, resource, referrer] = FIELDS;
    [(name, PLAYGROUND_POLICY), nosniff, opener, resource, referrer]
}

/// Digest-directory files are immutable.
pub const FILES_CACHE: &[u8] = b"public, max-age=31536000, immutable";
/// The page is revalidated on each load.
pub const PAGE_CACHE: &[u8] = b"no-cache";

/// The playground's `headers` text for a bundle under `directory` (`/bundle/<digest>`).
#[cfg_attr(not(test), expect(dead_code, reason = "the static playground uses the checked headers template"))]
pub fn headers(directory: &[u8]) -> Box<[u8]> {
    let policy = playground_fields();
    let cache = [directory, b"/*: Cache-Control: ", FILES_CACHE, b"\n/*: Cache-Control: ", PAGE_CACHE, b"\n"];
    let mut size = 0_usize;
    for (name, value) in policy {
        for part in [b"/*: ".as_slice(), name, b": ", value, b"\n"] {
            size = size.checked_add(part.len()).expect("headers text size");
        }
    }
    for part in cache {
        size = size.checked_add(part.len()).expect("headers text size");
    }
    let mut text = Writer::new(size);
    for (name, value) in policy {
        for part in [b"/*: ".as_slice(), name, b": ", value, b"\n"] {
            text.put(part).expect("headers text room");
        }
    }
    for part in cache {
        text.put(part).expect("headers text room");
    }
    text.finish()
}
