use super::fixtures::{self, chat as f};
use crate::pages::{
    Act, Request,
    chat::{Chat, OPENING_WEIGHT, Sending},
};
use jig_client::{Event, Readable, Told, Watchable, contract as c};
use skein_lib::{Duration, Queue, Time};
const OUTPUTS: u32 = 16;
const SERIAL: u64 = 37;
const EMPTY: u32 = 0;
const REPLY: u64 = 1;
#[test]
fn opening_a_chat_watches_task_and_reads_its_first_activity_page() {
    let mut out = Queue::with_capacity(OUTPUTS);
    let chat = Chat::new(f::TASK, crate::pages::chat::document::Mode::Chat, f::FIRST, &fixtures::limits(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: chat.document,
            subject: Watchable::Jig(c::Subject::Task { task: f::TASK })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle: chat.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None })
        }))
    );
    assert!(out.is_empty());
}
#[test]
fn a_chat_opens_to_its_opening_or_a_screen_and_a_half() {
    use jig_client::thread::window::{Edge, Edges, End};
    for (name, target) in [("short", 288), ("under", OPENING_WEIGHT - 1), ("long", OPENING_WEIGHT)] {
        let mut f = f::opened();
        let mut task = f::task();
        f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
        let request = skein_lib::bytes::copy_of(task.spec.specification.words.bytes());
        f::deliver(&mut f.client, f.watch, task);
        f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
        while f.out.pop().is_some() {}
        let cursor = c::Cursor::new(Box::from(*b"opening"), 7).unwrap();
        let words = if name == "short" {
            skein_lib::bytes::copy_of(FAST_REPLY)
        } else {
            // Two lines, one message and one text block carry the reply's bytes.
            let mut words = skein_lib::bytes::zeroed(usize::try_from(target - 4 * u64::from(c::LINE_ROOM)).unwrap());
            words.fill(b'x');
            words
        };
        let page = completed_activity(Some(cursor.clone()), &words);
        let mut weight = 0_u64;
        for row in &page.rows {
            weight = weight.checked_add(row.weight()).unwrap();
        }
        assert_eq!(weight, target, "{name}");
        let edges = Edges { before: Edge::End, after: Edge::More(cursor.clone()) };
        f.chat.told(
            Time::ZERO,
            &f.client,
            Told::Page { handle: f.chat.activity, page: activity_answer(page) },
            &mut f.out,
        );
        let older = Some(Request::Client(Event::Read {
            handle: f.chat.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: Some(cursor) }),
        }));
        if name == "long" {
            assert!(f.out.is_empty(), "{name}: the weight ends the opening read");
            assert!(f.chat.thread.reading.is_none());
            assert_eq!(f.chat.thread.lines.edges, edges, "the first page keeps its older cursor");
            assert!(!f.chat.thread.lines.edges.reaches(End::After));
            chat_act(&mut f.chat, &f.client, Act::Older, &mut f.out);
            assert_eq!(f.out.pop(), older, "the person reads from the retained cursor");
        } else {
            assert_eq!(f.out.pop(), older, "{name}: below the weight reads on");
            if name == "short" {
                let mut rows = skein_lib::List::with_capacity(1);
                rows.push(owner_activity(
                    1,
                    c::Activity::RunBegan { attempt: f::ATTEMPT, why: c::RunWhy::Made, resumed: false },
                ))
                .unwrap();
                f.chat.told(
                    Time::ZERO,
                    &f.client,
                    Told::Page {
                        handle: f.chat.activity,
                        page: activity_answer(c::ActivityPage { rows, before: None, after: None, cut: false }),
                    },
                    &mut f.out,
                );
                assert!(f.chat.thread.lines.edges.reaches(End::After));
                assert_eq!(f.chat.thread.lines.rows.len(), 3);
                assert_eq!(f.chat.request(&f.client).unwrap().0, request.as_ref());
            }
        }
        assert!(f.out.is_empty(), "{name}: no additional read");
    }
}
const FAST_REPLY: &[u8] = b"Hello from the local fake model.";
fn completed_activity(before: Option<c::Cursor>, words: &[u8]) -> c::ActivityPage {
    let mut blocks = skein_lib::List::with_capacity(1);
    blocks.push(c::Block::Text(c::Text::new(words.into(), u32::try_from(words.len()).unwrap()).unwrap())).unwrap();
    let jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(reply))) =
        f::activity_reply(c::Speaker::Assistant, blocks, 2)
    else {
        panic!("reply page")
    };
    let mut page = *reply;
    let turn = page.rows.swap_remove(0).unwrap();
    page.rows = skein_lib::List::with_capacity(2);
    page.rows
        .push(owner_activity(3, c::Activity::RunEnded { attempt: f::ATTEMPT, end: c::RunEnd::Parked, spent: 0 }))
        .unwrap();
    page.rows.push(turn).unwrap();
    page.before = before;
    page
}
#[test]
fn every_send_asks_for_the_composer() {
    let mut f = f::opened();
    let mut task = f::task();
    task.acts = fixtures::acts::offered(c::ActKind::Message, c::Admission::Allowed);
    f::deliver(&mut f.client, f.watch, task);
    for event in [
        Event::Watch { handle: crate::pages::me_handle(), subject: Watchable::Jig(c::Subject::Me) },
        Event::Up {
            up: jig_client::Up::Watched {
                watch: f.me,
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(fixtures::chats::me()))),
            },
        },
    ] {
        f.client.step_unlinked(Time::ZERO, skein_lib::Wall::EPOCH, event, &mut Queue::uncapped());
    }
    let env = fixtures::env(&fixtures::limits());
    let mut serial = SERIAL;
    assert_eq!(f.chat.composer_focus, 0);
    f.chat.act(&env, &f.client, Act::MessageWords { words: b"Hello".as_slice().into() }, &mut serial, &mut f.out);
    assert_eq!(f.chat.composer_focus, 0);
    f.chat.act(&env, &f.client, Act::Send, &mut serial, &mut f.out);
    assert_eq!(fixtures::acts::sent(&mut f.out).len(), 1);
    assert_eq!(f.chat.composer_focus, 1);
    assert!(f.chat.words.is_empty());
    f.chat.act(&env, &f.client, Act::Send, &mut serial, &mut f.out);
    assert!(fixtures::acts::sent(&mut f.out).is_empty());
    assert_eq!(f.chat.composer_focus, 2);
    for act in [Act::Stop, Act::AddDraft, Act::MessageWords { words: b"Again".as_slice().into() }] {
        f.chat.act(&env, &f.client, act, &mut serial, &mut f.out);
        assert_eq!(f.chat.composer_focus, 2);
    }
}

#[test]
fn change_something_focuses_the_composer_and_remembers_its_proposal() {
    let mut f = f::opened();
    assert_eq!(f.chat.change_asked, None);
    for count in [1, 2] {
        f.chat.act(
            &fixtures::env(&fixtures::limits()),
            &f.client,
            Act::ChangeProposal { proposal: PROPOSAL },
            &mut { SERIAL },
            &mut f.out,
        );
        assert_eq!(f.chat.composer_focus, count);
        assert_eq!(f.chat.change_asked, Some(PROPOSAL));
        assert!(fixtures::acts::sent(&mut f.out).is_empty());
    }
}
#[test]
fn a_sent_message_ends_the_change_asked() {
    let mut f = draft_ready();
    f.chat.change_asked = Some(PROPOSAL);
    f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::Send, &mut { SERIAL }, &mut f.out);
    assert!(fixtures::acts::sent(&mut f.out).is_empty());
    assert_eq!(f.chat.change_asked, Some(PROPOSAL), "empty Send keeps the request");
    f.chat.words = b"Wait three seconds instead".as_slice().into();
    f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::Send, &mut { SERIAL }, &mut f.out);
    assert_eq!(fixtures::acts::sent(&mut f.out).len(), 1);
    assert_eq!(f.chat.change_asked, None, "a message went out");
}
#[test]
fn a_decision_ends_the_change_asked() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL, DELEGATE_PROPOSAL]);
    f.chat.change_asked = Some(PROPOSAL);
    proposals(&mut f, &[PROPOSAL, DELEGATE_PROPOSAL]);
    assert_eq!(f.chat.change_asked, Some(PROPOSAL), "a refresh keeps the request");
    proposals(&mut f, &[PROPOSAL]);
    assert_eq!(f.chat.change_asked, Some(PROPOSAL), "another proposal's decision keeps the request");
    proposals(&mut f, &[]);
    assert_eq!(f.chat.change_asked, None, "the task document reports the proposal decided");
}

