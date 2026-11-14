use jig_client::{Ask, Asker, Handle, Key, Readable, Readout, Watchable, contract as c};
use jig_client_protocol_world::{
    client::Story,
    fixtures::{self, PROJECT},
};
use skein_lib::{Duration, length::len_u64};
// These handles belong to this story's single read, activity page and task watch.
const TURN: Handle = Handle { asker: Asker::JIG, number: 31 };
const ACTIVITY: Handle = Handle { asker: Asker::JIG, number: 32 };
const TASK_WATCH: Handle = Handle { asker: Asker::JIG, number: 33 };
// The cancellation is independent of the message request.
const END: Key = Key { asker: Asker::JIG, number: 92 };
const WORDS: Key = Key { asker: Asker::JIG, number: 91 };
// The fixture creates its first task at 1; 9 is absent.
const ABSENT: u64 = 9;
// The worker's turn follows the run beginning and durable message at position 4.
const COMMITTED_TURN: u64 = 4;
const WAIT: Duration = Duration::from_secs(1);
// A disconnect shorter than the read deadline lets the same store reopen.
const OFFLINE: Duration = Duration::from_millis(100);

#[test]
fn an_absent_turn_is_unknown_through_the_client() {
    let mut story = Story::live(fixtures::SEED);
    assert_eq!(
        read_once(&mut story, c::Read::Turn { task: ABSENT, attempt: 1, turn: 1 }),
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown))
    );
}

fn committed_story(mut story: Story) -> (Story, u64) {
    let task = story.running_task();
    let words = c::Text::new(Box::from(b"Please continue".as_slice()), 32).expect("bounded words");
    story.send(WORDS, Ask::Jig(Box::new(c::Ask::Message { project: PROJECT, task, words })));
    assert!(story.run_until(|s| !s.answered(WORDS).is_empty(), WAIT));
    story.watch(TASK_WATCH, Watchable::Jig(c::Subject::Task { task }));
    assert!(story.run_until(|s| s.thread_activity(TASK_WATCH, task) >= COMMITTED_TURN, WAIT));
    (story, task)
}

fn committed_turn(story: Story) -> (Story, c::ArchivedTurn) {
    let (mut story, task) = committed_story(story);
    story.read(ACTIVITY, Readable::Jig(c::Read::Activity { task, before: None }));
    assert!(story.run_until(|s| !s.pages(ACTIVITY).is_empty(), WAIT));
    let pages = story.pages(ACTIVITY);
    let [Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page)))] = pages.as_slice() else {
        panic!("activity page: {pages:?}")
    };
    let archive = page
        .rows
        .iter()
        .find_map(|line| {
            if let c::DecodedActivityLine { decoded: Some(archive), .. } = line { Some(archive.clone()) } else { None }
        })
        .expect("the committed turn's archive");
    (story, archive)
}

#[test]
fn a_committed_turn_is_read_whole_through_the_client() {
    let (mut story, archive) = committed_turn(Story::reading_chat(fixtures::SEED));
    let c::TurnRead::Decoded(turn) = &archive.turn else { panic!("decoded activity turn") };
    assert!(!turn.cut);
    assert_eq!(read_archive(&mut story, &archive), archive);
}

#[test]
fn an_activity_size_marker_keeps_its_bytes_through_the_client() {
    let body = fixtures::large_turn();
    let bytes = len_u64(body.len());
    assert!(bytes > u64::from(fixtures::TURN_PAGE_BYTES));
    let story = Story::reading_chat_with(fixtures::SEED, body, fixtures::domain_limits(), fixtures::turn_page_limits());
    let (_, archive) = committed_turn(story);
    assert_eq!(archive.turn, c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes } });
}

// Wait past the terminal for another exchange, and retain every Page for this handle.
fn read_once(story: &mut Story, read: c::Read) -> Readout {
    story.read(TURN, Readable::Jig(read));
    assert!(story.run_until(|s| !s.pages(TURN).is_empty(), WAIT));
    let exchanges = story.exchanges();
    assert!(!story.run_until(|_| false, WAIT));
    assert_eq!(story.exchanges(), exchanges);
    let pages = story.pages(TURN);
    let [page] = pages.as_slice() else { panic!("one terminal: {pages:?}") };
    page.clone()
}

fn archive_read(archive: &c::ArchivedTurn) -> c::Read {
    c::Read::Turn { task: archive.task, attempt: archive.name.attempt, turn: archive.name.number }
}

fn read_archive(story: &mut Story, archive: &c::ArchivedTurn) -> c::ArchivedTurn {
    let page = read_once(story, archive_read(archive));
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Turn(turn))) = page else { panic!("whole turn: {page:?}") };
    *turn
}

