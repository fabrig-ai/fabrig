//! Bundle routes and headers, over bytes handed to the server (skein/docs/design/ui/web/bundle.md, 4).
use skein_ui_example_server::{State, bundle, bundle::Bundle, routes};
use skein_ui_web_tests::get_with_headers;
use std::collections::BTreeMap;
use std::path::Path;

const DIGEST: &str = "0123456789abcdef";
const PAGE: &[u8] = b"<!doctype html><title>Notes</title>";
const JS: &[u8] = b"export default function init() {}";
const WASM: &[u8] = b"\0asm";
// Fixed gzip forms of JS and WASM, with mtime zero.
const JS_GZIP: &[u8] = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x02\xff\x4b\xad\x28\xc8\x2f\x2a\x51\x48\
    \x49\x4d\x4b\x2c\xcd\x29\x51\x48\x2b\xcd\x4b\x2e\xc9\xcc\xcf\x53\xc8\xcc\xcb\x2c\xd1\xd0\x54\
    \xa8\xae\x05\x00\xdc\x8b\xb0\x97\x21\x00\x00\x00";
const WASM_GZIP: &[u8] = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x02\xff\x63\x48\x2c\xce\x05\x00\
    \xdb\x49\x40\x5f\x04\x00\x00\x00";

fn bundle() -> Bundle {
    let files = BTreeMap::from([
        ("notes.js".into(), Box::from(JS)),
        ("notes.js.gz".into(), Box::from(JS_GZIP)),
        ("notes_bg.wasm".into(), Box::from(WASM)),
        ("notes_bg.wasm.gz".into(), Box::from(WASM_GZIP)),
    ]);
    Bundle::new(Box::from(PAGE), DIGEST.into(), files).expect("fixture bundle fits")
}

fn field<'a>(answer: &'a routes::Answer, name: &[u8]) -> Option<&'a [u8]> {
    answer.head.headers.iter().find(|header| header.name.eq_ignore_ascii_case(name)).map(|header| header.value.as_ref())
}

#[test]
fn the_servers_types_are_the_bundlers() {
    for (path, expected) in [
        ("a.html", Some("text/html; charset=utf-8")),
        ("a.js", Some("text/javascript")),
        ("a.wasm", Some("application/wasm")),
        ("a.css", Some("text/css")),
        ("a.svg", Some("image/svg+xml")),
        ("a.png", Some("image/png")),
        ("a.webp", Some("image/webp")),
        ("a.ico", Some("image/x-icon")),
        ("a.woff2", Some("font/woff2")),
        ("a.txt", Some("text/plain; charset=utf-8")),
        ("a.webmanifest", Some("application/manifest+json")),
        ("a.unknown", None),
    ] {
        assert_eq!(bundle::of(Path::new(path)), expected, "server {path}");
        assert_eq!(skein_ui_bundle::types::of(Path::new(path)), expected, "bundler {path}");
    }
}

#[test]
fn a_client_asking_gets_the_compressed_answer() {
    let bundle = bundle();
    for (name, kind, gzip, plain) in [
        ("notes.js", b"text/javascript".as_slice(), JS_GZIP, JS),
        ("notes_bg.wasm", b"application/wasm".as_slice(), WASM_GZIP, WASM),
    ] {
        let path = format!("/bundle/{DIGEST}/{name}");
        let call = get_with_headers(path.as_bytes(), &[(b"Accept-Encoding", b"gzip")]);
        let answer = routes::route(&mut State::new(), &bundle, &call, &[]);
        assert_eq!(answer.head.status, 200, "{name}");
        assert_eq!(answer.body.as_ref(), gzip, "{name}");
        let mut decoded = skein_lib::List::allocated(u32::try_from(plain.len()).expect("fixture fits u32"));
        skein_deflate::gzip::decode(&answer.body, &mut decoded).expect("gzip body");
        assert_eq!(decoded.as_slice(), plain, "{name}");
        assert_eq!(field(&answer, b"Content-Type"), Some(kind), "{name}");
        assert_eq!(field(&answer, b"Content-Encoding"), Some(b"gzip".as_slice()), "{name}");
        assert_eq!(field(&answer, b"Vary"), Some(b"Accept-Encoding".as_slice()), "{name}");
        assert_eq!(field(&answer, b"Cache-Control"), Some(b"public, max-age=31536000, immutable".as_slice()), "{name}");
        let names: Vec<_> = answer.head.headers.iter().take(4).map(|header| header.name.as_ref()).collect();
        assert_eq!(names, [b"Content-Type".as_slice(), b"Content-Encoding", b"Vary", b"Cache-Control"], "{name}");
    }
}

#[test]
fn a_client_not_asking_gets_the_plain_one() {
    let bundle = bundle();
    let identity = [(b"Accept-Encoding".as_slice(), b"identity".as_slice())];
    let refused = [(b"Accept-Encoding".as_slice(), b"gzip;q=0".as_slice())];
    for (name, plain) in [("notes.js", JS), ("notes_bg.wasm", WASM)] {
        let path = format!("/bundle/{DIGEST}/{name}");
        for headers in [&[][..], &identity, &refused] {
            let call = get_with_headers(path.as_bytes(), headers);
            let answer = routes::route(&mut State::new(), &bundle, &call, &[]);
            assert_eq!(answer.head.status, 200, "{name} {headers:?}");
            assert_eq!(answer.body.as_ref(), plain, "{name} {headers:?}");
            assert_eq!(field(&answer, b"Vary"), Some(b"Accept-Encoding".as_slice()), "{name} {headers:?}");
            assert_eq!(field(&answer, b"Content-Encoding"), None, "{name} {headers:?}");
        }
    }
}

#[test]
fn a_compressed_form_has_no_address() {
    let path = format!("/bundle/{DIGEST}/notes.js.gz");
    let call = get_with_headers(path.as_bytes(), &[(b"Accept-Encoding", b"gzip")]);
    assert_eq!(routes::route(&mut State::new(), &bundle(), &call, &[]).head.status, 404);
}

#[test]
fn the_page_does_not_negotiate_gzip() {
    let call = get_with_headers(b"/", &[(b"Accept-Encoding", b"gzip")]);
    let answer = routes::route(&mut State::new(), &bundle(), &call, &[]);
    assert_eq!(answer.body.as_ref(), PAGE);
    assert_eq!(field(&answer, b"Content-Encoding"), None);
    assert_eq!(field(&answer, b"Vary"), None);
}
