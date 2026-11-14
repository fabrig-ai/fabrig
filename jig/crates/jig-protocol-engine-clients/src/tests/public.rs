//! Public identifiers, admission and records (jig/docs/design/protocol/client/public-reads.md).
use super::fixtures as f;
use crate::{
    Component, Event, FromTransport, Kept, Limits, PublicRead, PublicReply, Request, SecretOwner, SecretsAnswer,
    SecretsRequest, Settings, ToTransport, Unusable,
    public::{Names, Of},
};
use jig_client_protocol_wire::{self as wire, public_v1 as w};
use jig_domain_core_vocabulary as c;
use skein_lib::{Env, List, Queue, Reader, ReplyTo, Rng, Secret, Time, Token, Wall, length::len_u64};
#[path = "public/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn identifiers_map_both_ways_apart_for_projects_and_tasks() {
    const SEED: u64 = 0xdecaf;
    let mut rng = Rng::new(SEED);
    let names = names();
    for _ in 0_u16..256 {
        let n = rng.next_u64();
        let project = u32::try_from(n & u64::from(u32::MAX)).unwrap();
        assert_eq!(names.number(names.project(project), Of::Project), u64::from(project), "seed {SEED}");
        assert_eq!(names.number(names.task(n), Of::Task), n, "seed {SEED}");
        assert_ne!(names.project(project), names.task(u64::from(project)), "seed {SEED}");
        assert_ne!(names.number(names.project(project), Of::Task), u64::from(project), "seed {SEED}");
        assert_ne!(names.number(names.task(u64::from(project)), Of::Project), u64::from(project), "seed {SEED}");
    }
}
#[test]
fn identifiers_are_the_same_after_a_restart() {
    let mut first = Step::configured(limits(), settings(), false);
    first.down(Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::PublicNames, value: None }));
    let Some(Request::Secrets(SecretsRequest::Keep { owner, value })) = first.out.pop() else {
        panic!("keep key");
    };
    first.down(Event::Secrets(SecretsAnswer::Kept { owner }));
    let before = first.component.public.names().unwrap().project(PROJECT);
    let task = first.component.public.names().unwrap().task(TASK);
    let mut settings = settings();
    settings.stream = 99;
    let mut restarted = Step::configured(limits(), settings, false);
    restarted.down(Event::Secrets(SecretsAnswer::Got { owner, value: Some(value) }));
    assert_eq!(restarted.component.public.names().unwrap().project(PROJECT), before);
    assert_eq!(restarted.component.public.names().unwrap().task(TASK), task);
    assert!(restarted.out.is_empty(), "the kept key is never replaced");
}
#[test]
fn the_key_is_drawn_and_kept_once_and_reads_are_busy_until_held() {
    let mut env = f::env();
    env.limits.public = limits();
    let mut out = Queue::uncapped();
    let component = Component::new(settings(), &env, &mut out).unwrap();
    let Some(Request::Secrets(SecretsRequest::Get { owner: SecretOwner::PublicNames })) = out.pop() else {
        panic!("get public key at start");
    };
    drop(component);
    let mut step = Step::configured(limits(), settings(), false);
    let exchange = step.send(read(1, &[]));
    assert_eq!(step.reply(exchange), PublicReply::Busy);
    step.down(Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::PublicNames, value: None }));
    let Some(Request::Secrets(SecretsRequest::Keep { owner, value })) = step.out.pop() else {
        panic!("keep once");
    };
    let mut rng = skein_lib::CryptoRng::new(&f::settings().seed, f::settings().stream).unwrap();
    assert_eq!(value.digest, crate::sign_in::draw(&mut rng));
    let exchange = step.send(read(1, &[]));
    assert_eq!(step.reply(exchange), PublicReply::Busy);
    step.down(Event::Secrets(SecretsAnswer::Got { owner, value: None }));
    assert!(step.out.is_empty());
    step.down(Event::Secrets(SecretsAnswer::Kept { owner }));
    step.send(read(1, &[]));
    let _held = step.admitted();
    assert!(step.out.is_empty());
}
#[test]
fn a_kind_past_answer_bytes_refuses_the_start() {
    let mut env = f::env();
    env.limits.public = limits();
    let mut configured = settings();
    configured.public.get_mut(0).unwrap().bound = BOUND + 1;
    assert_eq!(Component::new(configured, &env, &mut Queue::uncapped()).unwrap_err(), Unusable::Limits);
    let mut configured = settings();
    configured.public_bounds.lines = 257;
    assert_eq!(Component::new(configured, &env, &mut Queue::uncapped()).unwrap_err(), Unusable::Limits);
    assert!(Component::new(settings(), &env, &mut Queue::uncapped()).is_ok(), "at the answer bound");
}
#[test]
fn a_rate_of_zero_answers_every_read_busy() {
    let mut off = limits();
    off.rate = 0;
    let mut step = Step::configured(off, settings(), true);
    for request in [read(1, &[]), read(2, &[1]), read(3, &[1, 2])] {
        let exchange = step.send(request);
        assert_eq!(step.reply(exchange), PublicReply::Busy);
    }
}
#[test]
fn past_the_room_the_rate_or_the_bytes_a_read_is_busy_and_nothing_is_handed_on() {
    let base = limits();
    for (label, limits) in [
        ("room", crate::PublicLimits { reads: 1, ..base }),
        ("rate", crate::PublicLimits { rate_burst: 1, ..base }),
        ("bytes", crate::PublicLimits { bytes_burst: u64::from(BOUND), ..base }),
    ] {
        let mut step = Step::configured(limits, settings(), true);
        step.send(read(1, &[]));
        let _held = step.admitted();
        let exchange = step.send(read(1, &[]));
        assert_eq!(step.reply(exchange), PublicReply::Busy, "{label}");
    }
}
#[test]
fn an_unknown_kind_a_wrong_id_count_and_an_unserved_version_reach_no_engine() {
    let mut step = Step::new();
    for request in [
        read(0, &[]),
        PublicRead { owner: 99, ..read(1, &[]) },
        read(1, &[1]),
        read(2, &[]),
        read(3, &[1]),
        read(3, &[1, 2, 3]),
    ] {
        let exchange = step.send(request);
        assert_eq!(step.reply(exchange), PublicReply::Unknown);
    }
    for version in [0, 2] {
        let exchange = step.send(PublicRead { version, ..read(2, &[1]) });
        let PublicReply::NotServed { record } = step.reply(exchange) else {
            panic!("not served");
        };
        let record = wire::v22::NotServed::decode(&step.env.limits.wire, &mut Reader::new(&record)).unwrap();
        assert_eq!(record.served().len(), 1);
        let served = record.served().get(0).unwrap();
        assert_eq!((served.first(), served.last()), (1, 1));
    }
    assert_eq!(step.bytes(), BURST);
}
#[test]
fn every_well_formed_read_is_handed_on_alike() {
    let mut step = Step::new();
    for number in [1_u64, 2, u64::MAX] {
        let id = if let Ok(project) = u32::try_from(number) { names().project(project) } else { names().task(number) };
        step.send(read(2, &[id]));
        let (reply, mapped) = step.admitted();
        assert_eq!(
            mapped,
            c::PublicRead { owner: 0, version: 1, kind: 2, project: Some(names().number(id, Of::Project)), task: None }
        );
        step.answer(reply, c::PublicAnswer::Unknown);
        step.out.pop().unwrap();
    }
}
#[test]
fn reservations_are_given_back_on_every_end() {
    let mut step = Step::new();
    for answer in [
        c::PublicAnswer::Unknown,
        c::PublicAnswer::Busy,
        c::PublicAnswer::Document {
            decided: Time::ZERO,
            document: c::PublicDocument::Projects(c::PublicProjects {
                projects: Box::new([
                    c::PublicProject { project: 1, name: Box::from("x") },
                    c::PublicProject { project: 2, name: Box::from("y") },
                    c::PublicProject { project: 3, name: Box::from("z") },
                ]),
            }),
        },
    ] {
        let exchange = step.send(read(1, &[]));
        let (reply, _) = step.admitted();
        let expected = match &answer {
            c::PublicAnswer::Busy => PublicReply::Busy,
            c::PublicAnswer::Unknown | c::PublicAnswer::Document { .. } => PublicReply::Unknown,
        };
        step.answer(reply, answer);
        assert_eq!(step.reply(exchange), expected);
        assert_eq!(step.bytes(), BURST);
    }
    let exchange = step.send(read(1, &[]));
    let (reply, _) = step.admitted();
    step.down(Event::Transport(FromTransport::ExchangeGone { exchange }));
    step.document(reply, projects());
    assert!(step.out.is_empty());
    assert_eq!(step.bytes(), BURST);
    let exchange = step.send(read(1, &[]));
    let (reply, _) = step.admitted();
    step.document(reply, projects());
    let bytes = bytes(step.reply(exchange));
    assert_eq!(step.bytes(), BURST - len_u64(bytes.len()));
}
#[test]
fn bursts_bound_reads_at_once() {
    let mut limits = limits();
    limits.reads = 4;
    limits.rate_burst = 2;
    let mut step = Step::configured(limits, settings(), true);
    for _ in 0_u8..2 {
        step.send(read(1, &[]));
        let (reply, _) = step.admitted();
        step.answer(reply, c::PublicAnswer::Unknown);
        step.out.pop().unwrap();
    }
    let exchange = step.send(read(1, &[]));
    assert_eq!(step.reply(exchange), PublicReply::Busy);
    // Eight reads per second restore exactly one read in 125 ms, including fractional updates.
    step.env.now = Time::from_nanos(62_500_000);
    let exchange = step.send(read(1, &[]));
    assert_eq!(step.reply(exchange), PublicReply::Busy);
    step.env.now = Time::from_nanos(125_000_000);
    step.send(read(1, &[]));
    let _held = step.admitted();
    step.env.now = Time::ZERO;
    let exchange = step.send(read(1, &[]));
    assert_eq!(step.reply(exchange), PublicReply::Busy, "a backwards clock cannot refill");
}
#[test]
fn a_document_crosses_as_public_records_with_identifiers_alone() {
    let mut step = Step::new();
    let names = names();
    for (kind, ids, document) in [
        (1, Box::<[u64]>::from([]), projects()),
        (2, Box::from([names.project(PROJECT)]), board()),
        (3, Box::from([names.project(PROJECT), names.task(TASK)]), card()),
    ] {
        let exchange = step.send(read(kind, &ids));
        let (reply, _) = step.admitted();
        step.document(reply, document);
        let bytes = bytes(step.reply(exchange));
        for marker in
            [Box::<[u8]>::from(PROJECT.to_be_bytes()), Box::from(TASK.to_be_bytes()), Box::from(PARENT.to_be_bytes())]
        {
            assert!(skein_lib::bytes::find(&bytes, &marker).is_none());
        }
        match kind {
            1 => {
                let record =
                    w::PublicProjects::decode(&wire::public_limits(&INDEX).unwrap(), &mut Reader::new(&bytes)).unwrap();
                assert_eq!(record.max_age(), 5);
                assert_eq!(record.projects().get(0).unwrap().project(), names.project(PROJECT));
                assert_eq!(record.projects().get(0).unwrap().name(), b"Public project");
            }
            2 => {
                let record =
                    w::PublicBoard::decode(&wire::public_limits(&INDEX).unwrap(), &mut Reader::new(&bytes)).unwrap();
                assert_eq!(record.project(), names.project(PROJECT));
                assert_eq!(record.name(), b"Public project");
                assert_eq!(record.live().get(0).unwrap().line().task(), names.task(TASK));
                assert_eq!(record.live().get(0).unwrap().tree().get(0).unwrap().task(), names.task(PARENT));
            }
            3 => {
                let record =
                    w::PublicCard::decode(&wire::public_limits(&INDEX).unwrap(), &mut Reader::new(&bytes)).unwrap();
                assert_eq!(record.line().task(), names.task(TASK));
                assert_eq!(*record.line().parent(), Some(names.task(PARENT)));
                assert_eq!(record.lineage().get(0), Some(&names.task(PARENT)));
                assert_eq!(record.delegates().get(0).unwrap().task(), names.task(PARENT));
                assert_eq!(record.line().title(), b"Public task");
            }
            _ => unreachable!(),
        }
    }
}
#[test]
fn an_application_document_crosses_in_jigs_envelope() {
    let mut step = Step::new();
    let exchange = step.send(PublicRead {
        owner: 10,
        version: 2,
        kind: 1,
        ids: Box::new([names().project(PROJECT), names().task(TASK)]),
    });
    let (reply, mapped) = step.admitted();
    assert_eq!(mapped.task, Some(TASK));
    step.document(
        reply,
        c::PublicDocument::Application(c::PublicApplication {
            tasks: Box::new([TASK, PARENT, TASK]),
            record: Box::from(b"application".as_slice()),
        }),
    );
    let record = w::PublicApplication::decode(
        &wire::public_limits(&INDEX).unwrap(),
        &mut Reader::new(&bytes(step.reply(exchange))),
    )
    .unwrap();
    assert_eq!(record.tasks().as_slice(), &[names().task(TASK), names().task(PARENT), names().task(TASK)]);
    assert_eq!(record.record(), b"application");
}
#[test]
fn me_gives_a_public_projects_identifier_and_none_for_a_private_one() {
    let mut exchange = f::read_exchange();
    let mut frame = f::frame_two_projects();
    frame.projects.get_mut(0).unwrap().public = true;
    for held in [false, true] {
        if held {
            exchange.component.down(
                &exchange.env,
                Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::PublicNames, value: Some(kept()) }),
                &mut exchange.out,
            );
        }
        let (owner, reply, _) = exchange.read(&wire::v22::Read::Me);
        exchange
            .release_read(reply, c::reads::Answer::Document(Box::new(c::documents::Document::Frame(frame.clone()))));
        let wire::v22::ReadAnswer::Value(value) = exchange.read_answer(owner) else {
            panic!("me value");
        };
        let wire::v22::ReadValue::Me(value) =
            wire::v22::ReadValue::decode(&exchange.env.limits.wire, &mut Reader::new(value.record())).unwrap()
        else {
            panic!("me");
        };
        assert_eq!(
            frame_public_identifiers(&value, &exchange.env.limits.wire),
            [if held { Some(names().project(3)) } else { None }, None]
        );
        exchange.component.up(&exchange.env, &mut exchange.out);
        exchange.out.pop();
    }
}