#[test]
fn a_live_run_is_watched_and_dropped_when_it_ends() {
    let mut f = f::opened();
    let run = f.chat.run.expect("live run watched");
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Watch {
            handle: run,
            subject: Watchable::Jig(c::Subject::Run { task: f::TASK, turn: None, offset: EMPTY })
        }))
    );
    let mut task = f::task();
    f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(f.chat.run, None);
    assert_eq!(f::handles(&mut f.out).as_ref(), [run]);
}
#[test]
fn leaving_a_chat_unwatches_its_task_activity_and_run() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f.chat.leave(&mut f.out);
    assert_eq!(
        f::handles(&mut f.out).as_ref(),
        [f.chat.document, f.chat.activity, draft_handle(&f.chat), f.chat.run.expect("run")]
    );
}
fn open_run(f: &mut f::Fixture) -> skein_lib::Token {
    let mut emitted = Queue::with_capacity(OUTPUTS);
    while let Some(Request::Client(event)) = f.out.pop() {
        f.client.step_unlinked(Time::ZERO, skein_lib::Wall::EPOCH, event, &mut emitted);
    }
    while let Some(request) = emitted.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Run { .. }) },
        } = request
        {
            return watch;
        }
    }
    panic!("run opened")
}
fn run_text(from: u32, bytes: &[u8]) -> c::RunText {
    let length = u32::try_from(bytes.len()).unwrap();
    c::RunText {
        writing: None,
        attempt: f::ATTEMPT,
        began: Some(skein_lib::Wall::EPOCH),
        turn: c::ProvisionalTurnName { attempt: f::ATTEMPT, number: u32::try_from(f::POSITION).unwrap(), retry: 0 },
        from,
        through: from.checked_add(length).unwrap(),
        text: c::Text::new(Box::from(bytes), length).unwrap(),
        cut: false,
        call: None,
    }
}
fn run_delivery(f: &mut f::Fixture, watch: skein_lib::Token, delivery: c::Delivery) {
    f.client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Up { up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) } },
        &mut Queue::with_capacity(OUTPUTS),
    );
    let handle = f.chat.run.expect("run watched");
    for told in [Told::Changed { handle }, Told::Text { handle }] {
        assert!(f.chat.told(Time::ZERO, &f.client, told, &mut f.out));
    }
}
fn run_state(writing: Option<c::RunText>) -> c::Delivery {
    c::Delivery::RunState(Box::new(c::RunState {
        task: f::TASK,
        attempt: Some(f::ATTEMPT),
        committed_turn: 0,
        writing,
        call: None,
    }))
}
#[test]
fn a_chats_stream_suffix_extends_its_provisional_turn_until_committed() {
    let mut f = f::opened();
    let watch = open_run(&mut f);
    run_delivery(&mut f, watch, c::Delivery::Run(Box::new(run_text(0, b"The answer"))));
    assert_eq!(f.chat.thread.live.get(0).expect("provisional prefix").text.bytes(), b"The answer");
    run_delivery(&mut f, watch, c::Delivery::Run(Box::new(run_text(10, b" is forty-two."))));
    let handle = f.chat.run.unwrap();
    assert_eq!(f.client.text(handle).unwrap().text.bytes(), b"The answer is forty-two.");
    assert_eq!(f.chat.thread.live.get(0).expect("whole provisional turn").text.bytes(), b"The answer is forty-two.");

    let mut blocks = skein_lib::List::with_capacity(1);
    blocks.push(c::Block::Text(fixtures::acts::text(b"The answer is forty-two."))).unwrap();
    f::activity_blocks(&mut f, c::Speaker::Assistant, blocks, f::POSITION);
    assert!(f.chat.thread.live.is_empty(), "the committed turn replaces the provisional turn");
    let lines = f.chat.thread.lines();
    let [jig_client::thread::Line::Turn { turn, .. }] = lines.as_slice() else { panic!("one committed turn") };
    assert_eq!(
        turn.messages.get(0).unwrap().blocks.as_slice(),
        [c::Block::Text(fixtures::acts::text(b"The answer is forty-two."))]
    );
}
#[test]
fn a_chats_first_stream_text_follows_a_state_without_writing() {
    let mut f = f::opened();
    let watch = open_run(&mut f);
    run_delivery(&mut f, watch, run_state(None));
    assert!(f.chat.thread.live.is_empty());
    run_delivery(&mut f, watch, c::Delivery::Run(Box::new(run_text(0, b"The answer"))));
    assert_eq!(f.chat.thread.live.get(0).expect("first provisional turn").text.bytes(), b"The answer");
}
#[test]
fn a_chats_stream_can_arrive_before_its_first_run_state() {
    let mut f = f::opened();
    let watch = open_run(&mut f);
    run_delivery(&mut f, watch, c::Delivery::Run(Box::new(run_text(0, b"The answer"))));
    assert_eq!(f.chat.thread.attempt, Some(f::ATTEMPT));
    assert_eq!(f.chat.thread.live.get(0).expect("first provisional turn").text.bytes(), b"The answer");
}
#[test]
fn stop_sends_one_stop_and_shows_it_pending_until_answered() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let mut serial = SERIAL;
    for _ in [false, true] {
        f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::Stop, &mut serial, &mut f.out);
    }
    assert_eq!(fixtures::acts::sent(&mut f.out).as_ref(), [c::Ask::Stop { project: fixtures::PROJECT, task: f::TASK }]);
    assert_eq!(
        f.chat.pending.as_slice(),
        [(jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: SERIAL + 1 }, Sending::Stop)]
    );
    let key = f.chat.pending.as_slice()[0].0;
    f.chat.told(Time::ZERO, &f.client, Told::Answered { key, outcome: jig_client::Outcome::Unknown }, &mut f.out);
    assert!(f.chat.pending.is_empty());
    assert_eq!(f.chat.outcome, Some(jig_client::Outcome::Unknown));
}
fn chat_pages() -> (crate::pages::Pages, jig_client::Client) {
    let mut pages = crate::pages::Pages::new();
    let client = fixtures::acts::live_client();
    crate::pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &client,
        crate::pages::Event::Went { address: crate::Address::Chat { task: f::TASK } },
        &mut Queue::with_capacity(OUTPUTS),
    );
    (pages, client)
}
fn chat_told(pages: &mut crate::pages::Pages, client: &jig_client::Client, told: Told) -> Queue<Request> {
    let mut out = Queue::with_capacity(OUTPUTS);
    crate::pages::step(
        pages,
        &fixtures::env(&fixtures::limits()),
        client,
        crate::pages::Event::Told { told },
        &mut out,
    );
    out
}
fn owner_activity(position: u64, line: c::Activity) -> c::DecodedActivityLine {
    c::DecodedActivityLine {
        source: Box::new(c::ActivityLine { task: f::TASK, position, at: skein_lib::Wall::EPOCH, line }),
        decoded: None,
    }
}
fn owner_turn(position: u64, reply: u64) -> c::DecodedActivityLine {
    owner_activity(
        position,
        c::Activity::Turn { reply, attempt: f::ATTEMPT, turn: u32::try_from(reply).unwrap(), read: None },
    )
}
fn chat_move(
    pages: &mut crate::pages::Pages,
    client: &jig_client::Client,
    position: u64,
    reply: u64,
) -> Queue<Request> {
    let mut rows = skein_lib::List::with_capacity(1);
    rows.push(owner_turn(position, reply)).unwrap();
    chat_activity(pages, client, position, rows)
}
fn chat_activity(
    pages: &mut crate::pages::Pages,
    client: &jig_client::Client,
    position: u64,
    rows: skein_lib::List<c::DecodedActivityLine>,
) -> Queue<Request> {
    let mut task = f::task();
    task.activity = position;
    let document = pages.chat().unwrap().document;
    chat_told(
        pages,
        client,
        Told::Page {
            handle: document,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))),
        },
    );
    chat_told(
        pages,
        client,
        Told::Page {
            handle: pages.chat().unwrap().activity,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                before: None,
                after: None,
                cut: false,
            })))),
        },
    )
}
fn seen_key(pages: &crate::pages::Pages) -> jig_client::Key {
    for (key, kind) in &pages.chat().unwrap().pending {
        if *kind == Sending::Seen {
            return *key;
        }
    }
    panic!("automatic Seen pending")
}
fn seen_answer() -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
        c::DocumentOutcome::Amended { task: 9 },
    ))))
}
fn seen_ask(through: u64) -> c::Ask {
    c::Ask::Seen { through: c::SeenThrough::Conversation { task: f::TASK, through } }
}
#[test]
fn a_chats_first_loaded_page_is_seen_automatically() {
    let (mut pages, client) = chat_pages();
    assert_eq!(
        fixtures::acts::sent(&mut chat_move(&mut pages, &client, f::POSITION, REPLY)).as_ref(),
        [seen_ask(REPLY)]
    );
}
#[test]
fn turns_arriving_while_a_chat_is_open_are_seen_through_the_newest() {
    let (mut pages, client) = chat_pages();
    chat_move(&mut pages, &client, f::POSITION, REPLY);
    let key = seen_key(&pages);
    chat_told(&mut pages, &client, Told::Answered { key, outcome: seen_answer() });
    assert_eq!(
        fixtures::acts::sent(&mut chat_move(&mut pages, &client, f::POSITION + 2, REPLY + 1)).as_ref(),
        [seen_ask(REPLY + 1)]
    );
}
#[test]
fn a_chat_waits_for_the_pending_seen_before_sending_the_newest_reply() {
    let (mut pages, client) = chat_pages();
    chat_move(&mut pages, &client, f::POSITION, REPLY);
    let key = seen_key(&pages);
    assert!(fixtures::acts::sent(&mut chat_move(&mut pages, &client, f::POSITION + 1, REPLY + 1)).is_empty());
    let mut out = chat_told(&mut pages, &client, Told::Answered { key, outcome: seen_answer() });
    assert_eq!(fixtures::acts::sent(&mut out).as_ref(), [seen_ask(REPLY + 1)]);
    assert_ne!(seen_key(&pages), key);
}
#[test]
fn a_refused_or_failed_chat_seen_is_resent_at_the_next_move() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Unsent,
        jig_client::Outcome::Lost,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NotOffered,
            at: None,
        })),
    ] {
        let (mut pages, client) = chat_pages();
        chat_move(&mut pages, &client, f::POSITION, REPLY);
        let key = seen_key(&pages);
        assert!(fixtures::acts::sent(&mut chat_told(&mut pages, &client, Told::Answered { key, outcome })).is_empty());
        assert_eq!(pages.chat().unwrap().seen, 0);
        // A fresh page delivery is a move even when it repeats the current tail.
        let handle = pages.chat().unwrap().document;
        let mut out = chat_told(
            &mut pages,
            &client,
            Told::Page {
                handle,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(f::task())))),
            },
        );
        assert_eq!(fixtures::acts::sent(&mut out).as_ref(), [seen_ask(REPLY)]);
        assert_ne!(seen_key(&pages), key);
    }
}
#[test]
fn a_chat_sends_no_seen_without_a_new_loaded_reply() {
    let (mut pages, client) = chat_pages();
    chat_move(&mut pages, &client, f::POSITION, REPLY);
    let key = seen_key(&pages);
    chat_told(&mut pages, &client, Told::Answered { key, outcome: seen_answer() });
    assert!(fixtures::acts::sent(&mut chat_move(&mut pages, &client, f::POSITION, REPLY)).is_empty());
    assert!(fixtures::acts::sent(&mut chat_move(&mut pages, &client, f::POSITION - 1, REPLY)).is_empty());
    assert_eq!(pages.chat().unwrap().seen, REPLY);
}
#[test]
fn a_chat_sees_reply_numbers_instead_of_activity_positions() {
    let (mut pages, client) = chat_pages();
    let mut rows = skein_lib::List::with_capacity(4);
    rows.push(owner_activity(1, c::Activity::RunBegan { attempt: f::ATTEMPT, why: c::RunWhy::Made, resumed: false }))
        .unwrap();
    rows.push(owner_activity(
        2,
        c::Activity::Words {
            message: 1,
            by: c::Requester::Person(fixtures::PARTY),
            kind: c::MessageKind::Words,
            words: fixtures::acts::text(b"Hello"),
        },
    ))
    .unwrap();
    rows.push(owner_turn(3, 1)).unwrap();
    rows.push(owner_activity(4, c::Activity::RunEnded { attempt: f::ATTEMPT, end: c::RunEnd::Parked, spent: 0 }))
        .unwrap();
    let mut out = chat_activity(&mut pages, &client, 4, rows);
    assert_eq!(pages.chat().unwrap().thread.loaded, 4);
    assert_eq!(fixtures::acts::sent(&mut out).as_ref(), [seen_ask(1)]);
    assert_eq!(pages.chat().unwrap().seen, 1);
    let key = seen_key(&pages);
    chat_told(&mut pages, &client, Told::Answered { key, outcome: seen_answer() });
    assert_eq!(fixtures::acts::sent(&mut chat_move(&mut pages, &client, 5, 2)).as_ref(), [seen_ask(2)]);
    assert_eq!(pages.chat().unwrap().seen, 2);
}
#[test]
fn a_chat_with_activity_but_no_numbered_owner_turn_sends_no_seen() {
    let (mut pages, client) = chat_pages();
    let mut rows = skein_lib::List::with_capacity(3);
    rows.push(owner_activity(1, c::Activity::RunBegan { attempt: f::ATTEMPT, why: c::RunWhy::Made, resumed: false }))
        .unwrap();
    rows.push(owner_activity(
        2,
        c::Activity::Words {
            message: 1,
            by: c::Requester::Person(fixtures::PARTY),
            kind: c::MessageKind::Words,
            words: fixtures::acts::text(b"Hello"),
        },
    ))
    .unwrap();
    rows.push(owner_activity(
        4,
        c::Activity::Kept { attempt: f::ATTEMPT, pending: skein_lib::List::with_capacity(0), question: None },
    ))
    .unwrap();
    assert!(fixtures::acts::sent(&mut chat_activity(&mut pages, &client, 4, rows)).is_empty());
    assert_eq!(pages.chat().unwrap().thread.loaded, 4);
    assert_eq!(pages.chat().unwrap().seen, 0);
    assert!(pages.chat().unwrap().pending.is_empty());
}
#[test]
fn automatic_seen_preserves_the_last_acts_outcome_on_send_and_answer() {
    let mut f = f::opened();
    let mut serial = SERIAL;
    f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::Stop, &mut serial, &mut f.out);
    let key = f.chat.pending.as_slice()[0].0;
    f.chat.told(Time::ZERO, &f.client, Told::Answered { key, outcome: jig_client::Outcome::Unknown }, &mut f.out);
    let mut rows = skein_lib::List::with_capacity(1);
    rows.push(owner_turn(f::POSITION, REPLY)).unwrap();
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle: f.chat.activity,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                before: None,
                after: None,
                cut: false,
            })))),
        },
        &mut f.out,
    );
    f.chat.see(&mut serial, &mut f.out);
    assert_eq!(f.chat.outcome, Some(jig_client::Outcome::Unknown));
    assert_eq!(f.chat.outcome_kind, Some(Sending::Stop));
    let key = f.chat.pending.as_slice()[0].0;
    f.chat.told(Time::ZERO, &f.client, Told::Answered { key, outcome: seen_answer() }, &mut f.out);
    assert_eq!(f.chat.outcome, Some(jig_client::Outcome::Unknown));
    assert_eq!(f.chat.outcome_kind, Some(Sending::Stop));
}
fn root_event(root: &mut crate::Domain, event: crate::Event, out: &mut Queue<crate::Request>) {
    crate::step(root, &fixtures::env(&fixtures::limits()), event, out);
}
fn chat_with_loaded_turn() -> (crate::Domain, Queue<crate::Request>) {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let me = fixtures::start(&mut root, skein_lib::List::with_capacity(0));
    let mut out = Queue::uncapped();
    fixtures::confirm(&mut root, me, fixtures::PARTY, &mut out);
    root_event(
        &mut root,
        crate::Event::Went { place: crate::Place::Address(crate::Address::Chat { task: f::TASK }) },
        &mut out,
    );
    let mut watch = None;
    while let Some(request) = out.pop() {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Task { .. }) },
                },
        } = request
        {
            watch = Some(token);
        }
    }
    let mut task = f::task();
    task.acts = fixtures::acts::offered(c::ActKind::Message, c::Admission::Allowed);
    root_event(
        &mut root,
        crate::Event::Client {
            event: Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("chat task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut rows = skein_lib::List::with_capacity(1);
    rows.push(owner_turn(f::POSITION, REPLY)).unwrap();
    let handle = root.pages().chat().unwrap().activity;
    crate::route::told(
        &mut root,
        &fixtures::env(&limits),
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                before: None,
                after: None,
                cut: false,
            })))),
        },
        &mut out,
    );
    (root, out)
}
#[test]
fn an_automatic_seen_and_a_message_can_both_be_pending_and_answered() {
    const REQUESTS: u32 = 2;
    let (mut root, mut out) = chat_with_loaded_turn();
    for act in [Act::MessageWords { words: Box::from(*b"Hello") }, Act::Send] {
        root_event(&mut root, crate::Event::Act { action: crate::Action::Pages(act) }, &mut out);
    }
    let pending = root.pages().chat().unwrap().pending.clone();
    let mut kinds = skein_lib::List::with_capacity(REQUESTS);
    for (_, kind) in &pending {
        kinds.push(*kind).unwrap();
    }
    assert_eq!(kinds.as_slice(), [Sending::Seen, Sending::Message]);
    for (key, _) in &pending {
        assert!(root.client().pending(*key).is_some(), "both requests admitted to the real table");
    }
    root_event(&mut root, crate::Event::Client { event: Event::Stored }, &mut out);
    let mut attempts = skein_lib::List::with_capacity(REQUESTS);
    let mut asks = skein_lib::List::with_capacity(REQUESTS);
    while let Some(request) = out.pop() {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
        {
            attempts.push(attempt).unwrap();
            asks.push(*ask).unwrap();
        }
    }
    assert_eq!(
        asks.as_slice(),
        [
            seen_ask(REPLY),
            c::Ask::Message { project: fixtures::PROJECT, task: f::TASK, words: fixtures::acts::text(b"Hello") },
        ]
    );
    for (attempt, (_, kind)) in attempts.iter().zip(&pending) {
        let answer = match kind {
            Sending::Seen => c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Amended { task: 9 })),
            Sending::Message => {
                c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Said { task: f::TASK, message: 1 }))
            }
            Sending::Stop | Sending::Resume | Sending::Close => panic!("only Seen and Message pending"),
        };
        root_event(
            &mut root,
            crate::Event::Client {
                event: Event::Up {
                    up: jig_client::Up::Answer {
                        attempt: *attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(answer)),
                        why: None,
                    },
                },
            },
            &mut out,
        );
    }
    assert!(root.pages().chat().unwrap().pending.is_empty());
    for (key, _) in &pending {
        assert!(root.client().pending(*key).is_none(), "both requests answered");
    }
    assert_eq!(root.pages().chat().unwrap().seen, REPLY);
    assert_eq!(root.pages().chat().unwrap().outcome_kind, Some(Sending::Message));
}

#[test]
fn closing_a_chat_with_a_live_delegate_is_one_compound_request() {
    let mut f = f::with_delegate();
    f.chat.prepare_close(&f.client);
    assert!(fixtures::acts::sent(&mut f.out).is_empty(), "said before sent");
    f.chat.delegate_fate(f::DELEGATE, jig_client::shapes::Fate::Move);
    let mut serial = SERIAL;
    f.chat.close(&mut serial, &mut f.out);
    f.chat.close(&mut serial, &mut f.out);
    let expected = jig_client::shapes::close_chat(
        &fixtures::chats::me(),
        fixtures::PROJECT,
        f::TASK,
        &[jig_client::shapes::Delegate { task: f::DELEGATE, fate: jig_client::shapes::Fate::Move }],
        fixtures::acts::text(b"Chat closed"),
    )
    .expect("chat shape");
    assert_eq!(fixtures::acts::sent(&mut f.out).as_ref(), [expected], "jig's compound, once");
}
#[test]
fn the_confirmed_close_keeps_the_delegate_fates_and_grace_shown() {
    let mut f = f::with_delegate();
    f.chat.prepare_close(&f.client);
    let shown = f.chat.closing.clone().expect("confirmation request");
    let mut changed = f::task();
    changed.acts = fixtures::acts::offered(c::ActKind::Cancel, c::Admission::Allowed);
    f::deliver(&mut f.client, f.watch, changed);
    let mut serial = SERIAL;
    f.chat.close(&mut serial, &mut f.out);
    assert_eq!(fixtures::acts::sent(&mut f.out).as_ref(), [shown], "confirm the shown request");
}

const FIRST_CARD: u32 = 0;
const SECOND_CARD: u32 = 1;
const PROPOSAL: c::ProposalKey = fixtures::acts::PROPOSAL;
const DELEGATE_PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(f::DELEGATE), number: 4 };
const PROPOSALS: u32 = 2;
fn proposals(f: &mut f::Fixture, keys: &[c::ProposalKey]) {
    let mut task = f::task();
    task.proposals = skein_lib::List::with_capacity(PROPOSALS);
    for key in keys {
        task.proposals.push(*key).expect("proposals fit");
    }
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
}
fn reads(out: &mut Queue<Request>) -> Box<[(jig_client::Handle, c::ProposalKey)]> {
    let mut reads = skein_lib::List::with_capacity(PROPOSALS);
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Read { handle, read: Readable::Jig(c::Read::Proposal { proposal }) }) = request {
            reads.push((handle, proposal)).expect("bounded reads");
        }
    }
    reads.into_boxed()
}
#[test]
fn a_chats_proposals_are_read_once_and_keep_their_handles() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL, DELEGATE_PROPOSAL]);
    let first = reads(&mut f.out);
    assert_eq!(first.len(), 2);
    assert_eq!([first[0].1, first[1].1], [PROPOSAL, DELEGATE_PROPOSAL]);
    proposals(&mut f, &[DELEGATE_PROPOSAL, PROPOSAL]);
    assert!(reads(&mut f.out).is_empty(), "a retained proposal is not read again");
    assert_eq!(
        [f.chat.cards.get(FIRST_CARD).expect("first").handle, f.chat.cards.get(SECOND_CARD).expect("second").handle],
        [first[0].0, first[1].0],
        "retained handles follow proposal number order"
    );
}
#[test]
fn a_proposal_decided_elsewhere_leaves_the_chats_card() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    let handle = reads(&mut f.out)[0].0;
    card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
    proposals(&mut f, &[]);
    assert!(f.chat.cards.is_empty());
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Unwatch { handle })),
        "release the card's read capacity before reading its decision"
    );
    assert_eq!(reads(&mut f.out).len(), 1, "the departed proposal is read after closing the old handle");
    assert_eq!(f.chat.next_deadline(), None, "a removed card has no retry");
    f.chat.fire(Time::ZERO.saturating_add(Duration::from_secs(30)), &mut f.out);
    assert!(reads(&mut f.out).is_empty(), "a removed card is not read again");
}
fn rejecting_pages() -> crate::pages::Pages {
    let mut pages = crate::pages::Pages::new();
    crate::pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        crate::unlinked(&fixtures::limits(), fixtures::SEED).client(),
        crate::pages::Event::Went { address: crate::Address::Chat { task: f::TASK } },
        &mut Queue::with_capacity(OUTPUTS),
    );
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: f::FIRST };
    pages
        .cards_mut()
        .expect("chat cards")
        .push(crate::pages::inbox::Card {
            whole: None,
            object: c::ObjectKey::Proposal(PROPOSAL),
            group: c::InboxGroup::Proposals,
            project: Some(fixtures::PROJECT),
            position: None,
            unread: None,
            handle,
            page: Some(fixtures::acts::proposal(c::ActKind::Reject, c::Admission::Allowed)),
            words: None,
            key: None,
            outcome: None,
        })
        .expect("card fits");
    pages
}
#[test]
fn rejecting_a_proposal_in_place_is_said_before_it_is_sent() {
    let mut pages = rejecting_pages();
    let object = c::ObjectKey::Proposal(PROPOSAL);
    fixtures::acts::act(&mut pages, Act::CardWords { object, words: Box::from(*b"Too costly") });
    let mut said = fixtures::acts::act(&mut pages, fixtures::acts::decide(object, c::ActKind::Reject));
    assert!(fixtures::acts::sent(&mut said).is_empty());
    assert_eq!(pages.confirmation().expect("confirmation").act, c::ActKind::Reject);
    assert_eq!(pages.chat().expect("chat").cards.get(FIRST_CARD).expect("card").key, None);
    assert_eq!(
        fixtures::acts::sent(&mut fixtures::acts::act(&mut pages, Act::Confirm)).as_ref(),
        [c::Ask::Decide {
            project: fixtures::PROJECT,
            on: c::DecideOn::Proposal {
                proposal: PROPOSAL,
                running: None,
                decision: c::Decision::Rejected { reason: fixtures::acts::text(b"Too costly") }
            }
        }]
    );
}

#[test]
fn a_departed_proposal_keeps_its_identity_for_the_explanation() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    proposals(&mut f, &[]);
    assert_eq!(f.chat.departures(), &[c::ObjectKey::Proposal(PROPOSAL)]);
}

const DELEGATED: u64 = 2;
fn live_delegate() -> c::Task {
    let mut task = f::task();
    task.delegates = skein_lib::List::with_capacity(1);
    let mut chip = task.chip.clone();
    chip.task = DELEGATED;
    task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).expect("live delegate");
    task
}
#[test]
fn a_live_delegate_is_watched_before_its_call_names_it() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: f.chat.next };
    f::deliver(&mut f.client, f.watch, live_delegate());
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(watches(&mut f.out).as_ref(), [(handle, DELEGATED)]);
}
#[test]
fn a_delegate_that_ends_is_unwatched_once_nothing_names_it() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f::deliver(&mut f.client, f.watch, live_delegate());
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    let handle = watches(&mut f.out)[0].0;
    let mut task = f::task();
    task.delegates = skein_lib::List::with_capacity(0);
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(f::handles(&mut f.out).as_ref(), [handle]);
}
#[test]
fn a_chat_watches_a_delegated_task_when_only_its_result_remains() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let next = f.chat.next;
    let mut blocks = skein_lib::List::with_capacity(1);
    blocks
        .push(c::Block::Result {
            call: 8,
            summary: c::Text::new(Box::from(f::WAKE_SUMMARY), 128).expect("wake summary"),
            failed: false,
        })
        .expect("wake result");
    f::activity_blocks(&mut f, c::Speaker::Person, blocks, f::POSITION);
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: next };
    assert_eq!(watches(&mut f.out).as_ref(), [(handle, DELEGATED)]);
}

