use crate::{
    Request,
    address::{Filter, Kind},
    pages::Act,
    tests::fixtures,
};
use jig_client::{
    Readable,
    contract::{self, InboxGroup, ObjectKey, Read},
};
use skein_lib::{List, Queue};
const TASK: u64 = 9;
const MESSAGE: u64 = 3;
const PROJECT: u32 = fixtures::PROJECT;
const OTHER_PROJECT: u32 = 2;
const GOAL: u64 = 7;
const POSITION: u64 = 11;
const BELOW_TASK: u64 = 8;
const CASES: [(ObjectKey, InboxGroup, Read); 6] = [
    (
        ObjectKey::Question { task: TASK, message: MESSAGE },
        InboxGroup::Questions,
        Read::Question { task: TASK, message: MESSAGE },
    ),
    (
        ObjectKey::Proposal(contract::ProposalKey { proposer: contract::Proposer::Task(TASK), number: MESSAGE }),
        InboxGroup::Proposals,
        Read::Proposal {
            proposal: contract::ProposalKey { proposer: contract::Proposer::Task(TASK), number: MESSAGE },
        },
    ),
    (ObjectKey::Escalation(TASK), InboxGroup::Escalations, Read::Escalation { task: TASK }),
    (ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, Read::PersonTask { task: TASK }),
    (ObjectKey::Result(TASK), InboxGroup::Results, Read::Result { task: TASK }),
    (ObjectKey::Task(TASK), InboxGroup::Replies, Read::Task { task: TASK }),
];
fn rows(object: ObjectKey, group: InboxGroup, project: u32) -> List<contract::InboxEntry> {
    let mut rows = List::with_capacity(2);
    rows.push(fixtures::inbox_entry(object, group, project, POSITION)).expect("one row");
    rows
}
fn reads(out: &mut Queue<Request>) -> Box<[Read]> {
    let mut reads = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: jig_client::Down::Read { what: Readable::Jig(read), .. } },
        } = request
        {
            reads.push(read).expect("bounded reads");
        }
    }
    reads.into_boxed()
}
#[test]
fn the_inbox_watches_inbox_and_reads_each_entrys_object() {
    let (mut root, watch) = fixtures::inbox_home(Filter::ALL);
    for (object, group, expected) in CASES {
        let mut out = fixtures::inbox_entries(&mut root, watch, rows(object, group, PROJECT));
        assert_eq!(reads(&mut out).as_ref(), [expected], "{group:?}");
    }
}
#[test]
fn a_cut_person_question_reads_its_words() {
    for (cut, in_words) in [(true, true), (false, true), (true, false)] {
        let (mut root, watch) = fixtures::inbox_home(Filter::ALL);
        let mut out = fixtures::inbox_entries(
            &mut root,
            watch,
            rows(ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, PROJECT),
        );
        out.clear();
        let contract::ReadValue::PersonTask(mut person) = fixtures::acts::person(contract::ActKind::Answer) else {
            panic!("person fixture")
        };
        person.task.task = TASK;
        person.spec.specification.words.cut = cut;
        if !in_words {
            person.contract = contract::TaskContractDocument::Verdict { choices: List::with_capacity(0), cut: false };
        }
        let handle = root.pages().inbox().unwrap().cards.get(0).unwrap().handle;
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Page {
                handle,
                page: jig_client::Readout::Jig(contract::ReadResult::Value(contract::ReadValue::PersonTask(person))),
            },
            &mut out,
        );
        let expected = if cut && in_words {
            Box::<[Read]>::from([Read::TaskText { task: TASK, field: contract::TaskTextField::Specification, from: 0 }])
        } else {
            Box::<[Read]>::from([])
        };
        assert_eq!(reads(&mut out).as_ref(), expected.as_ref());
        assert!(root.pages().inbox().unwrap().known(), "a whole read does not delay known");
    }
}

