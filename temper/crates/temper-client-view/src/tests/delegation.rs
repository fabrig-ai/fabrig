use super::fixtures;
use crate::look;
use jig_client::contract as c;
use skein_fake_person::{Face, Find, Found, Target, TreeFace};
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{Address, Domain, Event, Request};
const CHANGE: u64 = 2;
const CHAT: u64 = 1;
const EXCHANGES: u32 = 64;
const STEPS: u32 = 4;
const TITLE: &[u8] = b"Export widget's table as CSV";
const MERGE: &[u8] = b"1a2b3c4abcdef0123456789abcdef0123456789abcd";
const WAKE_SUMMARY: &[u8] =
    b"Begin the work your brief describes.\n\n2: Result from task 2:\n1d0b14e4db018fed000000000000000300000000";
const LATER: &[u8] = b"The change is running.";
fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(Box::from(bytes), 256).expect("fixture text")
}
fn document(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(text(bytes))
}
pub(super) fn change() -> c::Task {
    let mut task = fixtures::task::document();
    task.chip.task = CHANGE;
    task.chip.title = document(TITLE);
    task.chip.phase = c::Phase::Running;
    task.hold = None;
    task.result = None;
    task.result_contract = c::TaskContractDocument::Change { connector: fixtures::FORGE, kind: 1, words: 256 };
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.ending = None;
    task
}
fn landed() -> c::Task {
    let mut task = change();
    task.requester = c::Requester::Task(CHAT);
    task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Change {
        kind: 1,
        part: c::Part { connector: fixtures::FORGE, kind: c::PartKind::Result, key: c::PartKey::Number(CHANGE) },
        words: document(MERGE),
    }))));
    task
}
fn limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.rows = 8;
    limits.client.domain.text = 256;
    limits.client.domain.long_text = 256;
    limits.client.domain.turn_text = 256;
    limits
}
fn up(root: &mut Domain, up: jig_client::Up, receiving: &temper_client_domain::Limits, out: &mut Queue<Request>) {
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *receiving },
        Event::Client { event: jig_client::Event::Up { up } },
        out,
    );
}

fn task_delivery(watch: skein_lib::Token, task: c::Task) -> jig_client::Up {
    jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))) }
}

fn root(documents: &[c::Task], called: &[u64], heard: bool) -> Domain {
    root_activity(documents, activity(called, heard), &[])
}
fn root_activity(documents: &[c::Task], activity: c::ActivityPage, delegates: &[u64]) -> Domain {
    root_chat(None, documents, activity, delegates)
}
fn root_chat(
    chat_document: Option<&c::Task>,
    documents: &[c::Task],
    activity: c::ActivityPage,
    delegates: &[u64],
) -> Domain {
    root_chat_limits(chat_document, documents, activity, delegates, &limits())
}
fn root_chat_limits(
    chat_document: Option<&c::Task>,
    documents: &[c::Task],
    activity: c::ActivityPage,
    delegates: &[u64],
    receiving: &temper_client_domain::Limits,
) -> Domain {
    root_chat_with_reads(chat_document, documents, activity, delegates, receiving).0
}
#[expect(
    clippy::too_many_lines,
    reason = "one fixture pumps the chat's read and watch responses while explicitly reading its history"
)]
fn root_chat_with_reads(
    chat_document: Option<&c::Task>,
    documents: &[c::Task],
    activity: c::ActivityPage,
    delegates: &[u64],
    receiving: &temper_client_domain::Limits,
) -> (Domain, Queue<Request>) {
    let mut reads = Queue::uncapped();
    let chat_number = match chat_document {
        Some(chat) => chat.chip.task,
        None => CHAT,
    };
    let (mut root, _, _) = fixtures::root_at(Address::Chats { filter: ChatsFilter::All }, receiving);
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *receiving },
        Event::Went { place: temper_client_domain::Place::Address(Address::Chat { task: chat_number }) },
        &mut out,
    );
    for _ in 0..EXCHANGES {
        let Some(request) = out.pop() else { break };
        match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read {
                                read,
                                what: jig_client::Readable::Jig(c::Read::Activity { before, .. }),
                            },
                    },
            } => {
                let page = fixtures::first_run::page(&activity, before.as_ref(), receiving.client.domain.rows);
                let more = page.before.is_some();
                up(
                    &mut root,
                    jig_client::Up::ReadDone {
                        read,
                        result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(page)))),
                    },
                    receiving,
                    &mut out,
                );
                if more {
                    temper_client_domain::step(
                        &mut root,
                        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *receiving },
                        Event::Act {
                            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Older),
                        },
                        &mut out,
                    );
                }
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { task: number }),
                            },
                    },
            } if number == chat_number => {
                let mut chat = match chat_document {
                    Some(chat) => chat.clone(),
                    None => change(),
                };
                chat.chip.task = chat_number;
                chat.delegates = List::with_capacity(u32::try_from(delegates.len()).expect("delegates fit"));
                for task in delegates {
                    let mut chip = change().chip;
                    chip.task = *task;
                    chat.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).expect("delegate");
                }
                chat.activity = activity.rows.get(0).expect("newest activity").source.position;
                up(&mut root, task_delivery(watch, chat), receiving, &mut out);
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { task: number }),
                            },
                    },
            } => {
                for task in documents {
                    if task.chip.task != number {
                        continue;
                    }
                    up(&mut root, task_delivery(watch, task.clone()), receiving, &mut out);
                }
            }
            request @ Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { what: jig_client::Readable::Jig(c::Read::Task { .. }), .. },
                    },
            } => reads.push(request),
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Push(_)
            | Request::Fact { .. } => {}
        }
    }
    assert!(out.is_empty(), "delegation fixture settles");
    (root, reads)
}

