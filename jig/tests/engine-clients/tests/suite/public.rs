//! Public component stories (jig/docs/design/protocol/client/public-reads.md).
use jig_client_protocol_wire::{self as wire, public_v1 as w, v27};
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{PublicKind, PublicLimits, PublicRead, PublicReply, Settings, public::Names};
use skein_http::{Header, Method};
use skein_lib::{CryptoRng, List, Reader, ReplyTo, Secret, Time, Writer};
// Each reservation is 8 KiB, enough for this world's two-project index.
const BOUND: u32 = 8192;
const INDEX: wire::PublicIndexBounds = wire::PublicIndexBounds { goals: 2, ended: 2, lines: 2, depth: 2, projects: 2 };
const PUBLIC_PROJECT: u32 = 7;
const PRIVATE_PROJECT: u32 = 8;
const PRIVATE_TASK: u64 = 19;
fn settings() -> Settings {
    let mut settings = fixtures::settings();
    settings.public_bounds = INDEX;
    settings.public = List::with_capacity(3);
    for kind in jig_protocol_engine_clients::jig_kinds(&INDEX) {
        settings.public.push(PublicKind { bound: BOUND, ..kind }).expect("three public kinds");
    }
    settings
}
fn limits() -> PublicLimits {
    PublicLimits {
        reads: 2,
        rate: 8,
        rate_burst: 8,
        bytes: u64::from(BOUND) * 2,
        bytes_burst: u64::from(BOUND) * 2,
        answer_bytes: BOUND,
        max_age: 5,
    }
}
fn peer(public: PublicLimits) -> Peer {
    let mut limits = fixtures::limits();
    limits.public = public;
    let mut bounds = fixtures::http_limits();
    bounds.public_calls = public.reads;
    Peer::with_bounds(&settings(), limits, bounds)
}
fn names() -> Names {
    let settings = settings();
    let mut rng = CryptoRng::new(&settings.seed, settings.stream).expect("32-byte fixture seed");
    let mut link = [0; 32];
    rng.fill(&mut link);
    rng.fill(&mut link);
    let mut key = [0; 32];
    rng.fill(&mut key);
    Names::new(&Secret::new(Box::new(key)))
}
fn read(kind: u16, ids: &[u64]) -> PublicRead {
    PublicRead { owner: 0, version: 1, kind, ids: Box::from(ids) }
}
fn admitted(peer: &mut Peer) -> (ReplyTo, core::PublicRead) {
    let Some(core::Event::PublicRead { reply, read }) = peer.take_core() else {
        panic!("public read to engine");
    };
    (reply, read)
}
fn signed_in_read() -> Box<[u8]> {
    let record = v27::Read::Me;
    let mut out = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut out).expect("measured record");
    let record = v27::Owned::new(
        &fixtures::limits().wire,
        v27::OwnedParts { owner: v27::Owner::Jig, version: wire::VERSION, record: out.finish() },
    )
    .expect("read envelope");
    let mut out = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut out).expect("measured record");
    out.finish()
}
fn projects() -> core::PublicAnswer {
    core::PublicAnswer::Document {
        decided: Time::ZERO,
        document: core::PublicDocument::Projects(core::PublicProjects {
            projects: Box::new([core::PublicProject { project: PUBLIC_PROJECT, name: Box::from("Public project") }]),
        }),
    }
}
#[test]
fn public_reads_past_the_room_rate_and_bytes_are_busy_while_signed_in_reads_are_answered() {
    let base = limits();
    for (label, limits) in [
        ("room", PublicLimits { reads: 1, ..base }),
        ("rate", PublicLimits { rate_burst: 1, ..base }),
        ("bytes", PublicLimits { bytes_burst: u64::from(BOUND), ..base }),
    ] {
        let mut peer = peer(limits);
        let bearer = fixtures::signed_in(&mut peer);
        peer.public_read(read(1, &[]));
        let (held, _) = admitted(&mut peer);
        let refused = peer.public_read(read(1, &[]));
        assert!(matches!(peer.public_reply(refused), Some(PublicReply::Busy)), "{label}");
        assert!(peer.take_core().is_none(), "{label}");
        let socket = peer.post(b"/jig/v1/read", signed_in_read(), Some(&bearer));
        let Some(core::Event::Read { reply, sign_in, .. }) = peer.take_core() else {
            panic!("signed-in read alongside {label}");
        };
        assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
        peer.answer_read(reply, core::reads::Answer::None(core::reads::Why::Gone));
        assert!(peer.answered(socket).is_some(), "{label}");
        peer.public_answer(held, core::PublicAnswer::Unknown);
    }
}
#[test]
fn two_reads_completing_at_once_and_one_whose_exchange_is_gone_leave_the_bucket_whole() {
    const AT_ONCE: u32 = 3;
    let mut limits = limits();
    limits.reads = AT_ONCE;
    limits.bytes_burst = u64::from(BOUND) * u64::from(AT_ONCE);
    let mut peer = peer(limits);
    let first = peer.public_read(read(1, &[]));
    let (first_reply, _) = admitted(&mut peer);
    let second = peer.public_read(read(1, &[]));
    let (second_reply, _) = admitted(&mut peer);
    let gone = peer.public_read(read(1, &[]));
    let (gone_reply, _) = admitted(&mut peer);
    peer.public_gone(gone);
    // Every completion has the same monotonic decision time.
    peer.public_answer(first_reply, core::PublicAnswer::Unknown);
    peer.public_answer(second_reply, core::PublicAnswer::Busy);
    peer.public_answer(gone_reply, projects());
    assert!(matches!(peer.public_reply(first), Some(PublicReply::Unknown)));
    assert!(matches!(peer.public_reply(second), Some(PublicReply::Busy)));
    assert!(peer.public_reply(gone).is_none());
    let mut replies = Vec::new();
    for _ in 0..AT_ONCE {
        let exchange = peer.public_read(read(1, &[]));
        assert!(peer.public_reply(exchange).is_none(), "the whole burst is available again");
        replies.push(admitted(&mut peer).0);
    }
    let refused = peer.public_read(read(1, &[]));
    assert!(matches!(peer.public_reply(refused), Some(PublicReply::Busy)));
    for reply in replies {
        peer.public_answer(reply, core::PublicAnswer::Unknown);
    }
}
#[test]
fn an_unknown_kind_a_private_project_an_identifier_naming_nothing_and_a_task_not_public_are_unknown_with_the_same_messages_to_the_engine()
 {
    let mut peer = peer(limits());
    let unknown = peer.public_read(read(99, &[]));
    assert!(matches!(peer.public_reply(unknown), Some(PublicReply::Unknown)));
    assert!(peer.take_core().is_none(), "an undeclared kind reaches no engine");
    let names = names();
    for (project, task) in [(PRIVATE_PROJECT, PRIVATE_TASK), (u32::MAX, u64::MAX), (PUBLIC_PROJECT, PRIVATE_TASK)] {
        let exchange = peer.public_read(read(3, &[names.project(project), names.task(task)]));
        let (reply, message) = admitted(&mut peer);
        assert_eq!(
            message,
            core::PublicRead { owner: 0, version: 1, kind: 3, project: Some(u64::from(project)), task: Some(task) }
        );
        peer.public_answer(reply, core::PublicAnswer::Unknown);
        assert!(matches!(peer.public_reply(exchange), Some(PublicReply::Unknown)));
    }
}
fn project_id(peer: &mut Peer) -> u64 {
    let exchange = peer.public_read(read(1, &[]));
    let (reply, _) = admitted(peer);
    peer.public_answer(reply, projects());
    let Some(PublicReply::Document { bytes, .. }) = peer.public_reply(exchange) else {
        panic!("public projects");
    };
    let projects =
        w::PublicProjects::decode(&wire::public_limits(&INDEX).expect("tiny public index"), &mut Reader::new(&bytes))
            .expect("public projects record");
    projects.projects().get(0).expect("one public project").project()
}
#[test]
fn identifiers_survive_a_restart() {
    let mut peer = peer(limits());
    let before = project_id(&mut peer);
    let mut restarted = settings();
    restarted.stream = 123;
    peer.restart(&restarted);
    assert_eq!(project_id(&mut peer), before);
    let exchange = peer.public_read(read(2, &[before]));
    let (reply, read) = admitted(&mut peer);
    assert_eq!(read.project, Some(u64::from(PUBLIC_PROJECT)));
    peer.public_answer(reply, core::PublicAnswer::Unknown);
    assert!(matches!(peer.public_reply(exchange), Some(PublicReply::Unknown)));
}

