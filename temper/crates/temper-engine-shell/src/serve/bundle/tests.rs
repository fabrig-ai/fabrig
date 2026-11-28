use super::{Bundle, Clients, Entry, EntryKind, Unread, embedded, prepare, read_with};
use crate::test_bundle_fixture::{BUNDLE_FILES, BUNDLE_LIMITS, PAGE};
use std::{ffi::OsStr, io, os::unix::ffi::OsStrExt, path::Path};

const ROOT: &str = "/fixture/bundle";
const CONFIGURED: &[u8] = b"[internal]\nbundle = \"/fixture/bundle\"\n";

/// Treat flat file values and any empty directories as directory entries, in fixture order.
fn directory_read(
    configured: &[u8],
    files: &[(&str, &[u8])],
    empty: &[&str],
    limits: Clients,
) -> Result<Bundle, String> {
    let root = Path::new(OsStr::from_bytes(configured));
    read_with(
        configured,
        limits,
        |at| {
            let relative = at.strip_prefix(root).expect("listing path under the bundle");
            let mut entries: Vec<Entry<_>> = Vec::new();
            for (path, is_file) in
                empty.iter().map(|path| (*path, false)).chain(files.iter().map(|(path, _)| (*path, true)))
            {
                let Ok(tail) = Path::new(path).strip_prefix(relative) else { continue };
                let mut parts = tail.iter();
                let Some(name) = parts.next() else { continue };
                if entries.iter().any(|entry| entry.name == name) {
                    continue;
                }
                let is_file = is_file && parts.next().is_none();
                entries.push(Entry {
                    name: name.to_owned(),
                    kind: move || Ok(if is_file { EntryKind::File } else { EntryKind::Directory }),
                });
            }
            Ok(entries.into_iter().map(Ok))
        },
        |at, max| {
            let relative = at.strip_prefix(root).expect("file path under the bundle");
            let (_, bytes) = files.iter().find(|(path, _)| Path::new(path) == relative).ok_or(Unread::Absent)?;
            if bytes.len() > max as usize { Err(Unread::TooLarge { max }) } else { Ok(Box::from(*bytes)) }
        },
    )
}

#[test]
fn a_bundle_directory_is_read_with_its_one_digest_directory() {
    let bundle = directory_read(ROOT.as_bytes(), BUNDLE_FILES, &[], BUNDLE_LIMITS).expect("bundle read");
    assert_eq!(bundle.page.as_ref(), PAGE);
    assert_eq!(bundle.etag.as_ref(), b"\"edaa28e315690734\"");
    assert_eq!(bundle.digest.as_ref(), b"0123456789abcdef");
    assert_eq!(bundle.files.len(), 2);
    let script = bundle.files.get(b"boot.js".as_slice()).expect("script");
    assert_eq!((script.kind, script.bytes.as_ref()), (b"text/javascript".as_slice(), b"boot();".as_slice()));
    let robots = bundle.files.get(b"files/robots.txt".as_slice()).expect("root file");
    assert_eq!(
        (robots.kind, robots.bytes.as_ref()),
        (b"text/plain; charset=utf-8".as_slice(), b"User-agent: *\nDisallow:\n".as_slice())
    );
}

#[test]
fn embedded_and_directory_files_make_the_same_bundle() {
    let mut files = BUNDLE_FILES.to_vec();
    // Root files beside index.html are ignored in both reads.
    files.push(("ignored.bin", b"ignored"));
    let [directory, binary] = both_reads(&files, BUNDLE_LIMITS);
    let directory = directory.expect("directory");
    let binary = binary.expect("embedded");
    assert_eq!(binary.page, directory.page);
    assert_eq!(binary.etag, directory.etag);
    assert_eq!(binary.digest, directory.digest);
    assert_eq!(binary.files.len(), directory.files.len());
    for (name, _) in BUNDLE_FILES.iter().skip(1) {
        let (_, relative) = name.split_once('/').expect("digest path");
        let from_directory = directory.files.get(relative.as_bytes()).expect("directory file");
        let from_embedded = binary.files.get(relative.as_bytes()).expect("embedded file");
        assert_eq!(from_embedded.kind, from_directory.kind);
        assert_eq!(from_embedded.bytes, from_directory.bytes);
    }
}

