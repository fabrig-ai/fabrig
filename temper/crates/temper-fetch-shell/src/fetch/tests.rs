use super::*;
use crate::testing::{digest as sha, zstd};
use std::cell::{Cell, RefCell};
use support::{CacheScript, Effect, FILE as CACHE_FILE, Name, Wire};
use temper_fetch::fetch::{self as model, Named, Pin};

#[path = "../../../../testing/forge_fetch.rs"]
mod forge_fetch;
mod support;

#[test]
fn fetch_effect_rows() {
    use forge_fetch::{CACHE, RELEASE, ROOTS, Row, Script};
    use model::Fetched;

    // This driver receives a pin; forge_fetch_rows checks Linux pin selection.
    let binary = Fetched::read(b"fetched forgejo 0 of 3 bytes\n").expect("Forgejo progress name").binary;
    let pin = Pin {
        binary,
        origin: Origin { host: b"mirror.example.invalid".as_slice().into(), port: 443 },
        path: RELEASE.into(),
        digest: [0x42; 32],
        file: CACHE.into(),
        packed: true,
    };
    for row in [Row::Digest, Row::Present, Row::MissingRoots] {
        let host = Script::new(Some(row));
        let mut stdout = Vec::new();
        let result = run_with(
            Fetch::new(Box::new([pin.clone()]), 32),
            || host.roots(Path::new(ROOTS)),
            |(), request| Ok(host.answer(request)),
            &mut stdout,
        );
        assert_eq!(result.expect_err("refused fetch").words(), row.diagnostic(&"42".repeat(32)));
        assert_eq!(stdout, row.stdout(), "{row:?}");
        host.check();
    }
}

#[test]
fn claude_cache_hit_and_refusals_request_only_metadata() {
    let file = Path::new("/cache/claude-code/2.1.311/claude");
    let size = 200;
    let present = "/cache/claude-code/2.1.311/claude: does not match the pinned digest";
    for (row, metadata, expected) in [
        ("sized hit", (true, size), Ok(())),
        ("wrong size", (true, size - 1), Err(present)),
        ("not a file", (false, size), Err(present)),
    ] {
        let requests = RefCell::new(Vec::new());
        let mut out = Vec::new();
        let result = claude_code_with(
            file,
            size,
            |candidate| {
                assert_eq!(candidate, file, "{row}");
                requests.borrow_mut().push("metadata");
                Ok(metadata)
            },
            |_| {
                requests.borrow_mut().push("prepare cache directory");
                Ok(())
            },
            |_| {
                requests.borrow_mut().push("lock");
                Ok(())
            },
            |_| {
                requests.borrow_mut().push("fetch");
                Ok(())
            },
            &mut out,
        )
        .map_err(|why| why.words());
        assert_eq!(result.as_ref().map_err(String::as_str), expected.as_ref().map_err(|why| *why), "{row}");
        assert_eq!(requests.into_inner(), ["metadata"], "{row}");
        assert!(out.is_empty(), "{row}: no fetching notice");
    }
}

#[test]
fn a_claude_cache_miss_is_checked_again_under_the_lock() {
    let file = Path::new("/cache/claude-code/2.1.311/claude");
    let size = 200;
    for found_under_lock in [true, false] {
        let requests = RefCell::new(Vec::new());
        let mut out = Vec::new();
        claude_code_with(
            file,
            size,
            |candidate| {
                assert_eq!(candidate, file);
                let mut requests = requests.borrow_mut();
                let under_lock = requests.last() == Some(&"lock");
                requests.push("metadata");
                if under_lock && found_under_lock { Ok((true, size)) } else { Err(io::ErrorKind::NotFound.into()) }
            },
            |candidate| {
                assert_eq!(candidate, file);
                requests.borrow_mut().push("prepare cache directory");
                Ok(())
            },
            |candidate| {
                assert_eq!(candidate, file);
                requests.borrow_mut().push("lock");
                Ok(())
            },
            |_| {
                requests.borrow_mut().push("fetch");
                Ok(())
            },
            &mut out,
        )
        .expect("cache hit or fetch");
        let mut expected = vec!["metadata", "prepare cache directory", "lock", "metadata"];
        if found_under_lock {
            assert_eq!(out, b"");
        } else {
            expected.push("fetch");
            assert_eq!(out, b"fetching Claude Code\n");
        }
        assert_eq!(requests.into_inner(), expected);
    }
}