fn activity(called: &[u64], heard: bool) -> c::ActivityPage {
    let mut objects = List::with_capacity(u32::try_from(called.len()).expect("tasks fit"));
    for task in called {
        objects.push(c::ObjectKey::Task(*task)).expect("task");
    }
    let mut blocks = List::with_capacity(3);
    blocks.push(c::Block::Text(text(b"I'll delegate this change in playground/widget."))).expect("agent words");
    blocks
        .push(c::Block::Call(c::Call {
            number: 7,
            tool: text(b"delegate"),
            input: text(b"{}"),
            outcome: Some(text(b"Delegated tasks: 2")),
            failed: false,
            objects,
            parts: List::with_capacity(0),
        }))
        .expect("call");
    blocks.push(c::Block::Result { call: 7, summary: text(b"Delegated tasks: 2"), failed: false }).expect("result");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).expect("message");
    let mut rows = List::with_capacity(4);
    if heard {
        for (position, from, words) in [(5, CHANGE, b"Carded result".as_slice()), (4, 3, b"Another task's result")] {
            rows.push(jig_client::contract::DecodedActivityLine {
                source: Box::new(c::ActivityLine {
                    task: CHAT,
                    position,
                    at: Wall::EPOCH,
                    line: c::Activity::Heard(c::Word {
                        number: position,
                        from: c::Requester::Task(from),
                        kind: c::MessageKind::Result(c::WordResult::Report),
                        words: text(words),
                        at: Wall::EPOCH,
                        hits: 0,
                        eligible: false,
                    }),
                }),
                decoded: None,
            })
            .expect("heard line");
        }
        let task = change();
        rows.push({
            let mut line = crate::tests::documents::result(
                1,
                Wall::EPOCH,
                c::Result {
                    task: task.chip,
                    ending: c::Ending::Done,
                    value: Some(c::ResultValue::Report(document(b"Duplicate result"))),
                    at: Some(Wall::EPOCH),
                    details: None,
                    acts: List::with_capacity(0),
                },
            );
            line.source.position = 3;
            line.source.at = Wall::EPOCH;
            line
        })
        .expect("result line");
    }
    rows.push(crate::tests::documents::turn(
        2,
        Wall::EPOCH,
        c::TurnName { attempt: 1, number: 1 },
        c::TurnRead::Decoded(c::Turn {
            messages,
            used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
            spent: 0,
            read_through: None,
            llm: None,
            cut: false,
        }),
    ))
    .expect("turn");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
