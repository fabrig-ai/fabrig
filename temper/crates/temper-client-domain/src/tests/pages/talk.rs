//! The role's chat in a talk (client/pages/talk.md, 1–2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary fixture tests collect and inspect the fake root's requests"
)]

use super::{
    fixtures as f,
    welcome::{deliver, event, up},
};
use crate::{
    Action, Address, Domain, Event, Place, Request,
    pages::{self, Act, chat::Talking, team},
};
use jig_client::{Down, Key, Readable, Readout, Told, UnlinkedRequest, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token, Wall};

fn talk() -> Address {
    Address::Talk { project: f::PROJECT, task: f::chat::TASK }
}
fn go(root: &mut Domain, address: Address) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(address) })
}
fn act(root: &mut Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(act) })
}
fn open(address: Address) -> (Domain, Token, Queue<Request>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::with_capacity(0));
    deliver(&mut root, me, c::Delivery::Me(Box::new(f::chats::me())));
    let out = go(&mut root, address);
    let watch = find_watch(&out, c::Subject::Task { task: f::chat::TASK });
    (root, watch, out)
}
fn find_watch(out: &Queue<Request>, subject: c::Subject) -> Token {
    for request in out {
        if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch, subject: found } } } =
            request
            && found == &Watchable::Jig(subject.clone())
        {
            return *watch;
        }
    }
    panic!("watch {subject:?}");
}
fn read(out: &Queue<Request>, wanted: c::Read) -> Token {
    for request in out {
        if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Read { read, what } } } = request
            && what == &Readable::Jig(wanted.clone())
        {
            return *read;
        }
    }
    panic!("read {wanted:?}");
}
fn nows(out: &Queue<Request>) -> Vec<Token> {
    let mut reads = Vec::new();
    for request in out {
        if let Request::Unlinked {
            request: UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(c::Read::Notes { .. }) } },
        } = request
        {
            reads.push(*read);
        }
    }
    reads
}
fn now_read() -> c::Read {
    c::Read::Notes {
        scope: c::NoteScope::Task { project: f::PROJECT, task: f::chat::TASK },
        description: Some(f::acts::text(b"now")),
        after: None,
    }
}
fn answer(root: &mut Domain, read: Token, value: c::ReadValue) -> Queue<Request> {
    up(root, Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) })
}
fn document(root: &mut Domain, watch: Token, task: c::Task) -> Queue<Request> {
    deliver(root, watch, c::Delivery::Task(Box::new(task)))
}
fn role() -> c::Task {
    let mut task = f::chat::task();
    task.acts = f::acts::offered(c::ActKind::Talk, c::Admission::Allowed);
    task.talk = Some(c::Talk { party: f::PARTY, opened: Wall::EPOCH, last: Wall::EPOCH });
    task
}
fn talking(root: &Domain) -> &Talking {
    root.pages().chat().unwrap().talk().unwrap()
}
fn sends(out: &Queue<Request>) -> Vec<c::Ask> {
    let mut asks = Vec::new();
    for request in out {
        if let Request::Unlinked {
            request: UnlinkedRequest::Down { down: Down::Send { request: jig_client::Ask::Jig(ask), .. } },
        } = request
        {
            asks.push(*ask.clone());
        }
    }
    asks
}
fn told(root: &mut Domain, told: Told) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), told, &mut out);
    out
}

#[test]
fn talk_address_roundtrips_and_rejects_malformed_tasks() {
    let address = Address::Talk { project: 3, task: 7 };
    assert_eq!(crate::address::print(&address).as_ref(), b"/projects/3/talk/7");
    assert_eq!(crate::address::parse(b"/projects/3/talk/7"), Place::Address(address));
    assert_eq!(crate::address::parse(b"/projects/3/talk/x"), Place::Missing);
}

