use super::*;
use crate::testing::digest as sha;
use std::cell::{Cell, RefCell};
use support::{CacheScript, Effect, Extracted, FILE as CACHE_FILE, Name, Wire};
use temper_fetch::fetch::{self as model, Bottle, Extraction, Named, Pin};

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
        bottle: None,
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
const BOTTLE_URL: &[u8] = b"https://mirror.test:8443/bottle";
const BOTTLE_HEAD: &[u8] = b"GET /bottle HTTP/1.1\r\nHost: mirror.test:8443\r\nUser-Agent: temper\r\nAuthorization: Bearer QQ==\r\nConnection: close\r\n\r\n";
const MIRROR_HEAD: &[u8] =
    b"GET /bottle HTTP/1.1\r\nHost: mirror.test:8443\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n";
const STORAGE_HEAD: &[u8] = b"GET /object?signature=a%2Fb HTTP/1.1\r\nHost: storage.test:9443\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n";
const RIGHT: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc";
const WRONG: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nxyz";
const GONE: &[u8] = b"HTTP/1.1 404 Missing\r\nContent-Length: 0\r\n\r\n";
const REDIRECT: &[u8] =
    b"HTTP/1.1 307 Redirect\r\nLocation: https://storage.test:9443/object?signature=a%2Fb\r\nContent-Length: 0\r\n\r\n";
const CHUNKED_PAST_BOUND: &[u8] =
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nab\r\n2\r\ncd\r\n0\r\n\r\n";
const LONG_PART: &[u8] = b"long interrupted part";
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
        bottle: None,
    }
}

fn forgejo() -> Named {
    model::Fetched::read(b"fetched forgejo 0 bytes").expect("Forgejo progress name").binary
}

