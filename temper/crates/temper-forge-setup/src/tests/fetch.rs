#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust for table fixtures and assertions (programming-model.md, 10.2)"
)]
use super::fixtures;
use crate::{
    check, limits, problem,
    setup::{Fetch, Mirror, Mode, Origin, Owned},
};

fn origin(host: &[u8], port: u16) -> Origin {
    Origin { host: host.into(), port }
}
fn read_owned(owned: &str) -> Result<Owned, Box<[u8]>> {
    let source = format!(r#"{{"owned":{owned}}}"#);
    let mut ps = problem::Problems::empty();
    let dirs = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let machine =
        crate::tests::machine_config::read(&fixtures::document(source.as_bytes()), 0, &limits::stock(), &dirs, &mut ps);
    if !ps.is_empty() {
        return Err(problem::lines(&ps));
    }
    match crate::tests::machine_config::entry(&machine.unwrap(), 2, crate::machine::BoxKind::Owned, &dirs).unwrap().mode
    {
        Mode::Owned(owned) => Ok(owned),
        Mode::Managed { .. } | Mode::Fake(_) => panic!("owned"),
    }
}
fn owned_json(forgejo: &str, runner: Option<&str>, mirror: Option<&str>, public: Option<&str>) -> String {
    let runner = runner.map_or("null".into(), |binary| format!(r#"{{"binary":"{binary}","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"}}"#));
    let mirror = mirror.map_or(String::new(), |mirror| format!(",\"mirror\":{mirror}"));
    let public = public.map_or(String::new(), |public| format!(",\"public\":{public}"));
    format!(
        r#"{{"remote":"http://127.0.0.1:3000","forgejo":"{forgejo}","runner":{runner},"listen":{{"socket":"127.0.0.1:3000"}},"person":{{"login":"ada","email":"ada@example.invalid"}}{mirror}{public}}}"#
    )
}
#[test]
fn owned_binaries_and_mirror() {
    let base = fixtures::owned();
    let mirror = r#"{"forgejo":"https://mirror.test","runner":"https://127.0.0.1:8443"}"#;
    for (forgejo, runner, mirrors) in [
        ("fetched", Some("/opt/forgejo/forgejo-runner"), None),
        ("/opt/forgejo/forgejo", Some("fetched"), None),
        ("fetched", Some("fetched"), None),
        ("/opt/forgejo/forgejo", Some("/opt/forgejo/forgejo-runner"), None),
        ("fetched", None, None),
        ("fetched", Some("fetched"), Some(mirror)),
        ("/opt/forgejo/forgejo", None, Some(mirror)),
    ] {
        let mut expected = base.clone();
        expected.fetch = Fetch {
            forgejo: forgejo == "fetched",
            runner: runner == Some("fetched"),
            mirror: mirrors
                .map(|_| Mirror { forgejo: origin(b"mirror.test", 443), runner: origin(b"127.0.0.1", 8443) }),
        };
        expected.forgejo = if expected.fetch.forgejo {
            b"/cache/forgejo/17.0.1/forgejo".as_slice().into()
        } else {
            forgejo.as_bytes().into()
        };
        expected.runner = runner.map(|binary| {
            let mut value = base.runner.clone().unwrap();
            value.binary = if binary == "fetched" {
                b"/cache/forgejo-runner/13.2.1/forgejo-runner".as_slice().into()
            } else {
                binary.as_bytes().into()
            };
            value
        });
        assert_eq!(read_owned(&owned_json(forgejo, runner, mirrors, None)), Ok(expected));
    }
    for bad in [
        "http://mirror.test",
        "https://mirror.test/",
        "https://mirror.test/releases",
        "https://u@mirror.test",
        "https://mirror.test?a",
        "https://mirror.test#a",
        "https://",
        "https://mirror.test:0",
        "https://mirror.test:65536",
    ] {
        let mirror = format!(r#"{{"forgejo":"{bad}","runner":"https://mirror.test"}}"#);
        assert_eq!(
            read_owned(&owned_json("fetched", None, Some(&mirror), None)),
            Err(b"application.machine.owned.mirror.forgejo: invalid form\n".as_slice().into()),
            "{bad}"
        );
    }
    for (mirror, expected) in [
        (r#"{"forgejo":"https://mirror.test"}"#, "application.machine.owned.mirror.runner: missing\n"),
        (
            r#"{"forgejo":"https://mirror.test","runner":"https://mirror.test","third":1}"#,
            "application.machine.owned.mirror.third: unknown member\n",
        ),
    ] {
        assert_eq!(read_owned(&owned_json("fetched", None, Some(mirror), None)), Err(expected.as_bytes().into()));
    }
}
#[test]
fn owned_public_address() {
    for (public, expected) in [
        (None, None),
        (Some(r#""https://git.example.com""#), Some(origin(b"git.example.com", 443))),
        (Some(r#""https://git.example.com:8443""#), Some(origin(b"git.example.com", 8443))),
    ] {
        let mut owned = fixtures::owned();
        owned.public = expected;
        assert_eq!(
            read_owned(&owned_json("/opt/forgejo/forgejo", Some("/opt/forgejo/forgejo-runner"), None, public)),
            Ok(owned)
        );
    }
}
#[test]
fn owned_public_address_refuses_invalid_forms() {
    for bad in [
        r#""http://git.example.com""#,
        r#""https://git.example.com/""#,
        r#""https://git.example.com/x""#,
        r#""https://git.example.com:0""#,
        r#""git.example.com""#,
        r#""https://git.example.com?q""#,
        r#""https://git.example.com#fragment""#,
        "42",
    ] {
        assert_eq!(
            read_owned(&owned_json("fetched", None, None, Some(bad))),
            Err(b"application.machine.owned.public: invalid form\n".as_slice().into()),
            "{bad}"
        );
    }
}
#[test]
fn fetch_bound_through_checked() {
    for (value, expected) in [
        (None, Ok(268_435_456)),
        (Some(1), Ok(1)),
        (Some(u64::MAX), Ok(u64::MAX)),
        (Some(0), Err(b"application.limits.overrides.setup.fetch_bytes: outside its type\n".as_slice())),
    ] {
        let overrides = value.map_or(Vec::new(), |value| vec![(b"setup.fetch_bytes".as_slice().into(), value)]);
        let got = check::limits(&overrides);
        match expected {
            Ok(expected) => assert_eq!(got.unwrap().1, expected),
            Err(expected) => assert_eq!(problem::lines(&got.unwrap_err()).as_ref(), expected),
        }
    }
}

use crate::fetch::{self, Event, Failure, Fetched, Named, Pin, Request, Unpacked};
use skein_forge::forgejo::pin::{Binary, Platform};
use skein_lib::{Queue, sha256::Sha256};

#[test]
fn a_fetched_line_reads_back() {
    for (binary, word) in [(Binary::Forgejo, "forgejo"), (Binary::Runner, "runner")] {
        for (done, total) in [
            (0, None),
            (1_048_576, None),
            (u64::MAX, None),
            (0, Some(1)),
            (1, Some(1)),
            (1_048_576, Some(122_057_320)),
            (0, Some(u64::MAX)),
            (u64::MAX, Some(u64::MAX)),
        ] {
            let fetched = Fetched { binary: Named::Forge(binary), done, total };
            let expected = match total {
                Some(total) => format!("fetched {word} {done} of {total} bytes\n"),
                None => format!("fetched {word} {done} bytes\n"),
            };
            let line = fetched.line();
            assert_eq!(fetched.word(), word.as_bytes());
            assert_eq!(line.as_ref(), expected.as_bytes());
            assert_eq!(Fetched::read(&line), Some(fetched));
            assert_eq!(Fetched::read(line.strip_suffix(b"\n").unwrap()), Some(fetched));
        }
    }
}

#[test]
fn only_a_fetched_line_reads() {
    for line in [
        "",
        "fetch forgejo 1 bytes",
        "Fetched forgejo 1 bytes",
        "fetched other 1 bytes",
        "fetched Forgejo 1 bytes",
        "fetched Runner 1 bytes",
        "fetched forgejo-runner 1 bytes",
        "fetched forgejo +1 bytes",
        "fetched forgejo -1 bytes",
        "fetched forgejo 1 of +2 bytes",
        "fetched forgejo 1 of -2 bytes",
        "fetched forgejo ١ bytes",
        "fetched forgejo 1 of ２ bytes",
        "fetched forgejo 1",
        "fetched forgejo 1 of 2",
        "fetched forgejo 0 of 0 bytes",
        "fetched forgejo 2 of 1 bytes",
        "fetched forgejo 18446744073709551616 bytes",
        "fetched forgejo 18446744073709551620 bytes",
        "fetched forgejo 1 of 18446744073709551616 bytes",
        "fetched forgejo 1 of 18446744073709551620 bytes",
        "fetched forgejo  bytes",
        "fetched forgejo 1 of  bytes",
        "fetched forgejo  of 2 bytes",
        " fetched forgejo 1 bytes",
        "fetched  forgejo 1 bytes",
        "fetched forgejo  1 bytes",
        "fetched forgejo 1  bytes",
        "fetched forgejo 1  of 2 bytes",
        "fetched forgejo 1 of  2 bytes",
        "fetched forgejo 1 of 2  bytes",
        "fetched forgejo\t1 bytes",
        "fetched forgejo 1\tof 2 bytes",
        "fetched forgejo 1 bytes ",
        "fetched forgejo 1 bytes\r\n",
        "fetched forgejo 1 bytes\n\n",
        "fetched forgejo 1 bytes\nfetched runner 1 bytes\n",
        "fetched forgejo 1 bytes\0",
    ] {
        assert_eq!(Fetched::read(line.as_bytes()), None, "{line:?}");
    }
}

fn sha(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finish()
}
fn unhex(text: &str) -> [u8; 32] {
    let mut value = [0; 32];
    for (value, digits) in value.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
        *value = u8::from_str_radix(core::str::from_utf8(digits).unwrap(), 16).unwrap();
    }
    value
}
#[test]
fn architecture_pins() {
    let release = [
        (
            Platform::Linux,
            "x86_64",
            [
                (
                    "/temper/forgejo/17.0.1/forgejo-17.0.1-linux-amd64.zst",
                    "51a8cdf4a1252e79f4b3855d7a7891af6c6635105420768c460e2f0690385d6f",
                ),
                (
                    "/temper/forgejo-runner/13.2.1/forgejo-runner-13.2.1-linux-amd64.zst",
                    "bb369fe1ecb50fdc983c0b1d993f19b0920df6e5864511892b3cdbf16d858b34",
                ),
            ],
        ),
        (
            Platform::Linux,
            "aarch64",
            [
                (
                    "/temper/forgejo/17.0.1/forgejo-17.0.1-linux-arm64.zst",
                    "1e99403746406e8199efc903ddd1940d5728a60e50241f7aa6c9f2b907bd8ec9",
                ),
                (
                    "/temper/forgejo-runner/13.2.1/forgejo-runner-13.2.1-linux-arm64.zst",
                    "eeddb7902f79abe488972f4189fbe3ea02762ee827d1b406b469420bf0d62dd9",
                ),
            ],
        ),
        (
            Platform::Macos,
            "aarch64",
            [
                (
                    "/temper/forgejo/17.0.1/forgejo-17.0.1-darwin-arm64.zst",
                    "4b6ed0e7e6ef034d298872125f69332e2aad49367435f66c58ee5eb2c5680cf6",
                ),
                (
                    "/temper/forgejo-runner/13.2.1/forgejo-runner-13.2.1-darwin-arm64.zst",
                    "2829f0125068f0ad92f908cb08c4d3c2f458cc22eec15fd675c332823cbece4b",
                ),
            ],
        ),
    ];
    let mut owned = fixtures::owned();
    let mirror = Mirror { forgejo: origin(b"mirror.test", 443), runner: origin(b"127.0.0.1", 8443) };
    for (platform, machine, releases) in release {
        for mirrored in [false, true] {
            owned.fetch = Fetch { forgejo: true, runner: true, mirror: mirrored.then(|| mirror.clone()) };
            let pins = fetch::pins(&owned, platform, machine).unwrap();
            assert_eq!(pins.len(), 2);
            let expected = [
                (Binary::Forgejo, &owned.forgejo, &mirror.forgejo),
                (Binary::Runner, &owned.runner.as_ref().unwrap().binary, &mirror.runner),
            ];
            for ((pin, (path, digest)), (binary, file, origin_if_mirrored)) in pins.iter().zip(releases).zip(expected) {
                assert_eq!(pin.binary, Named::Forge(binary), "{platform:?} {machine}");
                assert_eq!(&pin.file, file);
                let origin = if mirrored { origin_if_mirrored.clone() } else { origin(b"fabrig.ai", 443) };
                assert_eq!(pin.origin, origin);
                assert_eq!(pin.path.as_ref(), path.as_bytes());
                assert_eq!(pin.digest, unhex(digest));
                assert!(pin.packed);
            }
        }
    }
    owned.fetch = Fetch { forgejo: true, runner: true, mirror: None };
    for (platform, machine) in [(Platform::Macos, "x86_64"), (Platform::Macos, "riscv64"), (Platform::Linux, "riscv64")]
    {
        assert_eq!(fetch::pins(&owned, platform, machine), Err(Failure::Architecture(machine.as_bytes().into())));
    }
    for (forgejo, runner, wanted) in
        [(true, false, &[Binary::Forgejo][..]), (false, true, &[Binary::Runner][..]), (false, false, &[][..])]
    {
        owned.fetch = Fetch { forgejo, runner, mirror: None };
        let pins = fetch::pins(&owned, Platform::Linux, "x86_64").unwrap();
        assert!(pins.iter().map(|pin| pin.binary).eq(wanted.iter().copied().map(Named::Forge)));
    }
}
fn pin(file: &[u8]) -> Pin {
    Pin {
        binary: Named::Forge(Binary::Forgejo),
        origin: origin(b"mirror.test", 8443),
        path: b"/binary".as_slice().into(),
        digest: sha(b"abc"),
        file: file.into(),
        packed: true,
    }
}
fn hash(file: &[u8]) -> Request {
    Request::Hash { file: file.into() }
}
fn get() -> Request {
    Request::Get { origin: origin(b"mirror.test", 8443), path: b"/binary".as_slice().into() }
}
fn open(file: &str) -> Request {
    Request::Open { part: format!("{file}.zst.part").as_bytes().into() }
}
fn say(done: u64, total: Option<u64>) -> Request {
    Request::Say(Fetched { binary: Named::Forge(Binary::Forgejo), done, total })
}
fn write(offset: u64, bytes: &[u8]) -> Request {
    Request::Write { offset, bytes: bytes.into() }
}
fn finish(file: &str) -> Request {
    Request::Finish { part: format!("{file}.part").as_bytes().into(), file: file.as_bytes().into() }
}
fn remove(file: &str) -> Request {
    Request::Remove { part: file.as_bytes().into() }
}
fn unpack(file: &str, bound: u64) -> Request {
    Request::Unpack {
        packed: format!("{file}.zst.part").as_bytes().into(),
        part: format!("{file}.part").as_bytes().into(),
        bound,
    }
}
fn unpacked(bytes: &[u8]) -> Event {
    Event::Unpacked(Unpacked::File { digest: sha(bytes), bytes: u64::try_from(bytes.len()).unwrap() })
}
fn url() -> Box<[u8]> {
    b"https://mirror.test:8443/binary".as_slice().into()
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table of complete request and event traces")]
fn fetch_machine() {
    let first = |bound| {
        [
            (Event::Begin, hash(b"/cache/bin")),
            (Event::Hashed(None), get()),
            (answered(200, Some(3), None), open("/cache/bin")),
            (Event::Done, say(0, Some(3))),
            (Event::Done, Request::Read),
            (Event::Chunk(b"x".as_slice().into()), write(0, b"x")),
            (Event::Done, say(1, Some(3))),
            (Event::Done, Request::Read),
            (Event::Chunk(b"yz".as_slice().into()), write(1, b"yz")),
            (Event::Done, say(3, Some(3))),
            (Event::Done, Request::Read),
            (Event::Ended, unpack("/cache/bin", bound)),
            (unpacked(b"abc"), finish("/cache/bin")),
            (Event::Done, remove("/cache/bin.zst.part")),
        ]
    };
    let mut two = Vec::from(first(3));
    two.extend([
        (Event::Done, hash(b"/cache/two")),
        (Event::Hashed(None), get()),
        (answered(200, None, None), open("/cache/two")),
        (Event::Done, say(0, None)),
        (Event::Done, Request::Read),
        (Event::Chunk(b"xyz".as_slice().into()), write(0, b"xyz")),
        (Event::Done, Request::Read),
        (Event::Ended, say(3, None)),
        (Event::Done, unpack("/cache/two", 3)),
        (unpacked(b"abc"), finish("/cache/two")),
        (Event::Done, remove("/cache/two.zst.part")),
        (Event::Done, Request::End(Ok(()))),
    ]);
    let mut success = Vec::from(first(10));
    success.push((Event::Done, Request::End(Ok(()))));
    let mut at_bound = Vec::from(first(3));
    at_bound.push((Event::Done, Request::End(Ok(()))));
    for (name, count, bound, steps) in [
        ("absent two chunks", 1, 10, success),
        (
            "matching",
            1,
            3,
            vec![(Event::Begin, hash(b"/cache/bin")), (Event::Hashed(Some(sha(b"abc"))), Request::End(Ok(())))],
        ),
        (
            "present mismatch stops before second",
            2,
            3,
            vec![
                (Event::Begin, hash(b"/cache/bin")),
                (
                    Event::Hashed(Some(sha(b"x"))),
                    Request::End(Err(Failure::Present { file: b"/cache/bin".as_slice().into() })),
                ),
            ],
        ),
        (
            "302",
            1,
            3,
            vec![
                (Event::Begin, hash(b"/cache/bin")),
                (Event::Hashed(None), get()),
                (
                    Event::Answered { status: 302, length: None, location: None },
                    Request::End(Err(Failure::Status { url: url(), status: 302 })),
                ),
            ],
        ),
        (
            "404",
            1,
            3,
            vec![
                (Event::Begin, hash(b"/cache/bin")),
                (Event::Hashed(None), get()),
                (
                    Event::Answered { status: 404, length: None, location: None },
                    Request::End(Err(Failure::Status { url: url(), status: 404 })),
                ),
            ],
        ),
        (
            "length past bound",
            1,
            2,
            vec![
                (Event::Begin, hash(b"/cache/bin")),
                (Event::Hashed(None), get()),
                (
                    Event::Answered { status: 200, length: Some(3), location: None },
                    Request::End(Err(Failure::Bytes { url: url() })),
                ),
            ],
        ),
        ("length at bound", 1, 3, at_bound),
        (
            "stream past bound",
            1,
            2,
            vec![
                (Event::Begin, hash(b"/cache/bin")),
                (Event::Hashed(None), get()),
                (Event::Answered { status: 200, length: None, location: None }, open("/cache/bin")),
                (Event::Done, say(0, None)),
                (Event::Done, Request::Read),
                (Event::Chunk(b"ab".as_slice().into()), write(0, b"ab")),
                (Event::Done, Request::Read),
                (Event::Chunk(b"c".as_slice().into()), remove("/cache/bin.zst.part")),
                (Event::Done, Request::End(Err(Failure::Bytes { url: url() }))),
            ],
        ),
        ("both fetched", 2, 3, two),
        ("none", 0, 3, vec![(Event::Begin, Request::End(Ok(())))]),
    ] {
        let pins = match count {
            0_u8 => vec![],
            1 => vec![pin(b"/cache/bin")],
            2 => vec![pin(b"/cache/bin"), pin(b"/cache/two")],
            _ => unreachable!(),
        };
        let mut machine = fetch::Fetch::new(pins.into_boxed_slice(), bound);
        let mut out = Queue::uncapped();
        for (event, expected) in steps {
            fetch::step(&mut machine, event, &mut out);
            assert_eq!(out.pop(), Some(expected), "{name}");
            assert!(out.is_empty());
        }
    }
}
#[test]
fn failures_words_and_urls() {
    for (failure, expected) in [
        (Failure::Architecture(b"riscv64".as_slice().into()), "no pinned binaries for architecture riscv64"),
        (Failure::Present { file: b"/cache/bin".as_slice().into() }, "/cache/bin: does not match the pinned digest"),
        (Failure::Status { url: url(), status: 302 }, "https://mirror.test:8443/binary: HTTP status 302"),
        (Failure::Truncated { url: url() }, "https://mirror.test:8443/binary: truncated zstd frame"),
        (Failure::Damaged { url: url() }, "https://mirror.test:8443/binary: damaged zstd frame"),
        (
            Failure::UnpackedBytes { url: url() },
            "https://mirror.test:8443/binary: unpacked file past setup.fetch_bytes",
        ),
        (Failure::Trailing { url: url() }, "https://mirror.test:8443/binary: zstd frame followed by other bytes"),
        (Failure::Bytes { url: url() }, "https://mirror.test:8443/binary: body past setup.fetch_bytes"),
        (
            Failure::Digest { url: url(), pinned: sha(b"abc"), got: sha(b"x") },
            "https://mirror.test:8443/binary: digest differs: pinned ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad, got 2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881",
        ),
    ] {
        assert_eq!(failure.words().as_ref(), expected.as_bytes());
    }
    assert_eq!(fetch::url(&origin(b"mirror.test", 443), b"/binary").as_ref(), b"https://mirror.test/binary");
}

fn answered(status: u16, length: Option<u64>, location: Option<&[u8]>) -> Event {
    Event::Answered { status, length, location: location.map(Into::into) }
}

#[test]
fn a_redirect_with_a_location_is_refused() {
    let mut drive = Progress { machine: fetch::Fetch::new(Box::new([pin(b"/cache/bin")]), 3) };
    assert_eq!(drive.send(Event::Begin), hash(b"/cache/bin"));
    assert_eq!(drive.send(Event::Hashed(None)), get());
    assert_eq!(
        drive.send(answered(307, None, Some(b"https://storage.test/object"))),
        Request::End(Err(Failure::Status { url: url(), status: 307 })),
    );
}

#[test]
fn unpacking_refusals_remove_both_staging_files() {
    for (result, failure) in [
        (Unpacked::Truncated, Failure::Truncated { url: url() }),
        (Unpacked::Damaged, Failure::Damaged { url: url() }),
        (Unpacked::TooLong, Failure::UnpackedBytes { url: url() }),
        (Unpacked::File { digest: sha(b"abc"), bytes: 4 }, Failure::UnpackedBytes { url: url() }),
        (Unpacked::Trailing, Failure::Trailing { url: url() }),
        (
            Unpacked::File { digest: sha(b"x"), bytes: 1 },
            Failure::Digest { url: url(), pinned: sha(b"abc"), got: sha(b"x") },
        ),
    ] {
        let mut drive = Progress::start(pin(b"/cache/bin"), Some(3), 3);
        drive.chunk(0, b"xyz", Some(3), Some(3));
        assert_eq!(drive.send(Event::Ended), unpack("/cache/bin", 3));
        assert_eq!(drive.send(Event::Unpacked(result)), remove("/cache/bin.part"));
        assert_eq!(drive.send(Event::Done), remove("/cache/bin.zst.part"));
        assert_eq!(drive.send(Event::Done), Request::End(Err(failure)));
    }
}

struct Progress {
    machine: fetch::Fetch,
}
impl Progress {
    fn send(&mut self, event: Event) -> Request {
        let mut out = Queue::uncapped();
        fetch::step(&mut self.machine, event, &mut out);
        assert_eq!(out.len(), 1, "one request per event");
        out.pop().unwrap()
    }
    fn start(pin: Pin, length: Option<u64>, bound: u64) -> Self {
        let mut drive = Self { machine: fetch::Fetch::new(Box::new([pin]), bound) };
        assert_eq!(drive.send(Event::Begin), hash(b"/cache/bin"));
        assert_eq!(drive.send(Event::Hashed(None)), get());
        assert_eq!(drive.send(answered(200, length, None)), open("/cache/bin"));
        assert_eq!(drive.send(Event::Done), say(0, length.filter(|length| *length != 0)));
        assert_eq!(drive.send(Event::Done), Request::Read);
        drive
    }
    fn chunk(&mut self, offset: u64, bytes: &[u8], expected: Option<u64>, total: Option<u64>) {
        assert_eq!(self.send(Event::Chunk(bytes.into())), write(offset, bytes));
        if let Some(done) = expected {
            assert_eq!(self.send(Event::Done), say(done, total));
        }
        assert_eq!(self.send(Event::Done), Request::Read);
    }
    fn publish(&mut self) {
        assert_eq!(self.send(unpacked(b"abc")), finish("/cache/bin"));
        assert_eq!(self.send(Event::Done), remove("/cache/bin.zst.part"));
        assert_eq!(self.send(Event::Done), Request::End(Ok(())));
    }
}

#[test]
fn an_answer_with_a_size_says_each_percent_in_order() {
    let body = [b'x'; 200];
    let mut drive = Progress::start(pin(b"/cache/bin"), Some(200), 200);
    drive.chunk(0, &body[..1], None, Some(200));
    drive.chunk(1, &body[1..2], Some(2), Some(200));
    drive.chunk(2, &body[2..3], None, Some(200));
    drive.chunk(3, &body[3..10], Some(10), Some(200));
    drive.chunk(10, b"", None, Some(200));
    drive.chunk(10, &body[10..], Some(200), Some(200));
    assert_eq!(drive.send(Event::Ended), unpack("/cache/bin", 200));
    drive.publish();
    // An end below the next whole percent still reports its final bytes.
    let mut drive = Progress::start(pin(b"/cache/bin"), Some(u64::MAX), u64::MAX);
    drive.chunk(0, b"abc", None, Some(u64::MAX));
    assert_eq!(drive.send(Event::Ended), say(3, Some(u64::MAX)));
    assert_eq!(drive.send(Event::Done), unpack("/cache/bin", u64::MAX));
    drive.publish();
}

#[test]
fn an_answer_with_no_size_says_each_mib_and_its_end() {
    const MIB: usize = 1_048_576;
    let body = vec![b'x'; 2 * MIB + 1];
    let mut drive = Progress::start(pin(b"/cache/bin"), None, u64::MAX);
    drive.chunk(0, &body[..MIB - 1], None, None);
    drive.chunk(u64::try_from(MIB - 1).unwrap(), &body[MIB - 1..MIB], Some(u64::try_from(MIB).unwrap()), None);
    drive.chunk(u64::try_from(MIB).unwrap(), &body[MIB..2 * MIB], Some(u64::try_from(2 * MIB).unwrap()), None);
    drive.chunk(u64::try_from(2 * MIB).unwrap(), &body[2 * MIB..], None, None);
    assert_eq!(drive.send(Event::Ended), say(u64::try_from(2 * MIB + 1).unwrap(), None));
    assert_eq!(drive.send(Event::Done), unpack("/cache/bin", u64::MAX));
    drive.publish();
}

#[test]
fn a_body_past_its_size_is_refused() {
    for (initial, extra) in [(b"".as_slice(), b"abcd".as_slice()), (b"abc".as_slice(), b"d".as_slice())] {
        let mut drive = Progress::start(pin(b"/cache/bin"), Some(3), 10);
        if !initial.is_empty() {
            drive.chunk(0, initial, Some(3), Some(3));
        }
        assert_eq!(drive.send(Event::Chunk(extra.into())), remove("/cache/bin.zst.part"));
        let failure = Failure::Size { url: url() };
        assert_eq!(failure.words().as_ref(), b"https://mirror.test:8443/binary: body past its Content-Length");
        assert_eq!(drive.send(Event::Done), Request::End(Err(failure)));
    }
}

#[test]
fn a_zero_length_is_no_size() {
    for body in [b"abc".as_slice(), b"".as_slice()] {
        let mut drive = Progress::start(pin(b"/cache/bin"), Some(0), 3);
        drive.chunk(0, body, None, None);
        if body.is_empty() {
            assert_eq!(drive.send(Event::Ended), unpack("/cache/bin", 3));
        } else {
            assert_eq!(drive.send(Event::Ended), say(3, None));
            assert_eq!(drive.send(Event::Done), unpack("/cache/bin", 3));
        }
        drive.publish();
    }
}

#[test]
fn a_binary_already_there_says_nothing() {
    let mut drive = Progress { machine: fetch::Fetch::new(Box::new([pin(b"/cache/bin")]), 3) };
    assert_eq!(drive.send(Event::Begin), hash(b"/cache/bin"));
    assert_eq!(drive.send(Event::Hashed(Some(sha(b"abc")))), Request::End(Ok(())));
}