#[test]
fn talk_reads_chat_now_and_findings_and_refreshes_now_one_at_a_time() {
    let (mut root, watch, out) = open(talk());
    assert_eq!(root.pages().project(), Some(f::PROJECT));
    assert_eq!((talking(&root).project, talking(&root).task), (f::PROJECT, f::chat::TASK));
    let now_token = read(&out, now_read());
    assert_eq!(nows(&out), [now_token]);
    read(&out, c::Read::Activity { task: f::chat::TASK, before: None });
    assert!(
        out.iter().any(|request| {
            if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { subject, .. } } } = request {
                return Some(subject.clone()) == crate::forge::findings::watch(f::PROJECT, &f::limits().forge);
            }
            false
        }),
        "Talk's project findings are watched"
    );
    let mut task = role();
    let changed = document(&mut root, watch, task.clone());
    find_watch(&changed, c::Subject::Run { task: f::chat::TASK, turn: None, offset: 0 });
    assert!(nows(&changed).is_empty(), "initial document doesn't reread the opening Now");
    task.last_activity = Wall::from_nanos(1);
    assert!(nows(&document(&mut root, watch, task.clone())).is_empty(), "one Now in flight");
    let again = answer(&mut root, now_token, notes(Box::new([])));
    let now_token = read(&again, now_read());
    assert_eq!(nows(&again), [now_token]);
    answer(&mut root, now_token, notes(Box::new([])));
    assert!(nows(&document(&mut root, watch, task.clone())).is_empty(), "unchanged activity doesn't reread");
    task.last_activity = Wall::from_nanos(2);
    assert_eq!(nows(&document(&mut root, watch, task)).len(), 1);
    let closed = go(&mut root, Address::Accounts);
    let mut closing = false;
    for request in &closed {
        if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Close { .. } } } = request {
            closing = true;
        }
    }
    assert!(closing, "talk's watches close");
}

#[test]
fn talk_words_use_talk_permission_and_chat_words_still_use_message() {
    for (address, offered, expected) in [
        (talk(), c::ActKind::Talk, Some(true)),
        (talk(), c::ActKind::Message, None),
        (Address::Chat { task: f::chat::TASK }, c::ActKind::Message, Some(false)),
    ] {
        let (mut root, watch, _) = open(address);
        let mut task = role();
        task.acts = f::acts::offered(offered, c::Admission::Allowed);
        document(&mut root, watch, task);
        act(&mut root, Act::MessageWords { words: b"Hello".as_slice().into() });
        let out = act(&mut root, Act::Send);
        let asks = sends(&out);
        if let Some(is_talk) = expected {
            let words = f::acts::text(b"Hello");
            let ask = if is_talk {
                c::Ask::Talk { project: f::PROJECT, task: f::chat::TASK, words }
            } else {
                c::Ask::Message { project: f::PROJECT, task: f::chat::TASK, words }
            };
            assert_eq!(asks, [ask]);
            let chat = root.pages().chat().unwrap();
            assert!(chat.words.is_empty());
            assert_eq!(chat.thread.words.rows.len(), 1);
            let (key, _) = *chat.pending.get(0).unwrap();
            told(
                &mut root,
                Told::Answered {
                    key,
                    outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                        c::Outcome::Document(c::DocumentOutcome::Said { task: f::chat::TASK, message: 1 }),
                    ))),
                },
            );
            let chat = root.pages().chat().unwrap();
            assert!(chat.pending.is_empty());
            let mut sent = false;
            for line in &chat.thread.lines() {
                if let jig_client::thread::Line::Words {
                    state: jig_client::thread::Words::Sent { message: 1 }, ..
                } = line
                {
                    sent = true;
                }
            }
            assert!(sent);
        } else {
            assert!(asks.is_empty());
            assert_eq!(root.pages().chat().unwrap().words.as_ref(), b"Hello");
        }
    }
}