const FILE: &[u8] = b"/cache/claude";
const PINNED_BYTES: u64 = 8; // A synthetic build size, independent of the host's pinned build.

#[test]
fn a_regular_cached_build_at_the_pin_is_present() {
    assert!(claude_present(FILE, PINNED_BYTES, Ok((true, PINNED_BYTES))).expect("matching build"));
}

#[test]
fn a_cached_build_one_byte_short_is_refused() {
    let refusal = claude_present(FILE, PINNED_BYTES, Ok((true, PINNED_BYTES - 1))).expect_err("wrong size");
    assert!(matches!(refusal, Failed::Refused(temper_fetch::fetch::Failure::Present { .. })));
    assert_eq!(refusal.words(), "/cache/claude: does not match the pinned digest");
}

const PINNED_URL: &str = "https://mirror.test:8443/binary";
const PINNED_HEAD: &[u8] =
    b"GET /binary HTTP/1.1\r\nHost: mirror.test:8443\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n";
const STORAGE_HEAD: &[u8] = b"GET /object?signature=a%2Fb HTTP/1.1\r\nHost: storage.test:9443\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n";
const RIGHT: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc";
const WRONG: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nxyz";
const GONE: &[u8] = b"HTTP/1.1 404 Missing\r\nContent-Length: 0\r\n\r\n";
const REDIRECT: &[u8] =
    b"HTTP/1.1 307 Redirect\r\nLocation: https://storage.test:9443/object?signature=a%2Fb\r\nContent-Length: 0\r\n\r\n";
const CHUNKED_PAST_BOUND: &[u8] =
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nab\r\n2\r\ncd\r\n0\r\n\r\n";
const DIGEST_WORDS: &str = "digest differs: pinned ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad, got 3608bca1e44ea6c4d268eb6db02260269892c0b42b86bbf1e77a6fa16c3c9282";
const PRESENT_WORDS: &str = "/cache/binary: does not match the pinned digest";

fn mirror() -> Origin {
    Origin { host: b"mirror.test".as_slice().into(), port: 8443 }
}

fn pin(binary: Named) -> Pin {
    Pin {
        binary,
        origin: mirror(),
        path: b"/binary".as_slice().into(),
        digest: sha(b"abc"),
        file: CACHE_FILE.into(),
        packed: binary != Named::ClaudeCode,
    }
}

fn forgejo() -> Named {
    model::Fetched::read(b"fetched forgejo 0 bytes").expect("Forgejo progress name").binary
}

fn scripted(
    pin: Pin,
    bound: u64,
    files: &mut Download<CacheScript>,
    wire: &mut Wire<'_>,
    out: &mut dyn io::Write,
) -> Result<(), Failed> {
    let result = run_with(
        Fetch::new(Box::new([pin]), bound),
        || Ok(()),
        |(), request| files.answer(request, |request| wire.answer(request)),
        out,
    );
    files.close_part();
    result
}

fn opened(part: Name) -> [Effect; 3] {
    [Effect::Parent(0o700), Effect::Open(part, true, 0o600), Effect::Mode(part, 0o600)]
}

fn published() -> [Effect; 4] {
    [
        Effect::Mode(Name::Part, 0o755),
        Effect::Sync(Name::Part),
        Effect::Rename(Name::Part, Name::Binary),
        Effect::SyncParent,
    ]
}

fn wrote_part(len: usize) -> Vec<Effect> {
    let mut effects = vec![Effect::Hash];
    effects.extend(opened(Name::Part));
    effects.push(Effect::Write(Name::Part, 0, len));
    effects
}

fn saved_abc() -> Vec<Effect> {
    let mut effects = wrote_part(3);
    effects.extend(published());
    effects
}

fn discarded_part(len: usize) -> Vec<Effect> {
    let mut effects = wrote_part(len);
    effects.push(Effect::Remove(Name::Part));
    effects
}

