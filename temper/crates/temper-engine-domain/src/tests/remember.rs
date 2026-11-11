//! Root memory calls write by place (temper/docs/design/domain/agent/ask-temper.md, 4).
use crate::route::{host_route, remember};
use crate::{CallAnswer as C, RememberAction as A, RememberDid as D, RememberRefusal as R, Remembered as Answer};
use jig_domain_core::{self as core, NamedAction};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};

const CHAT: u64 = 1;
const PERSON: u64 = 7;
const RIGHT: Token = Token::new(99);
const BOUND: u32 = 12;
const BYTES: u32 = 120;
const REVISION: u32 = 4;
const NAMES: [u64; 12] = [2, 5, 9, 13, 17, 21, 25, 29, 33, 37, 41, 45];
const TEXTS: [&[u8]; 12] =
    [b"one", b"two", b"three", b"four", b"five", b"six", b"seven", b"eight", b"nine", b"ten", b"eleven", b"twelve"];

fn scope() -> notes::Scope {
    notes::Scope::Party { party: PERSON }
}

fn lines(count: usize) -> Box<[crate::Record]> {
    let mut rows = List::with_capacity(BOUND);
    for (&name, text) in NAMES.iter().zip(TEXTS).take(count) {
        rows.push(crate::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            scope: scope(),
            name,
            description: text.into(),
            revision: REVISION,
        }))))
        .unwrap();
    }
    rows.into_boxed()
}

fn texts(items: &[&[u8]]) -> List<Box<[u8]>> {
    let mut lines = List::with_capacity(BOUND);
    for text in items {
        lines.push(Box::from(*text)).unwrap();
    }
    lines
}

fn note(name: u64, text: &[u8], references: List<u64>, recalled: Option<u32>) -> NamedAction {
    NamedAction::Note {
        entry: notes::New {
            name,
            scope: scope(),
            description: text.into(),
            body: Box::new([]),
            references,
            author: notes::Author::Task { task: 0, attempt: 0 },
        },
        recalled,
    }
}

fn root(requester: tasks::Party) -> (crate::Domain, Env<crate::Limits>, crate::CallKey) {
    let (source, _, task) = super::live_task(None, tasks::Executor::Procedure { connector: 0, code: 1 });
    let mut row = source.core.tasks.task(task).unwrap().clone();
    row.requester = requester;
    let funding = *source.core.tasks.funding(row.funder).unwrap();
    let mut limits = super::resume_fixture::limits();
    limits.loads.rows = BOUND;
    limits.notes.load_rows = BOUND;
    limits.notes.entries_per_scope = BOUND;
    limits.notes.party_entries = BOUND;
    limits.notes.party_bytes = BYTES;
    limits.notes.description_bytes = BYTES;
    limits.notes.references = 2;
    let mut records = List::uncapped();
    if let tasks::Party::Task(number) = requester {
        let mut parent = row.clone();
        parent.number = number;
        parent.root = number;
        parent.requester = tasks::Party::Deployment { project: 1 };
        parent.delegates = Box::new([row.number]);
        parent.made = 2;
        row.root = number;
        row.depth = 1;
        records.push(crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(parent))))).unwrap();
    }
    for record in [
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(row)))),
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(funding))),
        crate::Record::Core(core::Record::People(jig_domain_people::Stored::Person {
            number: PERSON,
            identity: jig_domain_people::Identity {
                key: jig_domain_people::IdentityKey { provider: 0, subject: b"person".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        })),
    ] {
        records.push(record).unwrap();
    }
    let (mut domain, env) =
        super::resume_fixture::root_with_records(super::resume_fixture::config(7), &limits, records.as_slice());
    let key = crate::CallKey { task, attempt: 1, completion: 1, position: 0 };
    domain
        .call_renders
        .insert(
            RIGHT,
            host_route::Flight {
                task,
                attempt: 1,
                key: Some(key),
                note_decoding: false,
                job: None,
                page: None,
                render: host_route::FlightRender::Plain,
                name: b"call".as_slice().into(),
                tool: b"remember".as_slice().into(),
            },
        )
        .unwrap();
    (domain, env, key)
}