#[test]
fn leaving_sends_once_only_for_your_open_and_offered_talk_and_drops_the_answer() {
    for (party, offered, leaves) in [
        (Some(f::PARTY), true, true),
        (None, true, false),
        (Some(f::PARTY + 1), true, false),
        (Some(f::PARTY), false, false),
    ] {
        let (mut root, watch, _) = open(talk());
        let mut task = role();
        task.talk = party.map(|party| c::Talk { party, opened: Wall::EPOCH, last: Wall::EPOCH });
        task.acts = f::acts::offered(if offered { c::ActKind::Leave } else { c::ActKind::Talk }, c::Admission::Allowed);
        document(&mut root, watch, task);
        assert!(sends(&go(&mut root, talk())).is_empty(), "same page does not leave");
        let chat = root.pages().chat().unwrap();
        let next = chat.pending.len();
        let out = go(&mut root, Address::Accounts);
        assert_eq!(
            sends(&out),
            if leaves { vec![c::Ask::Leave { project: f::PROJECT, task: f::chat::TASK }] } else { vec![] }
        );
        assert!(sends(&go(&mut root, Address::Forges)).is_empty(), "a second navigation doesn't leave twice");
        let before = pages::shown(root.pages());
        assert_eq!(next, 0);
        // Leave has a Pages key, but no page owns its answer after navigation.
        let serial = root.pages.serial;
        told(
            &mut root,
            Told::Answered {
                key: Key { asker: crate::owner::Owner::Pages.asker(), number: serial },
                outcome: jig_client::Outcome::Unknown,
            },
        );
        assert_eq!(root.pages().address(), &Address::Forges);
        assert_eq!(pages::shown(root.pages()), before);
    }
}

fn activity(position: u64, line: c::Activity) -> c::DecodedActivityLine {
    c::DecodedActivityLine {
        source: Box::new(c::ActivityLine { task: f::chat::TASK, position, at: Wall::EPOCH, line }),
        decoded: None,
    }
}
fn ended(spent: u64) -> c::Activity {
    c::Activity::RunEnded { attempt: 1, end: c::RunEnd::Parked, spent }
}
#[test]
fn talking_facts_follow_the_reader_newest_opening_runs_budget_and_lowest_now() {
    let (mut root, watch, out) = open(talk());
    assert_eq!(
        (
            talking(&root).name(),
            talking(&root).open(),
            talking(&root).from(),
            talking(&root).cost(),
            talking(&root).spend(),
            talking(&root).now()
        ),
        (None, None, None, None, None, None)
    );
    let mut task = role();
    task.activity = 6;
    task.budget.numbers.spent = 22;
    task.budget.numbers.budget = 100;
    let expected = task.talk;
    let name = task.chip.title.bytes().to_vec();
    document(&mut root, watch, task.clone());
    assert_eq!(talking(&root).name(), Some(name.as_slice()));
    assert_eq!(talking(&root).open(), expected.as_ref());
    assert_eq!(talking(&root).spend(), Some((22, 100)));
    assert_eq!((talking(&root).from(), talking(&root).cost()), (None, None));
    let read = read(&out, c::Read::Activity { task: f::chat::TASK, before: None });
    answer(
        &mut root,
        read,
        c::ReadValue::Activity(Box::new(c::ActivityPage {
            rows: List::from_boxed(Box::new([
                activity(1, c::Activity::TalkOpened { party: f::PARTY }),
                activity(2, ended(50)),
                activity(3, c::Activity::TalkOpened { party: f::PARTY }),
                activity(4, ended(30)),
                activity(5, c::Activity::TalkOpened { party: f::PARTY + 1 }),
                activity(6, ended(12)),
            ])),
            before: None,
            after: None,
            cut: false,
        })),
    );
    assert_eq!((talking(&root).from(), talking(&root).cost()), (Some(3), Some(42)));
    let now = |number, description: &[u8], body: &[u8]| c::Note {
        number,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: f::PROJECT, task: f::chat::TASK },
        writer: c::NoteAuthor::Task { task: f::chat::TASK, attempt: 1 },
        acts: List::with_capacity(0),
        description: f::chat::document_text(description),
        body: f::chat::document_text(body),
        changed: Some(Wall::from_nanos(19)),
        tasks: List::with_capacity(0),
        cut: false,
    };
    answer(
        &mut root,
        read_now(&out),
        notes(Box::new([now(8, b"now", b"Later"), now(1, b"other", b"Not Now"), now(3, b"now", b"First")])),
    );
    assert_eq!(
        talking(&root).now(),
        Some(&team::Now { body: b"First".as_slice().into(), changed: Some(Wall::from_nanos(19)), cut: false })
    );
    assert_eq!(
        root.pages().chat().unwrap().request(root.client()),
        None,
        "a role's spec isn't its conversation's first entry"
    );
    task.talk.as_mut().unwrap().party = f::PARTY + 1;
    document(&mut root, watch, task.clone());
    assert_eq!((talking(&root).open(), talking(&root).from(), talking(&root).cost()), (None, None, None));
    task.talk = None;
    document(&mut root, watch, task);
    assert!(talking(&root).open().is_none());
    let (root, _, _) = open(Address::Chat { task: f::chat::TASK });
    assert!(root.pages().chat().unwrap().talk().is_none());
}
fn notes(rows: Box<[c::Note]>) -> c::ReadValue {
    c::ReadValue::Notes(Box::new(c::NotePage { rows: List::from_boxed(rows), before: None, after: None, cut: false }))
}
fn read_now(out: &Queue<Request>) -> Token {
    read(out, now_read())
}

