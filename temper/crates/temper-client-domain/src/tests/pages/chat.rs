use super::fixtures::{self, chat as f};
use crate::pages::{
    Act, Request,
    chat::{Chat, Sending},
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
            Sending::Stop | Sending::Close => panic!("only Seen and Message pending"),
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
fn a_chats_proposals_are_read_once_and_kept_in_the_tasks_order() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL, DELEGATE_PROPOSAL]);
    let first = reads(&mut f.out);
    assert_eq!(first.len(), 2);
    assert_eq!([first[0].1, first[1].1], [PROPOSAL, DELEGATE_PROPOSAL]);
    proposals(&mut f, &[DELEGATE_PROPOSAL, PROPOSAL]);
    assert!(reads(&mut f.out).is_empty(), "a retained proposal is not read again");
    assert_eq!(
        [f.chat.cards.get(FIRST_CARD).expect("first").handle, f.chat.cards.get(SECOND_CARD).expect("second").handle],
        [first[1].0, first[0].0],
        "delegate's proposal kept, ordered by the task"
    );
}
#[test]
fn a_proposal_decided_elsewhere_leaves_the_chats_card() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    let handle = reads(&mut f.out)[0].0;
    proposals(&mut f, &[]);
    assert!(f.chat.cards.is_empty());
    assert_eq!(f::handles(&mut f.out).as_ref(), [handle], "drop the card's handle too");
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
fn the_chat_composer_keeps_the_send_bound_when_receiving_grows() {
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
    f.chat.act(&fixtures::env(&limits), &f.client, Act::MessageWords { words: over }, &mut serial, &mut f.out);
    assert_eq!(f.chat.words, words, "an oversized edit keeps the previous words");
}

fn window_page(chat: &mut Chat, client: &jig_client::Client, out: &mut Queue<Request>, page: u32, pages: u32) {
    let mut rows = skein_lib::List::with_capacity(2);
    for offset in 0..2 {
        rows.push(crate::tests::documents::line(
            u64::from(pages.checked_sub(page).unwrap().checked_mul(2).unwrap().checked_sub(offset).unwrap()),
            skein_lib::Wall::EPOCH,
            c::Activity::RunBegan { attempt: 1, why: c::RunWhy::Made, resumed: false },
        ))
        .unwrap();
    }
    let cursor = c::Cursor::new(Box::from(*b"older"), 5).unwrap();
    chat.told(
        Time::ZERO,
        client,
        Told::Page {
            handle: chat.activity,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                before: if page.checked_add(1).unwrap() < pages { Some(cursor) } else { None },
                after: None,
                cut: false,
            })))),
        },
        out,
    );
}
#[test]
fn a_chat_fills_its_window_to_the_start_without_offering_older() {
    let limits = fixtures::limits();
    let client = jig_client::Client::unlinked(&limits.client, fixtures::SEED).unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut chat = Chat::new(f::TASK, crate::pages::chat::document::Mode::Chat, f::FIRST, &limits, &mut out);
    while out.pop().is_some() {}
    for page in 0..5 {
        window_page(&mut chat, &client, &mut out, page, 5);
        assert_eq!(!out.is_empty(), page < 4, "page {page}");
        while out.pop().is_some() {}
        assert!(!chat.offers_older());
    }
    assert_eq!(chat.thread.lines.rows.len(), 10);
    assert!(chat.thread.reading.is_none());
}
#[test]
fn a_full_chat_window_offers_older_and_reads_it_when_asked() {
    let limits = fixtures::limits();
    let client = jig_client::Client::unlinked(&limits.client, fixtures::SEED).unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut chat = Chat::new(f::TASK, crate::pages::chat::document::Mode::Chat, f::FIRST, &limits, &mut out);
    while out.pop().is_some() {}
    for page in 0..8 {
        window_page(&mut chat, &client, &mut out, page, 10);
        assert_eq!(!out.is_empty(), page < 7, "page {page}");
        while out.pop().is_some() {}
    }
    assert_eq!(chat.thread.lines.rows.len(), crate::pages::chat::LINES);
    assert!(chat.offers_older());
    let mut serial = SERIAL;
    chat.act(&fixtures::env(&limits), &client, Act::Older, &mut serial, &mut out);
    let Some(Request::Client(Event::Read { read: Readable::Jig(c::Read::Activity { before: Some(_), .. }), .. })) =
        out.pop()
    else {
        panic!("Older reads the preceding page");
    };
    window_page(&mut chat, &client, &mut out, 8, 10);
    assert!(out.is_empty());
    assert!(chat.offers_older());
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
    let mut call = f::delegate_call(number);
    call.tool = fixtures::acts::text(b"note");
    call.objects = skein_lib::List::with_capacity(0);
    call.outcome = Some(c::Text::new("Note 12 revision 1 written · 64 lines".as_bytes().into(), 64).unwrap());
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
    let mut f = f::opened();
    let mut serial = SERIAL;
    assert_eq!(f.chat.mode, Mode::Chat);
    for (act, mode, shown, count) in [
        (Act::ShowSection { line: 31 }, Mode::Split, Some(31), 1),
        (Act::Mode { mode: Mode::Draft }, Mode::Draft, Some(31), 1),
        (Act::ShowSection { line: 7 }, Mode::Draft, Some(7), 2),
        (Act::Mode { mode: Mode::Chat }, Mode::Chat, Some(7), 2),
    ] {
        f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, act, &mut serial, &mut f.out);
        assert_eq!((f.chat.mode, f.chat.shown, f.chat.shown_count), (mode, shown, count));
    }
}

fn start_over_pages() -> (crate::pages::Pages, jig_client::Client) {
    let mut f = f::with_delegate();
    let mut me = fixtures::chats::me();
    let project = me.projects.get_mut(0).unwrap();
    let mut configuring = project.templates.get(0).unwrap().clone();
    configuring.number = 3;
    let mut templates = skein_lib::List::with_capacity(2);
    templates.push(project.templates.get(0).unwrap().clone()).unwrap();
    templates.push(configuring).unwrap();
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
fn a_done_start_over_shows_configure() {
    let (mut pages, client) = start_over_pages();
    start_over_act(&mut pages, &client, Act::StartOver);
    start_over_act(&mut pages, &client, Act::Confirm);
    let mut out = answer_start_over(&mut pages, &client, seen_answer());
    assert_eq!(out.pop(), Some(Request::Show { address: crate::Address::Configure { project: fixtures::PROJECT } }));
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
    f.chat.told(
        Time::ZERO,
        &f.client,
        Told::Page { handle, page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Proposal(proposal))) },
        &mut f.out,
    );
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
                    action: c::RetainedAction::Settings(c::RetainedActionSettings {
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
    let mut limits = fixtures::limits();
    limits.client.domain.text = 128;
    f.chat.thread = jig_client::thread::Thread::new(f::TASK, &crate::pages::chat::thread_limits(&limits));
    f
}
#[test]
fn add_draft_sends_and_goes_to_split() {
    let mut f = draft_ready();
    f.chat.words = b"Unsent words".as_slice().into();
    let mut serial = SERIAL;
    for _ in 0_u32..2 {
        f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::AddDraft, &mut serial, &mut f.out);
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
    f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::AddDraft, &mut { SERIAL }, &mut f.out);
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