// Four compressible files and their forms, a root file, and the page: 10 files, 73 bytes.
const FORMED_COUNT: u32 = 10;
const FORMED_BYTES: u32 = 73;
const FORMED_FILES: &[(&str, &[u8])] = &[
    ("index.html", PAGE),
    // Forms deliberately precede files in the embedded input.
    ("0123456789abcdef/boot.js.gz", b"gz js"),
    ("0123456789abcdef/a/module.wasm.gz", b"gz wasm"),
    ("0123456789abcdef/a/style.css.gz", b"gz css"),
    ("0123456789abcdef/image.svg.gz", b"gz svg"),
    ("0123456789abcdef/boot.js", b"js"),
    ("0123456789abcdef/a/module.wasm", b"wasm"),
    ("0123456789abcdef/a/style.css", b"css"),
    ("0123456789abcdef/image.svg", b"svg"),
    ("0123456789abcdef/files/robots.txt", b"robots"),
];
fn both_reads(files: &[(&str, &[u8])], limits: Clients) -> [Result<Bundle, String>; 2] {
    [directory_read(ROOT.as_bytes(), files, &[], limits), embedded(files, limits)]
}

#[test]
fn a_bundle_reads_each_form_onto_its_file() {
    let limits = Clients { bundle_files: FORMED_COUNT, bundle_bytes: FORMED_BYTES };
    for read in both_reads(FORMED_FILES, limits) {
        let bundle = read.expect("both forms read at their bounds");
        assert_eq!(bundle.files.len(), 5);
        for (name, plain, gzip) in [
            (b"boot.js".as_slice(), b"js".as_slice(), b"gz js".as_slice()),
            (b"a/module.wasm", b"wasm", b"gz wasm"),
            (b"a/style.css", b"css", b"gz css"),
            (b"image.svg", b"svg", b"gz svg"),
        ] {
            let file = bundle.files.get(name).expect("plain file");
            assert_eq!(file.bytes.as_ref(), plain);
            assert_eq!(file.gzip.as_deref(), Some(gzip));
        }
        assert_eq!(bundle.files.get(b"files/robots.txt".as_slice()).expect("root file").gzip, None);
        assert!(bundle.files.get(b"boot.js.gz".as_slice()).is_none());
    }
}

#[test]
fn the_compressed_forms_count_against_the_bounds() {
    for (limits, why) in [
        (Clients { bundle_files: FORMED_COUNT - 1, ..BUNDLE_LIMITS }, "past clients.bundle_files"),
        (Clients { bundle_bytes: FORMED_BYTES - 1, ..BUNDLE_LIMITS }, "past clients.bundle_bytes"),
    ] {
        for read in both_reads(FORMED_FILES, limits) {
            assert!(read.expect_err("form exceeds bound").contains(why));
        }
    }
}

#[test]
fn a_compressed_form_with_no_file_refuses_the_start() {
    let files = [("index.html", PAGE), ("0123456789abcdef/absent.js.gz", b"gz".as_slice())];
    for read in both_reads(&files, BUNDLE_LIMITS) {
        assert!(read.expect_err("orphan form").contains("a compressed form with no file"));
    }
}

#[test]
fn a_gz_of_another_type_refuses_the_start() {
    let files = [("index.html", PAGE), ("0123456789abcdef/x.txt.gz", b"gz".as_slice())];
    for read in both_reads(&files, BUNDLE_LIMITS) {
        assert!(read.expect_err("type off closed list").contains("file type is not on the closed list"));
    }
}

fn serve_refusal(files: &[(&str, &[u8])], empty: &[&str]) -> String {
    use temper_config_shell::{Facts, Shape, parse};
    use temper_service_shell::configuration::Read;
    let facts = Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: false };
    let read = Read::from(parse(CONFIGURED, Shape::Standalone, &facts).expect("configuration file"));
    let directories = jig_protocol_config::Directories::given(b"/fixture/config", b"/fixture/data", b"/fixture/cache")
        .expect("configuration directories");
    let named = temper_service::startup::named(&read.document, &directories).expect("named startup inputs");
    // Use before_root's bundle step, then the same member mapping and prefix as serve.
    let why = prepare(named.bundle.as_deref(), named.bundle_limits, named.sign_in.is_some(), false, |path, limits| {
        directory_read(path, files, empty, limits)
    })
    .expect_err("bundle refused before opening the root");
    crate::prefixed("temper serve", &read.problems(&why))
}

