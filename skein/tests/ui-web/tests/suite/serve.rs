//! The server consumes a bundle without linking its producer.
use skein_scratch::Scratch;
use skein_ui_bundle::{Glue, Inputs};
use skein_ui_example_server::{
    State,
    bundle::{self, Bundle},
    routes,
};
use std::path::Path;
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
fn the_server_serves_a_bundle_it_did_not_make() {
    let scratch = Scratch::new("example-serves-bundle");
    let out = scratch.path().join("bundle");
    let inputs = Inputs {
        gzip: true,
        name: "notes".into(),
        prefix: "/bundle/".into(),
        stylesheets: vec![],
        pictures: vec![],
        page: None,
        files: None,
        noscript: "Scripts required.".into(),
    };
    let glue = Glue { js: b"export default function init() {}".to_vec(), wasm: b"\0asm".to_vec(), snippets: vec![] };
    let digest = skein_ui_bundle::bundle(&inputs, &glue, &out).expect("bundle written");
    let bundle = Bundle::read(&out).expect("server reads startup files");
    assert_eq!(bundle.digest, digest.to_string());
    let mut state = State::new();
    let answer = routes::route(&mut state, &bundle, &skein_ui_web_tests::get(b"/notes/7"), &[]);
    assert_eq!(answer.head.status, 200);
    assert_eq!(answer.body.as_ref(), std::fs::read(out.join("index.html")).expect("page").as_slice());
    for (name, expected, kind) in [
        ("notes.js", glue.js.as_slice(), "text/javascript"),
        ("notes_bg.wasm", glue.wasm.as_slice(), "application/wasm"),
    ] {
        let path = format!("/bundle/{digest}/{name}");
        let answer = routes::route(&mut state, &bundle, &skein_ui_web_tests::get(path.as_bytes()), &[]);
        assert_eq!(answer.head.status, 200, "{name}");
        assert_eq!(answer.body.as_ref(), expected, "{name}");
        for (name, expected) in
            [(b"Content-Type".as_slice(), kind.as_bytes()), (b"Cache-Control", b"public, max-age=31536000, immutable")]
        {
            let header =
                answer.head.headers.iter().find(|header| header.name.eq_ignore_ascii_case(name)).expect("file header");
            assert_eq!(header.value.as_ref(), expected);
        }
    }
}

fn compressed_bundle(scratch: &Scratch) -> (std::path::PathBuf, Bundle) {
    let out = scratch.path().join("bundle");
    let inputs = Inputs {
        gzip: true,
        name: "notes".into(),
        prefix: "/bundle/".into(),
        stylesheets: vec![],
        pictures: vec![],
        page: None,
        files: None,
        noscript: "Scripts required.".into(),
    };
    let glue = Glue { js: b"export default function init() {}".to_vec(), wasm: b"\0asm".to_vec(), snippets: vec![] };
    skein_ui_bundle::bundle(&inputs, &glue, &out).expect("bundle written");
    let bundle = Bundle::read(&out).expect("server reads bundle");
    (out, bundle)
}

fn field<'a>(answer: &'a routes::Answer, name: &[u8]) -> Option<&'a [u8]> {
    answer.head.headers.iter().find(|header| header.name.eq_ignore_ascii_case(name)).map(|header| header.value.as_ref())
}

#[test]
fn a_client_asking_gets_the_compressed_answer() {
    use std::io::Read;
    let scratch = Scratch::new("serve-gzip");
    let (out, bundle) = compressed_bundle(&scratch);
    for (name, kind) in [("notes.js", b"text/javascript".as_slice()), ("notes_bg.wasm", b"application/wasm")] {
        let path = format!("/bundle/{}/{name}", bundle.digest);
        let call = skein_ui_web_tests::get_with_headers(path.as_bytes(), &[(b"Accept-Encoding", b"gzip")]);
        let answer = routes::route(&mut State::new(), &bundle, &call, &[]);
        assert_eq!(answer.head.status, 200);
        let directory = out.join(&bundle.digest);
        assert_eq!(answer.body.as_ref(), std::fs::read(directory.join(format!("{name}.gz"))).expect("compressed file"));
        let mut decoded = Vec::new();
        flate2::read::GzDecoder::new(answer.body.as_ref()).read_to_end(&mut decoded).expect("gzip body");
        assert_eq!(decoded, std::fs::read(directory.join(name)).expect("plain file"));
        assert_eq!(field(&answer, b"Content-Type"), Some(kind));
        assert_eq!(field(&answer, b"Content-Encoding"), Some(b"gzip".as_slice()));
        assert_eq!(field(&answer, b"Vary"), Some(b"Accept-Encoding".as_slice()));
        assert_eq!(field(&answer, b"Cache-Control"), Some(b"public, max-age=31536000, immutable".as_slice()));
        let names: Vec<_> = answer.head.headers.iter().map(|header| header.name.as_ref()).collect();
        assert_eq!(&names[..4], &[b"Content-Type".as_slice(), b"Content-Encoding", b"Vary", b"Cache-Control"]);
    }
}

#[test]
fn a_client_not_asking_gets_the_plain_one() {
    let scratch = Scratch::new("serve-plain");
    let (out, bundle) = compressed_bundle(&scratch);
    for name in ["notes.js", "notes_bg.wasm"] {
        for encoding in [None, Some(b"identity".as_slice()), Some(b"gzip;q=0".as_slice())] {
            let path = format!("/bundle/{}/{name}", bundle.digest);
            let headers: Vec<_> = encoding.into_iter().map(|value| (b"Accept-Encoding".as_slice(), value)).collect();
            let call = skein_ui_web_tests::get_with_headers(path.as_bytes(), &headers);
            let answer = routes::route(&mut State::new(), &bundle, &call, &[]);
            assert_eq!(answer.head.status, 200);
            assert_eq!(answer.body.as_ref(), std::fs::read(out.join(&bundle.digest).join(name)).expect("plain file"));
            assert_eq!(field(&answer, b"Vary"), Some(b"Accept-Encoding".as_slice()));
            assert_eq!(field(&answer, b"Content-Encoding"), None);
        }
    }
}

#[test]
fn a_compressed_form_has_no_address() {
    let scratch = Scratch::new("serve-gzip-address");
    let (out, bundle) = compressed_bundle(&scratch);
    let path = format!("/bundle/{}/notes.js.gz", bundle.digest);
    let call = skein_ui_web_tests::get_with_headers(path.as_bytes(), &[(b"Accept-Encoding", b"gzip")]);
    assert_eq!(routes::route(&mut State::new(), &bundle, &call, &[]).head.status, 404);
    let call = skein_ui_web_tests::get_with_headers(b"/", &[(b"Accept-Encoding", b"gzip")]);
    let answer = routes::route(&mut State::new(), &bundle, &call, &[]);
    assert_eq!(answer.body.as_ref(), std::fs::read(out.join("index.html")).expect("page"));
    assert_eq!(field(&answer, b"Content-Encoding"), None);
    assert_eq!(field(&answer, b"Vary"), None);
}