fn watches(out: &mut Queue<Request>) -> Box<[(jig_client::Handle, u64)]> {
    let mut watches = skein_lib::List::with_capacity(OUTPUTS);
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task }) }) = request {
            watches.push((handle, task)).expect("bounded watches");
        }
    }
    watches.into_boxed()
}
#[test]
fn a_chat_watches_each_delegated_task_as_its_call_arrives() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let next = f.chat.next;
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: next };
    assert_eq!(watches(&mut f.out).as_ref(), [(handle, DELEGATED)], "fresh handle watches the delegated task");
    assert!(f.chat.delegation(&f.client, DELEGATED).is_none(), "wait for its document");
    let mut emitted = Queue::with_capacity(OUTPUTS);
    f.client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task: DELEGATED }) },
        &mut emitted,
    );
    let mut watch = None;
    while let Some(request) = emitted.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Task { task: DELEGATED }) },
        } = request
        {
            watch = Some(token);
        }
    }
    let mut task = f::change();
    task.chip.task = DELEGATED;
    f::deliver(&mut f.client, watch.expect("delegation opened"), task.clone());
    assert!(f.chat.told(Time::ZERO, &f.client, Told::Changed { handle }, &mut f.out));
    assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task));
    assert!(f.out.is_empty(), "a document refresh leaves its watch alone");
    assert!(f.chat.delegation(&f.client, 999).is_none(), "only held delegations are exposed");
}
#[test]
fn a_delegation_requested_by_another_owner_is_not_this_chats() {
    let mut f = f::opened();
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    for (requester, requested) in [(c::Requester::Person(DELEGATED), false), (c::Requester::Task(f::TASK), true)] {
        let mut task = f::change();
        task.chip.task = DELEGATED;
        task.requester = requester;
        f.chat.delegations.get_mut(0).expect("delegation").final_task = Some(Box::new(task.clone()));
        assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task));
        assert_eq!(f.chat.requested(&f.client, DELEGATED), if requested { Some(&task) } else { None });
    }
}
#[test]
fn a_chat_watches_the_newest_delegations_up_to_its_bound() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let bound = crate::pages::delegation::bound(&fixtures::limits());
    let mut tasks = skein_lib::List::with_capacity(bound + 1);
    for task in 1..=u64::from(bound + 1) {
        tasks.push(task).expect("calls");
    }
    f::activity(&mut f, tasks.as_slice(), f::POSITION);
    let watched = watches(&mut f.out);
    assert_eq!(watched.len(), usize::try_from(bound).expect("bound fits"));
    for (index, (_, task)) in watched.iter().enumerate() {
        assert_eq!(*task, u64::from(bound + 1) - u64::try_from(index).expect("bounded index"));
    }
    for delegation in &f.chat.delegations {
        assert_ne!(delegation.task, 1, "oldest is past the bound");
    }
}
#[test]
fn a_delegation_is_unwatched_as_its_call_leaves_the_thread() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    let handle = watches(&mut f.out)[0].0;
    f::activity(&mut f, &[], f::POSITION + 1);
    assert_eq!(f::handles(&mut f.out).as_ref(), [handle]);
    assert!(f.chat.delegations.is_empty());
    assert!(f.chat.delegation(&f.client, DELEGATED).is_none());
}
#[test]
fn a_newer_delegation_unwatches_the_one_past_the_bound() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let bound = crate::pages::delegation::bound(&fixtures::limits());
    let mut tasks = skein_lib::List::with_capacity(bound + 1);
    for task in 1..=u64::from(bound) {
        tasks.push(task).expect("calls");
    }
    f::activity(&mut f, tasks.as_slice(), f::POSITION);
    let old = watches(&mut f.out);
    let oldest = old.last().expect("oldest").0;
    tasks.push(u64::from(bound + 1)).expect("new call");
    f::activity(&mut f, tasks.as_slice(), f::POSITION + 1);
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Read {
            handle: f.chat.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None })
        }))
    );
    assert_eq!(f.out.pop(), Some(Request::Client(Event::Unwatch { handle: oldest })), "make watch room first");
    let new = watches(&mut f.out);
    assert_eq!(new.len(), 1);
    assert_eq!(new[0].1, u64::from(bound + 1));
    for (delegation, (handle, task)) in f.chat.delegations.iter().skip(1).zip(old.iter()) {
        assert_eq!((delegation.handle, delegation.task), (*handle, *task), "retained handles stay the same");
    }
}
#[test]
fn leaving_a_chat_unwatches_its_delegations() {
    let mut f = f::with_delegate();
    f::activity(&mut f, &[DELEGATED, 3], f::POSITION);
    let watched = watches(&mut f.out);
    f.chat.leave(&mut f.out);
    let left = f::handles(&mut f.out);
    assert_eq!(left.len(), 6, "document, activity, draft read, run and both delegations");
    assert_eq!(&left[4..], [watched[0].0, watched[1].0]);
}
#[test]
fn an_ended_delegation_reads_its_task_and_keeps_its_result() {
    let mut f = f::opened();
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    let handle = watches(&mut f.out)[0].0;
    for why in [jig_client::Why::SubjectEnded, jig_client::Why::Unknown] {
        assert!(f.chat.told(Time::ZERO, &f.client, Told::Ended { handle, why }, &mut f.out));
        assert_eq!(
            f.out.pop(),
            Some(Request::Client(Event::Read { handle, read: Readable::Jig(c::Read::Task { task: DELEGATED }) }))
        );
        assert!(f.out.is_empty());
    }
    let mut task = f::change();
    task.chip.task = DELEGATED;
    let result = f::result(&task, c::Ending::Done, Some(f::change_value(b"abcdef0123456789")));
    let ended = c::Ended {
        key: c::ObjectKey::Task(DELEGATED),
        value: c::EndedValue::Task { document: Box::new(task.clone()), result: Box::new(result.clone()) },
        at: None,
        by: None,
    };
    assert!(f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(Box::new(ended))))
        },
        &mut f.out
    ));
    task.result = Some(c::TaskResult::Document(Box::new(result)));
    assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task));
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task), "refresh keeps the final task");
    assert!(f.out.is_empty(), "no new requests for retained calls");
}

#[test]
fn the_chat_composer_holds_past_the_cue_bound() {
    let mut limits = fixtures::limits();
    limits.client.domain.text = 2048;
    limits.client.domain.long_text = 2048;
    limits.client.domain.turn_text = 2048;
    let mut f = f::opened();
    let mut serial = SERIAL;
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).unwrap();
    let mut words = skein_lib::bytes::zeroed(bound);
    words.fill(b'a');
    let mut over = skein_lib::bytes::zeroed(bound.checked_add(1).unwrap());
    over.fill(b'b');
    f.chat.act(&fixtures::env(&limits), &f.client, Act::MessageWords { words: words.clone() }, &mut serial, &mut f.out);
    assert_eq!(f.chat.words, words);
    f.chat.act(&fixtures::env(&limits), &f.client, Act::MessageWords { words: over.clone() }, &mut serial, &mut f.out);
    assert_eq!(f.chat.words, over, "an oversized edit replaces the previous words");
}

fn window_page(chat: &mut Chat, client: &jig_client::Client, out: &mut Queue<Request>, page: u32, pages: u32) {
    chat.told(
        Time::ZERO,
        client,
        Told::Page { handle: chat.activity, page: activity_answer(window_activity(page, pages)) },
        out,
    );
}
fn window_activity(page: u32, pages: u32) -> c::ActivityPage {
    let mut rows = skein_lib::List::with_capacity(WINDOW_PAGE_ROWS);
    for offset in 0..WINDOW_PAGE_ROWS {
        rows.push(crate::tests::documents::line(
            u64::from(
                pages.checked_sub(page).unwrap().checked_mul(WINDOW_PAGE_ROWS).unwrap().checked_sub(offset).unwrap(),
            ),
            skein_lib::Wall::EPOCH,
            c::Activity::RunBegan { attempt: 1, why: c::RunWhy::Made, resumed: false },
        ))
        .unwrap();
    }
    let cursor = c::Cursor::new(Box::from(*b"older"), 5).unwrap();
    c::ActivityPage {
        rows,
        before: if page.checked_add(1).unwrap() < pages { Some(cursor) } else { None },
        after: None,
        cut: false,
    }
}
#[test]
fn a_full_first_page_waits_for_the_persons_next_read() {
    let mut limits = fixtures::limits();
    limits.client.domain.document_bytes = u64::from(WINDOW_PAGE_ROWS * c::LINE_ROOM);
    let (mut chat, client, mut out) = reading_chat(&limits);
    window_page(&mut chat, &client, &mut out, 0, WINDOW_PAGES);
    assert!(out.is_empty(), "a page with rows waits for the person's next read");
    assert_eq!(chat.thread.lines.rows.len(), WINDOW_PAGE_ROWS);
    assert!(chat.thread.reading.is_none());
}
#[test]
fn initial_history_stops_when_the_cursor_does_not_advance() {
    let (mut chat, client, mut out) = reading_chat(&fixtures::limits());
    window_page(&mut chat, &client, &mut out, 0, WINDOW_PAGES);
    assert!(out.pop().is_some(), "initial history asks for the older page");
    window_page(&mut chat, &client, &mut out, 1, WINDOW_PAGES);
    assert!(out.is_empty(), "the repeated cursor stops automatic reading");
    assert!(chat.thread.reading.is_none());
    chat_act(&mut chat, &client, Act::Older, &mut out);
    assert!(out.pop().is_some(), "a person can still ask for older history");
}

#[test]
fn initial_history_keeps_the_newest_page_when_the_next_will_not_fit() {
    for newer in [false, true] {
        let mut limits = fixtures::limits();
        limits.client.domain.document_bytes = u64::from(3 * c::LINE_ROOM);
        let (mut chat, client, mut out) = reading_chat(&limits);
        window_page(&mut chat, &client, &mut out, 0, WINDOW_PAGES);
        assert!(out.pop().is_some(), "there is room to try an older page");
        let rows = chat.thread.lines.rows.clone();
        let edges = chat.thread.lines.edges.clone();
        if newer {
            chat.thread.committed = chat.thread.loaded.checked_add(1).unwrap();
        }
        window_page(&mut chat, &client, &mut out, 1, WINDOW_PAGES);
        assert_eq!(chat.thread.lines.rows, rows, "initial filling keeps the recent page whole");
        assert_eq!(chat.thread.lines.edges, edges, "the unread page keeps its cursor");
        let expected = if newer { Some(first_activity(&chat)) } else { None };
        assert_eq!(out.pop(), expected, "a newer announced reply still refreshes at the window limit");
        assert!(out.is_empty());
        assert_eq!(chat.thread.reading.is_some(), newer);
    }
}

#[test]
fn an_empty_opening_page_keeps_an_announced_tail_read() {
    for announced in [true, false] {
        let (mut chat, client, mut out) = reading_chat(&fixtures::limits());
        if announced {
            chat.thread.committed = 3;
        }
        let mut page = empty_activity();
        page.before = None;
        chat.told(Time::ZERO, &client, Told::Page { handle: chat.activity, page: activity_answer(page) }, &mut out);
        let expected = if announced { Some(first_activity(&chat)) } else { None };
        assert_eq!(out.pop(), expected, "a stale empty page still reads the announced tail");
        assert!(out.is_empty());
        if !announced {
            assert_eq!(chat.thread.moved(3), Some(c::Read::Activity { task: f::TASK, before: None }));
        }
        let cursor = c::Cursor::new(b"opening".as_slice().into(), 7).unwrap();
        chat.told(
            Time::ZERO,
            &client,
            Told::Page {
                handle: chat.activity,
                page: activity_answer(completed_activity(Some(cursor.clone()), FAST_REPLY)),
            },
            &mut out,
        );
        assert_eq!(
            out.pop(),
            Some(Request::Client(Event::Read {
                handle: chat.activity,
                read: Readable::Jig(c::Read::Activity { task: f::TASK, before: Some(cursor) }),
            })),
            "the first committed history still fills back to its opening"
        );
        window_page(&mut chat, &client, &mut out, 0, 1);
        assert_eq!(chat.thread.loaded, 3);
        assert!(chat.thread.reading.is_none());
        assert!(out.is_empty());
    }
}

#[test]
fn initial_empty_history_stops_at_its_page_budget() {
    let mut limits = fixtures::limits();
    limits.client.domain.document_bytes = u64::from(2 * c::LINE_ROOM);
    let (mut chat, client, mut out) = reading_chat(&limits);
    chat.thread.committed = 3;
    for (cursor, reads) in [(b"first".as_slice(), true), (b"second".as_slice(), false)] {
        let mut page = empty_activity();
        page.before = Some(c::Cursor::new(cursor.into(), 6).unwrap());
        chat.told(Time::ZERO, &client, Told::Page { handle: chat.activity, page: activity_answer(page) }, &mut out);
        assert_eq!(out.pop().is_some(), reads, "an empty page spends the initial read budget");
        assert!(out.is_empty());
    }
    assert!(chat.thread.reading.is_none());
}

#[test]
fn a_named_window_bounds_the_thread_and_its_lines() {
    const WINDOW_BYTES: u64 = 131_072;
    let mut limits = fixtures::limits();
    limits.client.domain.document_bytes = WINDOW_BYTES * 2;
    limits.client.domain.long_text = 64;
    limits.window_bytes = Some(WINDOW_BYTES);
    let window = crate::pages::chat::thread_limits(&limits);
    assert_eq!(window.bytes, WINDOW_BYTES);
    assert_eq!(window.text, limits.client.domain.long_text);
    assert_eq!(u64::from(window.lines), WINDOW_BYTES / u64::from(c::LINE_ROOM));

    limits.window_bytes = None;
    let whole = crate::pages::chat::thread_limits(&limits);
    assert_eq!(whole.bytes, limits.client.domain.document_bytes);
    assert_eq!(whole.text, limits.client.domain.long_text);
    assert_eq!(u64::from(whole.lines), limits.client.domain.document_bytes / u64::from(c::LINE_ROOM));
}
#[test]
fn a_window_is_refused_outside_long_text_and_document_bytes() {
    let mut limits = fixtures::limits();
    let least = u64::from(limits.client.domain.long_text);
    let most = limits.client.domain.document_bytes;
    for edge in [least, most] {
        limits.window_bytes = Some(edge);
        assert!(crate::valid(&limits), "window edge {edge} is admitted");
    }
    for outside in [least - 1, most + 1] {
        limits.window_bytes = Some(outside);
        assert!(!crate::valid(&limits), "window outside {outside} is refused");
    }
}
#[test]
fn a_chats_window_holds_more_than_sixteen_lines() {
    // The fixture grants 65536 document bytes; each line costs at least 64.
    const DOCUMENT_LINES: u32 = 1024;
    let limits = fixtures::limits();
    assert_eq!(crate::pages::chat::thread_limits(&limits).lines, DOCUMENT_LINES);
    let mut saturated = fixtures::limits();
    saturated.client.domain.document_bytes = u64::MAX;
    assert_eq!(crate::pages::chat::thread_limits(&saturated).lines, u32::MAX);
    let (mut chat, client, mut out) = reading_chat(&limits);
    for page in 0..WINDOW_PAGES {
        if page > 0 {
            chat.act(&fixtures::env(&limits), &client, Act::Older, &mut { SERIAL }, &mut out);
            while out.pop().is_some() {}
        }
        window_page(&mut chat, &client, &mut out, page, WINDOW_PAGES);
        while out.pop().is_some() {}
    }
    assert_eq!(chat.thread.lines.rows.len(), WINDOW_PAGES * WINDOW_PAGE_ROWS);
}

#[test]
fn a_change_call_names_its_task_in_the_chats_delegations() {
    let mut call = f::delegate_call(DELEGATED);
    call.tool = fixtures::acts::text(b"change");
    let mut delegated = skein_lib::List::with_capacity(2);
    crate::pages::delegation::delegated(&call, &mut delegated);
    assert_eq!(delegated.as_slice(), &[DELEGATED]);
}

#[test]
fn a_change_call_without_objects_reads_the_delegated_numbers() {
    let mut call = f::delegate_call(DELEGATED);
    call.tool = fixtures::acts::text(b"change");
    call.objects = skein_lib::List::with_capacity(0);
    let mut delegated = skein_lib::List::with_capacity(2);
    crate::pages::delegation::delegated(&call, &mut delegated);
    assert_eq!(delegated.as_slice(), &[2, 3]);
}

#[test]
fn a_goal_call_names_its_task_in_the_chats_delegations() {
    let mut call = f::delegate_call(DELEGATED);
    call.tool = fixtures::acts::text(b"goal");
    let mut delegated = skein_lib::List::with_capacity(2);
    crate::pages::delegation::delegated(&call, &mut delegated);
    assert_eq!(delegated.as_slice(), &[DELEGATED]);
}

#[test]
fn a_goal_call_without_objects_reads_the_delegated_numbers() {
    let mut call = f::delegate_call(DELEGATED);
    call.tool = fixtures::acts::text(b"goal");
    call.objects = skein_lib::List::with_capacity(0);
    let mut delegated = skein_lib::List::with_capacity(2);
    crate::pages::delegation::delegated(&call, &mut delegated);
    assert_eq!(delegated.as_slice(), &[2, 3]);
}