#[test]
fn titles_are_cut_on_a_character_boundary_and_keep_the_sources_cut() {
    let mut step = Step::new();
    let mut crossing = [b'x'; 257];
    for (slot, byte) in crossing.get_mut(255..).unwrap().iter_mut().zip("é".as_bytes()) {
        *slot = *byte;
    }
    let cases: [(Box<[u8]>, bool, bool, usize); 3] = [
        (Box::new([b'x'; 256]), false, false, 256),
        (Box::new(crossing), false, true, 255),
        (Box::from(b"short".as_slice()), true, true, 5),
    ];
    for (title, source_cut, expected_cut, length) in cases {
        let mut line = line(TASK);
        line.title = c::documents::Text { bytes: title, cut: source_cut };
        let exchange = step.send(read(3, &[names().project(PROJECT), names().task(TASK)]));
        let (reply, _) = step.admitted();
        step.document(
            reply,
            c::PublicDocument::Card(c::PublicCard { line, lineage: Box::new([]), delegates: Box::new([]), folded: 0 }),
        );
        let bytes = bytes(step.reply(exchange));
        let card = w::PublicCard::decode(&wire::public_limits(&INDEX).unwrap(), &mut Reader::new(&bytes)).unwrap();
        assert_eq!(card.line().title().len(), length);
        assert_eq!(card.line().title_cut(), expected_cut);
    }
}
#[test]
fn abort_returns_every_public_reservation() {
    let mut step = Step::new();
    for _ in 0_u8..2 {
        step.send(read(1, &[]));
        let _held = step.admitted();
    }
    step.down(Event::Abort);
    step.component.up(&step.env, &mut step.out);
    assert_eq!(step.bytes(), BURST);
}
#[test]
fn jig_kinds_reserve_the_configured_index_in_public_version_one() {
    // The proposed index figures, including the 64-project public-wire measurement.
    let bounds = wire::PublicIndexBounds { goals: 20, ended: 20, lines: 50, depth: 16, projects: 64 };
    let kinds = crate::jig_kinds(&bounds);
    for (kind, expected) in kinds.into_iter().zip([
        crate::PublicKind { owner: 0, kind: 1, first: 1, last: 1, task: false, bound: 4_872 },
        crate::PublicKind { owner: 0, kind: 2, first: 1, last: 1, task: false, bound: 716_448 },
        crate::PublicKind { owner: 0, kind: 3, first: 1, last: 1, task: true, bound: 18_041 },
    ]) {
        assert_eq!(kind, expected);
    }
}