fn text_page(task: u64, from: u64, total: u64, next: Option<u64>, words: &[u8]) -> contract::ReadResult {
    contract::ReadResult::Value(contract::ReadValue::TaskText(Box::new(contract::TaskText {
        task,
        field: contract::TaskTextField::Specification,
        from,
        total,
        next,
        text: contract::Text::new(words.into(), 32).unwrap(),
    })))
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one page-validation table checks continuation and the next refresh together"
)]
fn whole_question_pages_validate_finish_and_retry_on_refresh() {
    use crate::pages::inbox::QuestionWords;
    let first = text_page(TASK, 0, 12, Some(6), b"Whole ");
    let mut wrong_field = text_page(TASK, 0, 12, None, b"Whole words.");
    if let contract::ReadResult::Value(contract::ReadValue::TaskText(page)) = &mut wrong_field {
        page.field = contract::TaskTextField::Result;
    }
    for (name, pages, complete) in [
        ("whole", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 12, None, b"Whole words.")]), true),
        ("paged", Box::<[contract::ReadResult]>::from([first.clone(), text_page(TASK, 6, 12, None, b"words.")]), true),
        ("other task", Box::<[contract::ReadResult]>::from([text_page(TASK + 1, 0, 12, None, b"Whole words.")]), false),
        ("wrong from", Box::<[contract::ReadResult]>::from([text_page(TASK, 1, 13, None, b"Whole words.")]), false),
        ("past long_text", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 33, Some(6), b"Whole ")]), false),
        ("ended", Box::<[contract::ReadResult]>::from([contract::ReadResult::None(contract::ReadNone::Busy)]), false),
        (
            "wrong value",
            Box::<[contract::ReadResult]>::from([contract::ReadResult::Value(fixtures::acts::question())]),
            false,
        ),
        ("wrong field", Box::<[contract::ReadResult]>::from([wrong_field]), false),
        (
            "offset overflow",
            Box::<[contract::ReadResult]>::from([text_page(TASK, u64::MAX, 12, None, b"Whole words.")]),
            false,
        ),
        (
            "changed total",
            Box::<[contract::ReadResult]>::from([first.clone(), text_page(TASK, 6, 13, None, b"words.!")]),
            false,
        ),
        ("wrong next", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 12, Some(5), b"Whole ")]), false),
        ("no progress", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 12, Some(0), b"")]), false),
        ("short end", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 12, None, b"Whole ")]), false),
        ("past total", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 5, None, b"Whole ")]), false),
        ("next at total", Box::<[contract::ReadResult]>::from([text_page(TASK, 0, 6, Some(6), b"Whole ")]), false),
        (
            "later end",
            Box::<[contract::ReadResult]>::from([first, contract::ReadResult::None(contract::ReadNone::Busy)]),
            false,
        ),
    ] {
        let (mut root, watch) = fixtures::inbox_home(Filter::ALL);
        fixtures::inbox_entries(&mut root, watch, rows(ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, PROJECT));
        let contract::ReadValue::PersonTask(mut person) = fixtures::acts::person(contract::ActKind::Answer) else {
            panic!("person fixture")
        };
        person.task.task = TASK;
        person.spec.specification.words =
            contract::DocumentText { bytes: contract::Text::new(b"Whole".as_slice().into(), 32).unwrap(), cut: true };
        let person = contract::ReadValue::PersonTask(person);
        let inbox = root.pages().inbox().unwrap();
        let card_handle = inbox.cards.get(0).unwrap().handle;
        let whole_handle = jig_client::Handle { asker: card_handle.asker, number: inbox.next };
        let mut out = Queue::uncapped();
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Page {
                handle: card_handle,
                page: jig_client::Readout::Jig(contract::ReadResult::Value(person.clone())),
            },
            &mut out,
        );
        assert_eq!(
            reads(&mut out).as_ref(),
            [Read::TaskText { task: TASK, field: contract::TaskTextField::Specification, from: 0 }],
            "{name}"
        );
        let Some(QuestionWords::Reading(words)) = root.pages().inbox().unwrap().cards.get(0).unwrap().question_words()
        else {
            panic!("reading question")
        };
        assert_eq!(words, b"Whole");
        assert!(root.pages().inbox().unwrap().known());
        for (index, result) in pages.iter().enumerate() {
            crate::route::told(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                jig_client::Told::Page { handle: whole_handle, page: jig_client::Readout::Jig(result.clone()) },
                &mut out,
            );
            let expected = if index + 1 < pages.len() {
                Box::<[Read]>::from([Read::TaskText {
                    task: TASK,
                    field: contract::TaskTextField::Specification,
                    from: 6,
                }])
            } else {
                Box::<[Read]>::from([])
            };
            assert_eq!(reads(&mut out).as_ref(), expected.as_ref(), "{name}: continuation");
        }
        out.clear();
        let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
        match card.question_words() {
            Some(QuestionWords::Whole(words)) => {
                assert!(complete, "{name}");
                assert_eq!(words, b"Whole words.", "{name}");
            }
            Some(QuestionWords::Cut(words)) => {
                assert!(!complete, "{name}");
                assert_eq!(words, b"Whole", "{name}");
            }
            Some(QuestionWords::Reading(_)) | None => panic!("{name}: read did not end"),
        }
        let mut refresh = fixtures::inbox_entries(
            &mut root,
            watch,
            rows(ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, PROJECT),
        );
        let expected = if complete {
            Box::<[Read]>::from([Read::PersonTask { task: TASK }])
        } else {
            Box::<[Read]>::from([
                Read::PersonTask { task: TASK },
                Read::TaskText { task: TASK, field: contract::TaskTextField::Specification, from: 0 },
            ])
        };
        assert_eq!(
            reads(&mut refresh).as_ref(),
            expected.as_ref(),
            "{name}: the refresh restarts a short read at once"
        );
        let card_handle = root.pages().inbox().unwrap().cards.get(0).unwrap().handle;
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Page {
                handle: card_handle,
                page: jig_client::Readout::Jig(contract::ReadResult::Value(person)),
            },
            &mut out,
        );
        assert!(reads(&mut out).is_empty(), "{name}: the document does not duplicate a whole read");
    }
}

