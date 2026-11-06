use super::*;
#[test]
fn kinds_follow_the_closed_list() {
    assert_eq!(kind(b"index.html".as_slice()), Some(b"text/html; charset=utf-8".as_slice()));
    assert_eq!(kind(b"boot.js"), Some(b"text/javascript".as_slice()));
    assert_eq!(kind(b"app.wasm"), Some(b"application/wasm".as_slice()));
    assert_eq!(kind(b"app.css"), Some(b"text/css".as_slice()));
    assert_eq!(kind(b"image.svg"), Some(b"image/svg+xml".as_slice()));
    assert_eq!(kind(b"image.png"), Some(b"image/png".as_slice()));
    assert_eq!(kind(b"image.webp"), Some(b"image/webp".as_slice()));
    assert_eq!(kind(b"favicon.ico"), Some(b"image/x-icon".as_slice()));
    assert_eq!(kind(b"font.woff2"), Some(b"font/woff2".as_slice()));
    assert_eq!(kind(b"robots.txt"), Some(b"text/plain; charset=utf-8".as_slice()));
    assert_eq!(kind(b"app.webmanifest"), Some(b"application/manifest+json".as_slice()));
    for path in [b"dotless".as_slice(), b"a.b/c", b"script.exe"] {
        assert_eq!(kind(path), None);
    }
}
#[test]
fn files_stay_under_their_digest_and_robots_names_a_root_file() {
    let mut files = Map::with_capacity(1);
    files
        .insert(Box::from(b"files/robots.txt".as_slice()), File { kind: b"", bytes: Box::new([]), gzip: None })
        .unwrap();
    assert!(file(&files, b"d", b"/bundle/d/files/robots.txt").is_some());
    assert!(file(&files, b"d", b"/robots.txt").is_some());
    for path in [b"/bundle/x/files/robots.txt".as_slice(), b"/bundle/d/", b"/bundle/d/unknown.js"] {
        assert!(file(&files, b"d", path).is_none());
    }
}

#[test]
fn compressed_forms_name_only_compressible_files() {
    for (form, plain) in [
        (b"a/x.js.gz".as_slice(), b"a/x.js".as_slice()),
        (b"a/x.wasm.gz", b"a/x.wasm"),
        (b"a/x.css.gz", b"a/x.css"),
        (b"a/x.svg.gz", b"a/x.svg"),
    ] {
        assert_eq!(compressed_of(form), Some(plain));
    }
    for path in [b"x.txt.gz".as_slice(), b"x.gz", b"x.js", b"x.js.gz.gz"] {
        assert_eq!(compressed_of(path), None);
    }
    assert_eq!(kind(b"x.js.gz"), None);
}
