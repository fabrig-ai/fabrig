use super::fixtures;
use crate::{Event, FromApplication, Request, ToApplication, translate::watches};
use jig_client_protocol_wire::v20 as w;
use jig_domain_core as core;
use skein_lib::{Reader, Token};
const ENDED_WATCH: u64 = 17;
fn stream_world() -> (fixtures::Exchange, [u8; 16]) {
    let mut world = fixtures::Exchange::new();
    let stream = world.open_stream();
    (world, stream)
}
fn gone_watch(subject: w::Owned) -> (fixtures::Exchange, Token) {
    let mut env = fixtures::env();
    env.limits.streams = 1;
    let mut world = fixtures::Exchange::with_env(env);
    let stream = world.open_stream();
    world.open_record(stream, 17, subject);
    let watcher = *world.stream().watches.first().expect("watch").0;
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    let owner = *world.component.streams.first().expect("stream").0;
    world.component.down(
        &world.env,
        Event::Transport(crate::FromTransport::StreamGone { stream: owner }),
        &mut world.out,
    );
    while world.component.up(&world.env, &mut world.out) {}
    world.out.clear();
    (world, watcher)
}
#[test]
fn a_gone_stream_releases_its_slot_without_a_core_ending() {
    let (mut world, _) = gone_watch(fixtures::owned_subject(&fixtures::task_subject()));
    assert!(world.component.streams.is_empty(), "a lost connection cannot retain stream capacity");
    world.open_stream();
    assert_eq!(world.component.streams.len(), 1);
}
#[test]
fn a_late_core_document_after_stream_loss_is_acknowledged() {
    let (mut world, watcher) = gone_watch(fixtures::owned_subject(&fixtures::task_subject()));
    world.component.down(
        &world.env,
        Event::Delivery {
            watcher,
            document: Box::new(core::documents::Document::Frame(fixtures::frame_two_projects())),
        },
        &mut world.out,
    );
    let Some(Request::Core(event)) = world.out.pop() else { panic!("late delivery acknowledgement") };
    let core::Event::Delivered { watcher: got } = *event else { panic!("document acknowledgement") };
    assert_eq!(got, watcher);
    assert!(world.out.is_empty());
}
#[test]
fn a_late_application_document_after_stream_loss_is_acknowledged() {
    let (mut world, watcher) = gone_watch(fixtures::family_subject());
    world.component.down(
        &world.env,
        Event::Application(FromApplication::Delivery { watcher, family: 9, record: Box::new([77]) }),
        &mut world.out,
    );
    let Some(Request::Core(event)) = world.out.pop() else { panic!("late delivery acknowledgement") };
    let core::Event::Delivered { watcher: got } = *event else { panic!("document acknowledgement") };
    assert_eq!(got, watcher);
    assert!(world.out.is_empty());
}
#[test]
fn a_late_run_text_after_stream_loss_is_acknowledged() {
    let subject = w::Subject::Run(
        w::SubjectRun::new(&fixtures::wire(), w::SubjectRunParts { task: 7, turn: None, offset: 0 }).expect("run"),
    );
    let (mut world, watcher) = gone_watch(fixtures::owned_subject(&subject));
    let writing = fixtures::writing();
    world.component.down(
        &world.env,
        Event::RunText {
            watcher,
            turn: writing.turn,
            from: writing.from,
            to: writing.to,
            text: writing.text,
            cut: writing.cut,
            call: writing.call,
            writing: writing.writing,
        },
        &mut world.out,
    );
    let Some(Request::Core(event)) = world.out.pop() else { panic!("late text acknowledgement") };
    let core::Event::StreamDelivered { watcher: got } = *event else { panic!("text acknowledgement") };
    assert_eq!(got, watcher);
    assert!(world.out.is_empty());
}
#[test]
fn a_delivery_for_a_name_the_component_never_issued_is_ignored() {
    let mut world = fixtures::Exchange::new();
    let watcher = Token::new(u64::MAX);
    let writing = fixtures::writing();
    for event in [
        Event::Delivery {
            watcher,
            document: Box::new(core::documents::Document::Frame(fixtures::frame_two_projects())),
        },
        Event::Application(FromApplication::Delivery { watcher, family: 9, record: Box::new([77]) }),
        Event::RunText {
            watcher,
            turn: writing.turn,
            from: writing.from,
            to: writing.to,
            text: writing.text,
            cut: writing.cut,
            call: writing.call,
            writing: writing.writing,
        },
    ] {
        world.component.down(&world.env, event, &mut world.out);
        assert!(world.out.is_empty(), "a foreign name settles no watch");
    }
}
fn request_task(world: &mut fixtures::Exchange, stream: [u8; 16], name: u64) -> (Token, Token) {
    let owner = world.open_record(stream, name, fixtures::owned_subject(&fixtures::task_subject()));
    (owner, world.take_watch())
}
#[test]
fn a_zero_default_window_refuses_to_start() {
    let mut settings = fixtures::settings();
    settings.rows = 0;
    let mut out = skein_lib::Queue::with_capacity(4);
    assert_eq!(crate::Component::new(settings, &fixtures::env(), &mut out).err(), Some(crate::Unusable::Limits));
    assert!(out.is_empty());
}
#[test]
fn every_watch_subject_preserves_its_identity_and_window() {
    for (label, source, subject, rows) in fixtures::subject_cases() {
        let Some(core::Event::Watch { watcher, sign_in, subject: actual, window }) =
            watches::watch(Token::new(11), 41, source, 32)
        else {
            panic!("{label}: watch");
        };
        assert_eq!(
            (watcher, sign_in, actual, window.rows, window.after),
            (Token::new(11), 41, subject, rows, None),
            "{label}"
        );
    }
}
#[test]
fn an_application_watch_keeps_its_owner_and_rejects_foreign_deliveries() {
    let (mut world, stream) = stream_world();
    let owner = world.open_record(stream, 17, fixtures::family_subject());
    let Some(Request::Application(ToApplication::Watch { watcher, sign_in, family, version, record, .. })) =
        world.out.pop()
    else {
        panic!("family watch");
    };
    assert_eq!((sign_in, family, version, record), (41, 9, 2, Box::from([77])));
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    assert_eq!(
        w::OpenAnswer::decode(&fixtures::wire(), &mut Reader::new(world.response(owner).1)),
        Ok(w::OpenAnswer::Admitted)
    );
    world.component.down(
        &world.env,
        Event::Application(FromApplication::Delivery { watcher, family: 8, record: Box::new([77]) }),
        &mut world.out,
    );
    assert!(world.stream().watches.get(&watcher).expect("watch").delivery.is_none());
    world.component.down(
        &world.env,
        Event::Application(FromApplication::Delivery { watcher, family: 9, record: Box::new([88]) }),
        &mut world.out,
    );
    let packet =
        &world.stream().watches.get(&watcher).expect("watch").delivery.as_ref().expect("family delivery").event;
    let w::StreamEvent::Delivery(value) =
        w::StreamEvent::decode(&fixtures::wire(), &mut Reader::new(packet)).expect("event")
    else {
        panic!("delivery");
    };
    assert_eq!((value.watch(), value.document().version(), value.document().record()), (17, 2, &[88][..]));
}
#[test]
fn a_watch_admission_or_refusal_cannot_answer_another_exchange() {
    let (mut world, stream) = stream_world();
    let (first, watcher) = request_task(&mut world, stream, 17);
    world.component.down(&world.env, Event::Admitted { watcher: Token::new(999) }, &mut world.out);
    assert!(world.stream().watches.get(&watcher).expect("watch").opening.is_some());
    world.component.down(&world.env, Event::WatchRefused { watcher, why: core::WatchRefusal::Busy }, &mut world.out);
    let expected = w::OpenAnswer::Refused(
        w::OpenAnswerRefused::new(&fixtures::wire(), w::OpenAnswerRefusedParts { why: w::WatchRefusal::Busy })
            .expect("busy"),
    );
    assert_eq!(w::OpenAnswer::decode(&fixtures::wire(), &mut Reader::new(world.response(first).1)), Ok(expected));
    let (second, other) = request_task(&mut world, stream, 18);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    assert!(world.stream().watches.get(&other).expect("other watch").opening.is_some());
    assert!(
        world
            .component
            .pending
            .exchanges
            .get(&world.component.connections.get(&second).expect("second").name)
            .is_some()
    );
}
#[test]
fn waiting_for_room_keeps_one_delivery_per_watch() {
    let (mut world, stream) = stream_world();
    let (_, watcher) = request_task(&mut world, stream, 17);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    let document = core::documents::Document::Frame(fixtures::frame_two_projects());
    for _ in 0_u8..3 {
        world.component.down(
            &world.env,
            Event::Delivery { watcher, document: Box::new(document.clone()) },
            &mut world.out,
        );
    }
    assert!(world.stream().waiting);
    assert!(world.stream().watches.get(&watcher).expect("watch").delivery.is_some());
    assert_eq!(world.stream().message.iter().count(), 1);
    assert!(world.out.is_empty());
}