#[test]
fn a_kind_chosen_shows_the_inbox_at_its_address() {
    let (mut root, _) = fixtures::inbox_home(Filter::ALL);
    let filter = Filter { kind: Some(Kind::Question), ..Filter::ALL };
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Act { action: crate::Action::Pages(Act::Filter { filter }) },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Request::Show { address: crate::Address::Inbox { filter }, replace: false }));
    assert_eq!(&*crate::address::print(&crate::Address::Inbox { filter }), b"/?kind=question");
    assert!(out.is_empty());
}
#[test]
fn the_inbox_at_a_filtered_path_reads_that_filter() {
    let (mut root, _) = fixtures::inbox_home(Filter::ALL);
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Went { place: crate::address::parse(b"/?project=3&kind=question&goal=9") },
        &mut out,
    );
    let mut watch = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch: token,
                            subject: jig_client::Watchable::Jig(contract::Subject::Inbox { .. }),
                        },
                },
        } = request
        {
            watch = Some(token);
        }
    }
    let watch = watch.expect("filtered inbox watch");
    let mut out = fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    assert_eq!(
        reads(&mut out).as_ref(),
        [Read::Inbox {
            group: contract::InboxReadGroup::Needs,
            project: Some(3),
            kind: Some(InboxGroup::Questions),
            goal: Some(9),
            after: None,
        }]
    );
}
#[test]
fn a_filter_by_project_shows_only_that_projects_entries() {
    let (mut root, watch) = fixtures::inbox_home(Filter { project: Some(OTHER_PROJECT), ..Filter::ALL });
    let mut entries = rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT);
    entries
        .push(fixtures::inbox_entry(ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, OTHER_PROJECT, POSITION))
        .expect("second entry");
    let mut out = fixtures::inbox_entries(&mut root, watch, entries);
    assert_eq!(
        reads(&mut out).as_ref(),
        [contract::InboxReadGroup::Needs, contract::InboxReadGroup::Updates].map(|group| Read::Inbox {
            group,
            project: Some(OTHER_PROJECT),
            kind: None,
            goal: None,
            after: None,
        })
    );
    assert!(root.pages().inbox().unwrap().cards.is_empty(), "the party window cannot select the project");
    let handle = root.pages().inbox().unwrap().queries[0];
    let mut selected = rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT);
    selected
        .push(fixtures::inbox_entry(ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, OTHER_PROJECT, POSITION))
        .unwrap();
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Page {
            handle,
            page: jig_client::Readout::Jig(contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(
                contract::InboxPage { rows: selected, cut: false, before: None, after: None, selection: None },
            )))),
        },
        &mut out,
    );
    // Filtered sections are applied together.
    let updates = root.pages().inbox().unwrap().queries[1];
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Page {
            handle: updates,
            page: jig_client::Readout::Jig(contract::ReadResult::None(contract::ReadNone::Busy)),
        },
        &mut out,
    );
    assert_eq!(reads(&mut out).as_ref(), [Read::PersonTask { task: TASK }]);
}
#[test]
fn a_filter_by_kind_shows_only_that_kind() {
    let (mut root, watch) = fixtures::inbox_home(Filter { kind: Some(Kind::Question), ..Filter::ALL });
    let mut out =
        fixtures::inbox_entries(&mut root, watch, rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT));
    assert_eq!(reads(&mut out).as_ref(), []);
}
#[test]
fn a_filter_by_goal_shows_only_entries_below_it() {
    let (mut root, watch) = fixtures::inbox_home(Filter { goal: Some(GOAL), ..Filter::ALL });
    let mut out = fixtures::inbox_entries(
        &mut root,
        watch,
        rows(ObjectKey::Question { task: TASK, message: MESSAGE }, InboxGroup::Questions, PROJECT),
    );
    assert_eq!(
        reads(&mut out).as_ref(),
        [contract::InboxReadGroup::Needs, contract::InboxReadGroup::Updates].map(|group| Read::Inbox {
            group,
            project: None,
            kind: None,
            goal: Some(GOAL),
            after: None
        })
    );
    assert_eq!(root.pages().inbox().expect("home").cards.len(), 0, "owner membership comes from the filtered query");
    let handle = root.pages().inbox().expect("home").queries[0];
    let page = contract::InboxPage {
        cut: false,
        rows: rows(ObjectKey::PersonTask(BELOW_TASK), InboxGroup::PersonTasks, PROJECT),
        before: None,
        after: None,
        selection: None,
    };
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Page {
            handle,
            page: jig_client::Readout::Jig(contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(page)))),
        },
        &mut out,
    );
    section(&mut root, 1, None);
    assert_eq!(
        root.pages().inbox().expect("home").cards.get(0).expect("selected card").object,
        ObjectKey::PersonTask(BELOW_TASK)
    );
}
fn result_pages() -> crate::pages::Pages {
    fixtures::acts::pages(ObjectKey::Result(TASK), fixtures::acts::question())
}
fn open_result(pages: &mut crate::pages::Pages, through: u64) -> Queue<crate::pages::Request> {
    fixtures::acts::act(pages, Act::OpenResult { task: TASK, through })
}
fn sent_seen(out: &mut Queue<crate::pages::Request>, through: u64) -> jig_client::Key {
    let Some(crate::pages::Request::Client(jig_client::Event::Send { key, request, name: None })) = out.pop() else {
        panic!("Seen precedes Show");
    };
    assert_eq!(
        request,
        jig_client::Ask::Jig(Box::new(contract::Ask::Seen { through: contract::SeenThrough::Results { through } }))
    );
    key
}
fn shown(out: &mut Queue<crate::pages::Request>) {
    assert_eq!(out.pop(), Some(crate::pages::Request::Show { address: crate::Address::Task { task: TASK } }));
    assert!(out.is_empty());
}
fn answer_seen(pages: &mut crate::pages::Pages, key: jig_client::Key, outcome: jig_client::Outcome) {
    let mut out = Queue::with_capacity(16);
    crate::pages::step(
        pages,
        &fixtures::env(&fixtures::limits()),
        crate::unlinked(&fixtures::limits(), fixtures::SEED).client(),
        crate::pages::Event::Told { told: jig_client::Told::Answered { key, outcome } },
        &mut out,
    );
    assert!(out.is_empty(), "Seen answers draw and send nothing");
}
fn answered() -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Jig(contract::Answer::Done(contract::Outcome::Document(
        contract::DocumentOutcome::Amended { task: 9 },
    ))))
}
#[test]
fn following_a_result_sends_seen_then_shows_its_task() {
    let mut pages = result_pages();
    let mut out = open_result(&mut pages, POSITION);
    sent_seen(&mut out, POSITION);
    shown(&mut out);
}
#[test]
fn result_seen_is_forward_only_and_has_one_request_in_flight() {
    let mut pages = result_pages();
    let key = sent_seen(&mut open_result(&mut pages, POSITION), POSITION);
    shown(&mut open_result(&mut pages, POSITION + 1));
    answer_seen(&mut pages, key, answered());
    shown(&mut open_result(&mut pages, POSITION));
    shown(&mut open_result(&mut pages, POSITION - 1));
    let mut out = open_result(&mut pages, POSITION + 1);
    sent_seen(&mut out, POSITION + 1);
    shown(&mut out);
}
#[test]
fn a_refused_or_failed_result_seen_is_resent_on_the_next_follow() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Unsent,
        jig_client::Outcome::Lost,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(contract::Answer::Refused {
            refusal: contract::Refusal::NotOffered,
            at: None,
        })),
    ] {
        let mut pages = result_pages();
        let key = sent_seen(&mut open_result(&mut pages, POSITION), POSITION);
        answer_seen(&mut pages, key, answered());
        let key = sent_seen(&mut open_result(&mut pages, POSITION + 1), POSITION + 1);
        answer_seen(&mut pages, key, outcome);
        shown(&mut open_result(&mut pages, POSITION));
        let mut out = open_result(&mut pages, POSITION + 1);
        assert_ne!(sent_seen(&mut out, POSITION + 1), key);
        shown(&mut out);
    }
}
#[test]
fn going_to_a_results_task_from_another_page_sends_no_seen() {
    for address in
        [crate::Address::Chats { filter: crate::ChatsFilter::All }, crate::Address::Home { project: PROJECT }]
    {
        let mut pages = result_pages();
        shown(&mut fixtures::acts::act(&mut pages, Act::Go { address: crate::Address::Task { task: TASK } }));
        let client = crate::unlinked(&fixtures::limits(), fixtures::SEED);
        let mut out = Queue::uncapped();
        crate::pages::step(
            &mut pages,
            &fixtures::env(&fixtures::limits()),
            client.client(),
            crate::pages::Event::Went { address },
            &mut out,
        );
        assert!(fixtures::acts::sent(&mut out).is_empty());
        shown(&mut fixtures::acts::act(&mut pages, Act::Go { address: crate::Address::Task { task: TASK } }));
        shown(&mut open_result(&mut pages, POSITION));
    }
}
#[test]
fn result_seen_pending_and_answer_survive_navigation() {
    let mut pages = result_pages();
    let key = sent_seen(&mut open_result(&mut pages, POSITION), POSITION);
    let client = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    for address in [crate::Address::Task { task: TASK }, crate::Address::Inbox { filter: Filter::ALL }] {
        crate::pages::step(
            &mut pages,
            &fixtures::env(&fixtures::limits()),
            client.client(),
            crate::pages::Event::Went { address },
            &mut Queue::with_capacity(16),
        );
    }
    shown(&mut open_result(&mut pages, POSITION + 1));
    answer_seen(&mut pages, key, answered());
    shown(&mut open_result(&mut pages, POSITION));
}