fn check_cache(cache: &mut CacheScript, expected: Option<&[u8]>, effects: &[Effect], row: &str) {
    assert_eq!(cache.bytes(Name::Binary).as_deref(), expected, "{row}: cached binary");
    assert_eq!(cache.bytes(Name::Part), None, "{row}: no part left");
    assert_eq!(cache.bytes(Name::Packed), None, "{row}: no packed part left");
    assert_eq!(cache.effects, effects, "{row}: cache effects in order");
    cache.settled();
}

#[test]
fn pinned_fetch_rows() {
    for (row, content, expected) in
        [("matching", b"abc".as_slice(), Ok(())), ("present", b"xyz".as_slice(), Err(PRESENT_WORDS.into()))]
    {
        let mut files = Download::new(CacheScript::new(Some(content), None, None));
        let mut wire = Wire::new(&[]);
        let mut out = Vec::new();
        let result = scripted(pin(forgejo()), 3, &mut files, &mut wire, &mut out);
        assert_eq!(result.map_err(|why| why.words()), expected, "{row}");
        assert!(out.is_empty(), "{row}: present files say nothing");
        wire.check_heads(&[]);
        check_cache(&mut files.cache, Some(content), &[Effect::Hash], row);
    }
}

fn response(body: &[u8], chunked: bool) -> Vec<u8> {
    if chunked {
        let mut wire =
            format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n", body.len()).into_bytes();
        wire.extend_from_slice(body);
        wire.extend_from_slice(b"\r\n0\r\n\r\n");
        wire
    } else {
        let mut wire = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
        wire.extend_from_slice(body);
        wire
    }
}

fn get(path: &[u8]) -> WireRequest {
    WireRequest::Get { origin: mirror(), path: path.into() }
}

#[test]
fn fetch_request_heads() {
    let location = b"https://storage.test:9443/object?signature=a%2Fb";
    let mut wire = Wire::new(&[RIGHT, REDIRECT, CHUNKED_PAST_BOUND]);
    let ok = |length| Event::Answered { status: 200, length, location: None };
    assert_eq!(wire.answer(get(b"/binary?signature=a%2Fb")).unwrap(), ok(Some(3)));
    assert_eq!(wire.answer(WireRequest::Read).unwrap(), Event::Chunk(b"abc".as_slice().into()));
    assert_eq!(wire.answer(WireRequest::Read).unwrap(), Event::Ended);
    assert_eq!(
        wire.answer(get(b"/binary")).unwrap(),
        Event::Answered { status: 307, length: Some(0), location: Some(location.as_slice().into()) }
    );
    assert_eq!(
        wire.answer(WireRequest::Get {
            origin: Origin { host: b"storage.test".as_slice().into(), port: 9443 },
            path: b"/object?signature=a%2Fb".as_slice().into(),
        })
        .unwrap(),
        ok(None)
    );
    for piece in [b"ab", b"cd"] {
        assert_eq!(wire.answer(WireRequest::Read).unwrap(), Event::Chunk(piece.as_slice().into()));
    }
    assert_eq!(wire.answer(WireRequest::Read).unwrap(), Event::Ended);
    wire.check_heads(&[
        b"GET /binary?signature=a%2Fb HTTP/1.1\r\nHost: mirror.test:8443\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n",
        PINNED_HEAD, STORAGE_HEAD,
    ]);
}

const PACKED_BOUND: u64 = 1_048_576;

fn wrote_packed(bytes: usize) -> Vec<Effect> {
    let mut effects = vec![Effect::Hash];
    effects.extend(opened(Name::Packed));
    effects.push(Effect::Write(Name::Packed, 0, bytes));
    effects
}

fn unpacking(bytes: usize) -> Vec<Effect> {
    let mut effects = wrote_packed(bytes);
    effects.push(Effect::ReadPacked);
    effects.extend(opened(Name::Part));
    effects
}

fn published_packed(packed: usize, unpacked: usize) -> Vec<Effect> {
    let mut effects = unpacking(packed);
    effects.push(Effect::Write(Name::Part, 0, unpacked));
    effects.extend(published());
    effects.push(Effect::Remove(Name::Packed));
    effects
}

