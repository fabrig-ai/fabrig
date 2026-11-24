use super::fixtures;
use crate::translate::watches;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::{Event, documents as d};
use jig_domain_tasks as t;
use skein_lib::Token;
const QUERY_PROJECT: u32 = 3;
const QUERY_ROOT: u64 = 7;
const QUERY_PARENT: u64 = 8;
const QUERY_REQUESTER: u64 = 9;
const QUERY_WATCHER: Token = Token::new(11);
const QUERY_SIGN_IN: u64 = 41;
#[test]
fn a_task_query_subject_reaches_the_core_with_its_keys() {
    let query = w::TaskQuery {
        parts: skein_lib::List::with_capacity(0),
        project: Some(QUERY_PROJECT),
        requester: Some(w::Requester::Person(w::RequesterPerson { value: QUERY_REQUESTER })),
        root: Some(QUERY_ROOT),
        parent: Some(QUERY_PARENT),
        label: Some(5),
        phase: Some(w::PhaseClass::Held),
        executor: Some(w::ExecutorKind::Person),
        tracked: Some(true),
        order: w::QueryOrder::Priority,
    };
    let subject = w::Subject::Tasks(w::SubjectTasks { query, window: 2 });
    let Some(Event::Watch { watcher, sign_in, subject, window }) =
        watches::watch(QUERY_WATCHER, QUERY_SIGN_IN, subject, 32)
    else {
        panic!("watch")
    };
    assert_eq!((watcher, sign_in, window), (QUERY_WATCHER, QUERY_SIGN_IN, d::Window { rows: 2, after: None }));
    let d::Watched::Query { query } = subject else { panic!("task query") };
    assert_eq!(query.project, Some(QUERY_PROJECT));
    assert_eq!(query.requester, Some(t::Party::Person(QUERY_REQUESTER)));
    assert_eq!((query.root, query.parent, query.label), (Some(QUERY_ROOT), Some(QUERY_PARENT), Some(5)));
    assert_eq!(
        (query.phase, query.executor, query.tracked, query.order),
        (Some(d::PhaseClass::Held), Some(d::ExecutorKind::Person), Some(true), d::QueryOrder::Priority)
    );
}
#[test]
#[expect(clippy::too_many_lines, reason = "one passage follows admission, both row groups, delivery and continuation")]
fn task_parts_keep_their_places_across_pages_deliveries_and_continuations() {
    use crate::translate;
    use jig_domain_core_vocabulary::reads;
    use skein_lib::{List, Reader};
    let limits = fixtures::wire();
    let asks = List::from_boxed(Box::new([
        d::PartAsk { owner: 99, kind: 1, version: 2 },
        d::PartAsk { owner: 9, kind: 1, version: 2 },
        d::PartAsk { owner: 9, kind: 2, version: 2 },
        d::PartAsk { owner: 9, kind: 1, version: 3 },
    ]));
    let expected_query = d::TaskQuery { parts: asks.clone(), ..fixtures::tasks_document().query };
    let mut wire_asks = List::with_capacity(4);
    for ask in &asks {
        wire_asks
            .push(w::PartAsk { owner: w::Family { number: ask.owner }, kind: ask.kind, version: ask.version })
            .unwrap();
    }
    let query = w::TaskQuery {
        parts: wire_asks,
        project: expected_query.project,
        requester: None,
        root: expected_query.root,
        parent: expected_query.parent,
        label: expected_query.label,
        phase: None,
        executor: None,
        tracked: expected_query.tracked,
        order: w::QueryOrder::Number,
    };
    let value = w::Read::Tasks(w::ReadTasks { query: query.clone(), live: None, ended: None });
    for (core_parts, expected_parts) in [
        (
            [d::RowPart::Made(Box::from([255, 0, 1])), d::RowPart::Unknown],
            [
                w::RowPart::Value(w::Owned {
                    owner: w::Owner::Family(w::Family { number: 9 }),
                    version: 2,
                    record: Box::from([255, 0, 1]),
                }),
                w::RowPart::Unknown,
            ],
        ),
        (
            [d::RowPart::Failed(d::PartFailure::Busy), d::RowPart::Failed(d::PartFailure::TooLarge)],
            [
                w::RowPart::Failed(w::PartFailed { why: w::PartFailure::Busy }),
                w::RowPart::Failed(w::PartFailed { why: w::PartFailure::TooLarge }),
            ],
        ),
        (
            [d::RowPart::Failed(d::PartFailure::Unreadable), d::RowPart::Unknown],
            [w::RowPart::Failed(w::PartFailed { why: w::PartFailure::Unreadable }), w::RowPart::Unknown],
        ),
    ] {
        let mut exchange = fixtures::read_exchange();
        let (owner, reply, core_read) = exchange.read(&value);
        let reads::Read::Tasks { query: core_query, .. } = core_read else { panic!("task query") };
        assert_eq!(
            core_query.parts.as_slice(),
            &[d::PartAsk { owner: 9, kind: 1, version: 2 }, d::PartAsk { owner: 9, kind: 2, version: 2 },]
        );
        let mut lines = fixtures::tasks_document();
        lines.query = core_query.clone();
        lines.ended_mark = 73;
        lines.lines.get_mut(0).unwrap().parts = List::from_boxed(Box::new(core_parts));
        let cursor = reads::Cursor::Live {
            scope: reads::Scope::Tasks { query: core_query.clone() },
            after: d::Cursor::Number { number: 7 },
            store_after: None,
        };
        exchange.release_read(
            reply,
            reads::Answer::Tasks(Box::new(reads::TaskPage {
                live: lines.lines.clone(),
                live_next: Some(cursor.clone()),
                ended_mark: 73,
                ended: reads::EndedGroup::Rows { rows: lines.lines.clone(), next: Some(cursor), cut: false },
            })),
        );
        let w::ReadAnswer::Value(answer) = exchange.read_answer(owner) else { panic!("value") };
        let w::ReadValue::Tasks(page) =
            w::ReadValue::decode(&limits, &mut Reader::new(answer.record.as_ref())).unwrap()
        else {
            panic!("tasks")
        };
        assert_eq!(page.ended_mark, 73);
        let w::EndedGroup::Rows(ended) = &page.ended else { panic!("ended group") };
        let slots = translate::parts::plan(&asks, &fixtures::settings());
        let w::DeliveryValue::Tasks(delivery) =
            watches::delivery(&d::Document::Lines(lines), 2, &[], &slots, &limits).unwrap()
        else {
            panic!("delivery")
        };
        assert_eq!(delivery.ended_mark, 73);
        for bytes in [page.live.as_ref(), ended.rows.as_ref(), delivery.rows.as_ref()] {
            let mut reader = Reader::new(bytes);
            let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
            let row = w::TaskLine::decode(&limits, &mut Reader::new(&bytes)).unwrap();
            let [w::RowPart::NotServed(first), a, b, w::RowPart::NotServed(last)] = row.parts.as_slice() else {
                panic!("parts in order")
            };
            assert_eq!([a, b], [&expected_parts[0], &expected_parts[1]]);
            assert_eq!((&first.owner), &w::Owner::Family(w::Family { number: 99 }));
            assert_eq!((&last.owner), &w::Owner::Family(w::Family { number: 9 }));
            let ranges = &first.served;
            assert_eq!(ranges, (&last.served));
            assert_eq!(ranges.len(), 2);
            let family = ranges.get(1).unwrap();
            assert_eq!(((&family.owner), family.first, family.last), ((&last.owner), 2, 2));
        }
        let continuation = page.live_after.as_ref().unwrap().clone();
        let reads::Cursor::Live { scope: reads::Scope::Tasks { query: held }, .. } =
            translate::cursor::decode(continuation.value.as_ref()).unwrap()
        else {
            panic!("query cursor")
        };
        assert_eq!(held.parts, asks);
        let next = w::Read::Tasks(w::ReadTasks { query: query.clone(), live: Some(continuation), ended: None });
        let (_, _, read) = exchange.read(&next);
        let reads::Read::Tasks {
            query: next_query,
            live: Some(reads::Cursor::Live { scope: reads::Scope::Tasks { query: next_scope }, .. }),
            ..
        } = read
        else {
            panic!("continuation")
        };
        assert_eq!(next_query, core_query);
        assert_eq!(next_scope, core_query);
    }
}