fn call(domain: &mut crate::Domain, env: &Env<crate::Limits>, key: crate::CallKey, action: A) {
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    remember::call(domain, env, &mut decision, ReplyTo::new(RIGHT), key, action);
    assert!(decision.write_free(), "the root's load and checks keep no durable record");
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.core.counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
    if read_waiter(domain).is_some() {
        let mut out = Queue::uncapped();
        crate::decision::resume(&mut domain.journal, &mut out);
        let Some(crate::decision::JournalOutput::Deliver(crate::Delivery::Load { range, after, .. })) = out.pop()
        else {
            panic!("index load")
        };
        assert_eq!(
            range,
            crate::Range::Core(crate::CoreRange::Notes(notes::Range::Lines { scope: scope(), after: None }))
        );
        assert_eq!(after, None);
    }
}

fn read_waiter(domain: &crate::Domain) -> Option<Token> {
    for (id, read) in &domain.archive_reads {
        if let Some(crate::boundary::Read::Remember(_)) = read {
            return Some(id.token());
        }
    }
    None
}

fn waiter(domain: &crate::Domain) -> Token {
    read_waiter(domain).expect("remember index load")
}

fn load(domain: &mut crate::Domain, count: usize, split: bool) {
    let waiter = waiter(domain);
    let mut out = Queue::uncapped();
    let rows = lines(count);
    if split && count > 1 {
        let mut rows = rows.into_iter();
        let first = rows.next().unwrap();
        let mut tail = List::with_capacity(BOUND);
        for row in rows {
            tail.push(row).unwrap();
        }
        let next = crate::Key::Core(core::Key::Notes(notes::Key::Line { scope: scope(), name: NAMES[0] }));
        remember::loaded(domain, waiter, Box::new([first]), Some(next), None, &mut out);
        assert!(domain.work.is_empty(), "no write before the whole index");
        let Some(crate::Request::Store(crate::StoreRequest::Load { range, after: None, .. })) = out.pop() else {
            panic!("next index page")
        };
        assert_eq!(
            range,
            crate::Range::Core(crate::CoreRange::Notes(notes::Range::Lines { scope: scope(), after: Some(NAMES[0]) }))
        );
        remember::loaded(domain, waiter, tail.into_boxed(), None, None, &mut out);
    } else {
        remember::loaded(domain, waiter, rows, None, None, &mut out);
    }
    if let Some(crate::Request::Store(crate::StoreRequest::Load { range, .. })) = out.pop() {
        let crate::Range::Core(crate::CoreRange::Notes(notes::Range::Entry { name })) = range else {
            panic!("change entry")
        };
        let entry = notes::Entry {
            name,
            scope: scope(),
            description: b"two".as_slice().into(),
            body: Box::new([]),
            references: List::from_boxed(Box::new([77, 88])),
            author: notes::Author::Party { party: PERSON },
            revision: REVISION.checked_add(1).unwrap(),
            changed: Wall::EPOCH,
        };
        remember::loaded(
            domain,
            waiter,
            Box::new([crate::Record::Core(core::Record::Notes(notes::Record::Entry(entry)))]),
            None,
            None,
            &mut out,
        );
    }
    assert!(out.is_empty());
}