fn view(task: Option<c::Task>, called: &[u64], heard: bool) -> crate::View {
    view_documents(task.as_slice(), called, heard)
}
fn view_documents(documents: &[c::Task], called: &[u64], heard: bool) -> crate::View {
    let root = root(documents, called, heard);
    view_root(&root)
}
fn view_root(root: &Domain) -> crate::View {
    let mut view = crate::View::new(
        &crate::Limits { tree: fixtures::tree_limits() },
        &limits(),
        fixtures::words(),
        fixtures::FORGE,
    );
    fixtures::build(&mut view, root);
    view
}
fn find(view: &crate::View, target: Find) -> Option<skein_ui::NodeId> {
    let find = target.within(Box::new([
        Target::Role { role: Role::Log, name: Box::from(b"Conversation".as_slice()) },
        Target::Role { role: Role::Region, name: Box::from(b"Task 2".as_slice()) },
    ]));
    match TreeFace::new(view.tree()).find(&find) {
        Found::Node(id) => Some(id),
        Found::Absent | Found::Pending => None,
    }
}
fn status(view: &crate::View, name: &[u8], token: skein_ui::Style) {
    let id = find(view, Find::exactly_named(Role::Status, name)).expect("card status");
    assert_eq!(view.tree().find(id).expect("status").styles.as_slice(), [token]);
}
fn steps(view: &crate::View, done: bool) {
    let id = find(view, Find::exactly_named(Role::List, b"Steps")).expect("steps");
    let mut face = TreeFace::subtree(view.tree(), id).expect("list");
    let mut count = 0_u32;
    for node in view.tree().nodes() {
        if node.styles.as_slice().contains(&look::STEP) {
            count = count.checked_add(1).expect("step count");
            assert_eq!(node.styles.as_slice().contains(&look::STEP_DONE), done);
        }
    }
    assert_eq!(count, STEPS);
    for name in [b"Branch pushed".as_slice(), b"Pull request opened", b"CI passed", b"Landed"] {
        if let Found::Node(_) = face.find(&Find::exactly_named(Role::Item, name)) {
        } else {
            panic!("step absent");
        }
    }
}
fn after_call() -> c::ActivityPage {
    let mut rows = List::with_capacity(1);
    rows.push(crate::tests::documents::line(
        3,
        Wall::EPOCH,
        jig_client::contract::Activity::Heard(jig_client::contract::Word {
            number: 3,
            from: jig_client::contract::Requester::Task(0),
            kind: jig_client::contract::MessageKind::Words,
            words: text(LATER),
            at: Wall::EPOCH,
            hits: 1,
            eligible: true,
        }),
    ))
    .expect("later activity");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
fn last_turn(view: &crate::View) -> &[skein_ui::Node] {
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.role != Role::Log {
            continue;
        }
        let end = index.checked_add(usize::try_from(node.size).expect("log size")).expect("log end");
        let inner = index.checked_add(1).expect("inner column");
        assert_eq!(nodes[inner].styles.as_slice(), [look::THREAD_INNER]);
        let mut child = inner.checked_add(1).expect("first child");
        let mut last = None;
        while child < end {
            if nodes[child].styles.as_slice().contains(&look::TURN) {
                last = Some(child);
            }
            child = child.checked_add(usize::try_from(nodes[child].size).expect("child size")).expect("next child");
        }
        let last = last.expect("last turn");
        let end = last.checked_add(usize::try_from(nodes[last].size).expect("turn size")).expect("turn end");
        return &nodes[last..end];
    }
    panic!("conversation log absent");
}
fn change_cards(view: &crate::View) -> u32 {
    let mut cards = 0_u32;
    for node in view.tree().nodes() {
        if node.role == Role::Region && node.name.as_deref() == Some(b"Task 2") {
            cards = cards.checked_add(1).expect("card count");
        }
    }
    cards
}
#[test]
fn a_running_change_shows_its_card_after_the_conversation_once_its_call_has_left() {
    let view = view_root(&root_activity(&[change()], after_call(), &[CHANGE]));
    assert_eq!(change_cards(&view), 1);
    let region = fixtures::find(view.tree(), Role::Region, b"Task 2").expect("task card");
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    status(&view, b"Running", look::STATUS_ACTIVE);
    let last = last_turn(&view);
    assert_eq!(last[0].styles.as_slice(), [look::TURN]);
    assert_eq!(last[1].id, region, "the card follows the conversation's last line");
}
#[test]
fn a_live_delegate_carded_at_its_call_shows_once() {
    let view = view_root(&root_activity(&[change()], activity(&[CHANGE], false), &[CHANGE]));
    assert_eq!(change_cards(&view), 1);
}
#[test]
fn a_live_delegate_without_its_document_shows_nothing_after_the_conversation() {
    let view = view_root(&root_activity(&[], after_call(), &[CHANGE]));
    assert_eq!(change_cards(&view), 0);
    let last = last_turn(&view);
    assert_eq!(last[1].role, Role::Paragraph);
    assert!(
        skein_lib::bytes::find(&fixtures::nodes_text(last), LATER).is_some(),
        "the last region is still the conversation's line"
    );
}
#[test]
fn a_running_change_shows_its_card_at_its_call() {
    let mut view = view(Some(change()), &[CHANGE], false);
    let region = fixtures::find(view.tree(), Role::Region, b"Task 2").expect("task card");
    assert_eq!(view.tree().find(region).expect("card").styles.as_slice(), [look::CARD, look::TASK_CARD]);
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    status(&view, b"Running", look::STATUS_ACTIVE);
    assert!(find(&view, Find::exactly_named(Role::Heading { level: 3 }, TITLE)).is_some());
    steps(&view, false);
    let link = find(&view, Find::exactly_named(Role::Link, b"Task 2")).expect("task link");
    let mut link_text = None;
    for (index, node) in view.tree().nodes().iter().enumerate() {
        if node.id == link {
            link_text = Some(fixtures::node_text(view.tree(), index));
            break;
        }
    }
    assert_eq!(link_text.as_deref(), Some("Task 2 ↗".as_bytes()));
    assert_eq!(
        crate::act(&mut view, link, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: Address::Task { task: CHANGE }
        }))
    );
    assert_eq!(work_slots(&view), 0);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Delegated tasks: 2").is_none());
    let said = fixtures::text(view.tree());
    assert!(
        skein_lib::bytes::find(&said, b"I'll delegate").expect("agent")
            < skein_lib::bytes::find(&said, "T2 · CHANGE".as_bytes()).expect("card")
    );
}
#[test]
fn a_landed_change_ticks_its_steps_and_says_its_merge_commit() {
    let view = view(Some(landed()), &[CHANGE], false);
    status(&view, b"Landed", look::STATUS_DONE);
    assert!(find(&view, Find::text("CI passed. Landed · merge commit 1a2b3c4".as_bytes())).is_some());
    steps(&view, true);
}
fn completed_steps(task: c::Task) -> u32 {
    let view = view(Some(task), &[CHANGE], false);
    let mut count = 0_u32;
    for node in view.tree().nodes() {
        if node.states.as_slice().contains(&skein_ui::State::Completed) {
            assert_eq!(node.role, Role::Item);
            assert!(node.styles.as_slice().contains(&look::STEP));
            assert!(node.styles.as_slice().contains(&look::STEP_DONE));
            count = count.checked_add(1).expect("completed step count");
        }
    }
    count
}
#[test]
fn a_landed_cards_four_steps_are_completed_items() {
    assert_eq!(completed_steps(landed()), STEPS);
}
#[test]
fn a_running_cards_steps_are_not_completed() {
    assert_eq!(completed_steps(change()), 0);
}
fn wake(called: bool, summary: &[u8]) -> c::ActivityPage {
    let mut blocks = List::with_capacity(1);
    blocks.push(c::Block::Result { call: 8, summary: text(summary), failed: false }).expect("wake result");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Person, blocks }).expect("wake message");
    let mut rows = List::with_capacity(2);
    rows.push(crate::tests::documents::turn(
        3,
        Wall::EPOCH,
        c::TurnName { attempt: 2, number: 1 },
        c::TurnRead::Decoded(c::Turn {
            messages,
            used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
            spent: 0,
            read_through: None,
            llm: None,
            cut: false,
        }),
    ))
    .expect("wake turn");
    if called {
        for line in &activity(&[CHANGE], false).rows {
            rows.push(line.clone()).expect("call turn");
        }
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
#[test]
fn a_landed_change_shows_its_card_at_its_result_once_its_call_has_left() {
    let view = view_root(&root_activity(&[landed()], wake(false, WAKE_SUMMARY), &[]));
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_some());
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    status(&view, b"Landed", look::STATUS_DONE);
    assert_eq!(work_slots(&view), 0);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Begin the work your brief describes.").is_none());
}
#[test]
fn a_card_shows_once_at_its_call_when_its_result_shares_the_window() {
    let view = view_root(&root_activity(&[landed()], wake(true, WAKE_SUMMARY), &[]));
    let mut cards = 0_u32;
    for node in view.tree().nodes() {
        if node.role == Role::Region && node.name.as_deref() == Some(b"Task 2") {
            cards = cards.checked_add(1).expect("card count");
        }
    }
    assert_eq!(cards, 1);
    let said = fixtures::text(view.tree());
    let call = skein_lib::bytes::find(&said, b"I'll delegate").expect("call turn");
    let card = skein_lib::bytes::find(&said, "T2 · CHANGE".as_bytes()).expect("card");
    assert!(call < card, "the card stays at its call");
    let log = view.tree().nodes();
    let mut regions = 0_u32;
    for (at, node) in log.iter().enumerate() {
        if node.styles.as_slice().contains(&look::TURN) {
            regions = regions.checked_add(1).expect("region count");
            if regions > 1 {
                let end = at.checked_add(usize::try_from(node.size).expect("region size")).expect("region end");
                for child in &log[at..end] {
                    if child.role == Role::Paragraph {
                        assert!(
                            child.name.as_deref() != Some(b"Person") && child.name.as_deref() != Some(b"You"),
                            "the result region has no speaker shell"
                        );
                    }
                }
            }
        }
    }
    assert!(regions > 1, "request and turn regions");
    assert_eq!(work_slots(&view), 0);
}
#[test]
fn a_result_without_its_document_draws_no_input_work() {
    let view = view_root(&root_activity(&[], wake(false, WAKE_SUMMARY), &[]));
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_none());
    assert_eq!(work_slots(&view), 0);
    assert!(fixtures::find(view.tree(), Role::Paragraph, WAKE_SUMMARY).is_none());
}
#[test]
fn a_persons_result_draws_no_input_work() {
    let summary = b"...\n2: Result from task 2:\nwords";
    let mut task = change();
    task.requester = c::Requester::Person(CHANGE);
    let view = view_root(&root_activity(&[task], wake(false, summary), &[]));
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_none());
    assert_eq!(work_slots(&view), 0);
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Tool completed").is_none());
    assert!(fixtures::find(view.tree(), Role::Paragraph, summary).is_none());
}
#[test]
fn a_partly_carded_result_draws_no_input_work() {
    let summary = b"Result from task 2:\nlanded\n3: Result from task 3:\nwaiting";
    let view = view_root(&root_activity(&[landed()], wake(false, summary), &[]));
    assert_eq!(work_slots(&view), 0);
    assert!(fixtures::find(view.tree(), Role::Paragraph, summary).is_none());
}
#[test]
fn each_state_says_its_words_in_its_status_look() {
    for (ending, held, name, token) in [
        (None, true, b"Held".as_slice(), look::STATUS_HELD),
        (Some(c::ChipEnding::Done(c::ResultValue::Report(document(b"Done")))), false, b"Completed", look::STATUS_DONE),
        (Some(c::ChipEnding::Failed(document(b"Failed"))), false, b"Failed", look::STATUS_HELD),
        (
            Some(c::ChipEnding::Cancelled {
                reason: document(b"Cancelled"),
                by: c::Requester::Person(1),
                result: None,
                at: Wall::EPOCH,
            }),
            false,
            b"Cancelled",
            look::STATUS_QUIET,
        ),
    ] {
        let mut task = change();
        if held {
            task.chip.phase = c::Phase::Held;
        }
        if let Some(ending) = ending {
            task.chip.details.phase = c::ChipPhase::Ended;
            task.chip.details.details.ending = Some(ending);
        }
        let view = view(Some(task), &[CHANGE], false);
        status(&view, name, token);
        steps(&view, false);
    }
}
#[test]
fn a_report_has_no_steps() {
    let mut task = change();
    task.result_contract = c::TaskContractDocument::Report { words: 256 };
    task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Report(document(
        b"A **useful** report",
    ))))));
    let view = view(Some(task), &[CHANGE], false);
    assert!(find(&view, Find::text("T2 · REPORT".as_bytes())).is_some());
    assert!(find(&view, Find::exactly_named(Role::Paragraph, b"A useful report")).is_some());
    assert!(find(&view, Find::named(Role::List, b"Steps")).is_none());
}
#[test]
fn a_failed_change_says_its_reason() {
    let mut task = change();
    task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Failed(document(b"Checks failed")))));
    let view = view(Some(task), &[CHANGE], false);
    status(&view, b"Failed", look::STATUS_HELD);
    assert!(find(&view, Find::text(b"Checks failed")).is_some());
}
#[test]
fn a_delegation_without_its_document_shows_its_call_work() {
    let view = view(None, &[CHANGE], false);
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_none());
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"delegate").is_some());
    super::chat::assert_run_line(view.tree(), b"delegate", b"", &[b"delegate: Delegated tasks: 2"]);
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"delegate: Delegated tasks: 2").is_some());
    assert_eq!(work_slots(&view), 1);
}
#[test]
fn a_carded_task_hides_its_heard_result() {
    let view = view(Some(landed()), &[CHANGE], true);
    let text = fixtures::text(view.tree());
    assert!(skein_lib::bytes::find(&text, b"Carded result").is_none());
    assert!(skein_lib::bytes::find(&text, b"Duplicate result").is_none());
    assert!(skein_lib::bytes::find(&text, b"Another task's result").is_some());
}
#[test]
fn a_partly_carded_call_keeps_its_work() {
    let view = view(Some(change()), &[CHANGE, 3], false);
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    assert_eq!(work_slots(&view), 1);
}