#[test]
fn a_page_sized_turn_is_read_whole_once() {
    let body = fixtures::callable_turn();
    let bytes = len_u64(body.len());
    let story = Story::reading_chat_with(
        fixtures::SEED,
        body.clone(),
        fixtures::domain_limits(),
        fixtures::turn_overlap_limits(),
    );
    let (mut story, mut archive) = committed_turn(story);
    let (single, activity) = story.turn_row_costs(&archive, body);
    assert!(single <= fixtures::TURN_OVERLAP_BYTES, "single row: {single}, activity: {activity}");
    assert!(fixtures::TURN_OVERLAP_BYTES < activity);
    assert_eq!(archive.turn, c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes } });
    let single_turn = read_archive(&mut story, &archive);
    let c::TurnRead::Decoded(turn) = &single_turn.turn else { panic!("decoded turn") };
    assert!(!turn.cut);
    let [message] = turn.messages.as_slice() else { panic!("source message") };
    let [c::Block::Text(text), c::Block::Call(call)] = message.blocks.as_slice() else { panic!("source blocks") };
    assert_eq!(text.bytes(), fixtures::TURN_TEXT);
    assert_eq!(call.tool.bytes(), fixtures::TURN_TOOL);
    assert_eq!(call.input.bytes(), fixtures::TURN_INPUT);
    assert_eq!(call.number, fixtures::TURN_CALL_NUMBER);
    archive.turn = single_turn.turn.clone();
    assert_eq!(single_turn, archive);
}

fn restart_story(story: &mut Story) {
    story.restart_engine(OFFLINE);
    assert!(story.run_until(|s| s.client.link().state == jig_client::Link::Live, WAIT));
}

#[test]
fn a_turn_read_after_restart_keeps_its_archive() {
    let (mut story, archive) = committed_turn(Story::reading_chat(fixtures::SEED));
    restart_story(&mut story);
    assert_eq!(read_archive(&mut story, &archive), archive);
}

fn oversized_turn(client: jig_client_domain::Limits, core: &jig_test_domain::Limits) {
    let body = fixtures::large_turn();
    assert_eq!(len_u64(body.len()), fixtures::LARGE_TURN_BYTES);
    let story = Story::reading_chat_with(fixtures::SEED, body, client, *core);
    let (mut story, task) = committed_story(story);
    assert_eq!(
        read_once(&mut story, c::Read::Turn { task, attempt: 1, turn: 1 }),
        Readout::Jig(c::ReadResult::None(c::ReadNone::TooLarge { bytes: fixtures::LARGE_TURN_BYTES }))
    );
}

#[test]
fn a_single_turn_past_the_engine_bound_keeps_its_size() {
    oversized_turn(fixtures::domain_limits(), &fixtures::turn_page_limits());
}

#[test]
fn a_single_turn_past_the_client_bound_keeps_its_size() {
    let mut client = fixtures::domain_limits();
    // The body fits the engine's usual read bound, but not this client document.
    client.document_bytes = u64::from(fixtures::TURN_PAGE_BYTES);
    oversized_turn(client, &fixtures::owner_limits());
}

#[test]
fn an_invisible_turn_is_unknown_after_its_task_ends() {
    let (mut story, archive) = committed_turn(Story::reading_chat(fixtures::SEED));
    story.send(
        END,
        Ask::Jig(Box::new(c::Ask::Cancel {
            project: PROJECT,
            task: archive.task,
            reason: c::Text::new(b"done".as_slice().into(), 32).expect("bounded reason"),
            grace: None,
        })),
    );
    assert!(story.run_until(|s| s.phase(archive.task) == Some(c::Phase::Cancelled), WAIT));
    story.hide_turn_task();
    restart_story(&mut story);
    assert_eq!(read_once(&mut story, archive_read(&archive)), Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)));
}

fn activity_page(story: &mut Story, handle: Handle, read: c::Read) -> c::ActivityPage {
    story.read(handle, Readable::Jig(read));
    assert!(story.run_until(|s| !s.pages(handle).is_empty(), WAIT));
    let pages = story.pages(handle);
    let [Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page)))] = pages.as_slice() else {
        panic!("one activity page: {pages:?}")
    };
    *page.clone()
}