fn scripted(
    pin: Pin,
    bound: u64,
    files: &mut Download<CacheScript<'_>>,
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

fn check_cache(cache: &mut CacheScript<'_>, expected: Option<&[u8]>, effects: &[Effect], row: &str) {
    assert_eq!(cache.bytes(Name::Binary).as_deref(), expected, "{row}: cached binary");
    assert_eq!(cache.bytes(Name::Part), None, "{row}: no part left");
    assert_eq!(cache.bytes(Name::Archive), None, "{row}: no archive left");
    assert_eq!(cache.effects, effects, "{row}: cache effects in order");
    cache.settled();
}

#[test]
fn pinned_fetch_rows() {
    use Effect::Hash;
    for (row, initial, stale, response, expected, final_bytes, effects) in [
        ("wrong", None, None, Some(WRONG), Err(format!("{PINNED_URL}: {DIGEST_WORDS}")), None, discarded_part(3)),
        ("matching", Some(b"abc".as_slice()), None, None, Ok(()), Some(b"abc".as_slice()), vec![Hash]),
        (
            "present",
            Some(b"xyz".as_slice()),
            None,
            None,
            Err(PRESENT_WORDS.into()),
            Some(b"xyz".as_slice()),
            vec![Hash],
        ),
        ("leftover", None, Some(LONG_PART), Some(RIGHT), Ok(()), Some(b"abc".as_slice()), saved_abc()),
        ("404", None, None, Some(GONE), Err(format!("{PINNED_URL}: HTTP status 404")), None, vec![Hash]),
        (
            "chunked",
            None,
            None,
            Some(CHUNKED_PAST_BOUND),
            Err(format!("{PINNED_URL}: body past setup.fetch_bytes")),
            None,
            discarded_part(2),
        ),
    ] {
        let mut files = Download::new(CacheScript::new(initial, stale, None));
        let mut wire = Wire::new(response.as_slice());
        let mut out = Vec::new();
        let result = scripted(pin(forgejo()), 3, &mut files, &mut wire, &mut out);
        if row == "wrong" {
            assert!(matches!(result, Err(Failed::Refused(Failure::Digest { .. }))));
        }
        assert_eq!(result.map_err(|why| why.words()), expected, "{row}");
        if initial.is_some() {
            assert!(out.is_empty(), "{row}: present files say nothing");
        }
        wire.check_heads(if response.is_some() { &[PINNED_HEAD] } else { &[] });
        check_cache(&mut files.cache, final_bytes, &effects, row);
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

fn get(path: &[u8], token: bool) -> WireRequest {
    WireRequest::Get {
        origin: mirror(),
        path: path.into(),
        authorization: token.then(|| b"Bearer QQ==".as_slice().into()),
    }
}

#[test]
fn fetch_request_heads() {
    let location = b"https://storage.test:9443/object?signature=a%2Fb";
    let mut wire = Wire::new(&[RIGHT, REDIRECT, CHUNKED_PAST_BOUND]);
    let ok = |length| Event::Answered { status: 200, length, location: None };
    assert_eq!(wire.answer(get(b"/binary?signature=a%2Fb", false)).unwrap(), ok(Some(3)));
    assert_eq!(wire.answer(WireRequest::Read).unwrap(), Event::Chunk(b"abc".as_slice().into()));
    assert_eq!(wire.answer(WireRequest::Read).unwrap(), Event::Ended);
    assert_eq!(
        wire.answer(get(b"/bottle", true)).unwrap(),
        Event::Answered { status: 307, length: Some(0), location: Some(location.as_slice().into()) }
    );
    assert_eq!(
        wire.answer(WireRequest::Get {
            origin: Origin { host: b"storage.test".as_slice().into(), port: 9443 },
            path: b"/object?signature=a%2Fb".as_slice().into(),
            authorization: None,
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
        BOTTLE_HEAD, STORAGE_HEAD,
    ]);
}

#[test]
fn fetch_progress_rows() {
    let chunked = response(b"abc", true);
    for (row, response, refuses, expected) in [
        ("declared length", RIGHT, false, b"fetched runner 0 of 3 bytes\nfetched runner 3 of 3 bytes\n".as_slice()),
        ("chunked", chunked.as_slice(), false, b"fetched runner 0 bytes\nfetched runner 3 bytes\n".as_slice()),
        ("refusing writer", RIGHT, true, b"".as_slice()),
    ] {
        let mut files = Download::new(CacheScript::new(None, None, None));
        let mut wire = Wire::new(&[response]);
        let mut out = Vec::new();
        let mut full = &mut [][..];
        let writer: &mut dyn io::Write = if refuses { &mut full } else { &mut out };
        let runner = model::Fetched::read(b"fetched runner 0 bytes").expect("runner progress name").binary;
        scripted(pin(runner), 3, &mut files, &mut wire, writer).expect("the writer does not change the fetch result");
        assert_eq!(out, expected, "{row}");
        assert_eq!(full, b"");
        wire.check_heads(&[PINNED_HEAD]);
        check_cache(&mut files.cache, Some(b"abc"), &saved_abc(), row);
    }
}

#[derive(Clone, Copy)]
struct BottleRow<'a> {
    reply: &'a [u8],
    archive: &'a [u8],
    archive_pin: [u8; 32],
    binary_pin: [u8; 32],
    member: &'static [u8],
    bound: u64,
    token: bool,
    stale: Option<&'a [u8]>,
}

enum BottleCache {
    Unopened,
    ArchiveOverBound,
    ArchiveWritten,
    Extracted { written: usize, verdict: Extraction },
}

fn bottle_cache(row: BottleRow<'_>, cache: &BottleCache, succeeds: bool) -> Vec<Effect> {
    use Effect::{Extract, Hash, Remove, Write};
    use Name::{Archive, Part};
    let mut effects = vec![Hash];
    if !matches!(cache, BottleCache::Unopened) {
        effects.extend(opened(Archive));
        if !matches!(cache, BottleCache::ArchiveOverBound) {
            effects.push(Write(Archive, 0, row.archive.len()));
        }
        if matches!(cache, BottleCache::Extracted { .. }) {
            effects.extend(opened(Part));
            effects.push(Extract);
            if succeeds {
                effects.extend(published());
            } else {
                effects.push(Remove(Part));
            }
        }
        effects.push(Remove(Archive));
    }
    effects
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table covers all the bottle's redirect, archive and member rows")]
fn bottle_fetch_rows() {
    use BottleCache::{ArchiveOverBound, ArchiveWritten, Extracted as Member, Unopened};
    const BINARY_BYTES: usize = 32_768;
    let binary = vec![b'x'; BINARY_BYTES];
    let stale = vec![b'z'; BINARY_BYTES + 1];
    // Extraction sees the downloaded bytes and supplies a verdict. The real tar witness stays above.
    let archive = b"bottle archive";
    let good = response(archive, false);
    let chunked = response(archive, true);
    let broken = response(b"broken", false);
    let second_redirect = b"HTTP/1.1 307 Again\r\nLocation: https://storage.test/object\r\nContent-Length: 0\r\n\r\n";
    let row = BottleRow {
        reply: &good,
        archive,
        archive_pin: sha(archive),
        binary_pin: sha(&binary),
        member: b"bin/member",
        bound: BINARY_BYTES as u64,
        token: true,
        stale: None,
    };
    let digest = |pinned, got| Failure::Digest { url: BOTTLE_URL.into(), pinned, got };
    let bytes = || Failure::Bytes { url: BOTTLE_URL.into() };
    let extraction = || Failure::Extraction { url: BOTTLE_URL.into() };
    let member = || Member { written: BINARY_BYTES, verdict: Extraction::Digest(row.binary_pin) };
    let failed_member = || Member { written: 0, verdict: Extraction::Failed };
    let archive_bound = u64::try_from(archive.len()).unwrap() - 1;
    for (name, row, cache, expected) in [
        ("mirror", BottleRow { token: false, ..row }, member(), Ok(())),
        ("leftover", BottleRow { stale: Some(&stale), ..row }, member(), Ok(())),
        (
            "archive digest",
            BottleRow { archive_pin: sha(b"wrong"), ..row },
            ArchiveWritten,
            Err(digest(sha(b"wrong"), row.archive_pin)),
        ),
        (
            "binary digest",
            BottleRow { binary_pin: sha(b"wrong"), ..row },
            member(),
            Err(digest(sha(b"wrong"), row.binary_pin)),
        ),
        ("archive length", BottleRow { bound: archive_bound, ..row }, Unopened, Err(bytes())),
        ("archive stream", BottleRow { reply: &chunked, bound: archive_bound, ..row }, ArchiveOverBound, Err(bytes())),
        (
            "extraction bound",
            BottleRow { bound: row.bound - 1, ..row },
            Member { written: BINARY_BYTES / 2, verdict: Extraction::Bytes },
            Err(bytes()),
        ),
        ("missing member", BottleRow { member: b"bin/missing", ..row }, failed_member(), Err(extraction())),
        (
            "empty member",
            BottleRow { member: b"bin/empty", binary_pin: sha(b""), ..row },
            failed_member(),
            Err(extraction()),
        ),
        (
            "corrupt archive",
            BottleRow { reply: &broken, archive: b"broken", archive_pin: sha(b"broken"), ..row },
            failed_member(),
            Err(extraction()),
        ),
        (
            "second redirect",
            BottleRow { reply: second_redirect, ..row },
            Unopened,
            Err(Failure::Redirect { url: BOTTLE_URL.into() }),
        ),
        (
            "gone",
            BottleRow { reply: GONE, ..row },
            Unopened,
            Err(Failure::Gone { url: BOTTLE_URL.into(), status: 404 }),
        ),
    ] {
        let mut disk = CacheScript::new(None, row.stale, row.stale);
        if let Member { written, verdict } = &cache {
            disk.extraction = Some(Extracted {
                archive: row.archive,
                member: row.member,
                bound: row.bound,
                written: &binary[..*written],
                verdict: verdict.clone(),
            });
        }
        let mut files = Download::new(disk);
        let mut wire = Wire::new(&[REDIRECT, row.reply]);
        let pin = Pin {
            path: b"/bottle".as_slice().into(),
            digest: row.binary_pin,
            bottle: Some(Bottle { digest: row.archive_pin, member: row.member.into(), token: row.token }),
            ..pin(forgejo())
        };
        let result = scripted(pin, row.bound, &mut files, &mut wire, &mut Vec::new());
        let result = result.map_err(|why| {
            let Failed::Refused(refusal) = why else { panic!("{name}: expected a fetch refusal, got {why:?}") };
            refusal
        });
        assert_eq!(result, expected, "{name}");
        wire.check_heads(&[if row.token { BOTTLE_HEAD } else { MIRROR_HEAD }, STORAGE_HEAD]);
        let effects = bottle_cache(row, &cache, expected.is_ok());
        check_cache(&mut files.cache, expected.is_ok().then_some(binary.as_slice()), &effects, name);
    }
}

struct Held<'a>(&'a Cell<bool>);

impl Drop for Held<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn scripted_claude(
    files: &mut Download<CacheScript<'_>>,
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