const WAITING_ROLE: u32 = 2;
#[test]
fn a_task_document_carries_its_label_flags_activity_and_oldest_waiting_entry() {
    let limits = fixtures::wire_limits();
    let mut source = fixtures::task_with_hold();
    source.label = 5;
    source.flags = t::Flags {
        ended_by: t::EndedBy::Result,
        conversation: true,
        listed: true,
        retention: t::Retention::ForgottenOnceRead,
    };
    source.last_activity = skein_lib::Wall::from_nanos(13);
    source.waits_on = d::WaitsOn::Party {
        whom: jig_domain_people::Whom::Role { project: QUERY_PROJECT, role: WAITING_ROLE },
        entries: 3,
        task: QUERY_ROOT,
        kind: jig_domain_people::EntryKind::Question { message: fixtures::NUMBER },
    };
    let value = crate::translate::task::document(&source, &[], &limits).unwrap();
    assert_eq!(
        (value.label, value.flags.conversation, value.flags.listed, (&value.flags.retention), value.last_activity),
        (5, true, true, &w::Retention::ForgottenOnceRead, 13)
    );
    let w::WaitsOn::Party(wait) = &value.waits_on else { panic!("party") };
    let w::Whom::Role(whom) = &wait.whom else { panic!("role") };
    assert_eq!((whom.project, whom.role), (QUERY_PROJECT, WAITING_ROLE));
    assert_eq!((wait.entries, wait.task, (&wait.kind)), (3, QUERY_ROOT, &w::WaitedFor::Question));
}