// The delivery announces the fourth activity while the first page is pending.
const COMMITTED: u64 = 4;
fn pending_activity() -> f::Fixture {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let mut task = f::task();
    task.activity = COMMITTED;
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    f
}
fn activity_end(f: &mut f::Fixture, now: Time, why: c::ReadNone) {
    f.chat.told(
        now,
        &f.client,
        Told::Page { handle: f.chat.activity, page: jig_client::Readout::Jig(c::ReadResult::None(why)) },
        &mut f.out,
    );
}
fn first_activity(chat: &Chat) -> Request {
    Request::Client(Event::Read {
        handle: chat.activity,
        read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None }),
    })
}
#[test]
fn a_chats_busy_activity_read_is_asked_again_and_loads_the_committed_reply() {
    let mut f = pending_activity();
    assert!(f.out.is_empty(), "a delivery does not duplicate the pending read");
    activity_end(&mut f, Time::ZERO, c::ReadNone::Busy);
    let due = Time::ZERO.saturating_add(Duration::from_secs(1));
    assert_eq!(f.chat.next_deadline(), Some(due));
    f.chat.fire(Time::ZERO, &mut f.out);
    assert!(f.out.is_empty(), "the retry waits for its deadline");
    f.chat.fire(due, &mut f.out);
    assert_eq!(f.out.pop(), Some(first_activity(&f.chat)));
    assert!(f.out.is_empty());
    assert_eq!(f.chat.next_deadline(), None, "the retry is now in flight");
    let mut blocks = skein_lib::List::with_capacity(1);
    blocks.push(c::Block::Text(fixtures::acts::text(b"Committed reply"))).unwrap();
    f.chat.told(
        due,
        &f.client,
        Told::Page {
            handle: f.chat.activity,
            page: f::activity_reply(c::Speaker::Assistant, blocks.clone(), COMMITTED),
        },
        &mut f.out,
    );
    assert_eq!(f.chat.thread.loaded, COMMITTED);
    let lines = f.chat.thread.lines();
    let [jig_client::thread::Line::Turn { turn, .. }] = lines.as_slice() else { panic!("committed reply") };
    assert_eq!(turn.messages.get(0).unwrap().blocks, blocks);
    assert_eq!(f.chat.next_deadline(), None);
}
#[test]
fn a_chats_repeated_busy_reads_back_off_to_thirty_seconds_and_a_page_resets_it() {
    let mut f = pending_activity();
    let mut now = Time::ZERO;
    for seconds in [1, 2, 4, 8, 16, 30, 30] {
        activity_end(&mut f, now, c::ReadNone::Busy);
        let due = now.saturating_add(Duration::from_secs(seconds));
        assert_eq!(f.chat.next_deadline(), Some(due));
        f.chat.fire(due, &mut f.out);
        assert_eq!(f.out.pop(), Some(first_activity(&f.chat)));
        now = due;
    }
    f.chat.told(
        now,
        &f.client,
        Told::Page {
            handle: f.chat.activity,
            page: f::activity_reply(c::Speaker::Assistant, skein_lib::List::with_capacity(0), COMMITTED),
        },
        &mut f.out,
    );
    let mut task = f::task();
    task.activity = COMMITTED + 1;
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(now, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    while f.out.pop().is_some() {}
    activity_end(&mut f, now, c::ReadNone::Busy);
    assert_eq!(f.chat.next_deadline(), Some(now.saturating_add(Duration::from_secs(1))));
}
#[test]
fn a_lasting_activity_end_is_not_asked_again() {
    let mut f = pending_activity();
    activity_end(&mut f, Time::ZERO, c::ReadNone::Busy);
    let due = f.chat.next_deadline().unwrap();
    f.chat.fire(due, &mut f.out);
    while f.out.pop().is_some() {}
    activity_end(&mut f, due, c::ReadNone::Refused);
    assert_eq!(f.chat.next_deadline(), None);
    assert!(f.chat.thread.reading.is_none());
    f.chat.fire(due.saturating_add(Duration::from_secs(30)), &mut f.out);
    assert!(f.out.is_empty());
}
#[test]
fn leaving_a_chat_drops_its_activity_retry() {
    for event in [crate::pages::Event::Went { address: crate::Address::Accounts }, crate::pages::Event::Left] {
        let (mut pages, client) = chat_pages();
        let handle = pages.chat().unwrap().activity;
        chat_told(
            &mut pages,
            &client,
            Told::Page { handle, page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)) },
        );
        let due = Time::ZERO.saturating_add(Duration::from_secs(1));
        assert_eq!(crate::pages::next_deadline(&pages), Some(due));
        let limits = fixtures::limits();
        let mut env = fixtures::env(&limits);
        let mut out = Queue::with_capacity(OUTPUTS);
        crate::pages::step(&mut pages, &env, &client, event, &mut out);
        while out.pop().is_some() {}
        assert_eq!(crate::pages::next_deadline(&pages), None);
        env.now = due;
        crate::pages::fire(&mut pages, &env, &client, &mut out);
        assert!(out.is_empty());
    }
}

#[test]
fn a_chats_transient_older_read_keeps_its_cursor() {
    let mut f = pending_activity();
    window_page(&mut f.chat, &f.client, &mut f.out, 0, 2);
    chat_act(&mut f.chat, &f.client, Act::Older, &mut f.out);
    let asked = f.out.pop().expect("older read");
    let Request::Client(Event::Read { read: Readable::Jig(c::Read::Activity { before: Some(_), .. }), .. }) = &asked
    else {
        panic!("older read has its cursor")
    };
    activity_end(&mut f, Time::ZERO, c::ReadNone::Unreachable);
    let due = Time::ZERO.saturating_add(Duration::from_secs(1));
    f.chat.fire(due, &mut f.out);
    assert_eq!(f.out.pop(), Some(asked));
    assert!(f.out.is_empty());
}

fn draft_handle(chat: &Chat) -> jig_client::Handle {
    jig_client::Handle { asker: chat.document.asker, number: chat.document.number.checked_add(2).unwrap() }
}
fn take_draft_reads(f: &mut f::Fixture) -> usize {
    let mut count = 0_usize;
    while let Some(request) = f.out.pop() {
        if let Request::Client(Event::Read {
            handle,
            read: Readable::Jig(c::Read::Notes { scope, description, after }),
        }) = request
        {
            assert_eq!(handle, draft_handle(&f.chat));
            assert_eq!(scope, c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK });
            assert_eq!(description, None);
            assert_eq!(after, None);
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
fn draft_answer(f: &mut f::Fixture, page: jig_client::Readout) {
    f.chat.told(Time::ZERO, &f.client, Told::Page { handle: draft_handle(&f.chat), page }, &mut f.out);
}
pub(super) fn notes_page(notes: &[c::Note]) -> jig_client::Readout {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(notes.len()).unwrap());
    for note in notes {
        rows.push(note.clone()).unwrap();
    }
    jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
        rows,
        before: None,
        after: None,
        cut: false,
    }))))
}
pub(super) fn draft_note(number: u64, revision: u64, body: &[u8]) -> c::Note {
    c::Note {
        number,
        revision: c::Revision { value: revision },
        scope: c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK },
        description: f::document_text(b"A draft"),
        body: f::document_text(body),
        writer: c::NoteAuthor::Task { task: f::TASK, attempt: f::ATTEMPT },
        tasks: skein_lib::List::with_capacity(0),
        changed: Some(skein_lib::Wall::from_nanos(123)),
        acts: skein_lib::List::with_capacity(0),
        cut: false,
    }
}
fn note_call(f: &mut f::Fixture, number: u64, position: u64) {
    document_call(f, number, position, b"note", "Note 12 revision 1 written · 64 lines");
}
fn document_call(f: &mut f::Fixture, number: u64, position: u64, tool: &[u8], outcome: &str) {
    let mut call = f::delegate_call(number);
    call.tool = fixtures::acts::text(tool);
    call.objects = skein_lib::List::with_capacity(0);
    call.outcome = Some(c::Text::new(outcome.as_bytes().into(), 256).unwrap());
    let mut blocks = skein_lib::List::with_capacity(1);
    blocks.push(c::Block::Call(call)).unwrap();
    f::activity_blocks(f, c::Speaker::Assistant, blocks, position);
}
#[test]
fn reads_once_the_task_arrives() {
    // Task, activity, draft read and goal chip reserve four handles; the live run uses one.
    const RESERVED_AND_RUN: u64 = 5;
    let mut f = f::opened();
    assert_eq!(take_draft_reads(&mut f), 1);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(take_draft_reads(&mut f), 0);
    assert!(!f.chat.wanted, "the same task delivery does not request another read");
    assert_eq!(f.chat.next, f::FIRST.checked_add(RESERVED_AND_RUN).unwrap());
    // The final task's read is the other first-arrival path.
    let limits = fixtures::limits();
    let mut out = Queue::uncapped();
    let mut chat = Chat::new(f::TASK, crate::pages::chat::document::Mode::Chat, 20, &limits, &mut out);
    while out.pop().is_some() {}
    chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle: chat.document,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(f::task())))),
        },
        &mut out,
    );
    let mut notes = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Read { read: Readable::Jig(c::Read::Notes { scope, .. }), .. }) = request {
            assert_eq!(scope, c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK });
            notes = notes.checked_add(1).unwrap();
        }
    }
    assert_eq!(notes, 1);
}
#[test]
fn a_new_note_call_reads_again() {
    let mut f = f::opened();
    assert_eq!(take_draft_reads(&mut f), 1);
    draft_answer(&mut f, notes_page(&[]));
    note_call(&mut f, 11, f::POSITION);
    assert_eq!(take_draft_reads(&mut f), 1);
    assert_eq!(
        f.chat.newest_write(),
        crate::pages::chat::document::note_write("Note 12 revision 1 written · 64 lines".as_bytes())
    );
    draft_answer(&mut f, notes_page(&[]));
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(take_draft_reads(&mut f), 0, "the old note call is already seen");
}
#[test]
fn a_note_number_reused_in_a_later_turn_reads_again() {
    let mut f = f::opened();
    take_draft_reads(&mut f);
    draft_answer(&mut f, notes_page(&[draft_note(12, 1, b"# First")]));
    note_call(&mut f, 1, f::POSITION);
    take_draft_reads(&mut f);
    draft_answer(&mut f, notes_page(&[draft_note(12, 1, b"# First")]));
    note_call(&mut f, 1, f::POSITION + 1);
    assert_eq!(take_draft_reads(&mut f), 1, "each turn has its own call numbers");
    draft_answer(&mut f, notes_page(&[draft_note(12, 2, b"# Revised")]));
    assert_eq!(f.chat.draft.as_ref().unwrap().body.as_ref(), b"# Revised");
}
#[test]
fn one_read_out_wanted_once_more() {
    let mut f = f::opened();
    assert_eq!(take_draft_reads(&mut f), 1);
    note_call(&mut f, 11, f::POSITION);
    note_call(&mut f, 12, f::POSITION + 1);
    assert_eq!(take_draft_reads(&mut f), 0);
    assert!(f.chat.wanted);
    draft_answer(&mut f, notes_page(&[]));
    assert_eq!(take_draft_reads(&mut f), 1, "coalesced triggers ask once after the pending answer");
    assert!(!f.chat.wanted);
    draft_answer(&mut f, notes_page(&[]));
    assert_eq!(take_draft_reads(&mut f), 0);
}
#[test]
fn a_transient_end_retries_on_activitys_waits() {
    for why in [c::ReadNone::Busy, c::ReadNone::Unreachable, c::ReadNone::SignedOut] {
        let mut f = f::opened();
        take_draft_reads(&mut f);
        let mut now = Time::ZERO;
        for seconds in [1, 2, 4, 8, 16, 30, 30] {
            f.chat.told(
                now,
                &f.client,
                Told::Page { handle: draft_handle(&f.chat), page: jig_client::Readout::Jig(c::ReadResult::None(why)) },
                &mut f.out,
            );
            let due = now.saturating_add(Duration::from_secs(seconds));
            assert_eq!(f.chat.next_deadline(), Some(due));
            f.chat.fire(now, &mut f.out);
            assert_eq!(take_draft_reads(&mut f), 0);
            f.chat.fire(due, &mut f.out);
            assert_eq!(take_draft_reads(&mut f), 1);
            now = due;
        }
        draft_answer(&mut f, notes_page(&[]));
        assert_eq!(f.chat.next_deadline(), None);
    }
}
#[test]
fn off_a_call_to_note_reads_again() {
    let mut f = f::opened();
    let watch = open_run(&mut f);
    draft_answer(&mut f, notes_page(&[]));
    let c::Delivery::RunState(mut state) = run_state(None) else { panic!("run state") };
    state.call = Some(c::CurrentCall {
        identity: c::CallIdentity {
            owner: skein_lib::Token::new(1),
            sequence: 1,
            retry: 0,
            id: fixtures::acts::text(b"n1"),
        },
        tool: fixtures::acts::text(b"note"),
        input: fixtures::acts::text(b"{}"),
        cut: false,
        began: None,
    });
    run_delivery(&mut f, watch, c::Delivery::RunState(state));
    assert_eq!(take_draft_reads(&mut f), 0);
    run_delivery(&mut f, watch, run_state(None));
    assert_eq!(take_draft_reads(&mut f), 1);
}
#[test]
fn an_empty_answer_leaves_the_document() {
    let mut f = f::opened();
    take_draft_reads(&mut f);
    draft_answer(&mut f, notes_page(&[draft_note(12, 1, b"# Body")]));
    let saved = f.chat.draft.clone().unwrap();
    note_call(&mut f, 11, f::POSITION);
    take_draft_reads(&mut f);
    draft_answer(&mut f, notes_page(&[]));
    assert_eq!(f.chat.draft.as_ref(), Some(&saved));
    note_call(&mut f, 12, f::POSITION + 1);
    take_draft_reads(&mut f);
    draft_answer(&mut f, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)));
    assert_eq!(f.chat.draft.as_ref(), Some(&saved));
    assert_eq!(f.chat.next_deadline(), None);
}
#[test]
fn the_lowest_numbered_is_the_document() {
    let mut f = f::opened();
    take_draft_reads(&mut f);
    let mut foreign = draft_note(1, 1, b"Wrong scope");
    foreign.scope = c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK + 1 };
    draft_answer(&mut f, notes_page(&[draft_note(13, 1, b"Later note"), foreign, draft_note(12, 3, b"# Current")]));
    let draft = f.chat.draft.as_ref().unwrap();
    assert_eq!(
        (draft.note, draft.revision, draft.title.as_ref(), draft.body.as_ref(), draft.changed),
        (12, 3, b"A draft".as_slice(), b"# Current".as_slice(), Some(skein_lib::Wall::from_nanos(123)))
    );
}
#[test]
fn a_work_note_never_displaces_the_lowest_document_in_the_chats_scope() {
    let mut f = f::opened();
    take_draft_reads(&mut f);
    let mut work = draft_note(1, 1, b"Internal working memory");
    work.description = f::document_text(b"work: settings");
    let mut foreign = draft_note(2, 1, b"Another chat");
    foreign.scope = c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK + 1 };
    draft_answer(&mut f, notes_page(&[draft_note(13, 1, b"Later"), work, foreign, draft_note(12, 1, b"Document")]));
    assert_eq!(f.chat.draft.as_ref().expect("document").note, 12);
}
#[test]
fn a_chat_with_only_work_notes_has_no_document() {
    for previous_document in [false, true] {
        let mut f = f::opened();
        take_draft_reads(&mut f);
        if previous_document {
            draft_answer(&mut f, notes_page(&[draft_note(1, 1, b"Document")]));
            note_call(&mut f, 11, f::POSITION);
            take_draft_reads(&mut f);
        }
        let mut work = draft_note(1, 2, b"Internal working memory");
        work.description = f::document_text(b"work: settings");
        draft_answer(&mut f, notes_page(&[work]));
        assert!(f.chat.draft.is_none(), "work: notes must never be drawn, including a renamed document");
    }
}
#[test]
fn mode_and_show_section_are_page_state() {
    use crate::pages::chat::document::Mode;
    for (me, charter, picked, opening) in [
        (fixtures::chats::me(), 1, None, Mode::Chat),
        (super::chats::drafting_me(), 12, None, Mode::Split),
        (super::chats::drafting_me(), 1, None, Mode::Chat),
        (super::chats::drafting_me(), 12, Some(Mode::Chat), Mode::Chat),
    ] {
        let mut f = f::opened();
        let mut serial = SERIAL;
        let env = fixtures::env(&fixtures::limits());
        let forge = crate::forge::Side::new(&env.limits.forge, fixtures::SEED);
        let mut task = f::task();
        task.chip.executor = c::ChipExecutor::Agent { charter };
        f::deliver(&mut f.client, f.watch, task.clone());
        if let Some(mode) = picked {
            f.chat.act(&env, &f.client, Act::Mode { mode }, &mut serial, &mut f.out);
        }
        f.chat.goal_watch(&f.client, &forge, &mut f.out);
        for event in [
            Event::Watch { handle: crate::pages::me_handle(), subject: Watchable::Jig(c::Subject::Me) },
            Event::Up {
                up: jig_client::Up::Watched {
                    watch: f.me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        ] {
            f.client.step_unlinked(Time::ZERO, skein_lib::Wall::EPOCH, event, &mut Queue::uncapped());
        }
        f.chat.goal_watch(&f.client, &forge, &mut f.out);
        assert_eq!(f.chat.mode, opening, "charter {charter}, picked {picked:?}");
        for (act, mode, shown, count) in [
            (Act::ShowSection { line: 31 }, Mode::Split, Some(31), 1),
            (Act::Mode { mode: Mode::Draft }, Mode::Draft, Some(31), 1),
            (Act::ShowSection { line: 7 }, Mode::Draft, Some(7), 2),
            (Act::Mode { mode: Mode::Chat }, Mode::Chat, Some(7), 2),
        ] {
            f.chat.act(&env, &f.client, act, &mut serial, &mut f.out);
            f::deliver(&mut f.client, f.watch, task.clone());
            f.chat.goal_watch(&f.client, &forge, &mut f.out);
            assert_eq!((f.chat.mode, f.chat.shown, f.chat.shown_count), (mode, shown, count));
        }
    }
}

fn start_over_pages() -> (crate::pages::Pages, jig_client::Client) {
    let mut f = f::with_delegate();
    let mut me = fixtures::chats::me();
    let project = me.projects.get_mut(0).unwrap();
    let mut asking = project.templates.get(0).unwrap().clone();
    asking.number = 5;
    let mut templates = skein_lib::List::with_capacity(2);
    templates.push(project.templates.get(0).unwrap().clone()).unwrap();
    templates.push(asking).unwrap();
    project.templates = templates;
    f.client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched { watch: f.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
        },
        &mut Queue::uncapped(),
    );
    let mut task = f::task();
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.acts = fixtures::acts::offered(c::ActKind::Cancel, c::Admission::Allowed);
    f::deliver(&mut f.client, f.watch, task);
    let mut pages = crate::pages::Pages::new();
    crate::pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &f.client,
        crate::pages::Event::Went { address: crate::Address::Chat { task: f::TASK } },
        &mut Queue::uncapped(),
    );
    let handle = pages.chat().unwrap().document;
    chat_told(&mut pages, &f.client, Told::Changed { handle });
    let handle = draft_handle(pages.chat().unwrap());
    chat_told(&mut pages, &f.client, Told::Page { handle, page: notes_page(&[draft_note(1, 1, b"# How it works")]) });
    (pages, f.client)
}
fn start_over_act(pages: &mut crate::pages::Pages, client: &jig_client::Client, act: Act) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::pages::step(pages, &fixtures::env(&fixtures::limits()), client, crate::pages::Event::Act { act }, &mut out);
    out
}
#[test]
fn start_over_asks_first() {
    let (mut pages, client) = start_over_pages();
    let mut out = start_over_act(&mut pages, &client, Act::StartOver);
    assert!(fixtures::acts::sent(&mut out).is_empty());
    assert!(pages.confirmation().is_some());
    let chat = pages.chat().unwrap();
    assert!(chat.restarting);
    assert!(chat.closing.is_some());
    assert!(chat.pending.is_empty());
}
#[test]
fn confirmed_it_closes_with_the_chats_own_close() {
    let (mut pages, client) = start_over_pages();
    start_over_act(&mut pages, &client, Act::StartOver);
    let expected = pages.chat().unwrap().closing.clone().unwrap();
    let mut out = start_over_act(&mut pages, &client, Act::Confirm);
    assert_eq!(fixtures::acts::sent(&mut out).as_ref(), [expected]);
    assert!(pages.confirmation().unwrap().sent.is_some());
    assert_eq!(pages.chat().unwrap().pending.get(0).unwrap().1, Sending::Close);
    assert!(start_over_act(&mut pages, &client, Act::Confirm).is_empty());
}
fn answer_start_over(
    pages: &mut crate::pages::Pages,
    client: &jig_client::Client,
    outcome: jig_client::Outcome,
) -> Queue<Request> {
    let key = pages.chat().unwrap().pending.get(0).unwrap().0;
    let mut out = Queue::uncapped();
    crate::pages::step(
        pages,
        &fixtures::env(&fixtures::limits()),
        client,
        crate::pages::Event::Told { told: Told::Answered { key, outcome } },
        &mut out,
    );
    out
}
#[test]
fn a_done_start_over_shows_ask() {
    let (mut pages, client) = start_over_pages();
    start_over_act(&mut pages, &client, Act::StartOver);
    start_over_act(&mut pages, &client, Act::Confirm);
    let mut out = answer_start_over(&mut pages, &client, seen_answer());
    assert_eq!(out.pop(), Some(Request::Show { address: crate::Address::Ask { project: fixtures::PROJECT } }));
    assert!(out.is_empty());
    assert!(pages.confirmation().is_none());
    assert!(pages.chat().unwrap().closing.is_none());
    assert!(!pages.chat().unwrap().restarting);
}
#[test]
fn a_failed_close_keeps_its_fates_and_restarting() {
    let (mut pages, client) = start_over_pages();
    start_over_act(&mut pages, &client, Act::StartOver);
    let closing = pages.chat().unwrap().closing.clone();
    start_over_act(&mut pages, &client, Act::Confirm);
    let refused = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::NotOffered,
        at: None,
    }));
    assert!(answer_start_over(&mut pages, &client, refused.clone()).is_empty());
    assert_eq!(pages.address(), &crate::Address::Chat { task: f::TASK });
    assert_eq!(pages.chat().unwrap().outcome.as_ref(), Some(&refused));
    assert_eq!(pages.chat().unwrap().outcome_kind, Some(Sending::Close));
    assert!(pages.chat().unwrap().restarting);
    assert_eq!(pages.chat().unwrap().closing, closing);
    assert_eq!(pages.confirmation().unwrap().failed.as_ref(), Some(&refused));
    assert_eq!(fixtures::acts::sent(&mut start_over_act(&mut pages, &client, Act::Confirm)).len(), 1);
}
#[test]
fn cancel_changes_nothing() {
    let (mut pages, client) = start_over_pages();
    let draft = pages.chat().unwrap().draft.clone();
    start_over_act(&mut pages, &client, Act::StartOver);
    assert!(start_over_act(&mut pages, &client, Act::Cancel).is_empty());
    assert!(pages.confirmation().is_none());
    let chat = pages.chat().unwrap();
    assert!(chat.closing.is_none());
    assert!(!chat.restarting);
    assert!(chat.pending.is_empty());
    assert_eq!(chat.draft, draft);
    assert_eq!(pages.address(), &crate::Address::Chat { task: f::TASK });
}
fn departed_answer(f: &mut f::Fixture, key: c::ProposalKey, result: c::ReadResult) {
    proposals(f, &[key]);
    reads(&mut f.out);
    proposals(f, &[]);
    let reads = reads(&mut f.out);
    let [(handle, read)] = reads.as_ref() else { panic!("departed proposal read") };
    assert_eq!(*read, key);
    assert!(f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page { handle: *handle, page: jig_client::Readout::Jig(result) },
        &mut f.out
    ));
}
#[test]
fn a_departed_proposals_own_read_keeps_its_acceptance_time() {
    let mut f = f::opened();
    let at = skein_lib::Wall::from_nanos(50_580_000_000_000);
    departed_answer(
        &mut f,
        PROPOSAL,
        c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
            key: c::ObjectKey::Proposal(PROPOSAL),
            value: c::EndedValue::Proposal(c::ProposalHistory {
                reason: c::DocumentText::complete(
                    c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
                ),
                action: c::RetainedAction::Batch(skein_lib::List::with_capacity(0)),
                number: PROPOSAL.number,
                proposer: f::TASK,
                as_holder: true,
                state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(fixtures::PARTY) },
            }),
            at: Some(at),
            by: None,
        }))),
    );
    assert_eq!(f.chat.decided, Some((PROPOSAL.number, at)));
    proposals(&mut f, &[]);
    assert_eq!(f.chat.decided, Some((PROPOSAL.number, at)), "later task documents keep the decision");
}
#[test]
fn an_unknown_departed_proposal_keeps_its_number_without_a_time() {
    let mut f = f::opened();
    departed_answer(&mut f, PROPOSAL, c::ReadResult::None(c::ReadNone::Unknown));
    assert_eq!(f.chat.decided, None);
    assert_eq!(f.chat.departed_proposal, Some(PROPOSAL.number));
}