fn rendered(domain: &mut crate::Domain) -> C {
    let mut out = Queue::uncapped();
    crate::decision::resume(&mut domain.journal, &mut out);
    let Some(crate::decision::JournalOutput::Deliver(crate::Delivery::Host(host))) = out.pop() else {
        panic!("read answer")
    };
    let host_route::HostDelivery::Render { answer, .. } = *host else { panic!("read render") };
    answer
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the writes, their actions and joined answers")]
fn a_remember_call_keeps_changes_and_drops_by_place() {
    let at_bound = [b'x'; 120];
    let written = C::NoteWritten { name: 50, revision: 1, lines: 0, added: 0, removed: 0, first: 0 };
    let references = List::from_boxed(Box::new([CHAT]));
    let rows = [
        (
            "add to none",
            0,
            A::Add { text: b"  new  ".as_slice().into() },
            note(0, b"new", references.clone(), None),
            written.clone(),
            done(D::Kept, 1, &[b"new"]),
        ),
        (
            "add to three",
            3,
            A::Add { text: b"new".as_slice().into() },
            note(0, b"new", references.clone(), None),
            written.clone(),
            done(D::Kept, 4, &[b"one", b"two", b"three", b"new"]),
        ),
        (
            "add to eleven",
            11,
            A::Add { text: b"new".as_slice().into() },
            note(0, b"new", references.clone(), None),
            written.clone(),
            done(
                D::Kept,
                12,
                &[
                    b"one", b"two", b"three", b"four", b"five", b"six", b"seven", b"eight", b"nine", b"ten", b"eleven",
                    b"new",
                ],
            ),
        ),
        (
            "change keeps references",
            3,
            A::Change { line: 2, text: b" new ".as_slice().into() },
            note(5, b"new", List::from_boxed(Box::new([77, 88])), Some(REVISION)),
            written.clone(),
            done(D::Changed, 2, &[b"one", b"new", b"three"]),
        ),
        (
            "drop first",
            3,
            A::Drop { line: 1 },
            NamedAction::NoteDelete { scope: scope(), name: 2, recalled: REVISION },
            C::NoteDeleted { name: 2 },
            done(D::Dropped, 1, &[b"two", b"three"]),
        ),
        (
            "drop last",
            3,
            A::Drop { line: 3 },
            NamedAction::NoteDelete { scope: scope(), name: 9, recalled: REVISION },
            C::NoteDeleted { name: 9 },
            done(D::Dropped, 3, &[b"one", b"two"]),
        ),
        (
            "120 bytes",
            0,
            A::Add { text: at_bound.into() },
            note(0, &at_bound, references.clone(), None),
            written,
            done(D::Kept, 1, &[&at_bound]),
        ),
        (
            "moved add",
            3,
            A::Add { text: b"new".as_slice().into() },
            note(0, b"new", references.clone(), None),
            C::NoteRefused(notes::Refusal::Moved { revision: 5 }),
            Answer::Refused(R::Moved { action: D::Kept, line: 4 }),
        ),
        (
            "moved change",
            3,
            A::Change { line: 2, text: b"new".as_slice().into() },
            note(5, b"new", List::from_boxed(Box::new([77, 88])), Some(REVISION)),
            C::NoteRefused(notes::Refusal::Moved { revision: 5 }),
            Answer::Refused(R::Moved { action: D::Changed, line: 2 }),
        ),
        (
            "deleted drop",
            3,
            A::Drop { line: 3 },
            NamedAction::NoteDelete { scope: scope(), name: 9, recalled: REVISION },
            C::NoteRefused(notes::Refusal::Missing),
            Answer::Refused(R::Moved { action: D::Dropped, line: 3 }),
        ),
        (
            "notes busy",
            3,
            A::Add { text: b"new".as_slice().into() },
            note(0, b"new", references.clone(), None),
            C::NoteRefused(notes::Refusal::Busy),
            Answer::Refused(R::Busy),
        ),
        (
            "other note refusal",
            3,
            A::Add { text: b"new".as_slice().into() },
            note(0, b"new", references, None),
            C::NoteRefused(notes::Refusal::PastBound { bytes: 121, bound: BYTES }),
            Answer::Refused(R::Note(notes::Refusal::PastBound { bytes: 121, bound: BYTES })),
        ),
    ];
    for (rule, count, action, expected_action, answer, expected) in rows {
        let (mut domain, env, key) = root(tasks::Party::Person(PERSON));
        call(&mut domain, &env, key, action);
        load(&mut domain, count, true);
        let Some(crate::Work::Core(core::Event::NamedAction { to, key: found, action })) = domain.work.pop() else {
            panic!("{rule}: named write")
        };
        assert_eq!(to.into_token(), RIGHT, "{rule}");
        assert_eq!(found, key, "{rule}");
        match (action, expected_action) {
            (NamedAction::Note { entry, recalled }, NamedAction::Note { entry: expected, recalled: revision }) => {
                assert_eq!(entry, expected, "{rule}");
                assert_eq!(recalled, revision, "{rule}");
            }
            (
                NamedAction::NoteDelete { scope, name, recalled },
                NamedAction::NoteDelete { scope: expected, name: wanted, recalled: revision },
            ) => {
                assert_eq!((scope, name, recalled), (expected, wanted, revision), "{rule}");
            }
            (found, expected) => panic!("{rule}: {found:?} instead of {expected:?}"),
        }
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        host_route::render(&mut domain, &env.limits, &mut decision, ReplyTo::new(RIGHT), answer);
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.core.counters,
            &env.limits.journal,
            decision,
            false,
        )
        .unwrap();
        assert_eq!(rendered(&mut domain), C::Remember(Box::new(expected)), "{rule}");
        assert!(domain.work.is_empty());
        domain.archive_reads.reclaim();
        assert!(domain.archive_reads.is_empty());
    }
}

