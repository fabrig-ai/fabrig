//! Public component stories (jig/docs/design/protocol/client/public-reads.md).
use jig_client_protocol_wire::{self as wire, public_v1 as w, v20};
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{PublicKind, PublicLimits, PublicRead, PublicReply, Settings, public::Names};
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
    Peer::with_limits(&settings(), limits)
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
    let record = v20::Read::Me;
    let mut out = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut out).expect("measured record");
    let record = v20::Owned::new(
        &fixtures::limits().wire,
        v20::OwnedParts { owner: v20::Owner::Jig, version: 20, record: out.finish() },
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