#[test]
fn each_watch_ending_preserves_its_reason() {
    use jig_domain_views::End;
    for (label, source, expected) in [
        ("Closed", End::Unwatched, w::WatchEnd::Closed),
        ("SubjectEnded", End::Gone, w::WatchEnd::SubjectEnded),
        ("NotAllowed", End::NotAllowed, w::WatchEnd::NotAllowed),
        ("SignedOut", End::SignedOut, w::WatchEnd::SignedOut),
        ("Stopping", End::Stopping, w::WatchEnd::Stopping),
    ] {
        assert_eq!(watches::ended(source), expected, "{label}");
    }
}
#[test]
fn each_watch_refusal_preserves_its_reason() {
    use core::WatchRefusal;
    for (label, source, expected) in [
        ("Unknown", WatchRefusal::Unknown, w::WatchRefusal::Unknown),
        ("NotAllowed", WatchRefusal::NotAllowed, w::WatchRefusal::NotAllowed),
        ("Busy", WatchRefusal::Busy, w::WatchRefusal::Busy),
        ("SignedOut", WatchRefusal::SignedOut, w::WatchRefusal::SignedOut),
    ] {
        assert_eq!(watches::refused(source), expected, "{label}");
    }
}
#[test]
fn unwatch_answers_only_after_its_released_ending() {
    let (mut world, stream) = stream_world();
    let (_, watcher) = request_task(&mut world, stream, 17);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    let owner = world.unwatch(stream, 17);
    let Some(Request::Core(event)) = world.out.pop() else {
        panic!("unwatch");
    };
    let core::Event::Unwatch { watcher: actual } = *event else {
        panic!("unwatch");
    };
    assert_eq!(actual, watcher);
    let exchange = world.component.connections.get(&owner).expect("connection").name;
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    assert!(world.component.pending.exchanges.get(&exchange).is_some());
    world.component.down(
        &world.env,
        Event::WatchEnded { watcher, why: jig_domain_views::End::Unwatched },
        &mut world.out,
    );
    assert_eq!(
        w::CloseAnswer::decode(&fixtures::wire(), &mut Reader::new(world.response(owner).1)),
        Ok(w::CloseAnswer::Closed)
    );
}
fn ended_watch() -> (fixtures::Exchange, [u8; 16], Token) {
    let (mut world, stream) = stream_world();
    let (_, watcher) = request_task(&mut world, stream, ENDED_WATCH);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    world.component.down(&world.env, Event::WatchEnded { watcher, why: jig_domain_views::End::Gone }, &mut world.out);
    let owner = *world.component.streams.first().expect("stream").0;
    (world, stream, owner)
}
fn closed_replies(world: &mut fixtures::Exchange, owner: Token) -> usize {
    let mut replies = 0_usize;
    loop {
        while let Some(request) = world.out.pop() {
            if let Request::Transport(crate::ToTransport::Reply { exchange, reply }) = request
                && exchange == owner
            {
                let crate::Reply::Record(record) = reply else { panic!("close record") };
                assert_eq!(
                    w::CloseAnswer::decode(&fixtures::wire(), &mut Reader::new(&record)),
                    Ok(w::CloseAnswer::Closed)
                );
                replies = replies.saturating_add(1);
            }
        }
        if !world.component.up(&world.env, &mut world.out) {
            return replies;
        }
    }
}
#[test]
fn an_ended_watch_closes_once_when_its_stream_goes() {
    let (mut world, stream, stream_owner) = ended_watch();
    let owner = world.unwatch(stream, ENDED_WATCH);
    world.component.down(
        &world.env,
        Event::Transport(crate::FromTransport::StreamGone { stream: stream_owner }),
        &mut world.out,
    );
    assert_eq!(closed_replies(&mut world, owner), 1);
    assert_eq!(closed_replies(&mut world, owner), 0);
}
#[test]
fn an_ended_watch_closes_once_when_its_ending_leaves() {
    let (mut world, stream, stream_owner) = ended_watch();
    let owner = world.unwatch(stream, ENDED_WATCH);
    world.component.down(
        &world.env,
        Event::Transport(crate::FromTransport::StreamTaken { stream: stream_owner }),
        &mut world.out,
    );
    let replies = closed_replies(&mut world, owner);
    world.component.down(
        &world.env,
        Event::Transport(crate::FromTransport::StreamTaken { stream: stream_owner }),
        &mut world.out,
    );
    assert_eq!(replies + closed_replies(&mut world, owner), 1);
    assert_eq!(closed_replies(&mut world, owner), 0);
}
#[test]
fn a_run_watch_keeps_the_complete_provisional_position() {
    let limits = fixtures::wire();
    let rows = [
        ("Absent", None, None),
        (
            "Full",
            Some(
                w::ProvisionalTurnName::new(
                    &limits,
                    w::ProvisionalTurnNameParts { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX },
                )
                .expect("turn"),
            ),
            Some(jig_domain_views::TurnName { attempt: u64::MAX, turn: u32::MAX, retry: u32::MAX }),
        ),
    ];
    for (label, turn, expected) in rows {
        let source = w::Subject::Run(
            w::SubjectRun::new(&limits, w::SubjectRunParts { task: u64::MAX, turn, offset: u32::MAX }).expect("run"),
        );
        let Some(core::Event::WatchRun { watcher, sign_in, task, turn, offset }) =
            watches::watch(Token::new(11), 41, source, 32)
        else {
            panic!("{label}: run");
        };
        assert_eq!(
            (watcher, sign_in, task, turn, offset),
            (Token::new(11), 41, u64::MAX, expected, u32::MAX),
            "{label}"
        );
    }
}