#[test]
fn fetch_progress_rows() {
    let packed = zstd(b"abc");
    let bytes = packed.len();
    let declared = response(&packed, false);
    let chunked = response(&packed, true);
    for (row, response, refuses, expected) in [
        (
            "declared length",
            &declared,
            false,
            format!("fetched runner 0 of {bytes} bytes\nfetched runner {bytes} of {bytes} bytes\n"),
        ),
        ("chunked", &chunked, false, format!("fetched runner 0 bytes\nfetched runner {bytes} bytes\n")),
        ("refusing writer", &declared, true, String::new()),
    ] {
        let mut files = Download::new(CacheScript::new(None, None, None));
        let mut wire = Wire::new(&[response]);
        let mut out = Vec::new();
        let mut full = &mut [][..];
        let writer: &mut dyn io::Write = if refuses { &mut full } else { &mut out };
        let runner = model::Fetched::read(b"fetched runner 0 bytes").expect("runner progress name").binary;
        scripted(pin(runner), PACKED_BOUND, &mut files, &mut wire, writer)
            .expect("the writer does not change the fetch result");
        assert_eq!(out, expected.as_bytes(), "{row}");
        assert_eq!(full, b"");
        wire.check_heads(&[PINNED_HEAD]);
        check_cache(&mut files.cache, Some(b"abc"), &published_packed(bytes, 3), row);
    }
}

const FORGEJO_AMD64: &[u8] = b"/temper/forgejo/17.0.1/forgejo-17.0.1-linux-amd64.zst";

fn site_pin(name: Named, path: &[u8], digest: [u8; 32]) -> Pin {
    Pin {
        binary: name,
        origin: Origin { host: b"fabrig.ai".as_slice().into(), port: 443 },
        path: path.into(),
        digest,
        file: CACHE_FILE.into(),
        packed: true,
    }
}

fn site_head(path: &[u8]) -> String {
    let path = std::str::from_utf8(path).unwrap();
    format!("GET {path} HTTP/1.1\r\nHost: fabrig.ai\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n")
}

#[test]
fn packed_fetch_platforms() {
    let runner = model::Fetched::read(b"fetched runner 0 bytes").unwrap().binary;
    for (forgejo_path, runner_path) in [
        (FORGEJO_AMD64, b"/temper/forgejo-runner/13.2.1/forgejo-runner-13.2.1-linux-amd64.zst".as_slice()),
        (
            b"/temper/forgejo/17.0.1/forgejo-17.0.1-linux-arm64.zst",
            b"/temper/forgejo-runner/13.2.1/forgejo-runner-13.2.1-linux-arm64.zst",
        ),
        (
            b"/temper/forgejo/17.0.1/forgejo-17.0.1-darwin-arm64.zst",
            b"/temper/forgejo-runner/13.2.1/forgejo-runner-13.2.1-darwin-arm64.zst",
        ),
    ] {
        for (name, path) in [(forgejo(), forgejo_path), (runner, runner_path)] {
            // The release pins are for binaries up to 122 MB, not test fixtures.
            // Give the same request path a pin of the fixture's unpacked bytes.
            let body = [path, b" fixture bytes"].concat();
            let packed = zstd(&body);
            let response = response(&packed, false);
            let mut wire = Wire::new(&[&response]);
            let stale = [b'x'; 1024];
            let mut files = Download::new(CacheScript::new(None, Some(&stale), Some(&stale)));
            scripted(site_pin(name, path, sha(&body)), PACKED_BOUND, &mut files, &mut wire, &mut io::sink())
                .expect("the unpacked fixture matches its pin");
            let head = site_head(path);
            wire.check_heads(&[head.as_bytes()]);
            check_cache(&mut files.cache, Some(&body), &published_packed(packed.len(), body.len()), &head);
        }
    }
}

