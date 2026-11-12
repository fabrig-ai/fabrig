#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust tables drive step events and compare complete requests"
)]
use super::*;
const FILE: &[u8] = b"/cache/bin/0123456789ab/Linux-x86_64/temper-box";
const DIR: &[u8] = b"/cache/bin/0123456789ab/Linux-x86_64";
fn build() -> Build {
    Build {
        beside: b"/bin".as_slice().into(),
        cache: b"/cache".as_slice().into(),
        site: Origin { host: b"site.test".as_slice().into(), port: 443 },
        version: "0.0.0".into(),
        commit: "0123456789ab".into(),
        platform: b"Linux-x86_64".as_slice().into(),
        beside_exists: false,
    }
}
fn next(machine: &mut Find, event: Event) -> Request {
    let mut out = Queue::uncapped();
    step(machine, event, &mut out);
    assert_eq!(out.len(), 1);
    out.pop().unwrap()
}
fn candidate(line: &[u8], success: bool) -> Event {
    Event::Identity(Candidate { line: line.into(), success })
}
fn wanted() -> Event {
    candidate(&identity("0.0.0", "0123456789ab"), true)
}
fn get(path: &[u8]) -> Request {
    Request::Get { origin: build().site, path: path.into() }
}
fn sums_get() -> Request {
    get(b"/temper/v0/0123456789ab/SHA256SUMS")
}
fn file_get() -> Request {
    get(b"/temper/v0/0123456789ab/Linux-x86_64/temper-box")
}
fn clean_request() -> Request {
    Request::Clean { directory: DIR.into() }
}
fn cache_start(machine: &mut Find) {
    assert_eq!(next(machine, Event::Begin), Request::Lock { directory: DIR.into() });
    assert_eq!(next(machine, Event::Done), clean_request());
    assert_eq!(next(machine, Event::Done), Request::Cached { file: FILE.into() });
}
fn download() -> Find {
    let mut machine = Find::new(build());
    cache_start(&mut machine);
    assert_eq!(next(&mut machine, Event::Cached { file: None, sum: None }), sums_get());
    machine
}
fn head(length: Option<u64>) -> Event {
    Event::Answered { status: 200, length, location: None }
}
fn refusal(machine: &mut Find, request: Request, words: &str) {
    assert_eq!(request, clean_request());
    let Request::End(Err(why)) = next(machine, Event::Done) else { panic!("refusal") };
    assert!(core::str::from_utf8(&why.words()).unwrap().contains(words), "{why:?}");
}
fn served(machine: &mut Find, body: &[u8], length: Option<u64>) {
    let mut hash = Sha256::new();
    hash.update(body);
    let sums = joined(&[&hex(&hash.finish()), b"  Linux-x86_64/temper-box\n"]);
    assert_eq!(next(machine, head(None)), Request::Read);
    assert_eq!(next(machine, Event::Chunk(sums)), Request::Read);
    assert_eq!(next(machine, Event::Ended), file_get());
    assert_eq!(next(machine, head(length)), Request::Open { file: joined(&[FILE, b".gz.part"]) });
    assert_eq!(next(machine, Event::Done), Request::Say(Progress { done: 0, total: length.filter(|n| *n != 0) }));
    assert_eq!(next(machine, Event::Done), Request::Read);
}
#[test]
fn pieces() {
    assert_eq!(identity("0.0.0", "0123456789ab").as_ref(), b"temper-box 0.0.0 (0123456789ab)\n");
    for (commit, expected) in [
        ("0123456789ab", true),
        ("0123456789ab-dirty", false),
        ("unknown", false),
        ("0123456789aB", false),
        ("0123456789a", false),
    ] {
        assert_eq!(clean(commit.as_bytes()), expected, "{commit}");
    }
}
#[test]
fn sums_rules() {
    let hex = "0123456789abcdef".repeat(4);
    let name = b"Linux-x86_64/temper-box";
    let expected = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef].repeat(4);
    for separator in ["  ", " *"] {
        let text = format!("{hex}{separator}Linux-x86_64/temper-box\nnot-a-digest  Linux-x86_64/temper-engine\n");
        assert_eq!(sum(text.as_bytes(), name).unwrap().as_slice(), expected);
    }
    let line = format!("{hex}  Linux-x86_64/temper-box\n");
    for (text, expected) in [
        (format!("{hex}  Linux-x86_64/temper-engine\n"), Sums::Missing),
        (line.repeat(2), Sums::Doubled),
        (format!("{}  Linux-x86_64/temper-box\n", &hex[..63]), Sums::Malformed),
        (format!("{}  Linux-x86_64/temper-box\n", hex.to_uppercase()), Sums::Malformed),
        (format!("{hex} Linux-x86_64/temper-box\n"), Sums::Malformed),
        ("x".repeat(65_537), Sums::TooLong),
    ] {
        assert_eq!(sum(text.as_bytes(), name), Err(expected));
    }
    let mut exact = vec![b'x'; 65_536 - line.len()];
    exact.push(b'\n');
    exact.extend_from_slice(line.as_bytes());
    exact.remove(0);
    assert_eq!(exact.len(), 65_536);
    sum(&exact, name).unwrap();
}
#[test]
fn finding_rows() {
    for (beside, cached, expected) in [
        (Some(true), None, "beside"),
        (Some(false), Some((Some([1; 32]), Some([1; 32]), true)), "cache"),
        (None, Some((Some([1; 32]), Some([1; 32]), true)), "cache"),
        (None, Some((Some([1; 32]), Some([2; 32]), true)), "fetch"),
        (None, Some((Some([1; 32]), None, true)), "fetch"),
        (None, Some((None, Some([1; 32]), true)), "fetch"),
        (None, Some((Some([1; 32]), Some([1; 32]), false)), "refuse"),
    ] {
        let mut input = build();
        input.beside_exists = beside.is_some();
        let mut machine = Find::new(input);
        let first = next(&mut machine, Event::Begin);
        let first = if let Some(same) = beside {
            assert_eq!(first, Request::Identity { file: b"/bin/temper-box".as_slice().into() });
            next(&mut machine, if same { wanted() } else { candidate(b"another\n", true) })
        } else {
            first
        };
        if expected == "beside" {
            assert_eq!(first, Request::End(Ok(b"/bin/temper-box".as_slice().into())));
            continue;
        }
        assert_eq!(first, Request::Lock { directory: DIR.into() });
        assert_eq!(next(&mut machine, Event::Done), clean_request());
        assert_eq!(next(&mut machine, Event::Done), Request::Cached { file: FILE.into() });
        let (file, sum, same) = cached.unwrap();
        let answer = next(&mut machine, Event::Cached { file, sum });
        if file.is_some() && file == sum {
            assert_eq!(answer, Request::Identity { file: FILE.into() });
            let answer = next(&mut machine, if same { wanted() } else { candidate(b"wrong build\n", true) });
            if same {
                assert_eq!(answer, Request::End(Ok(FILE.into())));
                continue;
            }
            assert_eq!(answer, Request::RemoveCached { file: FILE.into() });
            let request = next(&mut machine, Event::Done);
            refusal(&mut machine, request, "wrong build\n");
        } else {
            assert_eq!(answer, Request::RemoveCached { file: FILE.into() });
            assert_eq!(next(&mut machine, Event::Done), sums_get());
        }
    }
    for commit in ["0123456789ab-dirty", "unknown"] {
        let mut input = build();
        input.commit = commit.into();
        let mut machine = Find::new(input);
        let Request::End(Err(why)) = next(&mut machine, Event::Begin) else { panic!("checkout refused before lock") };
        assert_eq!(
            core::str::from_utf8(&why.words()).unwrap(),
            format!(
                "no temper-box of this build (0.0.0 ({commit})) beside /bin/temper; a build from a checkout downloads none: build it with cargo build -p temper-box"
            )
        );
    }
    let mut input = build();
    input.platform = b"other-target".as_slice().into();
    let mut machine = Find::new(input);
    let directory = b"/cache/bin/0123456789ab/other-target".as_slice();
    assert_eq!(next(&mut machine, Event::Begin), Request::Lock { directory: directory.into() });
    assert_eq!(next(&mut machine, Event::Done), Request::Clean { directory: directory.into() });
    assert_eq!(next(&mut machine, Event::Done), Request::Cached { file: joined(&[directory, b"/temper-box"]) });
    assert_eq!(
        next(&mut machine, Event::Cached { file: None, sum: None }),
        Request::Clean { directory: directory.into() }
    );
    let Request::End(Err(Failure::Platform(target))) = next(&mut machine, Event::Done) else { panic!("target") };
    assert_eq!(target.as_ref(), b"other-target");
}
#[test]
fn download_rows() {
    for (length, body, words) in [(None, 65_537, "byte bound"), (Some(1), 2, "Content-Length")] {
        let mut machine = download();
        assert_eq!(next(&mut machine, head(length)), Request::Read);
        let answer = next(&mut machine, Event::Chunk(vec![0; body].into_boxed_slice()));
        refusal(&mut machine, answer, words);
    }
    for (binary, length) in [(false, SUMS_BYTES + 1), (true, SERVED_BYTES + 1)] {
        let mut machine = download();
        if binary {
            assert_eq!(next(&mut machine, head(None)), Request::Read);
            assert_eq!(
                next(
                    &mut machine,
                    Event::Chunk(
                        format!("{}  Linux-x86_64/temper-box\n", "0".repeat(64)).into_bytes().into_boxed_slice()
                    )
                ),
                Request::Read
            );
            assert_eq!(next(&mut machine, Event::Ended), file_get());
        }
        let answer = next(&mut machine, head(Some(length)));
        refusal(&mut machine, answer, "byte bound");
    }
    let mut machine = download();
    served(&mut machine, b"", None);
    let answer =
        next(&mut machine, Event::Chunk(vec![0; usize::try_from(SERVED_BYTES + 1).unwrap()].into_boxed_slice()));
    refusal(&mut machine, answer, "byte bound");
    let mut machine = download();
    served(&mut machine, b"", Some(0));
    let answer = next(&mut machine, Event::Chunk(Box::new([0])));
    refusal(&mut machine, answer, "Content-Length");
    for (body, unpacked, words) in [
        (b"".as_slice(), Unpacked::File { digest: [3; 32], bytes: UNPACKED_BYTES + 1 }, "512 MiB"),
        (b"", Unpacked::Invalid, "gzip"),
    ] {
        let mut machine = download();
        served(&mut machine, body, None);
        assert_eq!(
            next(&mut machine, Event::Ended),
            Request::Unpack {
                archive: joined(&[FILE, b".gz.part"]),
                part: joined(&[FILE, b".part"]),
                bound: UNPACKED_BYTES
            }
        );
        let answer = next(&mut machine, Event::Unpacked(unpacked));
        refusal(&mut machine, answer, words);
    }
    let mut machine = download();
    served(&mut machine, b"", None);
    assert_eq!(next(&mut machine, Event::Chunk(Box::new([1]))), Request::Write(Box::new([1])));
    assert_eq!(next(&mut machine, Event::Done), Request::Read);
    assert_eq!(next(&mut machine, Event::Ended), Request::Say(Progress { done: 1, total: None }));
    let answer = next(&mut machine, Event::Done);
    refusal(&mut machine, answer, "digest differs: wanted");
    for correct in [true, false] {
        let mut machine = download();
        served(&mut machine, b"", None);
        assert_eq!(
            next(&mut machine, Event::Ended),
            Request::Unpack {
                archive: joined(&[FILE, b".gz.part"]),
                part: joined(&[FILE, b".part"]),
                bound: UNPACKED_BYTES
            }
        );
        assert_eq!(
            next(&mut machine, Event::Unpacked(Unpacked::File { digest: [3; 32], bytes: UNPACKED_BYTES })),
            Request::Identity { file: joined(&[FILE, b".part"]) }
        );
        let answer = next(&mut machine, if correct { wanted() } else { candidate(b"wrong\n", false) });
        if !correct {
            refusal(&mut machine, answer, "identity differs: wrong");
            continue;
        }
        assert_eq!(answer, Request::WriteSum { file: FILE.into(), digest: [3; 32] });
        assert_eq!(next(&mut machine, Event::Done), Request::RenameSum { file: FILE.into() });
        assert_eq!(next(&mut machine, Event::Done), Request::Publish { file: FILE.into() });
        assert_eq!(next(&mut machine, Event::Done), Request::End(Ok(FILE.into())));
    }
}
#[test]
fn redirect_rows() {
    for (location, follow) in [
        ("https://site.test/other", true),
        ("/other", true),
        ("https://site.test:443/other", true),
        ("https://other.test/other", false),
        ("https://site.test:444/other", false),
        ("http://site.test/other", false),
        ("//other.test/other", false),
    ] {
        let mut machine = download();
        let answer = next(
            &mut machine,
            Event::Answered { status: 302, length: None, location: Some(location.as_bytes().into()) },
        );
        if follow {
            assert_eq!(answer, get(b"/other"));
        } else {
            assert_eq!(answer, clean_request());
            let Request::End(Err(why)) = next(&mut machine, Event::Done) else { panic!("redirect refusal") };
            let words = core::str::from_utf8(&why.words()).unwrap().to_owned();
            assert!(words.contains("https://site.test/temper/v0/0123456789ab/SHA256SUMS"));
            assert!(words.contains(location));
        }
    }
    let mut machine = download();
    for _ in 0_u8..3 {
        assert_eq!(
            next(
                &mut machine,
                Event::Answered { status: 307, length: None, location: Some(b"/other".as_slice().into()) }
            ),
            get(b"/other")
        );
    }
    let answer =
        next(&mut machine, Event::Answered { status: 307, length: None, location: Some(b"/other".as_slice().into()) });
    refusal(&mut machine, answer, "redirect refused");
    for status in [201, 404, 500] {
        let mut machine = download();
        let answer = next(&mut machine, Event::Answered { status, length: None, location: None });
        refusal(&mut machine, answer, &format!("HTTP status {status}"));
    }
}
#[test]
fn progress_rows() {
    for (length, chunks, expected) in [
        (Some(1000), vec![1, 8, 1, 15, 975], vec![0, 10, 25, 1000]),
        (None, vec![1, 1_048_574, 1, 2], vec![0, 1_048_576, 1_048_578]),
    ] {
        let body = vec![0; chunks.iter().sum()];
        let mut machine = download();
        served(&mut machine, &body, length);
        let mut said = vec![0];
        for chunk in chunks {
            let bytes = vec![0; chunk].into_boxed_slice();
            assert_eq!(next(&mut machine, Event::Chunk(bytes.clone())), Request::Write(bytes));
            let answer = next(&mut machine, Event::Done);
            match answer {
                Request::Say(progress) => {
                    said.push(progress.done);
                    let text = core::str::from_utf8(&progress.line()).unwrap().to_owned();
                    assert_eq!(
                        text,
                        match length {
                            Some(total) => format!("fetched temper-box {} of {total} bytes\n", progress.done),
                            None => format!("fetched temper-box {} bytes\n", progress.done),
                        }
                    );
                    assert_eq!(next(&mut machine, Event::Done), Request::Read);
                }
                Request::Read => {}
                other @ (Request::Identity { .. }
                | Request::Lock { .. }
                | Request::Clean { .. }
                | Request::Cached { .. }
                | Request::RemoveCached { .. }
                | Request::Get { .. }
                | Request::Open { .. }
                | Request::Write(_)
                | Request::Unpack { .. }
                | Request::WriteSum { .. }
                | Request::RenameSum { .. }
                | Request::Publish { .. }
                | Request::End(_)) => panic!("write answer {other:?}"),
            }
        }
        let mut answer = next(&mut machine, Event::Ended);
        if let Request::Say(progress) = answer {
            said.push(progress.done);
            answer = next(&mut machine, Event::Done);
        }
        assert_eq!(said, expected);
        assert_eq!(
            answer,
            Request::Unpack {
                archive: joined(&[FILE, b".gz.part"]),
                part: joined(&[FILE, b".part"]),
                bound: UNPACKED_BYTES
            }
        );
    }
}