#[test]
fn an_absent_stream_digest_is_answered_signed_out() {
    let mut world = fixtures::Exchange::new();
    let owner = world.stream_open();
    let Some(Request::Secrets(crate::SecretsRequest::Get { owner: wanted })) = world.out.pop() else {
        panic!("digest get");
    };
    world.component.down(
        &world.env,
        Event::Secrets(crate::SecretsAnswer::Got { owner: wanted, value: None }),
        &mut world.out,
    );
    assert_eq!(world.response(owner), (401, &[][..]));
    assert!(world.component.streams.is_empty());
}

#[test]
fn an_accounts_document_is_watched_but_never_read() {
    use jig_domain_core::{documents as d, reads};
    use skein_lib::List;
    let document = d::Document::Accounts(Box::new(d::Accounts {
        readiness: List::with_capacity(0),
        readiness_cut: false,
        program_hosts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        signing: List::with_capacity(0),
        last_end: None,
        usable: false,
        settled: true,
        cut: false,
    }));
    let limits = fixtures::wire();
    assert_eq!(
        watches::delivery(&document, 0, 1, &[], &limits),
        Ok(w::DeliveryValue::Accounts(
            w::Accounts::new(
                &limits,
                w::AccountsParts {
                    skein_bools: [false, true, false, false],
                    readiness: Box::new([]),
                    accounts: Box::from([]),
                    signing: Box::from([]),
                    last_end: None,
                    program_hosts: Box::new([]),
                }
            )
            .unwrap()
        ))
    );
    assert_eq!(
        crate::translate::objects::document(
            &reads::Answer::Document(Box::new(document)),
            &[],
            &crate::Private::none(),
            &limits
        ),
        Err(w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag })
    );
}