#[test]
fn packed_fetch_refusals() {
    // The declared-size RLE frame produces 1025 bytes (0x0301 + 256),
    // above the 1024-byte bound but well below it as served.
    const OVERSIZED: &[u8] = b"\x28\xb5\x2f\xfd\x60\x01\x03\x0b\x20\x00a";
    // No size in the header: a 1 KiB window, 1024 RLE bytes then one raw byte.
    const OVERSIZED_STREAM: &[u8] = b"\x28\xb5\x2f\xfd\x00\x00\x02\x20\x00a\x09\x00\x00b";
    let packed = zstd(b"xyz");
    let trailing = [&packed[..], b"!"].concat();
    for (body, bound, written, words) in [
        (&packed[..packed.len() - 1], PACKED_BOUND, 0, "truncated zstd frame"),
        (b"abc".as_slice(), PACKED_BOUND, 0, "damaged zstd frame"),
        (OVERSIZED, 1024, 0, "unpacked file past setup.fetch_bytes"),
        (OVERSIZED_STREAM, 1024, 0, "unpacked file past setup.fetch_bytes"),
        (&trailing, PACKED_BOUND, 3, "zstd frame followed by other bytes"),
        (&packed, PACKED_BOUND, 3, DIGEST_WORDS),
    ] {
        let response = response(body, false);
        let mut wire = Wire::new(&[&response]);
        let mut files = Download::new(CacheScript::new(None, None, None));
        let why =
            scripted(site_pin(forgejo(), FORGEJO_AMD64, sha(b"abc")), bound, &mut files, &mut wire, &mut io::sink())
                .expect_err("refused packed fetch");
        assert_eq!(why.words(), format!("https://fabrig.ai{}: {words}", std::str::from_utf8(FORGEJO_AMD64).unwrap()));
        wire.check_heads(&[site_head(FORGEJO_AMD64).as_bytes()]);
        let mut effects = unpacking(body.len());
        if written != 0 {
            effects.push(Effect::Write(Name::Part, 0, written));
        }
        effects.extend([Effect::Remove(Name::Part), Effect::Remove(Name::Packed)]);
        check_cache(&mut files.cache, None, &effects, words);
    }
}

#[test]
fn a_packed_fetch_refuses_status_and_body_bound() {
    let packed = zstd(b"abc");
    let declared = response(&packed, false);
    let mut chunked = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n".to_vec();
    let (prefix, rest) = packed.split_at(4);
    chunked.extend_from_slice(prefix);
    chunked.extend_from_slice(format!("\r\n{:x}\r\n", rest.len()).as_bytes());
    chunked.extend_from_slice(rest);
    chunked.extend_from_slice(b"\r\n0\r\n\r\n");
    let no_parts = || vec![Effect::Hash];
    let mut discarded = wrote_packed(prefix.len());
    discarded.push(Effect::Remove(Name::Packed));
    let small = u64::try_from(packed.len() - 1).unwrap();
    for (response, bound, words, effects) in [
        (GONE, PACKED_BOUND, "HTTP status 404", no_parts()),
        (REDIRECT, PACKED_BOUND, "HTTP status 307", no_parts()),
        (&declared, small, "body past setup.fetch_bytes", no_parts()),
        (&chunked, small, "body past setup.fetch_bytes", discarded),
    ] {
        let mut files = Download::new(CacheScript::new(None, None, None));
        let mut wire = Wire::new(&[response]);
        let why = scripted(pin(forgejo()), bound, &mut files, &mut wire, &mut io::sink()).expect_err("refused");
        assert_eq!(why.words(), format!("{PINNED_URL}: {words}"));
        wire.check_heads(&[PINNED_HEAD]);
        check_cache(&mut files.cache, None, &effects, words);
    }
}

#[test]
fn a_packed_body_past_its_content_length_is_discarded() {
    let packed = zstd(b"abc");
    let length = u64::try_from(packed.len() - 1).unwrap();
    let mut files = Download::new(CacheScript::new(None, None, None));
    // HTTP never emits excess body bytes. Supply the fault at its boundary
    // to witness the cache's cleanup when the machine detects it.
    let mut pieces = [Event::Chunk(packed[..4].into()), Event::Chunk(packed[4..].into())].into_iter();
    let why = run_with(
        Fetch::new(Box::new([pin(forgejo())]), PACKED_BOUND),
        || Ok(()),
        |(), request| {
            files.answer(request, |request| {
                Ok(match request {
                    WireRequest::Get { .. } => Event::Answered { status: 200, length: Some(length), location: None },
                    WireRequest::Read => pieces.next().expect("no demand after the extra bytes"),
                })
            })
        },
        &mut io::sink(),
    )
    .expect_err("refused excess body bytes");
    assert_eq!(why.words(), format!("{PINNED_URL}: body past its Content-Length"));
    assert!(pieces.next().is_none());
    let mut effects = wrote_packed(4);
    effects.push(Effect::Remove(Name::Packed));
    check_cache(&mut files.cache, None, &effects, "body past its Content-Length");
}

