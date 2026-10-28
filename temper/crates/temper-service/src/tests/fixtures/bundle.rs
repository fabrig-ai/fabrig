use crate::clients::bundle::{Bundle, File};
use skein_lib::{List, Map, Writer};

/// The bytes of `module.wasm`: one past a send, each of a kind that shows a chunk out of order.
pub const WASM_BYTES: u32 = 16_385;
pub fn head(method: skein_http::Method, target: &[u8], headers: &[(&[u8], &[u8])]) -> skein_http::server::Call {
    let mut fields = List::with_capacity(u32::try_from(headers.len()).expect("fixture fields"));
    for (name, value) in headers {
        fields.push(skein_http::Header { name: (*name).into(), value: (*value).into() }).expect("fixture fields");
    }
    skein_http::server::Call {
        method,
        target: target.into(),
        version: skein_http::Version::Http11,
        headers: fields.into_boxed(),
        body: skein_http::server::Body::None,
    }
}
pub fn wasm() -> Box<[u8]> {
    let mut bytes = Writer::new(usize::try_from(WASM_BYTES).expect("fixture size"));
    for at in 0..WASM_BYTES {
        bytes.put(&[u8::try_from(at % 251).expect("a byte")]).expect("fixture file");
    }
    bytes.finish()
}
pub fn bundle() -> Bundle {
    let mut files = Map::with_capacity(3);
    files
        .insert(
            Box::from(b"boot.js".as_slice()),
            File { kind: b"text/javascript", bytes: Box::from(b"boot();".as_slice()) },
        )
        .expect("boot file");
    files
        .insert(Box::from(b"module.wasm".as_slice()), File { kind: b"application/wasm", bytes: wasm() })
        .expect("module file");
    files
        .insert(
            Box::from(b"files/robots.txt".as_slice()),
            File { kind: b"text/plain; charset=utf-8", bytes: Box::from(b"User-agent: *\nDisallow: /\n".as_slice()) },
        )
        .expect("root file");
    Bundle {
        page: Box::from(b"<!doctype html><title>t</title>".as_slice()),
        etag: Box::from(b"\"0011223344556677\"".as_slice()),
        digest: Box::from(b"0123456789abcdef".as_slice()),
        files,
    }
}
/// A peer's bytes past its answer's head.
pub fn body(peer: &super::http::Peer) -> &[u8] {
    let received = peer.received();
    let at = skein_lib::bytes::find(received, b"\r\n\r\n").expect("answer head").checked_add(4).unwrap();
    received.get(at..).expect("answer body")
}
/// The page's policy as `answered` gives it, names lowercased (client/browser.md, 2.2).
pub const POLICY: [(&[u8], &[u8]); 5] = [
    (b"content-security-policy", b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"),
    (b"x-content-type-options", b"nosniff"),
    (b"cross-origin-opener-policy", b"same-origin"),
    (b"cross-origin-resource-policy", b"same-origin"),
    (b"referrer-policy", b"no-referrer"),
];