#[test]
fn reply_counts_and_source_cut_survive_both_inbox_documents() {
    let count = contract::Count { value: 5, limited: true };
    let mut line = super::chats::chat_line();
    line.chip.task = TASK;
    line.conversation = Some(contract::ConversationRead { seen: POSITION, unread: count });
    for owner in [false, true] {
        let (mut root, watch) = fixtures::inbox_home(Filter::ALL);
        let mut entries = rows(ObjectKey::Task(TASK), InboxGroup::Replies, PROJECT);
        entries.get_mut(0).unwrap().reply = Some(Box::new(line.clone()));
        let object = contract::InboxObject::Reply { message: MESSAGE };
        let cursor = contract::InboxCursor::Inbox { task: TASK, object, at: None };
        let delivery = if owner {
            let mut updates = List::with_capacity(1);
            updates
                .push(contract::InboxEntry {
                    projects: List::with_capacity(0),
                    reply: Some(Box::new(line.clone())),
                    task: contract::TaskLink::Live(line.chip.clone()),
                    project: Some(PROJECT),
                    object,
                    activity: None,
                    at: None,
                    unread: Some(true),
                    cursor,
                })
                .unwrap();
            contract::Delivery::Inbox(Box::new(contract::Inbox {
                party: fixtures::PARTY,
                needs: List::with_capacity(0),
                updates,
                next: Some(cursor),
                cached_needs: 0,
                cached_updates: count,
                results_seen: 0,
                updates_time_complete: true,
                selection: None,
            }))
        } else {
            contract::Delivery::Inbox(Box::new({
                let counts = contract::Needs { proposals: 0, questions: 0, escalations: 0, person_tasks: 0 };
                crate::tests::documents::inbox(
                    entries,
                    counts
                        .questions
                        .saturating_add(counts.proposals)
                        .saturating_add(counts.escalations)
                        .saturating_add(counts.person_tasks),
                    count,
                    0,
                    true,
                )
            }))
        };
        let mut out = Queue::uncapped();
        crate::step(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: contract::WatchEvent::Delivered(delivery) },
                },
            },
            &mut out,
        );
        let inbox = root.pages().inbox().unwrap();
        assert_eq!(inbox.cards.get(0).unwrap().unread, Some(count));
        assert_eq!(inbox.cut, [true, true]);
    }
}
#[test]
fn filtered_cut_is_kept_for_each_section_and_both_page_shapes() {
    use jig_client::{Readout, Told};
    for owner in [false, true] {
        let (mut root, watch) = fixtures::inbox_home(Filter { goal: Some(GOAL), ..Filter::ALL });
        let mut out = fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
        let handles = root.pages().inbox().unwrap().queries;
        for (index, handle) in handles.into_iter().enumerate() {
            let page = if owner {
                contract::ReadValue::Inbox(Box::new(contract::InboxPage {
                    selection: None,
                    cut: index == 1,
                    rows: List::with_capacity(0),
                    before: None,
                    after: None,
                }))
            } else {
                contract::ReadValue::Inbox(Box::new(contract::InboxPage {
                    cut: index == 1,
                    rows: List::with_capacity(0),
                    before: None,
                    after: None,
                    selection: None,
                }))
            };
            crate::route::told(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                Told::Page { handle, page: Readout::Jig(contract::ReadResult::Value(page)) },
                &mut out,
            );
        }
        assert_eq!(root.pages().inbox().unwrap().cut, [false, true]);
    }
}