fn headers(cookie: Option<&[u8]>, bearer: Option<&[u8]>) -> Box<[Header]> {
    let mut fields: Vec<(&[u8], &[u8])> = vec![
        (b"host", fixtures::HOST.as_bytes()),
        (b"content-type", b"application/octet-stream"),
        (b"jig-client", b"browser"),
    ];
    if let Some(cookie) = cookie {
        fields.push((b"cookie", cookie));
    }
    if let Some(bearer) = bearer {
        fields.push((b"authorization", bearer));
    }
    fields.into_iter().map(|(name, value)| Header { name: Box::from(name), value: Box::from(value) }).collect()
}

#[test]
fn public_reads_with_no_credential_cookie_bearer_and_both_are_answered_alike() {
    let mut peer = peer(limits());
    let bearer = fixtures::signed_in(&mut peer);
    let token = bearer.strip_prefix(b"Bearer ").expect("fixture bearer");
    let cookie = [b"jig-sign-in=".as_slice(), token].concat();
    let me = peer.http(Method::Post, b"/jig/v1/read", signed_in_read(), headers(Some(&cookie), None));
    let Some(core::Event::Read { reply, sign_in, .. }) = peer.take_core() else {
        panic!("the cookie belongs to a signed-in person");
    };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    peer.answer_read(reply, core::reads::Answer::Document(Box::new(fixtures::me_document(7))));
    assert!(matches!(
        v27::ReadAnswer::decode(&v27::CEILINGS, &mut Reader::new(peer.answered(me).expect("me answer")))
            .expect("me record"),
        v27::ReadAnswer::Value(_)
    ));
    let mut baseline = None;
    for (cookie, bearer) in [
        (None, None),
        (Some(cookie.as_slice()), None),
        (None, Some(bearer.as_ref())),
        (Some(cookie.as_slice()), Some(bearer.as_ref())),
    ] {
        let socket = peer.http(Method::Get, b"/jig/v1/public/0/1/1", Box::new([]), headers(cookie, bearer));
        let (reply, got) = admitted(&mut peer);
        assert_eq!(got, core::PublicRead { owner: 0, version: 1, kind: 1, project: None, task: None });
        peer.public_answer(reply, projects());
        assert!(peer.take_core().is_none(), "only the public read reaches the core");
        assert_eq!(peer.response(socket).0, Some(200));
        let bytes = peer.public_http_bytes(socket);
        if let Some(baseline) = &baseline {
            assert_eq!(bytes, baseline, "credentials change no byte");
        } else {
            baseline = Some(bytes.to_vec());
        }
    }
}