pub(super) fn step_roles() {
    let view = view(Some(landed()), &[CHANGE], false);
    for (token, role) in [(look::STEPS, Role::List), (look::STEP, Role::Item), (look::STEP_DONE, Role::Item)] {
        let mut found = false;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&token) {
                assert_eq!(node.role, role);
                found = true;
            }
        }
        assert!(found);
    }
}

#[test]
fn a_calls_cards_keep_its_task_order() {
    let mut other = change();
    other.chip.task = 3;
    let view = view_documents(&[change(), other], &[3, CHANGE], false);
    let mut cards = List::with_capacity(2);
    for node in view.tree().nodes() {
        if node.styles.as_slice().contains(&look::TASK_CARD) {
            cards.push(node.name.as_deref().expect("card name")).expect("two cards");
        }
        assert!(!node.styles.as_slice().contains(&look::WORK));
    }
    assert_eq!(cards.as_slice(), [b"Task 3".as_slice(), b"Task 2"]);
}

fn work_slots(view: &crate::View) -> u32 {
    let mut count = 0_u32;
    for node in view.tree().nodes() {
        if node.role == Role::Slot && node.styles.as_slice().contains(&look::WORK) {
            count = count.checked_add(1).expect("work slot count");
        }
    }
    count
}

fn first_run_view(dark: bool, landed_change: bool) -> crate::View {
    use fixtures::first_run as run;
    let mut receiving = limits();
    receiving.client.domain.rows = 3;
    receiving.client.domain.text = 512;
    receiving.client.domain.long_text = 512;
    receiving.client.domain.turn_text = 2048;
    let mut chat = change();
    chat.chip.task = if dark { 4 } else { CHAT };
    chat.requester = c::Requester::Person(fixtures::chat::PARTY);
    chat.chip.title = c::DocumentText::complete(run::text(if dark { run::DARK } else { run::CSV }));
    chat.spec.specification.words = chat.chip.title.clone();
    chat.chip.phase = c::Phase::Idle;
    chat.flags.ended_by = c::EndedBy::Requester;
    chat.acts = List::with_capacity(3);
    for act in [c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel] {
        chat.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).expect("chat act");
    }
    chat.activity = if dark {
        4
    } else if landed_change {
        9
    } else {
        5
    };
    let document = if landed_change { landed() } else { change() };
    let root = root_chat_limits(
        Some(&chat),
        &[document],
        run::activity(dark, landed_change, receiving.client.domain.turn_text),
        if dark || landed_change { &[] } else { &[CHANGE] },
        &receiving,
    );
    let mut view = crate::View::new(
        &crate::Limits { tree: fixtures::tree_limits() },
        &receiving,
        fixtures::words(),
        fixtures::FORGE,
    );
    fixtures::build(&mut view, &root);
    view
}
fn in_order(said: &[u8], expected: &[&[u8]]) {
    let mut rest = said;
    for value in expected {
        let at = skein_lib::bytes::find(rest, value).expect("next transcript text");
        rest = &rest[at.checked_add(value.len()).expect("matched text end")..];
    }
}
#[test]
fn the_csv_chat_receives_the_whole_turn_and_anchors_the_landed_card_at_its_call() {
    use fixtures::first_run as run;
    let view = first_run_view(false, true);
    super::chat::assert_run_line(view.tree(), b"wait", b"", &[b"wait"]);
    let said = fixtures::text(view.tree());
    in_order(
        &said,
        &[
            b"You",
            run::CSV,
            b"Temper",
            run::DELEGATING,
            "T2 · CHANGE".as_bytes(),
            b"wait",
            b"Temper",
            run::LANDED,
            b"wait",
        ],
    );
    for hidden in [
        b"Run 1 began".as_slice(),
        b"Earlier lines omitted",
        b"Run 1 ended",
        b"Begin the work your brief describes.",
        b"Milestone",
        b"Delegated tasks",
        run::MERGE,
        b"Instructions",
        b"Tool completed",
    ] {
        assert!(skein_lib::bytes::find(&said, hidden).is_none(), "{hidden:?}");
    }
    assert_eq!(TreeFace::new(view.tree()).find(&Find::exactly_named(Role::Paragraph, b"delegate")), Found::Absent);
    assert!(fixtures::find(view.tree(), Role::Button, b"Older").is_none());
    assert!(fixtures::find(view.tree(), Role::Status, b"Read").is_none());
}
// QA frame 004: the change's result reached the parked chat, before the
// resumed turn and before its independently watched task document caught up.
// Whole, the first turn's delegate call anchors the card; cut, the heard
// result does, and it is never drawn as bare words (client/view.md, 5).
#[test]
fn the_csv_chat_keeps_its_card_and_header_together_as_the_result_arrives() {
    for turn_text in [2048, 512] {
        csv_chat_hears_its_result(turn_text);
    }
}
fn csv_chat_hears_its_result(turn_text: u32) {
    use fixtures::first_run as run;
    let mut receiving = limits();
    receiving.client.domain.rows = 3;
    receiving.client.domain.text = 512;
    receiving.client.domain.long_text = 512;
    receiving.client.domain.turn_text = turn_text;
    let mut activity = run::activity(false, true, receiving.client.domain.turn_text);
    let mut rows = List::with_capacity(6);
    for line in &activity.rows {
        if line.source.position <= 6 {
            let mut line = line.clone();
            if let c::Activity::Heard(word) = &mut line.source.line {
                word.words = run::text(b"1d0b14e4db018fed000000000000000300000000");
            }
            rows.push(line).expect("before resumed turn");
        }
    }
    activity.rows = rows;
    let mut chat = change();
    chat.chip.task = CHAT;
    chat.requester = c::Requester::Person(fixtures::chat::PARTY);
    chat.waits_on = c::WaitsOn::News;
    chat.wake.words = c::WakeRule::Immediate;
    let c::ChipPhaseFacts { details, .. } = chat.chip.details.as_mut();
    details.live_delegates = 0;
    let mut task = change();
    task.requester = c::Requester::Task(CHAT);
    let (mut root, mut reads) = root_chat_with_reads(Some(&chat), &[task], activity, &[], &receiving);
    let mut view = crate::View::new(
        &crate::Limits { tree: fixtures::tree_limits() },
        &receiving,
        fixtures::words(),
        fixtures::FORGE,
    );
    fixtures::build(&mut view, &root);
    assert!(fixtures::find(view.tree(), Role::Status, b"Earlier text omitted").is_none());
    assert_csv_card(&view, false);
    let Some(Request::Unlinked {
        request:
            jig_client::UnlinkedRequest::Down {
                down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task: CHANGE }) },
            },
    }) = reads.pop()
    else {
        panic!("hearing T2's result refreshes its document")
    };
    assert!(reads.is_empty(), "one final document read");
    let mut out = Queue::uncapped();
    up(
        &mut root,
        jig_client::Up::ReadDone {
            read,
            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(landed())))),
        },
        &receiving,
        &mut out,
    );
    fixtures::build(&mut view, &root);
    assert_csv_card(&view, true);
}
fn assert_csv_card(view: &crate::View, landed: bool) {
    assert_eq!(change_cards(view), 1, "exactly one card for the change, at its call or its heard result");
    let mut bare_hex = false;
    for (index, _) in view.tree().nodes().iter().enumerate() {
        let text = fixtures::node_text(view.tree(), index);
        if text.len() == 40 {
            let mut hex = true;
            for byte in &text {
                if !byte.is_ascii_hexdigit() {
                    hex = false;
                }
            }
            bare_hex |= hex;
        }
    }
    assert!(!bare_hex, "the card owns the result; no bare merge commit text node");
    status(
        view,
        if landed { b"Landed" } else { b"Running" },
        if landed { look::STATUS_DONE } else { look::STATUS_ACTIVE },
    );
    assert_eq!(fixtures::find(view.tree(), Role::Status, b"waiting on 1 delegate").is_some(), !landed);
    assert_eq!(fixtures::find(view.tree(), Role::Status, b"waiting on your words").is_some(), landed);
}
#[test]
fn the_csv_chat_stops_waiting_when_its_card_lands_before_the_chat_document() {
    use fixtures::first_run as run;
    let mut receiving = limits();
    receiving.client.domain.rows = 3;
    receiving.client.domain.text = 512;
    receiving.client.domain.long_text = 512;
    receiving.client.domain.turn_text = 2048;
    let mut chat = change();
    chat.chip.task = CHAT;
    chat.requester = c::Requester::Person(fixtures::chat::PARTY);
    chat.waits_on = c::WaitsOn::Delegates;
    chat.wake.words = c::WakeRule::Immediate;
    let c::ChipPhaseFacts { details, .. } = chat.chip.details.as_mut();
    details.live_delegates = 1;
    let mut task = change();
    task.requester = c::Requester::Task(CHAT);
    for landed_change in [false, true] {
        let document = if landed_change { landed() } else { task.clone() };
        let root = root_chat_limits(
            Some(&chat),
            &[document],
            run::activity(false, false, receiving.client.domain.turn_text),
            &[CHANGE],
            &receiving,
        );
        let mut view = crate::View::new(
            &crate::Limits { tree: fixtures::tree_limits() },
            &receiving,
            fixtures::words(),
            fixtures::FORGE,
        );
        fixtures::build(&mut view, &root);
        assert_csv_card(&view, landed_change);
    }
}
#[test]
fn the_dark_theme_chat_has_its_whole_answer_and_waits_with_its_composer() {
    use fixtures::first_run as run;
    let view = first_run_view(true, false);
    let said = fixtures::text(view.tree());
    in_order(&said, &[b"You", run::DARK, b"Temper", run::DARK_ANSWER]);
    for hidden in [b"Done".as_slice(), b"Ended", b"finish", b"Milestone", b"Tool completed", b"Instructions"] {
        assert!(skein_lib::bytes::find(&said, hidden).is_none(), "{hidden:?}");
    }
    let mut composer = false;
    for node in view.tree().nodes() {
        composer |= node.styles.as_slice().contains(&look::SESSION_FOOT);
    }
    assert!(composer);
}