#[test]
fn a_retained_settings_card_keeps_the_documents_acceptance_time() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    let handle = reads(&mut f.out)[0].0;
    card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::Value(settings_proposal(PROPOSAL))));
    proposals(&mut f, &[]);
    assert_eq!(f.chat.cards.len(), 1, "settings keeps its card after the decision");
    let at = skein_lib::Wall::from_nanos(50_580_000_000_000);
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
                key: c::ObjectKey::Proposal(PROPOSAL),
                value: c::EndedValue::Proposal(c::ProposalHistory {
                    reason: c::DocumentText::complete(
                        c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
                    ),
                    action: c::RetainedAction::Settings(c::RetainedActionSettings {
                        projects: skein_lib::List::with_capacity(0),
                        against: 0,
                        items: skein_lib::List::with_capacity(0),
                        made: Some(1),
                    }),
                    number: PROPOSAL.number,
                    proposer: f::TASK,
                    as_holder: true,
                    state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(fixtures::PARTY) },
                }),
                at: Some(at),
                by: None,
            })))),
        },
        &mut f.out,
    );
    assert_eq!(f.chat.decided, Some((PROPOSAL.number, at)));
    let Some(c::ReadValue::Ended(_)) = &f.chat.cards.get(0).unwrap().page else {
        panic!("settings keeps its ended proposal card")
    };
}

fn proposal_read_handle(
    reads: &[(jig_client::Handle, c::ProposalKey)],
    proposal: c::ProposalKey,
) -> jig_client::Handle {
    for (handle, key) in reads {
        if *key == proposal {
            return *handle;
        }
    }
    panic!("proposal read missing: {proposal:?}")
}
fn card_page(f: &mut f::Fixture, now: Time, handle: jig_client::Handle, page: jig_client::Readout) {
    f.chat.told(now, &f.client, Told::Page { handle, page }, &mut f.out);
}
fn pending_card() -> (f::Fixture, jig_client::Handle) {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    let handle = reads(&mut f.out)[0].0;
    (f, handle)
}
fn settings_proposal(key: c::ProposalKey) -> c::ReadValue {
    let c::ReadValue::Proposal(mut proposal) = fixtures::acts::proposal(c::ActKind::Accept, c::Admission::Allowed)
    else {
        panic!("proposal fixture")
    };
    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 0,
        items: skein_lib::List::with_capacity(0),
        review: None,
        made: None,
    }));
    proposal.key = key;
    c::ReadValue::Proposal(proposal)
}
fn accepted_proposal(key: c::ProposalKey, at: skein_lib::Wall) -> jig_client::Readout {
    jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Proposal(key),
        value: c::EndedValue::Proposal(c::ProposalHistory {
            reason: c::DocumentText::complete(
                c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
            ),
            action: c::RetainedAction::Batch(skein_lib::List::with_capacity(0)),
            number: key.number,
            proposer: match key.proposer {
                c::Proposer::Task(task) => task,
                c::Proposer::Party(party) => party,
            },
            as_holder: true,
            state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(fixtures::PARTY) },
        }),
        at: Some(at),
        by: None,
    }))))
}

#[test]
fn a_card_read_that_ends_busy_is_asked_again() {
    for page in [
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)),
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
        jig_client::Readout::Limit,
    ] {
        let (mut f, handle) = pending_card();
        proposals(&mut f, &[PROPOSAL, DELEGATE_PROPOSAL]);
        let other = reads(&mut f.out)[0].0;
        card_page(&mut f, Time::ZERO, handle, page.clone());
        let later = Time::ZERO.saturating_add(Duration::from_millis(500));
        card_page(&mut f, later, other, page.clone());
        let due = Time::ZERO.saturating_add(Duration::from_secs(1));
        assert_eq!(f.chat.next_deadline(), Some(due), "{page:?}");
        f.chat.fire(Time::ZERO.saturating_add(Duration::from_millis(999)), &mut f.out);
        assert!(reads(&mut f.out).is_empty(), "the read waits until its deadline");
        f.chat.fire(due, &mut f.out);
        assert_eq!(reads(&mut f.out).as_ref(), [(handle, PROPOSAL)]);
        f.chat.fire(due, &mut f.out);
        assert!(reads(&mut f.out).is_empty(), "only one retry is in flight");
        let later_due = later.saturating_add(Duration::from_secs(1));
        assert_eq!(f.chat.next_deadline(), Some(later_due), "each card keeps its deadline");
        f.chat.fire(later_due, &mut f.out);
        assert_eq!(reads(&mut f.out).as_ref(), [(other, DELEGATE_PROPOSAL)]);
        assert_eq!(f.chat.next_deadline(), None);
    }
}
#[test]
fn the_wait_doubles_and_a_value_resets_it() {
    let (mut f, handle) = pending_card();
    let mut now = Time::ZERO;
    for seconds in [1, 2, 4, 8, 16, 30, 30] {
        card_page(&mut f, now, handle, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
        let due = now.saturating_add(Duration::from_secs(seconds));
        assert_eq!(f.chat.next_deadline(), Some(due));
        f.chat.fire(due, &mut f.out);
        assert_eq!(reads(&mut f.out).as_ref(), [(handle, PROPOSAL)]);
        now = due;
    }
    card_page(&mut f, now, handle, jig_client::Readout::Jig(c::ReadResult::Value(settings_proposal(PROPOSAL))));
    assert_eq!(f.chat.next_deadline(), None);
    proposals(&mut f, &[]);
    reads(&mut f.out);
    card_page(&mut f, now, handle, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
    assert_eq!(f.chat.next_deadline(), Some(now.saturating_add(Duration::from_secs(1))));
}
#[test]
fn a_lasting_end_is_not_asked_again() {
    for why in
        [c::ReadNone::Gone, c::ReadNone::Refused, c::ReadNone::Unknown, c::ReadNone::TooLarge { bytes: u64::MAX }]
    {
        let (mut f, handle) = pending_card();
        let value = settings_proposal(PROPOSAL);
        card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::Value(value)));
        proposals(&mut f, &[]);
        reads(&mut f.out);
        card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
        card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::None(why)));
        assert_eq!(f.chat.cards.get(FIRST_CARD).unwrap().page, None, "{why:?}");
        assert_eq!(f.chat.next_deadline(), None, "{why:?}");
        f.chat.fire(Time::ZERO.saturating_add(Duration::from_secs(30)), &mut f.out);
        assert!(reads(&mut f.out).is_empty(), "{why:?}");
    }
}
#[test]
fn a_retained_card_s_failed_re_read_keeps_its_page_and_asks_again() {
    let (mut f, handle) = pending_card();
    let value = settings_proposal(PROPOSAL);
    card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::Value(value.clone())));
    proposals(&mut f, &[]);
    reads(&mut f.out);
    card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
    assert_eq!(f.chat.cards.get(FIRST_CARD).unwrap().page.as_ref(), Some(&value));
    proposals(&mut f, &[]);
    assert!(reads(&mut f.out).is_empty(), "a refresh respects the retry wait");
    let due = Time::ZERO.saturating_add(Duration::from_secs(1));
    assert_eq!(f.chat.next_deadline(), Some(due));
    f.chat.fire(due, &mut f.out);
    assert_eq!(reads(&mut f.out).as_ref(), [(handle, PROPOSAL)]);
    card_page(&mut f, due, handle, accepted_proposal(PROPOSAL, skein_lib::Wall::EPOCH));
    let Some(c::ReadValue::Ended(_)) = &f.chat.cards.get(FIRST_CARD).unwrap().page else {
        panic!("the retry delivers the ended proposal")
    };
    assert_eq!(f.chat.next_deadline(), None);
}
#[test]
fn cards_hold_their_proposals_in_number_order() {
    let mut f = f::opened();
    proposals(&mut f, &[DELEGATE_PROPOSAL, PROPOSAL]);
    let initial = reads(&mut f.out);
    let handle = proposal_read_handle(&initial, PROPOSAL);
    card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::Value(settings_proposal(PROPOSAL))));
    proposals(&mut f, &[DELEGATE_PROPOSAL, PROPOSAL]);
    assert_eq!(
        [f.chat.cards.get(FIRST_CARD).unwrap().object, f.chat.cards.get(SECOND_CARD).unwrap().object],
        [c::ObjectKey::Proposal(PROPOSAL), c::ObjectKey::Proposal(DELEGATE_PROPOSAL)]
    );
    proposals(&mut f, &[DELEGATE_PROPOSAL]);
    assert_eq!(
        [f.chat.cards.get(FIRST_CARD).unwrap().object, f.chat.cards.get(SECOND_CARD).unwrap().object],
        [c::ObjectKey::Proposal(PROPOSAL), c::ObjectKey::Proposal(DELEGATE_PROPOSAL)],
        "retained settings cards keep their number order"
    );
}
#[test]
fn an_older_proposal_s_answer_keeps_the_newest_decision() {
    for retained in [false, true] {
        let mut f = f::opened();
        proposals(&mut f, &[DELEGATE_PROPOSAL, PROPOSAL]);
        let initial = reads(&mut f.out);
        if retained {
            for (handle, key) in &initial {
                card_page(
                    &mut f,
                    Time::ZERO,
                    *handle,
                    jig_client::Readout::Jig(c::ReadResult::Value(settings_proposal(*key))),
                );
            }
        }
        proposals(&mut f, &[]);
        let departed = reads(&mut f.out);
        let at = skein_lib::Wall::from_nanos(50_580_000_000_000);
        for key in [DELEGATE_PROPOSAL, PROPOSAL] {
            let handle = proposal_read_handle(&departed, key);
            card_page(&mut f, Time::ZERO, handle, accepted_proposal(key, at));
        }
        assert_eq!(f.chat.decided, Some((DELEGATE_PROPOSAL.number, at)), "retained: {retained}");
        assert_eq!(f.chat.departed_proposal, Some(DELEGATE_PROPOSAL.number), "retained: {retained}");
    }
}

