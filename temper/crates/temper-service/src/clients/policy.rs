//! The page policy for the clients' listener and static playground
//! (temper/docs/design/client/browser.md, 2.2; temper/docs/design/playground.md, 7).
use alloc::boxed::Box;
use skein_http::{Header, server::Response};
use skein_lib::{List, Writer, bytes};

/// Which page the policy is for: the clients' listener or the static playground.
#[derive(Debug)]
pub enum Page {
    Service,
    Playground,
}

/// The policy's fields; the playground's CSP adds `worker-src 'self'`.
pub const fn fields(page: Page) -> [(&'static [u8], &'static [u8]); 5] {
    let csp: &[u8] = match page {
        Page::Service => b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        Page::Playground => b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; worker-src 'self'",
    };
    [
        (b"Content-Security-Policy", csp),
        (b"X-Content-Type-Options", b"nosniff"),
        (b"Cross-Origin-Opener-Policy", b"same-origin"),
        (b"Cross-Origin-Resource-Policy", b"same-origin"),
        (b"Referrer-Policy", b"no-referrer"),
    ]
}

/// jig's reference policy, on every answer of the clients' listener.
pub const FIELDS: [(&[u8], &[u8]); 5] = fields(Page::Service);

/// Digest-directory files are immutable.
pub const FILES_CACHE: &[u8] = b"public, max-age=31536000, immutable";
/// The page is revalidated on each load.
pub const PAGE_CACHE: &[u8] = b"no-cache";

/// The playground's `headers` text for a bundle under `directory` (`/bundle/<digest>`).
#[cfg_attr(not(test), expect(dead_code, reason = "the static playground uses the checked headers template"))]
pub fn headers(directory: &[u8]) -> Box<[u8]> {
    let policy = fields(Page::Playground);
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

/// Append each field the response does not hold, its name compared ignoring case.
pub(crate) fn add(response: &mut Response) {
    let capacity = u32::try_from(response.headers.len()).expect("bounded response headers");
    let mut headers = List::with_capacity(
        capacity.checked_add(u32::try_from(FIELDS.len()).expect("policy field count")).expect("policy fields"),
    );
    for header in core::mem::replace(&mut response.headers, Box::new([])) {
        headers.push(header).expect("existing fields");
    }
    for (name, value) in fields(Page::Service) {
        let mut present = false;
        for header in &headers {
            present |= header.name.eq_ignore_ascii_case(name);
        }
        if !present {
            headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).expect("policy room");
        }
    }
    response.headers = headers.into_boxed();
}