#[test]
fn a_page_beside_a_line_reads_both_ways_through_the_engine() {
    // Four durable lines fit two pages; the worker then waits without adding rows.
    const EARLIER: Handle = Handle { asker: Asker::JIG, number: 34 };
    const LATER: Handle = Handle { asker: Asker::JIG, number: 35 };
    const NEWEST: Handle = Handle { asker: Asker::JIG, number: 36 };
    let (mut story, task) = committed_story(Story::reading_chat(fixtures::SEED));
    let tail = activity_page(&mut story, ACTIVITY, c::Read::Activity { task, before: None });
    assert_eq!(tail.rows.iter().map(|row| row.source.position).collect::<Vec<_>>(), [4, 3]);
    assert!(tail.before.is_some());
    assert_eq!(tail.after, None);
    let earlier =
        activity_page(&mut story, EARLIER, c::Read::ActivityBeside { task, position: 3, side: c::Side::Earlier });
    assert_eq!(earlier.rows.iter().map(|row| row.source.position).collect::<Vec<_>>(), [2, 1]);
    assert_eq!(earlier.before, None);
    assert_eq!(earlier.after, None);
    // Keep the oldest line, then read upward beside held lines to recover the end.
    let later = activity_page(&mut story, LATER, c::Read::ActivityBeside { task, position: 1, side: c::Side::Later });
    assert_eq!(later.rows.iter().map(|row| row.source.position).collect::<Vec<_>>(), [2, 3]);
    assert_eq!(later.before, None);
    assert!(later.after.is_some());
    let newest = activity_page(&mut story, NEWEST, c::Read::ActivityBeside { task, position: 3, side: c::Side::Later });
    assert_eq!(newest.before, None);
    assert_eq!(newest.after, None);
    let mut window = vec![earlier.rows.get(1).expect("oldest line").clone()];
    window.extend(later.rows.into_boxed().into_vec());
    window.extend(newest.rows.into_boxed().into_vec());
    assert_eq!(window.iter().map(|row| row.source.position).collect::<Vec<_>>(), [1, 2, 3, 4]);
    assert!(window.last().expect("committed turn").decoded.is_some());
    let recovered = window.get(2..).expect("recovered tail").iter().rev().cloned().collect::<Vec<_>>();
    assert_eq!(recovered, tail.rows.into_boxed().into_vec());
}

#[test]
fn a_message_past_text_reads_back_whole_through_the_engine() {
    // The fixture's people.words is 64; a 33-byte message passes its client's text of 32.
    let mut client = fixtures::domain_limits();
    client.text = 32;
    client.long_text = 128;
    let core = fixtures::owner_limits();
    let words = vec![b'w'; 33].into_boxed_slice();
    assert!(words.len() > usize::try_from(client.text).expect("text fits"));
    assert!(words.len() <= usize::try_from(client.long_text).expect("long_text fits"));
    assert!(words.len() <= usize::try_from(core.core.people.words).expect("people.words fits"));
    let mut story = Story::reading_chat_with(fixtures::SEED, Box::from(fixtures::TURN_BYTES), client, core);
    let task = story.running_task();
    story.send(
        WORDS,
        Ask::Jig(Box::new(c::Ask::Message {
            project: PROJECT,
            task,
            words: c::Text::new(words.clone(), 128).expect("whole words"),
        })),
    );
    assert!(story.run_until(|s| !s.answered(WORDS).is_empty(), WAIT));
    assert!(
        story.answered(WORDS).iter().any(|answer| matches!(
            answer,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_)))
        ))
    );
    let page = activity_page(&mut story, ACTIVITY, c::Read::Activity { task, before: None });
    let read = page
        .rows
        .iter()
        .find_map(|row| match &row.source.line {
            c::Activity::Words { words, by: c::Requester::Person(_), .. } => Some(words),
            _ => None,
        })
        .expect("person's activity line");
    assert_eq!(read.bytes(), words.as_ref());
}

// The component's core peer supplies the carrier; the core world checks who gets each mark.
fn private_turn() -> jig_domain_core::TurnRecord {
    use smith_transcript::v7 as s;
    let mut turn = smith_transcript::decode_archived(&s::CEILINGS, &fixtures::callable_turn())
        .expect("callable turn")
        .into_parts();
    turn.system = Some(Box::from(b"## Preferences\n\n1. A private preference\n\n## Work\nDo the work.\n".as_slice()));
    let turn = s::Turn::new(&s::CEILINGS, turn).expect("private section");
    let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(turn.measure()));
    turn.encode(&mut bytes).expect("measured turn");
    jig_domain_core::TurnRecord {
        opens_window: true,
        task: 1,
        attempt: 1,
        turn: 1,
        spent: 1,
        read: None,
        at: skein_lib::Wall::EPOCH,
        transcript: bytes.finish(),
    }
}