fn section(root: &mut crate::Domain, index: usize, rows: Option<List<contract::InboxEntry>>) {
    let handle = root.pages().inbox().unwrap().queries[index];
    let result = match rows {
        Some(rows) => contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(contract::InboxPage {
            rows,
            cut: false,
            before: None,
            after: None,
            selection: None,
        }))),
        None => contract::ReadResult::None(contract::ReadNone::Busy),
    };
    page(root, handle, result);
}
fn page(root: &mut crate::Domain, handle: jig_client::Handle, result: contract::ReadResult) {
    crate::route::told(
        root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Page { handle, page: jig_client::Readout::Jig(result) },
        &mut Queue::uncapped(),
    );
}
fn filtered() -> Filter {
    Filter { project: Some(PROJECT), ..Filter::ALL }
}
fn question_rows() -> List<contract::InboxEntry> {
    rows(fixtures::acts::QUESTION, InboxGroup::Questions, PROJECT)
}
fn ready(filter: Filter) -> (crate::Domain, skein_lib::Token) {
    let (mut root, watch) = fixtures::inbox_home(filter);
    fixtures::inbox_entries(&mut root, watch, question_rows());
    if filter.project.is_some() {
        section(&mut root, 0, Some(question_rows()));
        section(&mut root, 1, Some(rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT)));
    }
    let cards = &root.pages().inbox().unwrap().cards;
    let mut handles = List::with_capacity(cards.len());
    for card in cards {
        handles.push(card.handle).unwrap();
    }
    for handle in &handles {
        page(&mut root, *handle, contract::ReadResult::Value(fixtures::acts::question()));
    }
    (root, watch)
}
fn objects(root: &crate::Domain) -> Box<[ObjectKey]> {
    let cards = &root.pages().inbox().unwrap().cards;
    let mut objects = List::with_capacity(cards.len());
    for card in cards {
        objects.push(card.object).unwrap();
    }
    objects.into_boxed()
}
#[test]
fn a_refresh_keeps_each_cards_page_until_its_read_answers() {
    let (mut root, watch) = ready(Filter::ALL);
    fixtures::inbox_entries(&mut root, watch, question_rows());
    let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
    assert_eq!(card.page, Some(fixtures::acts::question()));
    assert!(!root.pages().inbox().unwrap().delivered(root.client()));
    let handle = card.handle;
    page(&mut root, handle, contract::ReadResult::Value(fixtures::acts::person(contract::ActKind::Take)));
    assert_eq!(
        root.pages().inbox().unwrap().cards.get(0).unwrap().page,
        Some(fixtures::acts::person(contract::ActKind::Take))
    );
    assert!(root.pages().inbox().unwrap().delivered(root.client()));
}
#[test]
fn a_failed_read_keeps_the_page_it_had() {
    let (mut root, watch) = ready(Filter::ALL);
    fixtures::inbox_entries(&mut root, watch, question_rows());
    let handle = root.pages().inbox().unwrap().cards.get(0).unwrap().handle;
    page(&mut root, handle, contract::ReadResult::None(contract::ReadNone::Busy));
    assert_eq!(root.pages().inbox().unwrap().cards.get(0).unwrap().page, Some(fixtures::acts::question()));
    assert!(root.pages().inbox().unwrap().delivered(root.client()));
}
#[test]
fn a_filtered_refresh_keeps_the_cards_until_both_reads_answer() {
    for first in [0, 1] {
        let (mut root, watch) = ready(filtered());
        let before = objects(&root);
        fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
        assert_eq!(objects(&root), before);
        section(&mut root, first, Some(List::with_capacity(0)));
        assert_eq!(objects(&root), before);
        for card in &root.pages().inbox().unwrap().cards {
            assert!(card.page.is_some());
        }
        assert!(!root.pages().inbox().unwrap().delivered(root.client()));
        section(&mut root, 1 - first, Some(List::with_capacity(0)));
        assert!(objects(&root).is_empty());
        assert!(root.pages().inbox().unwrap().delivered(root.client()));
    }
}
#[test]
fn a_failed_section_read_keeps_that_sections_cards() {
    for failed in [0, 1] {
        let (mut root, watch) = ready(filtered());
        let kept = objects(&root)[failed];
        fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
        section(&mut root, failed, None);
        section(&mut root, 1 - failed, Some(List::with_capacity(0)));
        assert_eq!(objects(&root).as_ref(), [kept]);
        assert_eq!(root.pages().inbox().unwrap().cards.get(0).unwrap().page, Some(fixtures::acts::question()));
    }
}
#[test]
fn a_card_gone_from_the_rows_leaves_at_the_swap() {
    let (mut root, watch) = ready(filtered());
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    section(&mut root, 0, Some(List::with_capacity(0)));
    assert!(objects(&root).contains(&fixtures::acts::QUESTION));
    section(&mut root, 1, None);
    assert!(!objects(&root).contains(&fixtures::acts::QUESTION));
    let mut out = Queue::uncapped();
    crate::pages::step(
        &mut root.pages,
        &fixtures::env(&fixtures::limits()),
        &root.client,
        crate::pages::Event::Act {
            act: Act::Needs(crate::pages::needs::Act::Choose {
                object: fixtures::acts::QUESTION,
                words: Box::from(b"stale".as_slice()),
            }),
        },
        &mut out,
    );
    assert!(out.is_empty(), "{out:?}");
    assert!(!root.pages().needs().held(fixtures::acts::QUESTION));
}
#[test]
fn overlapping_filtered_refreshes_show_the_newest_rows() {
    let (mut root, watch) = ready(filtered());
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    section(&mut root, 0, Some(List::with_capacity(0)));
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    section(&mut root, 0, Some(question_rows()));
    section(&mut root, 1, Some(List::with_capacity(0)));
    assert_eq!(objects(&root).as_ref(), [fixtures::acts::QUESTION]);
    // Reused handles can deliver the newer read after the first pair swaps.
    section(&mut root, 0, Some(rows(ObjectKey::PersonTask(BELOW_TASK), InboxGroup::PersonTasks, PROJECT)));
    assert_eq!(objects(&root).as_ref(), [ObjectKey::PersonTask(BELOW_TASK)]);
}

