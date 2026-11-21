use skein_ui_example_server::bundle;
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