#[test]
fn a_me_watch_maps_its_public_project_identifier() {
    let mut exchange = f::read_exchange();
    exchange.component.down(
        &exchange.env,
        Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::PublicNames, value: Some(kept()) }),
        &mut exchange.out,
    );
    let stream = exchange.open_stream();
    exchange.open_record(stream, 17, f::owned_subject(&wire::v22::Subject::Me));
    let watcher = exchange.take_watch();
    exchange.component.down(&exchange.env, Event::Admitted { watcher }, &mut exchange.out);
    let mut frame = f::frame_two_projects();
    frame.projects.get_mut(0).unwrap().public = true;
    exchange.component.down(
        &exchange.env,
        Event::Delivery { watcher, document: Box::new(c::documents::Document::Frame(frame)) },
        &mut exchange.out,
    );
    let packet = &exchange.stream().watches.get(&watcher).unwrap().delivery.as_ref().unwrap().event;
    let wire::v22::StreamEvent::Delivery(value) =
        wire::v22::StreamEvent::decode(&exchange.env.limits.wire, &mut Reader::new(packet)).unwrap()
    else {
        panic!("me delivery");
    };
    let wire::v22::DeliveryValue::Me(value) =
        wire::v22::DeliveryValue::decode(&exchange.env.limits.wire, &mut Reader::new(value.document().record()))
            .unwrap()
    else {
        panic!("me value");
    };
    assert_eq!(frame_public_identifiers(&value, &exchange.env.limits.wire), [Some(names().project(3)), None]);
}