#[test]
fn a_bundle_page_without_a_head_refuses_the_start_for_its_metadata() {
    assert_eq!(
        serve_refusal(BUNDLE_FILES, &[]),
        format!(
            "temper serve: {}:2: internal.bundle: /fixture/bundle: the page has no </head> for its sign-in metadata\n",
            temper_config_shell::FILE
        )
    );
}

#[test]
fn a_directory_that_is_no_bundle_refuses_the_start() {
    let mut unsupported = BUNDLE_FILES.to_vec();
    unsupported.push(("0123456789abcdef/files/data.bin", b"x"));
    let second_digest = ["fedcba9876543210"];
    for (name, files, empty, expected) in [
        ("no-page", &BUNDLE_FILES[1..], &[][..], "/fixture/bundle/index.html: absent"),
        ("no-digest", &BUNDLE_FILES[..1], &[][..], "/fixture/bundle: no digest directory"),
        (
            "two-digests",
            BUNDLE_FILES,
            &second_digest[..],
            "/fixture/bundle/fedcba9876543210: more than one digest directory",
        ),
        (
            "off-the-list",
            unsupported.as_slice(),
            &[][..],
            "/fixture/bundle/0123456789abcdef/files/data.bin: file type is not on the closed list",
        ),
    ] {
        assert_eq!(
            serve_refusal(files, empty),
            format!("temper serve: {}:2: internal.bundle: {expected}\n", temper_config_shell::FILE),
            "row {name}"
        );
    }
}

#[test]
fn listing_refusals_precede_entry_kinds() {
    use io::ErrorKind::PermissionDenied;
    let limits = Clients { bundle_files: 1, bundle_bytes: 1 };
    for (listing, expected) in [
        (
            [Ok("z"), Ok("y"), Ok("x"), Err(PermissionDenied)],
            format!("application.clients.bundle: {ROOT}/x: too many entries"),
        ),
        (
            [Ok("z"), Err(PermissionDenied), Ok("x"), Ok("y")],
            format!("application.clients.bundle: {ROOT}: {}", io::Error::from(PermissionDenied)),
        ),
    ] {
        let why = read_with(
            ROOT.as_bytes(),
            limits,
            |at| {
                assert_eq!(at, Path::new(ROOT));
                Ok(listing.into_iter().map(|entry| {
                    entry.map_err(io::Error::from).map(|name| Entry {
                        name: name.into(),
                        kind: || panic!("the listing's refusal comes before any kind read"),
                    })
                }))
            },
            |_, _| Ok(Box::new([])),
        )
        .expect_err("refused listing");
        assert_eq!(why, expected);
    }
}

#[test]
fn directory_reads_page_first_then_sorted_files_at_remaining_credit() {
    use std::cell::Cell;
    let reads =
        [("index.html", 6, b"I".as_slice()), ("0123456789abcdef/a.js", 5, b"aa"), ("0123456789abcdef/b.js", 3, b"bbb")];
    let next = Cell::new(0);
    let limits = Clients { bundle_files: 3, bundle_bytes: 6 };
    let root = Path::new(ROOT);
    read_with(
        ROOT.as_bytes(),
        limits,
        |at| {
            assert_eq!(next.get(), 1, "list only after reading the page, before reading any asset");
            let entries = if at == root {
                vec![("0123456789abcdef", EntryKind::Directory), ("index.html", EntryKind::File)]
            } else {
                assert_eq!(at, root.join("0123456789abcdef"));
                vec![("b.js", EntryKind::File), ("a.js", EntryKind::File)]
            };
            Ok(entries.into_iter().map(|(name, kind)| Ok(Entry { name: name.into(), kind: move || Ok(kind) })))
        },
        |at, max| {
            let (name, credit, bytes) = reads[next.get()];
            assert_eq!(at, root.join(name));
            assert_eq!(max, credit, "credit for {name}");
            next.set(next.get() + 1);
            Ok(Box::from(bytes))
        },
    )
    .expect("read at both bounds");
    assert_eq!(next.get(), reads.len());
}