fn draft_limits() -> crate::Limits {
    let mut limits = fixtures::limits();
    limits.client.domain.long_text = 128;
    limits
}
fn draft_ready() -> f::Fixture {
    let mut f = f::opened();
    f.client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Watch { handle: crate::pages::me_handle(), subject: Watchable::Jig(c::Subject::Me) },
        &mut Queue::uncapped(),
    );
    let mut me = fixtures::chats::me();
    let member = me.projects.get_mut(0).unwrap();
    let mut goal = member.templates.get(0).unwrap().clone();
    goal.number = 2;
    goal.tracked = Some(1);
    goal.flags.conversation = false;
    let mut templates = skein_lib::List::with_capacity(2);
    templates.push(member.templates.get(0).unwrap().clone()).unwrap();
    templates.push(goal).unwrap();
    member.templates = templates;
    f.client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched { watch: f.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
        },
        &mut Queue::uncapped(),
    );
    let mut task = f::task();
    task.acts = fixtures::acts::offered(c::ActKind::Message, c::Admission::Allowed);
    f::deliver(&mut f.client, f.watch, task);
    while f.out.pop().is_some() {}
    let limits = draft_limits();
    f.chat.thread = jig_client::thread::Thread::new(f::TASK, &crate::pages::chat::thread_limits(&limits));
    f
}
#[test]
fn add_draft_sends_and_goes_to_split() {
    let mut f = draft_ready();
    f.chat.words = b"Unsent words".as_slice().into();
    let mut serial = SERIAL;
    for _ in 0_u32..2 {
        f.chat.act(&fixtures::env(&draft_limits()), &f.client, Act::AddDraft, &mut serial, &mut f.out);
    }
    assert_eq!(f.chat.mode, crate::pages::chat::document::Mode::Split);
    assert!(f.chat.writing);
    assert_eq!(f.chat.words.as_ref(), b"Unsent words");
    let sent = fixtures::acts::sent(&mut f.out);
    let [c::Ask::Message { project, task, words }] = sent.as_ref() else { panic!("one draft message: {sent:?}") };
    assert_eq!(
        (*project, *task, words.bytes()),
        (fixtures::PROJECT, f::TASK, crate::pages::chat::document::WRITE_THIS_UP)
    );
    assert_eq!(f.chat.pending.len(), 1);
    assert_eq!(f.chat.pending.get(0).unwrap().1, Sending::Message);
    assert!(f.chat.thread.words.pending(f.chat.pending.get(0).unwrap().0));
}
#[test]
fn a_refused_add_keeps_the_pane_and_its_words() {
    let mut f = draft_ready();
    f.chat.act(&fixtures::env(&draft_limits()), &f.client, Act::AddDraft, &mut { SERIAL }, &mut f.out);
    let key = f.chat.pending.get(0).unwrap().0;
    let refused = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::NotOffered,
        at: None,
    }));
    f.chat.told(Time::ZERO, &f.client, Told::Answered { key, outcome: refused.clone() }, &mut f.out);
    assert_eq!(f.chat.mode, crate::pages::chat::document::Mode::Split);
    assert!(f.chat.writing);
    assert_eq!(f.chat.outcome, Some(refused));
    assert_eq!(f.chat.outcome_kind, Some(Sending::Message));
    assert!(f.chat.pending.is_empty());
}

#[test]
fn a_page_of_working_notes_does_not_hide_a_later_document() {
    let mut f = f::opened();
    take_draft_reads(&mut f);
    let mut work = draft_note(1, 1, b"Internal working memory");
    work.description = f::document_text(b"work: settings");
    let jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(mut page))) = notes_page(&[work]) else {
        panic!("notes page")
    };
    let after = c::Cursor::new(Box::from(*b"next"), 4).unwrap();
    page.after = Some(after.clone());
    draft_answer(&mut f, jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))));
    assert!(f.chat.draft.is_none());
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Read {
            handle: draft_handle(&f.chat),
            read: Readable::Jig(c::Read::Notes {
                scope: c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK },
                description: None,
                after: Some(after),
            }),
        })),
        "continue past working notes rather than claiming the chat has no document"
    );
    draft_answer(&mut f, notes_page(&[draft_note(2, 1, b"Visible document")]));
    assert_eq!(f.chat.draft.as_ref().unwrap().note, 2);
    assert_eq!(f.chat.next_deadline(), None);
    assert!(f.out.is_empty());
}

// Ten pages of two short lines exceed a four-line byte window.
const WINDOW_PAGES: u32 = 10;
const WINDOW_PAGE_ROWS: u32 = 2;
const WINDOW_BYTES: u64 = 4 * 64;
fn reading_chat(limits: &crate::Limits) -> (Chat, jig_client::Client, Queue<Request>) {
    let client = jig_client::Client::unlinked(&limits.client, fixtures::SEED).unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    let chat = Chat::new(f::TASK, crate::pages::chat::document::Mode::Chat, f::FIRST, limits, &mut out);
    while out.pop().is_some() {}
    (chat, client, out)
}
fn activity_answer(page: c::ActivityPage) -> jig_client::Readout {
    jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(page))))
}
fn empty_activity() -> c::ActivityPage {
    c::ActivityPage {
        rows: skein_lib::List::with_capacity(0),
        before: Some(c::Cursor::new(Box::from(*b"older"), 5).unwrap()),
        after: None,
        cut: false,
    }
}
fn chat_act(chat: &mut Chat, client: &jig_client::Client, act: Act, out: &mut Queue<Request>) {
    chat.act(&fixtures::env(&fixtures::limits()), client, act, &mut { SERIAL }, out);
}
#[test]
fn newer_is_read_on_a_chats_page() {
    let (mut pages, client) = chat_pages();
    let handle = pages.chat().unwrap().activity;
    let mut page = window_activity(0, WINDOW_PAGES);
    page.after = empty_activity().before;
    chat_told(&mut pages, &client, Told::Page { handle, page: activity_answer(page) });
    let mut out = Queue::with_capacity(OUTPUTS);
    for _ in [false, true] {
        crate::pages::step(
            &mut pages,
            &fixtures::env(&fixtures::limits()),
            &client,
            crate::pages::Event::Act { act: Act::Newer },
            &mut out,
        );
    }
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle,
            read: Readable::Jig(c::Read::ActivityBeside {
                task: f::TASK,
                position: u64::from(WINDOW_PAGES * WINDOW_PAGE_ROWS),
                side: c::Side::Later,
            }),
        }))
    );
    assert!(out.is_empty(), "an outstanding read is not duplicated");
}
#[test]
fn the_end_comes_back_after_enough_older() {
    use jig_client::thread::window::End;
    let mut limits = fixtures::limits();
    limits.client.domain.document_bytes = WINDOW_BYTES;
    let (mut chat, client, mut out) = reading_chat(&limits);
    let mut serial = SERIAL;
    for page in 0..WINDOW_PAGES {
        if page > 0 {
            if chat.thread.reading.is_none() {
                chat.act(&fixtures::env(&limits), &client, Act::Older, &mut serial, &mut out);
            }
            assert!(out.pop().is_some(), "older page {page} is read");
        }
        window_page(&mut chat, &client, &mut out, page, WINDOW_PAGES);
    }
    assert!(!chat.thread.lines.edges.reaches(End::Before), "older reads let the newest lines go");
    for page in (0..WINDOW_PAGES - 1).rev() {
        let position = chat.thread.lines.rows.get(0).unwrap().source.position;
        chat.act(&fixtures::env(&limits), &client, Act::Newer, &mut serial, &mut out);
        assert_eq!(
            out.pop(),
            Some(Request::Client(Event::Read {
                handle: chat.activity,
                read: Readable::Jig(c::Read::ActivityBeside { task: f::TASK, position, side: c::Side::Later }),
            }))
        );
        let mut answer = window_activity(page, WINDOW_PAGES);
        answer.after = if page == 0 { None } else { empty_activity().before };
        chat.told(Time::ZERO, &client, Told::Page { handle: chat.activity, page: activity_answer(answer) }, &mut out);
        assert!(out.is_empty());
    }
    assert!(chat.thread.lines.edges.reaches(End::Before));
    assert_eq!(chat.thread.lines.rows.get(0).unwrap().source.position, u64::from(WINDOW_PAGES * WINDOW_PAGE_ROWS));
}
#[test]
fn latest_and_sending_go_to_the_end_and_read_it_where_it_went() {
    use jig_client::thread::window::Edge;
    for (act, sends, second_end) in [(Act::Latest, 0_u32, 2), (Act::Send, 1, 1)] {
        for (edge, reads) in [(Edge::Dropped, true), (Edge::End, false)] {
            let mut f = send_ready(b"Next", c::Admission::Allowed);
            f.chat.thread.lines.edges.before = edge;
            chat_act(&mut f.chat, &f.client, act.clone(), &mut f.out);
            assert_eq!(f.chat.to_end(), 1, "{act:?} requests the end");
            let mut read_count = 0;
            let mut send_count = 0;
            while let Some(request) = f.out.pop() {
                match request {
                    Request::Client(Event::Read { handle, read }) => {
                        assert_eq!(handle, f.chat.activity);
                        assert_eq!(read, Readable::Jig(c::Read::Activity { task: f::TASK, before: None }));
                        read_count += 1;
                    }
                    Request::Client(Event::Send { .. }) => send_count += 1,
                    other @ (Request::Push(_)
                    | Request::Screen { .. }
                    | Request::KeepLook { .. }
                    | Request::Client(_)
                    | Request::Show { .. }
                    | Request::Replace { .. }
                    | Request::OpenTab { .. }) => {
                        panic!("unexpected {other:?}")
                    }
                }
            }
            assert_eq!(read_count, u32::from(reads));
            assert_eq!(send_count, sends);
            f.chat.words = Box::from(*b"Again");
            chat_act(&mut f.chat, &f.client, act.clone(), &mut f.out);
            assert_eq!(f.chat.to_end(), second_end, "{act:?}: a press requests the end; a duplicate send stays");
            assert!(f.out.is_empty(), "an outstanding read or send is not duplicated");
        }
    }
}
#[test]
fn an_empty_page_with_more_reads_on() {
    let (mut chat, client, mut out) = reading_chat(&fixtures::limits());
    let older = empty_activity().before;
    chat.told(
        Time::ZERO,
        &client,
        Told::Page { handle: chat.activity, page: activity_answer(empty_activity()) },
        &mut out,
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle: chat.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: older }),
        }))
    );
    assert!(out.is_empty());
}
#[test]
fn a_failed_older_keeps_the_page_and_can_be_retried() {
    use jig_client::thread::window::End;
    let (mut chat, client, mut out) = reading_chat(&fixtures::limits());
    window_page(&mut chat, &client, &mut out, 0, WINDOW_PAGES);
    let read = out.pop().expect("initial older read");
    let rows = chat.thread.lines.rows.clone();
    let edges = chat.thread.lines.edges.clone();
    chat.words = Box::from(*b"Keep these words");
    chat.mode = crate::pages::chat::document::Mode::Split;
    chat.shown = Some(1);
    chat.told(
        Time::ZERO,
        &client,
        Told::Page { handle: chat.activity, page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) },
        &mut out,
    );
    assert_eq!(chat.thread.read_failed, Some(End::After));
    assert_eq!(chat.thread.lines.rows, rows);
    assert_eq!(chat.thread.lines.edges, edges);
    assert_eq!(chat.words.as_ref(), b"Keep these words");
    assert_eq!(chat.mode, crate::pages::chat::document::Mode::Split);
    assert_eq!(chat.shown, Some(1));
    assert_eq!(chat.to_end(), 0);
    assert!(out.is_empty());
    chat_act(&mut chat, &client, Act::Older, &mut out);
    assert_eq!(out.pop(), Some(read));
    assert_eq!(chat.thread.read_failed, None);
}

#[test]
fn a_send_that_sends_nothing_leaves_the_page_at_its_place() {
    use jig_client::thread::window::Edge;
    for (words, admission) in
        [(b"".as_slice(), c::Admission::Allowed), (b"Next".as_slice(), c::Admission::Refused(c::Refusal::NotOffered))]
    {
        let mut f = send_ready(words, admission);
        f.chat.thread.lines.edges.before = Edge::Dropped;
        chat_act(&mut f.chat, &f.client, Act::Send, &mut f.out);
        assert_eq!(f.chat.to_end(), 0);
        assert_eq!(f.chat.words.as_ref(), words);
        assert!(f.out.is_empty());
    }
}

fn send_ready(words: &[u8], admission: c::Admission) -> f::Fixture {
    let mut f = f::with_delegate();
    let mut task = f::task();
    task.acts = fixtures::acts::offered(c::ActKind::Message, admission);
    f.chat.final_task = Some(Box::new(task));
    f.chat.thread.reading = None;
    f.chat.words = words.into();
    f
}

#[test]
fn an_ended_read_leaves_latest_free() {
    use jig_client::thread::window::{Edge, End};
    let (mut chat, client, mut out) = reading_chat(&fixtures::limits());
    window_page(&mut chat, &client, &mut out, 0, WINDOW_PAGES);
    assert!(out.pop().is_some(), "initial older read");
    chat.thread.lines.edges.before = Edge::Dropped;
    chat.told(
        Time::ZERO,
        &client,
        Told::Page {
            handle: chat.activity,
            page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Malformed)),
        },
        &mut out,
    );
    assert_eq!(chat.thread.read_failed, Some(End::After));
    assert_eq!(chat.next_deadline(), None);
    chat.fire(Time::ZERO.saturating_add(Duration::from_secs(120)), &mut out);
    assert!(out.is_empty(), "an undecodable answer is not asked again by time alone");
    chat_act(&mut chat, &client, Act::Latest, &mut out);
    assert_eq!(out.pop(), Some(first_activity(&chat)), "Latest asks immediately after a lasting end");
    assert!(out.is_empty());
}

// client/pages/README.md: the person's placed stop and its independently named revision.
const PAUSE_REVISION: u64 = 19;
const PAUSE_WORDS: &[u8] = b"And then?";
fn paused_task() -> c::Task {
    let mut task = f::task();
    f::phase(&mut task, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: PAUSE_REVISION });
    task.chip.details.details.hold = Some(c::ChipHold {
        placed: Some(c::PlacedHold {
            number: 1,
            owner: c::HoldOwner::Party { person: fixtures::PARTY },
            mode: c::HoldMode::Stop,
            scope: c::HoldScope::Subtree { task: f::TASK },
            reason: Box::default(),
            placed_at: skein_lib::Wall::EPOCH,
            revision: c::Revision { value: 2 },
            lifted: false,
            may_lift: true,
        }),
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: PAUSE_REVISION,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    });
    task.acts = skein_lib::List::with_capacity(2);
    for act in [c::ActKind::Message, c::ActKind::Release] {
        task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    task
}
fn pause_fixture(task: c::Task) -> f::Fixture {
    let mut f = f::with_delegate();
    f::deliver(&mut f.client, f.watch, task);
    f.chat.words = PAUSE_WORDS.into();
    f
}
fn pause_act(f: &mut f::Fixture, act: Act) -> Box<[c::Ask]> {
    f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, act, &mut { SERIAL }, &mut f.out);
    fixtures::acts::sent(&mut f.out)
}
#[test]
fn resume_on_a_paused_chat_releases_at_its_hold_revision() {
    let mut f = pause_fixture(paused_task());
    assert_eq!(
        pause_act(&mut f, Act::Resume).as_ref(),
        [c::Ask::Release {
            project: fixtures::PROJECT,
            task: f::TASK,
            revision: c::Revision { value: PAUSE_REVISION },
            again: None,
        }]
    );
    assert_eq!(f.chat.pending.len(), 1);
    assert!(pause_act(&mut f, Act::Resume).is_empty(), "one resume in flight");
}
#[test]
fn resume_requires_an_own_pause_and_release_offered() {
    let mut unavailable = paused_task();
    unavailable.acts = fixtures::acts::offered(c::ActKind::Message, c::Admission::Allowed);
    let mut foreign = paused_task();
    foreign.chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().owner =
        c::HoldOwner::Task { task: f::TASK };
    for task in [unavailable, foreign, f::task()] {
        let mut f = pause_fixture(task);
        assert!(pause_act(&mut f, Act::Resume).is_empty());
    }
}
#[test]
fn a_message_to_a_paused_chat_sends_its_words_and_release_together() {
    let mut f = pause_fixture(paused_task());
    let sent = pause_act(&mut f, Act::Send);
    let [c::Ask::Compound { project, operations }] = sent.as_ref() else { panic!("one compound: {sent:?}") };
    assert_eq!(*project, fixtures::PROJECT);
    assert_eq!(
        operations.as_slice(),
        [
            c::Operation::Message {
                project: fixtures::PROJECT,
                task: f::TASK,
                words: fixtures::acts::text(PAUSE_WORDS)
            },
            c::Operation::Release {
                project: fixtures::PROJECT,
                task: f::TASK,
                revision: c::Revision { value: PAUSE_REVISION },
                again: None
            },
        ]
    );
    assert_eq!(f.chat.pending.len(), 1);
    let (key, kind) = *f.chat.pending.get(0).unwrap();
    assert_eq!(kind, Sending::Message);
    let words = f.chat.thread.words.rows.get(0).unwrap();
    assert_eq!((words.request, words.place, words.state), (Some(key), 0, jig_client::thread::Words::BeingSent));
    assert!(f.chat.words.is_empty());
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Answered {
            key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::Compound { project: fixtures::PROJECT, operations: 2, first_message: 41 },
            )))),
        },
        &mut f.out,
    );
    assert_eq!(f.chat.thread.words.rows.get(0).unwrap().state, jig_client::thread::Words::Sent { message: 41 });
}
#[test]
fn a_message_to_a_chat_held_otherwise_is_words_alone() {
    let mut task = paused_task();
    task.chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().owner =
        c::HoldOwner::Task { task: f::TASK };
    let mut f = pause_fixture(task);
    assert_eq!(
        pause_act(&mut f, Act::Send).as_ref(),
        [c::Ask::Message { project: fixtures::PROJECT, task: f::TASK, words: fixtures::acts::text(PAUSE_WORDS) }]
    );
}
#[test]
fn a_refused_message_to_a_paused_chat_stays_in_the_composer_s_refusal() {
    let mut f = pause_fixture(paused_task());
    pause_act(&mut f, Act::Send);
    let key = f.chat.pending.get(0).unwrap().0;
    let outcome = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::NotOffered,
        at: Some(1),
    }));
    f.chat.told(Time::ZERO, &f.client, Told::Answered { key, outcome: outcome.clone() }, &mut f.out);
    assert!(f.chat.thread.words.rows.is_empty());
    assert_eq!(f.chat.outcome_kind, Some(Sending::Message));
    assert_eq!(f.chat.outcome, Some(outcome));
    assert_eq!(f.chat.task(&f.client).unwrap().chip.phase, c::Phase::Held);
    assert!(fixtures::acts::sent(&mut f.out).is_empty());
}