fn private_turn_over_wire(record: jig_domain_core::TurnRecord, person: u64, withheld: bool) -> c::ArchivedTurn {
    use jig_client_protocol_wire::v22 as w;
    use jig_domain_core::{self as core, reads};
    use jig_engine_clients_kit::fixtures as engine;
    use skein_lib::{List, Reader, Writer};
    let mut settings = engine::settings();
    settings.private = jig_protocol_transcript::Private {
        title: Box::from(b"Preferences".as_slice()),
        tools: List::from_boxed(Box::new([Box::from(fixtures::TURN_TOOL)])),
    };
    let mut peer = engine::Peer::started(&settings);
    let exchange = peer.post_link(&peer.printed_link());
    peer.answer_signed_in(engine::FIRST_SIGN_IN, person);
    peer.answer_kept(engine::FIRST_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    let mut bearer = Writer::new(61);
    bearer.put(b"Bearer ").expect("prefix");
    skein_lib::base64::encode(
        skein_lib::base64::Alphabet::Url,
        answer.token().as_ref().expect("fake bearer").expose(),
        &mut bearer,
    )
    .expect("bearer fits");
    let bearer = bearer.finish();
    let limits = engine::limits().wire;
    let read = c::Read::Turn { task: record.task, attempt: record.attempt, turn: record.turn };
    let owned = jig_client_protocol_link::translate::reads::owned(Readable::Jig(read.clone()), &limits)
        .expect("turn read envelope");
    let mut bytes = Writer::new(skein_lib::length::usize_of(owned.measure()));
    owned.encode(&mut bytes).expect("measured read");
    let socket = peer.post(b"/jig/v1/read", bytes.finish(), Some(&bearer));
    let Some(core::Event::Read { reply, sign_in, read: requested }) = peer.take_core() else {
        panic!("read reaches core")
    };
    assert_eq!(sign_in, engine::FIRST_SIGN_IN);
    assert_eq!(requested, reads::Read::Turn { task: record.task, attempt: record.attempt, turn: record.turn });
    assert!(peer.answered(socket).is_none());
    peer.answer_read(
        reply,
        reads::Answer::Ended(Box::new(reads::Ended {
            value: if withheld { reads::EndedValue::WithheldTurn(record) } else { reads::EndedValue::Turn(record) },
            at: Some(skein_lib::Wall::EPOCH),
            by: None,
        })),
    );
    let w::ReadAnswer::Value(envelope) =
        w::ReadAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("wire answer")))
            .expect("read answer")
    else {
        panic!("turn value")
    };
    let limits = jig_client_protocol_link::Limits::new(fixtures::domain_limits(), 1).expect("client limits");
    let c::ReadValue::Turn(archive) =
        jig_client_protocol_link::translate::reads::value(envelope.record(), &limits).expect("client decodes turn")
    else {
        panic!("decoded archive")
    };
    *archive
}

#[test]
fn a_turns_private_parts_reach_its_person_alone_over_the_wire() {
    const P: u64 = jig_engine_clients_kit::fixtures::FIRST_PERSON;
    const Q: u64 = P + 1;
    for (person, withheld) in [(P, false), (Q, true)] {
        let record = private_turn();
        let whole = record.transcript.clone();
        let archive = private_turn_over_wire(record, person, withheld);
        assert_eq!((archive.task, archive.name.attempt, archive.name.number, archive.cumulative), (1, 1, 1, 1));
        let c::TurnRead::Decoded(turn) = archive.turn else { panic!("whole decoded turn") };
        assert!(!turn.cut);
        let expected_system = if withheld {
            format!("## Preferences\n\n{}\n\n## Work\nDo the work.\n", jig_protocol_transcript::WITHHELD).into_bytes()
        } else {
            b"## Preferences\n\n1. A private preference\n\n## Work\nDo the work.\n".to_vec()
        };
        let system = turn
            .messages
            .iter()
            .find(|message| message.speaker == c::Speaker::System)
            .expect("system text is inspected by a whole-turn read");
        let [c::Block::Text(text)] = system.blocks.as_slice() else { panic!("system block") };
        assert_eq!(text.bytes(), expected_system, "person {person}'s section");
        let message =
            turn.messages.iter().find(|message| message.speaker != c::Speaker::System).expect("source message");
        let [c::Block::Text(text), c::Block::Call(call)] = message.blocks.as_slice() else { panic!("source blocks") };
        assert_eq!(text.bytes(), fixtures::TURN_TEXT);
        assert_eq!(call.tool.bytes(), fixtures::TURN_TOOL);
        assert_eq!(call.number, fixtures::TURN_CALL_NUMBER);
        assert_eq!(call.input.bytes(), if withheld { b"{}" } else { fixtures::TURN_INPUT });
        if !withheld {
            let limits = jig_protocol_transcript::Limits::new(
                4096,
                c::TurnLimits { messages: 8, blocks: 8, text: 4096, outcome: 128 },
            );
            assert_eq!(turn, jig_protocol_transcript::decode(&limits, &whole).expect("requester's whole turn"));
        }
    }
}