fn owned() -> v27::Owned {
    v27::Owned::new(
        &v27::CEILINGS,
        v27::OwnedParts { owner: v27::Owner::Jig, version: wire::VERSION, record: Box::default() },
    )
    .expect("owned read")
}
fn anonymous_body(route: &str) -> Box<[u8]> {
    match route.as_bytes() {
        b"request" => {
            let record = v27::Keyed::new(
                &v27::CEILINGS,
                v27::KeyedParts {
                    key: [7; 16],
                    owner: v27::Owner::Jig,
                    version: wire::VERSION,
                    request: Box::default(),
                    value: None,
                },
            )
            .expect("keyed request");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode keyed");
            writer.finish()
        }
        b"read" => {
            let record = owned();
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode read");
            writer.finish()
        }
        b"watch" => {
            let record = v27::Open::new(&v27::CEILINGS, v27::OpenParts { stream: [7; 16], watch: 1, subject: owned() })
                .expect("watch");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode watch");
            writer.finish()
        }
        b"unwatch" => {
            let record = v27::WatchClose::new(&v27::CEILINGS, v27::WatchCloseParts { stream: [7; 16], watch: 1 })
                .expect("watch close");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode close");
            writer.finish()
        }
        b"stream-gone" => {
            let record = v27::Gone::new(&v27::CEILINGS, v27::GoneParts { stream: [7; 16] }).expect("gone");
            let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
            record.encode(&mut writer).expect("encode gone");
            writer.finish()
        }
        b"sign-out" | b"stream" => Box::default(),
        _ => unreachable!("route table"),
    }
}

#[test]
fn every_other_exchange_without_a_credential_or_with_an_unknown_cookie_is_signed_out() {
    for cookie in [None, Some(b"jig-sign-in=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".as_slice())] {
        let mut peer = peer(limits());
        for route in ["request", "read", "watch", "unwatch", "stream-gone", "stream", "sign-out"] {
            let path = format!("/jig/v1/{route}");
            let socket = peer.http(
                if route == "stream" { Method::Get } else { Method::Post },
                path.as_bytes(),
                anonymous_body(route),
                headers(cookie, None),
            );
            let wire = peer.public_http_bytes(socket);
            let head_end = wire.windows(4).position(|bytes| bytes == b"\r\n\r\n").expect("HTTP head");
            let head = std::str::from_utf8(&wire[..head_end]).expect("ASCII head");
            assert!(
                !head.lines().any(|line| line.to_ascii_lowercase().starts_with("access-control-")),
                "{route}: no CORS"
            );
            let (status, bytes) = peer.response(socket);
            assert_eq!(status, Some(if route == "stream" { 401 } else { 200 }), "{route}, cookie {cookie:?}");
            let mut input = Reader::new(bytes);
            match route {
                "request" => assert_eq!(
                    v27::KeyedAnswer::decode(&v27::CEILINGS, &mut input).expect("keyed answer"),
                    v27::KeyedAnswer::SignedOut
                ),
                "read" => assert!(
                    matches!(v27::ReadAnswer::decode(&v27::CEILINGS, &mut input).expect("read answer"), v27::ReadAnswer::None(row) if row.value() == &v27::ReadNone::SignedOut)
                ),
                "watch" => assert!(
                    matches!(v27::OpenAnswer::decode(&v27::CEILINGS, &mut input).expect("watch answer"), v27::OpenAnswer::Refused(row) if row.why() == &v27::WatchRefusal::SignedOut)
                ),
                "unwatch" => assert_eq!(
                    v27::CloseAnswer::decode(&v27::CEILINGS, &mut input).expect("close answer"),
                    v27::CloseAnswer::SignedOut
                ),
                "sign-out" => assert_eq!(
                    v27::SignOutAnswer::decode(&v27::CEILINGS, &mut input).expect("sign-out answer"),
                    v27::SignOutAnswer::SignedOut
                ),
                "stream-gone" => assert!(bytes.is_empty(), "stream end acknowledged without a record"),
                "stream" => {}
                other => panic!("route {other}"),
            }
            assert!(peer.take_core().is_none(), "{route}: no engine work for a signed-out visitor");
        }
    }
}