#[test]
fn a_failed_chat_has_one_end_line_with_its_reason() {
    let mut chat = change();
    chat.chip.task = CHAT;
    chat.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Failed(document(b"No capacity")))));
    let view = view_root(&root_chat(Some(&chat), &[], after_call(), &[]));
    assert!(fixtures::find(view.tree(), Role::Paragraph, "Failed · No capacity".as_bytes()).is_some());
}

#[test]
fn an_ended_chat_has_no_composer_or_chooser() {
    let mut chat = change();
    chat.chip.task = CHAT;
    chat.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Failed(document(b"No capacity")))));
    let view = view_root(&root_chat(Some(&chat), &[], after_call(), &[]));
    for node in view.tree().nodes() {
        for token in [look::SESSION_FOOT, look::CHOICE] {
            assert!(!node.styles.as_slice().contains(&token));
        }
        assert!(!(node.role == Role::Disclosure && node.name.as_deref() == Some(b"Model")));
    }
}

#[test]
fn cleared_and_uncleared_cards_keep_their_verification_words() {
    for (words, label) in [
        (b"1a2b3c4\nCleared by fast run 2.".as_slice(), b"Cleared".as_slice()),
        (
            b"1a2b3c4\nNot cleared: no verdict within the verification bound.".as_slice(),
            b"Landed, uncleared".as_slice(),
        ),
    ] {
        let mut task = landed();
        task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Change {
            kind: 1,
            part: c::Part { connector: fixtures::FORGE, kind: c::PartKind::Result, key: c::PartKey::Number(CHANGE) },
            words: document(words),
        }))));
        let view = view(Some(task), &[CHANGE], false);
        status(&view, label, look::STATUS_DONE);
        let newline = skein_lib::bytes::find(words, b"\n").unwrap();
        for line in [words.get(..newline).unwrap(), words.get(newline.checked_add(1).unwrap()..).unwrap()] {
            assert!(find(&view, Find::text(line)).is_some());
        }
        steps(&view, true);
    }
}