#[test]
fn paused_is_the_person_s_own_stop_alone() {
    let me = fixtures::chats::me();
    let chip = paused_task().chip;
    let mut foreign = chip.clone();
    foreign.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().owner =
        c::HoldOwner::Party { person: fixtures::PARTY + 1 };
    let mut ancestor = chip.clone();
    ancestor.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().scope =
        c::HoldScope::Subtree { task: f::TASK + 1 };
    let mut lifted = chip.clone();
    lifted.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().lifted = true;
    let mut drain = chip.clone();
    drain.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().mode = c::HoldMode::Drain;
    let mut exceeded = chip.clone();
    exceeded.details.details.hold.as_mut().unwrap().placed = None;
    exceeded.details.phase = c::ChipPhase::Held {
        reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: 1 }),
        revision: PAUSE_REVISION,
    };
    for (name, chip, expected) in [
        ("own stop", chip, true),
        ("another person", foreign, false),
        ("ancestor", ancestor, false),
        ("lifted", lifted, false),
        ("drain", drain, false),
        ("exceeded", exceeded, false),
        ("live", f::task().chip, false),
    ] {
        assert_eq!(crate::pages::chat::paused(&chip, &me), expected, "{name}");
    }
}

fn making_page(f: &mut f::Fixture, number: u64) {
    making_lines(f, &[number]);
}
fn making_lines(f: &mut f::Fixture, numbers: &[u64]) {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(numbers.len()).unwrap());
    for (index, number) in numbers.iter().enumerate() {
        rows.push(owner_activity(
            u64::try_from(index).unwrap().checked_add(1).unwrap(),
            c::Activity::Decided { revision: 1, proposal: Some(c::DecidedProposal { number: *number, made: true }) },
        ))
        .unwrap();
    }
    f.chat.thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let handle = f.chat.activity;
    card_page(f, Time::ZERO, handle, activity_answer(c::ActivityPage { rows, before: None, after: None, cut: false }));
}
#[test]
fn a_making_line_asks_for_its_proposal() {
    let mut f = f::opened();
    making_page(&mut f, 1);
    let first = reads(&mut f.out);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].1, c::ProposalKey { proposer: c::Proposer::Task(f::TASK), number: 1 });
}
fn read_making_card(f: &mut f::Fixture) -> (jig_client::Handle, c::ProposalKey) {
    making_page(f, 1);
    reads(&mut f.out)[0]
}
#[test]
fn a_second_page_with_the_line_asks_once() {
    let mut f = f::opened();
    let (handle, _) = read_making_card(&mut f);
    making_page(&mut f, 1);
    assert!(reads(&mut f.out).is_empty());
    assert_eq!(f.chat.cards.get(0).unwrap().handle, handle);
}
#[test]
fn a_decided_card_stays_when_the_document_arrives_after_its_line() {
    let mut f = f::opened();
    let (handle, key) = read_making_card(&mut f);
    card_page(&mut f, Time::ZERO, handle, accepted_proposal(key, skein_lib::Wall::EPOCH));
    proposals(&mut f, &[]);
    making_page(&mut f, 1);
    assert!(reads(&mut f.out).is_empty(), "document and line must not restart the read");
    assert!(f.chat.departures().is_empty());
    assert_eq!(f.chat.cards.get(0).unwrap().handle, handle);
}
#[test]
fn a_gone_proposal_has_no_card_and_is_not_asked_again() {
    let mut f = f::opened();
    let (handle, _) = read_making_card(&mut f);
    card_page(&mut f, Time::ZERO, handle, jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Gone)));
    proposals(&mut f, &[]);
    making_page(&mut f, 1);
    assert!(reads(&mut f.out).is_empty());
    assert_eq!(f.chat.cards.len(), 1, "remember the answered read");
    assert!(f.chat.cards.get(0).unwrap().page.is_none(), "no drawable page");
}
#[test]
fn a_full_list_lets_a_decided_card_out_of_the_window_go() {
    let key = c::ProposalKey { proposer: c::Proposer::Task(f::TASK), number: 1 };
    for answer in [
        accepted_proposal(key, skein_lib::Wall::EPOCH),
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Gone)),
    ] {
        let mut f = f::opened();
        f.chat.cards = skein_lib::List::with_capacity(1);
        let (handle, _) = read_making_card(&mut f);
        card_page(&mut f, Time::ZERO, handle, answer);
        making_lines(&mut f, &[1, 2]);
        assert!(reads(&mut f.out).is_empty(), "a card whose making line stands keeps its place at the bound");
        making_page(&mut f, 2);
        let second = reads(&mut f.out);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].1.number, 2);
        assert_eq!(f.chat.cards.len(), 1);
        assert_eq!(f.chat.cards.get(0).unwrap().handle, second[0].0);
        assert_ne!(second[0].0, handle);
    }
}

fn composer_fixture(limits: &crate::Limits) -> f::Fixture {
    let mut f = send_ready(b"", c::Admission::Allowed);
    let task = f.chat.final_task.take();
    f.chat = Chat::new(f::TASK, crate::pages::chat::document::Mode::Chat, f::FIRST, limits, &mut Queue::uncapped());
    f.chat.final_task = task;
    super::composer::deliver_me(&mut f);
    f
}
fn composer_act(f: &mut f::Fixture, limits: &crate::Limits, act: Act) {
    f.chat.act(&fixtures::env(limits), &f.client, act, &mut { SERIAL }, &mut f.out);
}
#[test]
fn a_kilobyte_pasted_is_sent_whole() {
    let limits = super::composer::limits();
    let mut f = composer_fixture(&limits);
    let words = super::composer::words(super::composer::BOUND);
    composer_act(&mut f, &limits, Act::MessageWords { words: words.clone() });
    assert_eq!(f.chat.words, words);
    composer_act(&mut f, &limits, Act::Send);
    assert_eq!(
        fixtures::acts::sent(&mut f.out).as_ref(),
        [c::Ask::Message {
            project: fixtures::PROJECT,
            task: f::TASK,
            words: c::Text::new(words, super::composer::HOLD).unwrap()
        }]
    );
    assert!(f.chat.words.is_empty());
}
#[test]
fn a_paste_past_the_bound_stays_and_sends_nothing() {
    let limits = super::composer::limits();
    for &(typed, kept, beyond, excess) in super::composer::PAST {
        let mut f = composer_fixture(&limits);
        composer_act(&mut f, &limits, Act::MessageWords { words: super::composer::words(typed) });
        let words = super::composer::words(kept);
        assert_eq!(f.chat.words, words);
        assert_eq!(f.chat.beyond, beyond);
        composer_act(&mut f, &limits, Act::Send);
        assert!(fixtures::acts::sent(&mut f.out).is_empty());
        assert!(f.chat.pending.is_empty());
        assert!(f.chat.thread.words.rows.is_empty());
        assert_eq!(f.chat.words, words);
        assert_eq!(crate::pages::composer::over(&f.chat.words, f.chat.beyond, super::composer::BOUND), excess);
        composer_act(&mut f, &limits, Act::MessageWords { words: Box::from(*b"short") });
        assert_eq!(f.chat.beyond, 0);
        composer_act(&mut f, &limits, Act::Send);
        assert_eq!(fixtures::acts::sent(&mut f.out).len(), 1);
    }
}

#[test]
fn a_send_the_thread_refuses_keeps_the_words_and_says_so() {
    let limits = super::composer::limits();
    let mut f = composer_fixture(&limits);
    f.chat.thread = jig_client::thread::Thread::new(
        f::TASK,
        &jig_client::thread::Limits { lines: 1, bytes: 4, text: super::composer::HOLD, live: super::composer::HOLD },
    );
    composer_act(&mut f, &limits, Act::MessageWords { words: Box::from(*b"Hello") });
    composer_act(&mut f, &limits, Act::Send);
    assert_eq!(f.chat.words.as_ref(), b"Hello");
    assert!(fixtures::acts::sent(&mut f.out).is_empty());
    assert!(f.chat.pending.is_empty());
    assert!(f.chat.thread.words.rows.is_empty());
    assert_eq!(f.chat.outcome, Some(jig_client::Outcome::Limit));
    assert_eq!(f.chat.outcome_kind, Some(Sending::Message));
}

fn paused_composer_fixture(limits: &crate::Limits) -> f::Fixture {
    let mut f = composer_fixture(limits);
    f.chat.final_task = None;
    f::deliver(&mut f.client, f.watch, paused_task());
    f
}
#[test]
fn a_paused_chat_sends_a_kilobyte_and_release_together() {
    let limits = super::composer::limits();
    let mut f = paused_composer_fixture(&limits);
    let words = super::composer::words(super::composer::BOUND);
    composer_act(&mut f, &limits, Act::MessageWords { words: words.clone() });
    assert_eq!(f.chat.words, words);
    composer_act(&mut f, &limits, Act::Send);
    let sent = fixtures::acts::sent(&mut f.out);
    let [c::Ask::Compound { project, operations }] = sent.as_ref() else { panic!("one compound: {sent:?}") };
    assert_eq!(*project, fixtures::PROJECT);
    assert_eq!(
        operations.as_slice(),
        [
            c::Operation::Message {
                project: fixtures::PROJECT,
                task: f::TASK,
                words: c::Text::new(words, super::composer::HOLD).unwrap(),
            },
            c::Operation::Release {
                project: fixtures::PROJECT,
                task: f::TASK,
                revision: c::Revision { value: PAUSE_REVISION },
                again: None,
            },
        ]
    );
    assert_eq!(f.chat.pending.len(), 1);
    assert_eq!(f.chat.thread.words.rows.len(), 1, "compound words wait to be read back");
    assert!(f.chat.words.is_empty());
    assert_eq!(f.chat.beyond, 0);
}
#[test]
fn a_paused_chat_past_the_bound_keeps_its_words_without_releasing() {
    let limits = super::composer::limits();
    for &(typed, kept, beyond, excess) in super::composer::PAST {
        let mut f = paused_composer_fixture(&limits);
        composer_act(&mut f, &limits, Act::MessageWords { words: super::composer::words(typed) });
        composer_act(&mut f, &limits, Act::Send);
        assert!(fixtures::acts::sent(&mut f.out).is_empty(), "no message or release past the bound");
        assert!(f.chat.pending.is_empty());
        assert!(f.chat.thread.words.rows.is_empty());
        assert_eq!(f.chat.words, super::composer::words(kept));
        assert_eq!(f.chat.beyond, beyond);
        assert_eq!(crate::pages::composer::over(&f.chat.words, f.chat.beyond, super::composer::BOUND), excess);
        composer_act(&mut f, &limits, Act::MessageWords { words: Box::from(*b"short") });
        assert_eq!(f.chat.beyond, 0);
        composer_act(&mut f, &limits, Act::Send);
        let sent = fixtures::acts::sent(&mut f.out);
        let [c::Ask::Compound { .. }] = sent.as_ref() else { panic!("one compound: {sent:?}") };
    }
}
#[test]
fn a_paused_chat_reports_thread_refusal_and_keeps_its_words() {
    let limits = super::composer::limits();
    let mut f = paused_composer_fixture(&limits);
    f.chat.thread = jig_client::thread::Thread::new(
        f::TASK,
        &jig_client::thread::Limits { lines: 1, bytes: 4, text: super::composer::HOLD, live: super::composer::HOLD },
    );
    composer_act(&mut f, &limits, Act::MessageWords { words: Box::from(*b"Hello") });
    composer_act(&mut f, &limits, Act::Send);
    assert_eq!(f.chat.words.as_ref(), b"Hello");
    assert_eq!(f.chat.beyond, 0);
    assert!(fixtures::acts::sent(&mut f.out).is_empty(), "thread refusal cannot release the chat");
    assert!(f.chat.pending.is_empty());
    assert!(f.chat.thread.words.rows.is_empty());
    assert_eq!(f.chat.outcome, Some(jig_client::Outcome::Limit));
    assert_eq!(f.chat.outcome_kind, Some(Sending::Message));
}

fn request_fixture(cut: bool) -> f::Fixture {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let mut task = f::task();
    task.spec.specification.words = f::document_text(b"Kept");
    task.spec.specification.words.cut = cut;
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    f
}
fn request_read(out: &mut Queue<Request>, from: u64) -> jig_client::Handle {
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Read {
            handle,
            read: Readable::Jig(c::Read::TaskText { task: f::TASK, field: c::TaskTextField::Specification, from: at }),
        }) = request
        {
            assert_eq!(at, from);
            return handle;
        }
    }
    panic!("a cut request reads its specification from {from}")
}
fn request_piece(f: &mut f::Fixture, handle: jig_client::Handle, from: u64, bytes: &[u8], next: Option<u64>) {
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(Box::new(c::TaskText {
                task: f::TASK,
                field: c::TaskTextField::Specification,
                from,
                total: 11,
                next,
                text: fixtures::acts::text(bytes),
            })))),
        },
        &mut f.out,
    );
}
#[test]
fn a_cut_request_is_read_whole() {
    let mut f = request_fixture(true);
    assert_eq!(f.chat.request(&f.client), Some((b"Kept".as_slice(), false)));
    let handle = request_read(&mut f.out, 0);
    request_piece(&mut f, handle, 0, b"Kept ", Some(5));
    assert_eq!(f.chat.request(&f.client), Some((b"Kept ".as_slice(), false)));
    assert_eq!(request_read(&mut f.out, 5), handle);
    request_piece(&mut f, handle, 5, b"whole!", None);
    assert_eq!(f.chat.request(&f.client), Some((b"Kept whole!".as_slice(), true)));
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert!(f.out.is_empty(), "a repeated task delivery does not restart the read");
    f.chat.leave(&mut f.out);
    assert!(f::handles(&mut f.out).contains(&handle), "leaving releases the read handle");
}
#[test]
fn an_uncut_request_asks_nothing() {
    let f = request_fixture(false);
    assert_eq!(f.chat.request(&f.client), Some((b"Kept".as_slice(), true)));
    assert!(f.out.is_empty());
}
#[test]
fn a_request_read_that_ends_keeps_what_it_has() {
    let mut f = request_fixture(true);
    let handle = request_read(&mut f.out, 0);
    request_piece(&mut f, handle, 0, b"Kept ", Some(5));
    request_read(&mut f.out, 5);
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page { handle, page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Gone)) },
        &mut f.out,
    );
    assert_eq!(f.chat.request(&f.client), Some((b"Kept ".as_slice(), false)));
    assert_eq!(f.chat.next_deadline(), None);
    f.chat.fire(Time::from_nanos(60_000_000_000), &mut f.out);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert!(f.out.is_empty(), "a lasting end never restarts");
}

#[test]
fn a_transient_request_end_retries_with_activitys_waits() {
    let mut f = request_fixture(true);
    let handle = request_read(&mut f.out, 0);
    request_piece(&mut f, handle, 0, b"Kept ", Some(5));
    request_read(&mut f.out, 5);
    let mut now = Time::ZERO;
    for (why, delay) in [(c::ReadNone::Busy, 1), (c::ReadNone::Unreachable, 2)] {
        f.chat.told(
            now,
            &f.client,
            Told::Page { handle, page: jig_client::Readout::Jig(c::ReadResult::None(why)) },
            &mut f.out,
        );
        let due = now.saturating_add(Duration::from_secs(delay));
        assert_eq!(f.chat.next_deadline(), Some(due));
        assert_eq!(f.chat.request(&f.client), Some((b"Kept ".as_slice(), false)));
        f.chat.fire(Time::from_nanos(due.as_nanos() - 1), &mut f.out);
        assert!(f.out.is_empty());
        f.chat.fire(due, &mut f.out);
        assert_eq!(request_read(&mut f.out, 5), handle);
        now = due;
    }
    request_piece(&mut f, handle, 5, b"whole!", None);
    assert_eq!(f.chat.next_deadline(), None);
    assert_eq!(f.chat.request(&f.client), Some((b"Kept whole!".as_slice(), true)));
}

#[test]
fn a_request_rejects_pieces_that_do_not_follow() {
    let good = c::TaskText {
        task: f::TASK,
        field: c::TaskTextField::Specification,
        from: 5,
        total: 11,
        next: None,
        text: fixtures::acts::text(b"whole!"),
    };
    for (rule, piece) in [
        ("task", c::TaskText { task: f::TASK + 1, ..good.clone() }),
        ("field", c::TaskText { field: c::TaskTextField::Instructions, ..good.clone() }),
        ("overlap", c::TaskText { from: 4, total: 11, next: Some(10), ..good.clone() }),
        ("gap", c::TaskText { from: 6, text: fixtures::acts::text(b"hole!"), ..good.clone() }),
        ("changed total", c::TaskText { total: 12, next: Some(11), ..good.clone() }),
        ("past total", c::TaskText { text: fixtures::acts::text(b"too long"), ..good.clone() }),
        ("wrong next", c::TaskText { text: fixtures::acts::text(b"whole"), next: Some(9), ..good.clone() }),
        ("no progress", c::TaskText { text: fixtures::acts::text(b""), next: Some(5), ..good.clone() }),
        ("next at total", c::TaskText { next: Some(11), ..good.clone() }),
        ("unfinished end", c::TaskText { text: fixtures::acts::text(b"whole"), ..good.clone() }),
        ("overflow", c::TaskText { from: u64::MAX, ..good.clone() }),
    ] {
        let mut f = request_fixture(true);
        let handle = request_read(&mut f.out, 0);
        request_piece(&mut f, handle, 0, b"Kept ", Some(5));
        request_read(&mut f.out, 5);
        f.chat.told(
            Time::ZERO,
            &f.client,
            Told::Page {
                handle,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(Box::new(piece)))),
            },
            &mut f.out,
        );
        assert_eq!(f.chat.request(&f.client), Some((b"Kept ".as_slice(), false)), "{rule}");
        assert_eq!(f.chat.next_deadline(), None, "{rule}");
        request_piece(&mut f, handle, 5, b"whole!", None);
        assert_eq!(f.chat.request(&f.client), Some((b"Kept ".as_slice(), false)), "{rule}: stopped");
        assert!(f.out.is_empty(), "{rule}");
    }
}