enum Load {
    Index(usize),
    Failed,
    Cut,
    Busy,
    NoPerson,
    NoProgress,
    WrongScope,
    NoRoom,
}

#[test]
#[expect(clippy::too_many_lines, reason = "one refusal table drives the same root load seam and retry")]
fn a_refused_remember_call_writes_nothing() {
    let rows = [
        (
            "full",
            Load::Index(12),
            tasks::Party::Person(PERSON),
            A::Add { text: b"new".as_slice().into() },
            R::Full { bound: BOUND },
        ),
        (
            "121 bytes",
            Load::Index(3),
            tasks::Party::Person(PERSON),
            A::Add { text: Box::new([b'x'; 121]) },
            R::TooLong { bytes: 121, bound: BYTES },
        ),
        (
            "empty",
            Load::Index(3),
            tasks::Party::Person(PERSON),
            A::Add { text: b"  ".as_slice().into() },
            R::NotOneLine,
        ),
        (
            "two lines",
            Load::Index(3),
            tasks::Party::Person(PERSON),
            A::Change { line: 2, text: b"one\ntwo".as_slice().into() },
            R::NotOneLine,
        ),
        (
            "trailing newline",
            Load::Index(3),
            tasks::Party::Person(PERSON),
            A::Add { text: b"one\n".as_slice().into() },
            R::NotOneLine,
        ),
        ("no line", Load::Index(3), tasks::Party::Person(PERSON), A::Drop { line: 4 }, R::NoLine { line: 4, lines: 3 }),
        (
            "zero line",
            Load::Index(3),
            tasks::Party::Person(PERSON),
            A::Change { line: 0, text: b"new".as_slice().into() },
            R::NoLine { line: 0, lines: 3 },
        ),
        ("no load room", Load::NoRoom, tasks::Party::Person(PERSON), A::Drop { line: 1 }, R::Unread),
        ("store failed", Load::Failed, tasks::Party::Person(PERSON), A::Drop { line: 1 }, R::Unread),
        ("store cut", Load::Cut, tasks::Party::Person(PERSON), A::Drop { line: 1 }, R::Unread),
        ("busy load", Load::Busy, tasks::Party::Person(PERSON), A::Drop { line: 1 }, R::Busy),
        ("task requester", Load::NoPerson, tasks::Party::Task(2), A::Drop { line: 1 }, R::NoPerson),
        (
            "deployment requester",
            Load::NoPerson,
            tasks::Party::Deployment { project: 1 },
            A::Drop { line: 1 },
            R::NoPerson,
        ),
        ("no progress", Load::NoProgress, tasks::Party::Person(PERSON), A::Drop { line: 1 }, R::Unread),
        ("wrong scope", Load::WrongScope, tasks::Party::Person(PERSON), A::Drop { line: 1 }, R::Unread),
    ];
    for (rule, load, requester, action, expected) in rows {
        let (mut domain, env, key) = root(requester);
        if let Load::Busy = load {
            for _ in 0..domain.archive_reads.capacity() {
                domain.archive_reads.insert(Some(crate::boundary::Read::Notes { owner: Token::new(1) })).unwrap();
            }
        }
        call(&mut domain, &env, key, action.clone());
        let mut out = Queue::uncapped();
        match load {
            Load::Index(count) => self::load(&mut domain, count, true),
            Load::NoRoom => {
                let waiter = waiter(&domain);
                let mut issued = Queue::uncapped();
                for number in 0..env.limits.loads.loads {
                    crate::loads::begin(
                        &mut domain.loads,
                        Token::new(u64::from(number)),
                        crate::Range::Core(crate::CoreRange::Tasks),
                        None,
                        1,
                        &mut issued,
                    )
                    .unwrap();
                }
                crate::restart::request_load(
                    &mut domain,
                    waiter,
                    crate::Range::Core(crate::CoreRange::Notes(notes::Range::Lines { scope: scope(), after: None })),
                    None,
                    &mut out,
                );
            }
            Load::Failed => {
                let waiter = waiter(&domain);
                remember::failed(&mut domain, waiter);
            }
            Load::Cut => {
                let waiter = waiter(&domain);
                remember::loaded(
                    &mut domain,
                    waiter,
                    lines(3),
                    None,
                    Some(crate::loads::Cut { rows: 1, bytes: 1 }),
                    &mut out,
                );
            }
            Load::Busy | Load::NoPerson => {}
            Load::NoProgress => {
                let waiter = waiter(&domain);
                let next = crate::Key::Core(core::Key::Notes(notes::Key::Line { scope: scope(), name: 2 }));
                remember::loaded(&mut domain, waiter, Box::new([]), Some(next), None, &mut out);
            }
            Load::WrongScope => {
                let waiter = waiter(&domain);
                let line = notes::Line {
                    scope: notes::Scope::Party { party: 8 },
                    name: 2,
                    description: b"other".as_slice().into(),
                    revision: REVISION,
                };
                remember::loaded(
                    &mut domain,
                    waiter,
                    Box::new([crate::Record::Core(core::Record::Notes(notes::Record::Line(line)))]),
                    None,
                    None,
                    &mut out,
                );
            }
        }
        assert!(out.is_empty(), "{rule}: no further load");
        assert!(domain.work.is_empty(), "{rule}: no write");
        assert_eq!(rendered(&mut domain), C::Remember(Box::new(Answer::Refused(expected))), "{rule}");
        assert!(!domain.core.call_parts.contains_key(&key), "{rule}: no call record");
        domain.archive_reads.reclaim();
        if !busy(&load) {
            assert!(domain.archive_reads.is_empty(), "{rule}: no retained read");
            // The same call key reads again rather than replaying a recorded refusal.
            call(&mut domain, &env, key, action);
            match load {
                Load::NoPerson => {
                    assert_eq!(rendered(&mut domain), C::Remember(Box::new(Answer::Refused(R::NoPerson))));
                }
                Load::Index(_) | Load::Failed | Load::Cut | Load::NoProgress | Load::WrongScope | Load::NoRoom => {
                    let _waiter = waiter(&domain);
                    assert!(domain.work.is_empty(), "{rule}: retry loads afresh");
                }
                Load::Busy => unreachable!(),
            }
        }
    }
}

fn busy(load: &Load) -> bool {
    match load {
        Load::Busy => true,
        Load::Index(_)
        | Load::Failed
        | Load::Cut
        | Load::NoPerson
        | Load::NoProgress
        | Load::WrongScope
        | Load::NoRoom => false,
    }
}

fn done(did: D, line: u32, lines: &[&[u8]]) -> Answer {
    Answer::Done { did, line, lines: texts(lines), bound: BOUND }
}