// jig/docs/design/protocol/client/README.md, 5 and 10.
#[test]
fn a_web_pages_first_deliveries_fit_together() {
    const WATCHES: u32 = 26;
    const DOCUMENT_BYTES: usize = 16_384;
    let mut env = fixtures::env();
    env.limits.watches_per_stream = WATCHES;
    env.limits.exchanges = WATCHES + 1; // The stream owns a connection too.
    env.limits.wire = w::CEILINGS;
    env.limits.queued_bytes =
        crate::limits::stream_queue_bytes(WATCHES, &env.limits.wire).expect("first-delivery queue");
    let mut world = fixtures::Exchange::with_env(env);
    let stream = world.open_stream();
    for name in 1..=WATCHES {
        world.open_record(stream, u64::from(name), fixtures::family_subject());
        let Some(Request::Application(ToApplication::Watch { watcher, .. })) = world.out.pop() else {
            panic!("application watch");
        };
        world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
        world.component.down(
            &world.env,
            Event::Application(FromApplication::Delivery {
                watcher,
                family: 9,
                record: fixtures::message_bytes(DOCUMENT_BYTES),
            }),
            &mut world.out,
        );
        assert!(!world.stream().closing(), "first delivery of watch {name}");
    }
    assert_eq!(world.stream().watches.len(), WATCHES);
    assert!(world.stream().waiting, "the hello is still in flight");
    for (_, watch) in &world.stream().watches {
        assert!(watch.delivery.is_some());
    }
}