struct Held<'a>(&'a Cell<bool>);

impl Drop for Held<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn scripted_claude(
    files: &mut Download<CacheScript>,
    wire: &mut Wire<'_>,
    out: &mut dyn io::Write,
    calls: &RefCell<Vec<&'static str>>,
) -> Result<(), Failed> {
    let files = RefCell::new(files);
    let file = path(CACHE_FILE);
    let held = Cell::new(false);
    let result = claude_code_with(
        file,
        3,
        |candidate| {
            assert_eq!(candidate, file);
            calls.borrow_mut().push(if held.get() { "metadata under lock" } else { "metadata" });
            let bytes = files.borrow_mut().cache.bytes(Name::Binary).ok_or(io::ErrorKind::NotFound)?;
            Ok((true, u64::try_from(bytes.len()).unwrap()))
        },
        |candidate| {
            assert_eq!(candidate, file);
            calls.borrow_mut().push("prepare");
            Ok(())
        },
        |candidate| {
            assert_eq!(candidate, file);
            calls.borrow_mut().push("lock");
            assert!(!held.replace(true));
            Ok(Held(&held))
        },
        |out| {
            assert!(held.get(), "the cache lock covers the download");
            calls.borrow_mut().push("download");
            let mut files = files.borrow_mut();
            let result = run_with(
                Fetch::new(Box::new([pin(Named::ClaudeCode)]), 3),
                || {
                    calls.borrow_mut().push("roots");
                    Ok(())
                },
                |(), request| files.answer(request, |request| wire.answer(request)),
                out,
            );
            files.close_part();
            result
        },
        out,
    );
    assert!(!held.get(), "the cache lock is released on return");
    result
}

#[test]
fn claude_download_rows() {
    use Effect::Hash;
    let long = response(b"abcd", false);
    for (row, initial, response, expected, final_bytes, effects) in [
        ("missing", None, Some(RIGHT), Ok(()), Some(b"abc".as_slice()), saved_abc()),
        ("digest", None, Some(WRONG), Err(format!("{PINNED_URL}: {DIGEST_WORDS}")), None, discarded_part(3)),
        (
            "bound",
            None,
            Some(long.as_slice()),
            Err(format!("{PINNED_URL}: body past the pinned size")),
            None,
            vec![Hash],
        ),
        (
            "size",
            Some(b"other size".as_slice()),
            None,
            Err(PRESENT_WORDS.into()),
            Some(b"other size".as_slice()),
            vec![],
        ),
    ] {
        let mut files = Download::new(CacheScript::new(initial, None, None));
        let mut wire = Wire::new(response.as_slice());
        let calls = RefCell::new(Vec::new());
        let mut out = Vec::new();
        let result = scripted_claude(&mut files, &mut wire, &mut out, &calls);
        if row == "digest" {
            assert!(matches!(result, Err(Failed::Refused(Failure::Digest { .. }))));
        }
        assert_eq!(result.map_err(|why| why.words()), expected, "{row}");
        if initial.is_some() {
            assert_eq!(calls.into_inner(), ["metadata"], "{row}");
            assert!(out.is_empty(), "{row}");
        } else {
            assert_eq!(
                calls.into_inner(),
                ["metadata", "prepare", "lock", "metadata under lock", "download", "roots"],
                "{row}"
            );
        }
        wire.check_heads(if response.is_some() { &[PINNED_HEAD] } else { &[] });
        check_cache(&mut files.cache, final_bytes, &effects, row);
        if row == "missing" {
            assert_eq!(
                out,
                b"fetching Claude Code\nfetched claude-code 0 of 3 bytes\nfetched claude-code 3 of 3 bytes\n"
            );
            out.clear();
            let calls = RefCell::new(Vec::new());
            let mut wire = Wire::new(&[]);
            scripted_claude(&mut files, &mut wire, &mut out, &calls).expect("second call is a cache hit");
            assert_eq!(calls.into_inner(), ["metadata"]);
            assert_eq!(out, b"");
            wire.check_heads(&[]);
            check_cache(&mut files.cache, final_bytes, &effects, "second call");
        }
    }
}