#[test]
fn delegation_links_keep_their_name_and_choose_the_shells_words() {
    let root = root(&[change()], &[CHANGE], false);
    for (words, link_text, browser) in
        [(fixtures::words(), "Task 2 ↗".as_bytes(), false), (fixtures::browser_words(), b"Open".as_slice(), true)]
    {
        let mut view =
            crate::View::new(&crate::Limits { tree: fixtures::tree_limits() }, &limits(), words, fixtures::FORGE);
        fixtures::build(&mut view, &root);
        let mut links = 0_u32;
        let mut link = None;
        for (at, node) in view.tree().nodes().iter().enumerate() {
            if node.role == Role::Link && node.name.as_deref() == Some(b"Task 2") {
                links = links.checked_add(1).expect("link count");
                link = Some(at);
            }
        }
        assert_eq!(links, 1);
        assert!(find(&view, Find::exactly_named(Role::Link, b"Task 2")).is_some());
        let at = link.expect("link index");
        assert_eq!(fixtures::node_text(view.tree(), at).as_ref(), link_text);
        assert_eq!(find(&view, Find::exactly_named(Role::Paragraph, b"Change")).is_some(), browser);
    }
}
#[test]
fn tracked_delegations_say_started_goal_in_the_browser_alone() {
    let mut task = change();
    task.priority = Some(1);
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    let root = root(&[task], &[CHANGE], false);
    for (words, browser) in [(fixtures::words(), false), (fixtures::browser_words(), true)] {
        let mut view =
            crate::View::new(&crate::Limits { tree: fixtures::tree_limits() }, &limits(), words, fixtures::FORGE);
        fixtures::build(&mut view, &root);
        assert_eq!(find(&view, Find::text(b"Started goal")).is_some(), browser);
        let title = find(&view, Find::exactly_named(Role::Heading { level: 3 }, TITLE)).expect("title");
        assert!(view.tree().find(title).expect("title").styles.as_slice().contains(&look::TOOL_FILE));
    }
}
#[test]
fn a_browser_held_delegation_keeps_the_held_token() {
    let mut task = change();
    task.chip.phase = c::Phase::Held;
    let root = root(&[task], &[CHANGE], false);
    let mut view = crate::View::new(
        &crate::Limits { tree: fixtures::tree_limits() },
        &limits(),
        fixtures::browser_words(),
        fixtures::FORGE,
    );
    fixtures::build(&mut view, &root);
    status(&view, b"Held", look::STATUS_HELD);
}