#[test]
fn weekly_reads_only_an_open_seven_day_project_period() {
    const DAY: u64 = 86_400_000_000_000;
    for (days, closed, expected) in [(7, false, true), (6, false, false), (7, true, false)] {
        let (mut root, _, out) = open(talk());
        assert!(!team::weekly(&root, f::PROJECT));
        let read = read(&out, c::Read::Project { project: f::PROJECT });
        let mut project = super::team::project(Box::new([]), false);
        project.number = f::PROJECT;
        project.current_period = Some(c::PoolLedger {
            source: c::Funder::Period { project: f::PROJECT, period: 1 },
            parent: None,
            numbers: c::Numbers { budget: 100, spent: 10, spent_below: 0, reserved: 0 },
            left: 90,
            closed,
            began: Some(Wall::from_nanos(DAY)),
            next: Some(Wall::from_nanos((days + 1) * DAY)),
        });
        answer(&mut root, read, c::ReadValue::Project(Box::new(project)));
        assert_eq!(team::weekly(&root, f::PROJECT), expected);
        assert!(!team::weekly(&root, f::PROJECT + 1));
    }
}

#[test]
fn accepting_settings_on_a_talk_card_moves_runs_but_on_a_chat_stays() {
    for (address, running) in [(talk(), c::Running::Moves), (Address::Chat { task: f::chat::TASK }, c::Running::Stays)]
    {
        let (mut root, watch, _) = open(address);
        let key = f::acts::PROPOSAL;
        let mut task = role();
        task.proposals = List::from_boxed(Box::new([key]));
        let out = document(&mut root, watch, task);
        let read = read(&out, c::Read::Proposal { proposal: key });
        let c::ReadValue::Proposal(mut proposal) = f::acts::proposal(c::ActKind::Accept, c::Admission::Allowed) else {
            panic!("proposal fixture")
        };
        proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
            against: 0,
            items: List::with_capacity(0),
            review: None,
            made: None,
        }));
        answer(&mut root, read, c::ReadValue::Proposal(proposal));
        assert!(
            sends(&act(
                &mut root,
                Act::Decide { object: c::ObjectKey::Proposal(key), revision: None, act: c::ActKind::Accept }
            ))
            .is_empty()
        );
        let out = act(&mut root, Act::Confirm);
        assert_eq!(
            sends(&out),
            [c::Ask::Decide {
                project: f::PROJECT,
                on: c::DecideOn::Proposal { proposal: key, running: Some(running), decision: c::Decision::Accepted }
            }]
        );
    }
}