fn application_watch(world: &mut fixtures::Exchange, stream: [u8; 16], name: u64) -> Token {
    world.open_record(stream, name, fixtures::family_subject());
    let Some(Request::Application(ToApplication::Watch { watcher, .. })) = world.out.pop() else {
        panic!("application watch");
    };
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    watcher
}
fn family_delivery(watcher: Token, bytes: usize) -> Event {
    Event::Application(FromApplication::Delivery { watcher, family: 9, record: fixtures::message_bytes(bytes) })
}
#[test]
fn a_stream_queue_includes_the_in_flight_record_at_its_exact_bound() {
    for short in [false, true] {
        let (mut world, stream) = stream_world();
        let watcher = application_watch(&mut world, stream, 1);
        let packet = watches::application_packet(9, 2, Box::new([77; 128]), 1, &world.env.limits.wire).expect("packet");
        let exact = world.stream().bytes + packet.measure();
        world.env.limits.queued_bytes = exact - u32::from(short);
        world.component.down(&world.env, family_delivery(watcher, 128), &mut world.out);
        assert_eq!(world.stream().closing(), short);
        if short {
            let Some(Request::StreamFailed { sign_in, why }) = world.out.pop() else { panic!("queue diagnostic") };
            assert_eq!(sign_in, fixtures::NUMBER);
            assert_eq!(why, crate::StreamFailure::QueueBound { queued_bytes: exact - 1 });
        }
        assert!(world.out.is_empty());
    }
}
#[test]
fn an_untranslatable_document_names_its_path_and_closes_only_once() {
    let (mut world, stream) = stream_world();
    let first = application_watch(&mut world, stream, 1);
    let second = application_watch(&mut world, stream, 2);
    for watcher in [first, second] {
        world.component.down(&world.env, family_delivery(watcher, 257), &mut world.out);
    }
    assert!(world.stream().closing());
    let Some(Request::StreamFailed { sign_in, why }) = world.out.pop() else { panic!("translation diagnostic") };
    assert_eq!(sign_in, fixtures::NUMBER);
    assert_eq!(
        why,
        crate::StreamFailure::Translation(w::Problem {
            path: w::Path::OwnedRecord,
            reason: skein_codec::Reason::Bound
        })
    );
    assert!(world.out.is_empty(), "one diagnostic per stream");
}

#[test]
fn an_untranslatable_me_document_also_names_its_sign_in() {
    let (mut world, stream) = stream_world();
    let (_, watcher) = request_task(&mut world, stream, 1);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    world.env.limits.wire.owned_record = 1;
    world.component.down(
        &world.env,
        Event::Delivery {
            watcher,
            document: Box::new(core::documents::Document::Frame(fixtures::frame_two_projects())),
        },
        &mut world.out,
    );
    let Some(Request::StreamFailed { sign_in, why }) = world.out.pop() else { panic!("me diagnostic") };
    assert_eq!(sign_in, fixtures::NUMBER);
    assert_eq!(
        why,
        crate::StreamFailure::Translation(w::Problem {
            path: w::Path::OwnedRecord,
            reason: skein_codec::Reason::Bound
        })
    );
    assert!(world.stream().closing());
    assert!(world.out.is_empty());
}