#[test]
fn a_waiting_confirmation_survives_its_row_leaving() {
    let (mut root, watch) = ready(Filter::ALL);
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: 1 };
    root.pages.inbox.as_mut().unwrap().cards.get_mut(0).unwrap().key = Some(key);
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    assert_eq!(objects(&root).as_ref(), [fixtures::acts::QUESTION]);
    let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
    assert_eq!(card.key, Some(key));
    assert_eq!(card.page, Some(fixtures::acts::question()));
}

fn settings_rows() -> List<contract::InboxEntry> {
    let key = contract::ProposalKey { proposer: contract::Proposer::Task(TASK), number: MESSAGE };
    let mut entries = rows(ObjectKey::Proposal(key), InboxGroup::Proposals, 0);
    let row = entries.get_mut(0).unwrap();
    row.projects = List::with_capacity(2);
    row.projects.push(PROJECT).unwrap();
    row.projects.push(OTHER_PROJECT).unwrap();
    entries
}
#[test]
fn a_settings_proposal_is_read_in_each_project_it_shows_in() {
    for project in [PROJECT, OTHER_PROJECT] {
        let (mut root, watch) = fixtures::inbox_home(Filter { project: Some(project), ..Filter::ALL });
        let mut out = fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
        let handle = root.pages().inbox().unwrap().queries[0];
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Page {
                handle,
                page: jig_client::Readout::Jig(contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(
                    contract::InboxPage {
                        rows: settings_rows(),
                        cut: false,
                        before: None,
                        after: None,
                        selection: None,
                    },
                )))),
            },
            &mut out,
        );
        section(&mut root, 1, Some(List::with_capacity(0)));
        let cards = &root.pages().inbox().unwrap().cards;
        assert_eq!(cards.len(), 1, "project {project} must show the settings proposal");
        assert_eq!(cards.get(0).unwrap().project, Some(0), "the decision stays deployment-scoped");
    }
}
#[test]
fn a_failed_refresh_keeps_a_settings_proposal_in_each_project_it_shows_in() {
    for project in [PROJECT, OTHER_PROJECT] {
        let (mut root, watch) = fixtures::inbox_home(Filter { project: Some(project), ..Filter::ALL });
        fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
        section(&mut root, 0, Some(settings_rows()));
        section(&mut root, 1, Some(List::with_capacity(0)));
        let before = objects(&root);
        assert_eq!(before.len(), 1);

        fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
        section(&mut root, 0, None);
        section(&mut root, 1, Some(List::with_capacity(0)));
        assert_eq!(objects(&root), before, "project {project} keeps the settings proposal");
        assert_eq!(root.pages().inbox().unwrap().cards.get(0).unwrap().project, Some(0));
    }
}
#[test]
fn the_rail_counts_a_settings_proposal_in_each_project_it_shows_in() {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    let out = super::welcome::deliver(&mut root, me, contract::Delivery::Me(Box::new(fixtures::chats::me())));
    let mut watch = None;
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch: token,
                            subject: jig_client::Watchable::Jig(contract::Subject::Inbox { .. }),
                        },
                },
        } = request
        {
            watch = Some(*token);
        }
    }
    fixtures::inbox_entries(&mut root, watch.expect("standing inbox watch"), settings_rows());
    for project in [PROJECT, OTHER_PROJECT] {
        assert_eq!(
            root.pages().needs().count(root.client(), root.forge().approvals(), project).count,
            1,
            "project {project}"
        );
    }
    assert_eq!(root.pages().needs().count(root.client(), root.forge().approvals(), 3).count, 0);
}

#[test]
fn a_lost_section_read_is_no_answer() {
    let (mut root, watch) = fixtures::inbox_home(filtered());
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    let queries = root.pages().inbox().unwrap().queries;
    for handle in queries {
        page(&mut root, handle, contract::ReadResult::None(contract::ReadNone::Unreachable));
    }
    let inbox = root.pages().inbox().unwrap();
    assert!(inbox.delivered(root.client()));
    assert!(!inbox.known());
    assert!(!inbox.answered(contract::InboxReadGroup::Needs));
    assert!(!inbox.answered(contract::InboxReadGroup::Updates));
}
#[test]
fn known_stands_through_a_refresh() {
    let (mut root, watch) = ready(filtered());
    assert!(root.pages().inbox().unwrap().known());
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    let inbox = root.pages().inbox().unwrap();
    assert!(!inbox.delivered(root.client()));
    assert!(inbox.known());
    assert!(inbox.answered(contract::InboxReadGroup::Needs));
}