/// The browser chat's card hides progress steps (temper/docs/design/client/web/testing.md, 6).
#[test]
#[expect(clippy::disallowed_macros, reason = "the view's styles and their browser sheet jointly hide progress")]
fn a_browser_chat_card_hides_its_steps() {
    let root = root(&[change()], &[CHANGE], false);
    let view = fixtures::browser_page_view(&root);
    let list = find(&view, Find::exactly_named(Role::List, b"Steps")).expect("semantic progress list");
    assert!(view.tree().find(list).expect("progress list").styles.as_slice().contains(&look::STEPS));
    let card = fixtures::find(view.tree(), Role::Region, b"Task 2").expect("change card");
    assert!(view.tree().find(card).expect("change card").styles.as_slice().contains(&look::TASK_CARD));
    let mut session = false;
    for node in view.tree().nodes() {
        session |= node.styles.as_slice().contains(&look::SESSION);
    }
    assert!(session, "the card is in the browser session");
    let sheet = include_str!("../../../temper-client-shell-web/temper.css");
    let rule = sheet
        .split_once("#app > .temper-session .temper-task-card > :is(.temper-meta, .temper-steps) {")
        .expect("chat card's hidden parts")
        .1
        .split_once('}')
        .expect("style rule")
        .0;
    assert!(rule.contains("display: none;"), "chat cards hide their progress steps");
}