#[test]
fn a_request_past_long_text_stops_before_keeping_a_piece() {
    let mut f = request_fixture(true);
    let handle = request_read(&mut f.out, 0);
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::TaskText(Box::new(c::TaskText {
                task: f::TASK,
                field: c::TaskTextField::Specification,
                from: 0,
                total: u64::from(fixtures::limits().client.domain.long_text) + 1,
                next: Some(5),
                text: fixtures::acts::text(b"Other"),
            })))),
        },
        &mut f.out,
    );
    assert_eq!(f.chat.request(&f.client), Some((b"Kept".as_slice(), false)));
    assert_eq!(f.chat.next_deadline(), None);
    assert!(f.out.is_empty());
}

#[test]
fn a_terminal_chat_reads_the_question_it_waits_on() {
    use crate::pages::{self, terminal};
    for terminal in [false, true] {
        for wait in [0_u8, 1, 2] {
            let mut fixture = f::with_delegate();
            let mut task = f::task();
            f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
            task.run_wait = None;
            task.waits_on = match wait {
                0 => c::WaitsOn::Party {
                    whom: c::Whom::Person(fixtures::chats::me().party.number),
                    entries: 1,
                    task: f::TASK,
                    kind: c::WaitedFor::Question,
                },
                1 => c::WaitsOn::Delegates,
                2 => c::WaitsOn::Nothing,
                _ => unreachable!(),
            };
            f::deliver(&mut fixture.client, fixture.watch, task.clone());
            let mut pages = pages::Pages::new();
            let env = fixtures::env(&fixtures::limits());
            let mut out = Queue::uncapped();
            pages::step(
                &mut pages,
                &env,
                &fixture.client,
                pages::Event::Went { address: crate::Address::Chat { task: f::TASK } },
                &mut out,
            );
            if terminal {
                pages::step(
                    &mut pages,
                    &env,
                    &fixture.client,
                    pages::Event::Act { act: Act::Terminal(terminal::Act::Enter) },
                    &mut out,
                );
            }
            assert_eq!(pages.inbox().is_some(), terminal && wait == 0, "terminal {terminal}, wait {wait}");
            let Some(inbox) = pages.inbox() else { continue };
            let entries = inbox.entries;
            let query = inbox.queries[0];
            assert!(pages.chat().unwrap().next >= inbox.next);
            while out.pop().is_some() {}
            for _ in 0_u8..2 {
                pages::step(
                    &mut pages,
                    &env,
                    &fixture.client,
                    pages::Event::Told { told: Told::Changed { handle: entries } },
                    &mut out,
                );
                assert_eq!(
                    out.pop(),
                    Some(Request::Client(Event::Read {
                        handle: query,
                        read: Readable::Jig(c::Read::Inbox {
                            group: c::InboxReadGroup::Needs,
                            project: Some(task.project),
                            kind: Some(c::InboxGroup::Questions),
                            goal: Some(f::TASK),
                            after: None,
                        }),
                    }))
                );
                assert!(out.is_empty());
            }
            pages::step(
                &mut pages,
                &env,
                &fixture.client,
                pages::Event::Act { act: Act::Terminal(terminal::Act::Leave) },
                &mut out,
            );
            assert!(pages.inbox().is_none());
            assert!(f::handles(&mut out).contains(&entries));
            pages::step(
                &mut pages,
                &env,
                &fixture.client,
                pages::Event::Act { act: Act::Terminal(terminal::Act::Enter) },
                &mut out,
            );
            let entries = pages.inbox().unwrap().entries;
            task.waits_on = c::WaitsOn::Nothing;
            f::deliver(&mut fixture.client, fixture.watch, task);
            let document = pages.chat().unwrap().document;
            pages::step(
                &mut pages,
                &env,
                &fixture.client,
                pages::Event::Told { told: Told::Changed { handle: document } },
                &mut out,
            );
            assert!(pages.inbox().is_none());
            assert!(f::handles(&mut out).contains(&entries));
        }
    }
}

#[test]
fn a_new_document_call_reads_again() {
    let mut f = f::opened();
    take_draft_reads(&mut f);
    draft_answer(&mut f, notes_page(&[]));
    document_call(
        &mut f,
        11,
        f::POSITION,
        b"document",
        "Note 12 revision 1 written · 64 lines\ndocument: \"Title\" written, 64 lines.",
    );
    assert_eq!(take_draft_reads(&mut f), 1);
    assert_eq!(
        f.chat.newest_write(),
        crate::pages::chat::document::note_write("Note 12 revision 1 written · 64 lines".as_bytes())
    );
}

const VIEW_ADDRESS: &[u8] = b"way Free";
const VIEW_TEXT: &[u8] = b"```view\nway Free\n```";
const VIEW_REVISION: u32 = 7;

fn view_reads(out: &mut Queue<Request>) -> skein_lib::List<(jig_client::Handle, Box<[u8]>)> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut reads = skein_lib::List::with_capacity(crate::Limits::chat_views());
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Read { handle, read: Readable::Family { scope, window, record } }) = request {
            assert_eq!(record.family, wire::FORGE);
            assert_eq!(record.version, wire::CLIENT_VERSION);
            let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
            let w::Read::View(view) = reading.into_parts().read else { panic!("chat reads a view") };
            assert_eq!(scope, c::Scope::Project(view.project()));
            assert_eq!(view.project(), fixtures::PROJECT);
            assert_eq!(window, c::Window { rows: 0, after: None });
            reads.push((handle, view.into_parts().address)).unwrap();
        }
    }
    reads
}
fn view_blocks(texts: &[&[u8]]) -> skein_lib::List<c::Block> {
    let mut blocks = skein_lib::List::with_capacity(u32::try_from(texts.len()).unwrap());
    for text in texts {
        blocks
            .push(c::Block::Text(c::Text::new(Box::from(*text), u32::try_from(text.len()).unwrap()).unwrap()))
            .unwrap();
    }
    blocks
}
fn show_view_text(f: &mut f::Fixture, texts: &[&[u8]], project: u32) {
    let mut task = f::task();
    task.project = project;
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle: f.chat.activity,
            page: f::activity_reply(c::Speaker::Assistant, view_blocks(texts), f::POSITION),
        },
        &mut f.out,
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks shared reads, their bound and their release")]
fn view_addresses_share_reads_and_release_on_departure() {
    use crate::pages::chat::Shown;
    struct Case<'a> {
        texts: &'a [&'a [u8]],
        draft: Option<&'a [u8]>,
        project: u32,
        expected: &'a [&'a [u8]],
        reads: u32,
        oldest: Option<&'a Shown>,
    }
    let bounded = numbered_views(0, 32);
    let overflow = numbered_views(0, 33);
    let newest = numbered_views(1, 33);
    let bounded_text = borrowed_view_bytes(&bounded.texts);
    let overflow_text = borrowed_view_bytes(&overflow.texts);
    let at_bound = borrowed_view_bytes(&bounded.addresses);
    let newest = borrowed_view_bytes(&newest.addresses);
    for (name, case) in [
        (
            "turn",
            Case {
                texts: &[VIEW_TEXT],
                draft: None,
                project: fixtures::PROJECT,
                expected: &[VIEW_ADDRESS],
                reads: 1,
                oldest: None,
            },
        ),
        (
            "twice",
            Case {
                texts: &[VIEW_TEXT, VIEW_TEXT],
                draft: None,
                project: fixtures::PROJECT,
                expected: &[VIEW_ADDRESS],
                reads: 1,
                oldest: None,
            },
        ),
        (
            "document",
            Case {
                texts: &[],
                draft: Some(VIEW_TEXT),
                project: fixtures::PROJECT,
                expected: &[VIEW_ADDRESS],
                reads: 1,
                oldest: None,
            },
        ),
        (
            "shared document",
            Case {
                texts: &[VIEW_TEXT],
                draft: Some(VIEW_TEXT),
                project: fixtures::PROJECT,
                expected: &[VIEW_ADDRESS],
                reads: 1,
                oldest: None,
            },
        ),
        (
            "no project",
            Case { texts: &[VIEW_TEXT], draft: None, project: 0, expected: &[VIEW_ADDRESS], reads: 0, oldest: None },
        ),
        (
            "at bound",
            Case {
                texts: bounded_text.as_slice(),
                draft: None,
                project: fixtures::PROJECT,
                expected: at_bound.as_slice(),
                reads: 32,
                oldest: Some(&Shown::Asking),
            },
        ),
        (
            "newest past bound",
            Case {
                texts: overflow_text.as_slice(),
                draft: None,
                project: fixtures::PROJECT,
                expected: newest.as_slice(),
                reads: 32,
                oldest: None,
            },
        ),
    ] {
        let mut f = f::opened();
        while f.out.pop().is_some() {}
        show_view_text(&mut f, case.texts, case.project);
        if let Some(body) = case.draft {
            draft_answer(&mut f, notes_page(&[draft_note(1, 1, body)]));
        }
        let reads = view_reads(&mut f.out);
        assert_eq!(reads.len(), case.reads, "{name}");
        for address in case.expected {
            assert_eq!(
                f.chat.view(address),
                Some(&if case.project == 0 { Shown::Failed } else { Shown::Asking }),
                "{name}"
            );
        }
        assert_eq!(f.chat.view(b"a0"), case.oldest, "{name}");
        f.chat.told(Time::ZERO, &f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
        assert!(view_reads(&mut f.out).is_empty(), "unchanged text: {name}");
        f.chat.leave(&mut f.out);
        let released = f::handles(&mut f.out);
        for (handle, _) in &reads {
            assert!(released.contains(handle), "{name}");
        }
        let mut unique = skein_lib::Map::with_capacity(crate::Limits::chat_views());
        for (_, address) in &reads {
            assert!(unique.insert(address.clone(), ()).unwrap().is_none(), "equal addresses share one read: {name}");
        }
    }
}

struct NumberedViews {
    texts: skein_lib::List<Box<[u8]>>,
    addresses: skein_lib::List<Box<[u8]>>,
}
fn numbered_views(first: u64, end: u64) -> NumberedViews {
    let count = u32::try_from(end.checked_sub(first).unwrap()).unwrap();
    let mut texts = skein_lib::List::with_capacity(count);
    let mut addresses = skein_lib::List::with_capacity(count);
    for number in first..end {
        texts.push(numbered_view_text(number)).unwrap();
        addresses.push(numbered_view_address(number)).unwrap();
    }
    NumberedViews { texts, addresses }
}
fn borrowed_view_bytes(values: &skein_lib::List<Box<[u8]>>) -> skein_lib::List<&[u8]> {
    let mut borrowed = skein_lib::List::with_capacity(values.len());
    for value in values {
        borrowed.push(value.as_ref()).unwrap();
    }
    borrowed
}

fn numbered_view_address(number: u64) -> Box<[u8]> {
    let digits = skein_lib::Decimal::of(number);
    let mut writer = skein_lib::Writer::new(digits.as_bytes().len().checked_add(1).unwrap());
    writer.put(b"a").unwrap();
    writer.put(digits.as_bytes()).unwrap();
    writer.finish()
}
fn numbered_view_text(number: u64) -> Box<[u8]> {
    let address = numbered_view_address(number);
    let mut writer = skein_lib::Writer::new(address.len().checked_add(b"```view\n\n```".len()).unwrap());
    writer.put(b"```view\n").unwrap();
    writer.put(&address).unwrap();
    writer.put(b"\n```").unwrap();
    writer.finish()
}

fn view_value() -> temper_client_protocol_wire::v1::View {
    use temper_client_protocol_wire::v1 as w;
    w::View::NotDrawn(
        w::NotDrawn::new(&w::CEILINGS, w::NotDrawnParts { words: b"Not drawn yet".as_slice().into() }).unwrap(),
    )
}
fn view_page() -> jig_client::Readout {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view: view_value() }).unwrap();
    jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) }
}
fn settings_watch(client: &mut jig_client::Client) -> skein_lib::Token {
    let mut out = Queue::uncapped();
    client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Watch { handle: crate::forge::ways::handle(), subject: Watchable::Jig(c::Subject::Settings) },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Settings) },
        } = request
        {
            return watch;
        }
    }
    panic!("settings watch opened");
}
fn view_settings(client: &mut jig_client::Client, watch: skein_lib::Token, newest: u32) {
    client.step_unlinked(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                    newest,
                    settings: skein_lib::List::with_capacity(0),
                    held: skein_lib::List::with_capacity(0),
                    pending: skein_lib::List::with_capacity(0),
                }))),
            },
        },
        &mut Queue::uncapped(),
    );
}

#[test]
fn view_answers_keep_drawings_until_a_new_revision_answers() {
    use crate::pages::chat::Shown;
    use temper_client_protocol_wire as wire;
    for (name, page, expected, refreshes) in [
        ("answer", view_page(), Shown::Drawn(Box::new(view_value())), 1),
        ("refused", jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), Shown::Failed, 0),
        (
            "malformed",
            jig_client::Readout::Family { family: wire::FORGE, record: Some(Box::from(*b"bad")) },
            Shown::Failed,
            0,
        ),
        ("absent", jig_client::Readout::Family { family: wire::FORGE, record: None }, Shown::Failed, 0),
        ("not served", jig_client::Readout::NotServed { family: Some(wire::FORGE), served: None }, Shown::Failed, 0),
    ] {
        let mut pages = crate::pages::Pages::new();
        let mut client = f::opened().client;
        crate::pages::step(
            &mut pages,
            &fixtures::env(&fixtures::limits()),
            &client,
            crate::pages::Event::Went { address: crate::Address::Chat { task: f::TASK } },
            &mut Queue::uncapped(),
        );
        let settings = settings_watch(&mut client);
        view_settings(&mut client, settings, VIEW_REVISION);
        chat_told(&mut pages, &client, Told::Changed { handle: crate::forge::ways::handle() });
        let document = pages.chat().unwrap().document;
        chat_told(
            &mut pages,
            &client,
            Told::Page {
                handle: document,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(f::task())))),
            },
        );
        let activity = pages.chat().unwrap().activity;
        let mut out = chat_told(
            &mut pages,
            &client,
            Told::Page {
                handle: activity,
                page: f::activity_reply(c::Speaker::Assistant, view_blocks(&[VIEW_TEXT]), f::POSITION),
            },
        );
        let reads = view_reads(&mut out);
        assert_eq!(reads.len(), 1, "{name}");
        let handle = reads.get(0).unwrap().0;
        chat_told(&mut pages, &client, Told::Page { handle, page });
        assert_eq!(pages.chat().unwrap().view(VIEW_ADDRESS), Some(&expected), "{name}");
        for revision in [VIEW_REVISION, VIEW_REVISION - 1, VIEW_REVISION + 1] {
            view_settings(&mut client, settings, revision);
            let mut out = chat_told(&mut pages, &client, Told::Changed { handle: crate::forge::ways::handle() });
            let reads = view_reads(&mut out);
            assert_eq!(
                reads.len(),
                if revision > VIEW_REVISION { refreshes } else { 0 },
                "{name}: revision {revision}"
            );
            assert_eq!(pages.chat().unwrap().view(VIEW_ADDRESS), Some(&expected), "keep the drawing: {name}");
        }
        for _ in 0..refreshes {
            chat_told(
                &mut pages,
                &client,
                Told::Page { handle, page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) },
            );
            assert_eq!(pages.chat().unwrap().view(VIEW_ADDRESS), Some(&Shown::Failed));
        }
    }
}

#[test]
fn transient_view_reads_retry_with_the_settings_waits() {
    use crate::pages::chat::Shown;
    for page in [
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)),
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
        jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut)),
        jig_client::Readout::Limit,
    ] {
        let mut f = f::opened();
        show_view_text(&mut f, &[VIEW_TEXT], fixtures::PROJECT);
        let reads = view_reads(&mut f.out);
        let handle = reads.get(0).unwrap().0;
        let mut now = Time::ZERO;
        for seconds in [1, 2, 4, 8, 16, 30, 30] {
            f.chat.told(now, &f.client, Told::Page { handle, page: page.clone() }, &mut f.out);
            let due = now.saturating_add(Duration::from_secs(seconds));
            assert_eq!(f.chat.next_deadline(), Some(due));
            assert_eq!(f.chat.view(VIEW_ADDRESS), Some(&Shown::Failed));
            f.chat.fire(now, &mut f.out);
            assert!(view_reads(&mut f.out).is_empty());
            f.chat.fire(due, &mut f.out);
            assert_eq!(view_reads(&mut f.out).as_slice(), [(handle, Box::from(VIEW_ADDRESS))]);
            now = due;
        }
        f.chat.told(now, &f.client, Told::Page { handle, page: view_page() }, &mut f.out);
        assert_eq!(f.chat.next_deadline(), None);
        assert_eq!(f.chat.view(VIEW_ADDRESS), Some(&Shown::Drawn(Box::new(view_value()))));
    }
}
