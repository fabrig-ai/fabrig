//! What a task waits on, from its live subtree and the entries in it.
use crate::tests::*;
use fixtures::{make_batch, restore};

fn pair() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    let parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
    let child = new_row(&core, 2, tasks::Funder::Task(1), None);
    make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
    (core, limits)
}

fn document_matching_line(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> Task {
    let line = task_line(core, limits, 1, row, None).unwrap();
    let document = task_from_row(core, limits, 1, row, SINCE).unwrap();
    assert_eq!(document.wait, line.wait);
    assert_eq!(document.nearest, line.nearest);
    document
}

fn question(number: u64, at: u64) -> tasks::Word {
    tasks::Word {
        number,
        from: tasks::Party::Task(2),
        kind: tasks::MessageKind::Question,
        words: Box::from(&b"Question?"[..]),
        at: Wall::from_nanos(at),
        hits: 1,
        eligible: true,
    }
}

#[test]
fn an_askers_credit_keeps_question_words_while_it_closes() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.question_lines =
        Box::new([tasks::QuestionLine { message: 3, line: parent.activity, answer: None, resolution: None }]);
    let mut child = core.tasks.task(2).unwrap().clone();
    let question = tasks::OpenQuestion { asker: 2, number: 3, line: child.activity };
    child.questions = Box::new([tasks::QuestionCredit {
        number: 3,
        answerer: tasks::Party::Task(1),
        asked: tasks::Party::Task(1),
        line: child.activity,
        offer: None,
    }]);
    child.phase = tasks::Phase::Closing(tasks::Closing {
        stage: tasks::Stage::Effects,
        ending: tasks::Ending::Failed { reason: Box::new([]) },
    });
    restore(&mut core, &limits, Box::new([parent, child]));
    assert!(core.remember_question(&tasks::ActivityLine {
        task: question.asker,
        position: question.line,
        at: Wall::EPOCH,
        line: tasks::Line::Words {
            message: question.number,
            by: tasks::Party::Task(2),
            kind: tasks::MessageKind::Question,
            words: b"Which form?".as_slice().into()
        },
    }));
    assert_eq!(core.tasks.open_questions(1).as_slice(), [question]);
    core.prune_questions(2);
    assert_eq!(core.open_question_words(question), Some(b"Which form?".as_slice()));
    document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
    document_matching_line(&core, &limits, core.tasks.task(2).unwrap());
}

#[test]
fn a_task_with_live_delegates_and_no_entries_waits_on_its_delegates() {
    let (core, limits) = pair();
    assert_eq!(core.waits_on(&limits, 1), WaitsOn::Delegates);
    document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
}

#[test]
fn a_question_to_the_requester_names_that_party_and_one_entry() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.questions = Box::new([own_credit(&parent, 7)]);
    parent.inbox = Box::new([question(3, 10)]);
    parent.last_message = 3;
    parent.question_lines =
        Box::new([tasks::QuestionLine { message: 3, line: parent.activity, answer: None, resolution: None }]);
    let child = core.tasks.task(2).unwrap().clone();
    restore(&mut core, &limits, Box::new([parent, child]));
    let position = core.tasks.task(1).unwrap().activity;
    assert!(core.remember_question(&tasks::ActivityLine {
        task: 1,
        position,
        at: Wall::from_nanos(10),
        line: tasks::Line::Words {
            message: 3,
            by: tasks::Party::Task(2),
            kind: tasks::MessageKind::Question,
            words: b"Question?".as_slice().into()
        }
    }));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Person(1),
            entries: 1,
            task: 1,
            kind: people::EntryKind::Question { message: 3 },
        }
    );
    document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
}

#[test]
fn the_oldest_of_two_waiting_entries_names_whom() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.questions = Box::new([own_credit(&parent, 7)]);
    parent.inbox = Box::new([question(3, 10)]);
    parent.last_message = 3;
    parent.question_lines =
        Box::new([tasks::QuestionLine { message: 3, line: parent.activity, answer: None, resolution: None }]);
    let mut child = core.tasks.task(2).unwrap().clone();
    child.revisions.hold = 1;
    child.escalation = tasks::Escalation::Waiting {
        revision: 1,
        holder: tasks::EscalationHolder::Person(2),
        entry: 4,
        since: Wall::from_nanos(9),
    };
    child.phase =
        tasks::Phase::Held { why: tasks::Hold::Budget, source: None, was: tasks::Was::Active(tasks::Active::Due) };
    restore(&mut core, &limits, Box::new([parent, child]));
    let position = core.tasks.task(1).unwrap().activity;
    assert!(core.remember_question(&tasks::ActivityLine {
        task: 1,
        position,
        at: Wall::from_nanos(10),
        line: tasks::Line::Words {
            message: 3,
            by: tasks::Party::Task(2),
            kind: tasks::MessageKind::Question,
            words: b"Question?".as_slice().into()
        }
    }));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Person(2),
            entries: 2,
            task: 2,
            kind: people::EntryKind::Escalation { revision: 1 },
        }
    );
    document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
}

#[test]
fn a_proposal_below_says_the_party_is_waited_on_for_that_proposal() {
    let (mut core, limits) = pair();
    let parent = core.tasks.task(1).unwrap().clone();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.revision = 1;
    child.proposal = Some(Box::new(tasks::Proposal {
        number: 4,
        proposer: 2,
        project: 1,
        action: tasks::ProposalAction::Release { task: 1 },
        reason: Box::new([]),
        as_holder: false,
        state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(1), since: Wall::EPOCH },
    }));
    restore(&mut core, &limits, Box::new([parent, child]));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Person(1),
            entries: 1,
            task: 2,
            kind: people::EntryKind::Proposal { proposer: Proposer::Task(2), number: 4 },
        }
    );
    document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
}

#[test]
fn a_person_task_below_names_its_role() {
    let (mut core, limits) = pair();
    let parent = core.tasks.task(1).unwrap().clone();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.executor = tasks::Executor::Person(tasks::PersonAddress::Role(1));
    child.person_wait = Some(tasks::PersonWait {
        since: Wall::EPOCH,
        stall: Duration::from_secs(1),
        deadline: Wall::from_nanos(1_000_000_000),
        notice: None,
        offer: None,
    });
    restore(&mut core, &limits, Box::new([parent, child]));
    assert_eq!(
        core.waits_on(&limits, 1),
        WaitsOn::Party {
            whom: people::Whom::Role { project: 1, role: 1 },
            entries: 1,
            task: 2,
            kind: people::EntryKind::PersonTask,
        }
    );
    document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
}

#[test]
fn an_idle_task_with_nothing_below_waits_on_news() {
    let (mut core, limits) = pair();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.phase = tasks::Phase::Active(tasks::Active::Idle);
    restore(&mut core, &limits, Box::new([child]));
    assert_eq!(core.waits_on(&limits, 2), WaitsOn::News);
    document_matching_line(&core, &limits, core.tasks.task(2).unwrap());
}

#[test]
fn a_working_task_with_live_delegates_waits_on_nothing() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    parent.attempt = 1;
    parent.run_since = Some(Wall::EPOCH);
    let child = core.tasks.task(2).unwrap().clone();
    restore(&mut core, &limits, Box::new([parent, child]));
    assert_eq!(core.waits_on(&limits, 1), WaitsOn::Nothing);
    let document = document_matching_line(&core, &limits, core.tasks.task(1).unwrap());
    assert_eq!(document.wait, None);
    assert_eq!(document.nearest, None);
}

#[test]
fn a_task_not_live_waits_on_nothing() {
    let (core, limits) = pair();
    assert_eq!(core.waits_on(&limits, 99), WaitsOn::Nothing);
}

#[test]
fn a_kept_idle_row_waits_on_dependencies_before_delegates_and_cuts_oldest_first() {
    let (mut core, mut limits) = tree_core();
    let parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
    let older = new_row(&core, 3, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch_at(&mut core, &limits, tasks::Party::Person(1), Box::new([older]), Wall::from_nanos(10));
    let newer = new_row(&core, 2, tasks::Funder::Task(1), None);
    fixtures::make_batch_at(&mut core, &limits, tasks::Party::Task(1), Box::new([newer]), Wall::from_nanos(20));
    limits.documents.lines = 1;
    let mut row = core.tasks.task(1).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.dependencies = Box::new([2, 3]);
    row.waiting_on = Box::new([2, 3]);
    let WaitsOn::Dependencies { pending, count } = document_matching_line(&core, &limits, &row).waits_on else {
        panic!("kept")
    };
    assert_eq!(pending.as_slice(), &[3]);
    assert_eq!(count, 2);
    row.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    assert_eq!(document_matching_line(&core, &limits, &row).waits_on, WaitsOn::Nothing);
}

fn own_credit(row: &tasks::TaskRecord, number: u64) -> tasks::QuestionCredit {
    tasks::QuestionCredit { number, answerer: row.requester, asked: row.requester, line: row.activity, offer: None }
}

#[test]
fn an_idle_task_waits_on_its_oldest_own_question_before_dependencies_and_delegates() {
    let (mut core, limits) = pair();
    let delegate = new_row(&core, 3, tasks::Funder::Task(2), None);
    make_batch(&mut core, &limits, tasks::Party::Task(2), Box::new([delegate]));
    let mut row = core.tasks.task(2).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.waiting_on = Box::new([1]);
    row.questions = Box::new([own_credit(&row, 9), own_credit(&row, 7)]);
    assert_eq!(tasks::own_question(&row), Some(7));
    assert_eq!(document_matching_line(&core, &limits, &row).waits_on, WaitsOn::Question { number: 7 });
    row.waiting_on = Box::new([]);
    row.delegates = Box::new([]);
    assert_eq!(document_matching_line(&core, &limits, &row).waits_on, WaitsOn::Question { number: 7 });
    row.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    assert_eq!(document_matching_line(&core, &limits, &row).waits_on, WaitsOn::Nothing);
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.questions = Box::new([]);
    assert_eq!(document_matching_line(&core, &limits, &row).waits_on, WaitsOn::News);
}

#[test]
fn asking_and_answering_an_own_question_change_the_task_document_wait() {
    let (mut core, limits) = pair();
    let parent = core.tasks.task(1).unwrap().clone();
    let ledger = *core.tasks.funding(tasks::Funder::Period { project: 1, period: 1 }).unwrap();
    let mut child = core.tasks.task(2).unwrap().clone();
    child.phase = tasks::Phase::Active(tasks::Active::Idle);
    restore(&mut core, &limits, Box::new([parent, child]));
    send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) });
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    let document_wait = |core: &Core| document_matching_line(core, &limits, core.tasks.task(2).unwrap()).waits_on;
    assert_eq!(document_wait(&core), WaitsOn::News);
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::Message {
            reply_to: ReplyTo::new(Token::new(90)),
            project: 1,
            operation: tasks::Message { task: 1, word: question(7, 10) },
        }),
    );
    assert!(!core.tasks.task(2).unwrap().questions.is_empty(), "{out:?}");
    assert_eq!(document_wait(&core), WaitsOn::Question { number: 7 });
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::Message {
            reply_to: ReplyTo::new(Token::new(91)),
            project: 1,
            operation: tasks::Message {
                task: 2,
                word: tasks::Word {
                    number: 8,
                    from: tasks::Party::Task(1),
                    kind: tasks::MessageKind::Answer { question: 7 },
                    words: b"Use this form".as_slice().into(),
                    at: Wall::from_nanos(11),
                    hits: 1,
                    eligible: true,
                },
            },
        }),
    ));
    assert!(core.tasks.task(2).unwrap().questions.is_empty());
    assert_eq!(document_wait(&core), WaitsOn::Nothing, "the answer wakes the next run");
}

use crate::{AtBound, Bound, Nearest, Unbounded, Wait, WaitKind, WaitObject};
use tasks::EscalationHolder as Holder;

const SINCE: Wall = Wall::from_nanos(100);
const SOON: Wall = Wall::from_nanos(300);
const FINAL: Holder = Holder::Role { project: 1, role: 0 };
const REQUESTER: Holder = Holder::Person(1);
const PARENT: Holder = Holder::Task(1);

fn at(since: Wall, figure: Duration, then: AtBound) -> Bound {
    Bound::At { at: Wall::from_nanos(since.as_nanos().checked_add(figure.as_nanos()).unwrap()), then }
}

fn remember(core: &mut Core, task: u64, line: u64, message: u64) {
    core.question_activity
        .insert(
            (task, line),
            tasks::ActivityLine {
                task,
                position: line,
                at: SINCE,
                line: tasks::Line::Words {
                    message,
                    by: tasks::Party::Task(task),
                    kind: tasks::MessageKind::Question,
                    words: b"Which?".as_slice().into(),
                },
            },
        )
        .expect("room for the fixture's open question");
}

#[test]
fn party_question_uses_asking_time_and_distinguishes_pass_up_from_overdue() {
    let (mut core, limits) = pair();
    let mut row = core.tasks.task(1).unwrap().clone();
    row.question_lines =
        Box::new([tasks::QuestionLine { message: 7, line: row.activity, answer: None, resolution: None }]);
    remember(&mut core, row.number, row.activity, 7);
    let mut expected = Wait {
        task: 1,
        kind: WaitKind::Party,
        object: WaitObject::Entry { kind: people::EntryKind::Question { message: 7 }, at: Holder::Person(1) },
        since: SINCE,
        bound: at(SINCE, limits.waits.party, AtBound::PassUp),
        hears: Some(REQUESTER),
    };
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(document.waits.get(0), Some(&expected));
    row.project = 99; // No project policy, covering holder or readable document.
    expected.bound = at(SINCE, limits.waits.party, AtBound::Overdue);
    expected.hears = None;
    assert_eq!(crate::runs::waits::read(&core, &limits, &row).waits.first, Some(expected));
}

#[test]
fn party_entries_keep_the_escalation_proposal_and_person_clocks() {
    let (core, limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.phase =
        tasks::Phase::Held { why: tasks::Hold::Budget, source: None, was: tasks::Was::Active(tasks::Active::Due) };
    row.escalation = tasks::Escalation::Waiting { holder: Holder::Task(1), revision: 1, entry: 7, since: SINCE };
    row.proposal = Some(Box::new(tasks::Proposal {
        number: 8,
        proposer: 2,
        project: 1,
        action: tasks::ProposalAction::Release { task: 2 },
        reason: Box::new([]),
        as_holder: false,
        state: tasks::ProposalState::Pending {
            holder: tasks::ProposalHolder::Policy { project: 1, kind: tasks::ProposalKind::Release },
            since: SINCE,
        },
    }));
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(
        document.waits.as_slice(),
        &[
            Wait {
                task: 2,
                kind: WaitKind::Party,
                object: WaitObject::Entry { kind: people::EntryKind::Escalation { revision: 1 }, at: Holder::Task(1) },
                since: SINCE,
                bound: at(SINCE, limits.tasks.escalation_stall, AtBound::PassUp),
                hears: Some(PARENT),
            },
            Wait {
                task: 2,
                kind: WaitKind::Party,
                object: WaitObject::Entry {
                    kind: people::EntryKind::Proposal { proposer: Proposer::Task(2), number: 8 },
                    at: FINAL
                },
                since: SINCE,
                bound: at(SINCE, limits.tasks.proposal_stall, AtBound::Overdue),
                hears: Some(PARENT),
            },
        ]
    );
    row.escalation = tasks::Escalation::Waiting { holder: FINAL, revision: 1, entry: 7, since: SINCE };
    let stalls = limits.tasks.escalation_stall.max(limits.tasks.proposal_stall);
    let past = Wall::from_nanos(SINCE.as_nanos() + stalls.as_nanos() + 1);
    let document = task_from_row(&core, &limits, 1, &row, past).unwrap();
    let expected = [limits.tasks.escalation_stall, limits.tasks.proposal_stall];
    assert_eq!(document.waits.len(), 2);
    for (record, stall) in document.waits.iter().zip(expected) {
        assert_eq!(record.bound, at(SINCE, stall, AtBound::Overdue));
        let Bound::At { at, .. } = record.bound else { panic!("a final role has a stall bound") };
        assert!(at < past);
    }
    row.escalation = tasks::Escalation::Unheld { revision: 1 };
    row.proposal = None;
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.person_wait = Some(tasks::PersonWait {
        since: SINCE,
        stall: limits.tasks.person_stall,
        deadline: SOON,
        notice: None,
        offer: None,
    });
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(
        document.wait,
        Some(Wait {
            task: 2,
            kind: WaitKind::Party,
            object: WaitObject::Entry { kind: people::EntryKind::PersonTask, at: FINAL },
            since: SINCE,
            bound: at(SINCE, limits.tasks.person_stall, AtBound::Overdue),
            hears: Some(PARENT),
        })
    );
}

#[test]
fn a_person_offer_moves_the_one_party_wait_and_its_bound() {
    const ADDRESSEE: u64 = 6;
    const FIRST_OFFER: Wall = Wall::from_nanos(200);
    const FINAL_OFFER: Wall = Wall::from_nanos(300);
    let (core, limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Due);
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Person(ADDRESSEE));
    let stall = limits.tasks.person_stall;
    let deadline = Wall::from_nanos(SINCE.as_nanos() + stall.as_nanos());
    let notice = tasks::PersonStallNotice { message: 7, at: FIRST_OFFER, activity: 1 };
    let cases = [
        (None, Holder::Person(ADDRESSEE), SINCE, AtBound::PassUp),
        (Some(tasks::PersonAddress::Person(1)), REQUESTER, FIRST_OFFER, AtBound::PassUp),
        (Some(tasks::PersonAddress::Role(0)), FINAL, FINAL_OFFER, AtBound::Overdue),
    ];
    for (address, holder, since, then) in cases {
        let offer = match address {
            Some(holder) => Some(tasks::PersonOffer { holder, since }),
            None => None,
        };
        row.person_wait =
            Some(tasks::PersonWait { since: SINCE, stall, deadline, notice: offer.is_some().then_some(notice), offer });
        let document = document_matching_line(&core, &limits, &row);
        let expected = Wait {
            task: row.number,
            kind: WaitKind::Party,
            object: WaitObject::Entry { kind: people::EntryKind::PersonTask, at: holder },
            since,
            bound: at(since, stall, then),
            hears: Some(PARENT),
        };
        assert_eq!(document.waits.as_slice(), core::slice::from_ref(&expected));
        assert_eq!(document.wait, Some(expected));
    }
}

#[test]
fn requester_question_precedes_pending_dependency_and_respects_the_deadline() {
    let (mut core, limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.questions = Box::new([own_credit(&row, 7)]);
    row.waiting_on = Box::new([1]);
    remember(&mut core, row.number, row.activity, 7);
    let mut expected = Wait {
        task: 2,
        kind: WaitKind::Question,
        object: WaitObject::Question { number: 7, answerer: 1 },
        since: SINCE,
        bound: at(SINCE, limits.waits.question, AtBound::PassUp),
        hears: Some(PARENT),
    };
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(document.waits.get(0), Some(&expected));
    assert_eq!(document.waits.get(1).unwrap().kind, WaitKind::Dependency);
    row.authority.budget.deadline = Some(SOON);
    expected.bound = Bound::At { at: SOON, then: AtBound::PassUp };
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(document.wait, Some(expected));
    assert_eq!(document.nearest, Some(Nearest { at: SOON, then: AtBound::PassUp, kind: WaitKind::Question }));
}

#[test]
fn dependency_records_use_addition_time_and_keep_the_count_past_the_cap() {
    let (core, mut limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.created_at = Wall::from_nanos(10);
    row.phase = tasks::Phase::Active(tasks::Active::Due);
    row.executor = tasks::Executor::Procedure { connector: 2, code: 1 };
    row.waiting_on = Box::new([3, 4]);
    row.added = Box::new([tasks::Added { task: 3, at: SINCE, by: tasks::Party::Task(1) }]);
    let mut expected = [
        Wait {
            task: 2,
            kind: WaitKind::Dependency,
            object: WaitObject::Task(4),
            since: row.created_at,
            bound: Bound::None(Unbounded::NoDeadline),
            hears: Some(PARENT),
        },
        Wait {
            task: 2,
            kind: WaitKind::Dependency,
            object: WaitObject::Task(3),
            since: SINCE,
            bound: Bound::None(Unbounded::NoDeadline),
            hears: Some(PARENT),
        },
    ];
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(document.waits.as_slice(), &expected);
    assert_eq!(document.nearest, None);
    row.authority.budget.deadline = Some(SOON);
    limits.documents.lines = 1;
    expected[0].bound = Bound::At { at: SOON, then: AtBound::Hold };
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(document.wait_count, 2);
    assert_eq!(document.waits.as_slice(), &expected[..1]);
}

#[test]
fn resource_wait_names_the_holder_and_counts_from_the_latest_join() {
    let (mut core, limits) = tree_core();
    let name = tasks::Name { connector: 2, path: Box::new([b"one".as_slice().into()]) };
    let hold = tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits };
    for event in [
        tasks::Event::Kinds { connector: 2, kinds: Box::new([tasks::Kind { connector: 2, kind: 1, hold }]) },
        tasks::Event::Resource { name: name.clone(), hold },
        tasks::Event::ResourceAccess { project: 1, name: name.clone(), access: tasks::ResourceAccess::Writable },
    ] {
        send_task(&mut core, &limits, event);
    }
    for (task, made) in [(1, Wall::EPOCH), (2, SINCE)] {
        let mut member = new_row(&core, task, tasks::Funder::Period { project: 1, period: 1 }, None);
        member.holdings = Box::new([tasks::Holding::Write { resource: name.clone(), kind: 1 }]);
        fixtures::make_batch_at(&mut core, &limits, tasks::Party::Person(1), Box::new([member]), made);
    }
    let mut row = core.tasks.task(2).unwrap().clone();
    let mut expected = Wait {
        task: 2,
        kind: WaitKind::Resource,
        object: WaitObject::Resource { name, holder: Some(1) },
        since: SINCE,
        bound: at(SINCE, limits.tasks.hold_wait, AtBound::Hold),
        hears: Some(REQUESTER),
    };
    assert_eq!(core.waits(2).first, Some(expected.clone()));
    assert_eq!(core.waits_on(&limits, 2), WaitsOn::Nothing);
    document_matching_line(&core, &limits, &row);
    row.hold_bound_since = Some(SOON);
    expected.since = SOON;
    expected.bound = at(SOON, limits.tasks.hold_wait, AtBound::Hold);
    assert_eq!(document_matching_line(&core, &limits, &row).wait, Some(expected));
}

#[test]
fn delegates_use_their_making_time_and_replace_news() {
    const PASSED: Wall = Wall::from_nanos(SINCE.as_nanos() - 1);
    let (mut core, limits) = pair();
    let delegate = new_row(&core, 3, tasks::Funder::Task(1), None);
    fixtures::make_batch_at(&mut core, &limits, tasks::Party::Task(1), Box::new([delegate]), SINCE);
    let mut row = core.tasks.task(1).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.delegates = Box::new([3, 2]);
    let mut expected = [
        Wait {
            task: 1,
            kind: WaitKind::Delegate,
            object: WaitObject::Task(2),
            since: Wall::EPOCH,
            bound: Bound::None(Unbounded::NoDeadline),
            hears: Some(REQUESTER),
        },
        Wait {
            task: 1,
            kind: WaitKind::Delegate,
            object: WaitObject::Task(3),
            since: SINCE,
            bound: Bound::None(Unbounded::NoDeadline),
            hears: Some(REQUESTER),
        },
    ];
    let document = document_matching_line(&core, &limits, &row);
    assert_eq!(document.waits.as_slice(), &expected);
    assert_eq!(document.nearest, None);
    row.authority.budget.deadline = Some(SOON);
    for wait in &mut expected {
        wait.bound = Bound::At { at: SOON, then: AtBound::Hold };
    }
    assert_eq!(document_matching_line(&core, &limits, &row).waits.as_slice(), &expected);

    // A delegate made after the deadline is due as soon as its wait begins.
    row.authority.budget.deadline = Some(PASSED);
    expected[0].bound = Bound::At { at: PASSED, then: AtBound::Hold };
    expected[1].bound = Bound::At { at: SINCE, then: AtBound::Hold };
    assert_eq!(document_matching_line(&core, &limits, &row).waits.as_slice(), &expected);
}

#[test]
fn parked_news_keeps_four_subscriptions_and_the_timer_but_standing_has_no_bound() {
    let (core, limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.phase_since = SINCE;
    row.subscriptions = Box::new([6, 7, 8, 9, 10].map(|number| tasks::Subscription {
        number,
        kind: if number == 10 {
            tasks::SubscriptionKind::Timer { at: SOON, period: None }
        } else {
            tasks::SubscriptionKind::Topic { connector: 2, topic: number }
        },
    }));
    let mut expected = Wait {
        task: 2,
        kind: WaitKind::News,
        since: SINCE,
        hears: Some(PARENT),
        object: WaitObject::News { subscriptions: List::from_boxed(Box::new([6, 7, 8, 9])), timer: Some(SOON) },
        bound: at(SINCE, limits.waits.news, AtBound::Hold),
    };
    assert_eq!(document_matching_line(&core, &limits, &row).wait, Some(expected.clone()));
    expected.bound = Bound::None(Unbounded::Standing);
    row.flags.ended_by = tasks::EndedBy::Standing;
    assert_eq!(document_matching_line(&core, &limits, &row).wait, Some(expected.clone()));
    row.flags.ended_by = tasks::EndedBy::Requester;
    row.flags.conversation = true;
    assert_eq!(document_matching_line(&core, &limits, &row).wait, Some(expected));
}

#[test]
fn a_goal_names_a_descendants_escalation_but_its_own_nearer_bound() {
    let (mut core, limits) = pair();
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.tracked = Some(1);
    parent.phase = tasks::Phase::Active(tasks::Active::Idle);
    parent.waiting_on = Box::new([2]);
    parent.authority.budget.deadline = Some(SOON);
    let mut child = core.tasks.task(2).unwrap().clone();
    child.escalation = tasks::Escalation::Waiting { holder: Holder::Person(1), revision: 1, entry: 7, since: SINCE };
    restore(&mut core, &limits, Box::new([child]));
    let document = document_matching_line(&core, &limits, &parent);
    let first = document.wait.expect("party wait below the goal");
    assert_eq!((first.task, first.kind), (2, WaitKind::Party));
    assert_eq!(document.nearest, Some(Nearest { at: SOON, then: AtBound::Hold, kind: WaitKind::Dependency }));
}

#[test]
fn a_goal_keeps_its_descendants_question_and_the_earliest_bound() {
    let (mut core, limits) = pair();
    let mut goal = core.tasks.task(1).unwrap().clone();
    goal.tracked = Some(1);
    goal.phase = tasks::Phase::Active(tasks::Active::Idle);
    goal.waiting_on = Box::new([2]);
    let leaf = new_row(&core, 3, tasks::Funder::Period { project: 1, period: 1 }, None);
    make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([leaf]));
    let mut leaf = core.tasks.task(3).unwrap().clone();
    leaf.question_lines =
        Box::new([tasks::QuestionLine { message: 7, line: leaf.activity, answer: None, resolution: None }]);
    remember(&mut core, leaf.number, leaf.activity, 7);
    let mut child = core.tasks.task(2).unwrap().clone();
    // Supply the wait walk with a person-facing question below the direct delegate.
    child.delegates = Box::new([3]);
    restore(&mut core, &limits, Box::new([goal.clone(), child, leaf]));

    let later = Wall::from_nanos(SINCE.as_nanos() + limits.waits.party.as_nanos());
    let question = Wait {
        task: 3,
        kind: WaitKind::Party,
        object: WaitObject::Entry { kind: people::EntryKind::Question { message: 7 }, at: REQUESTER },
        since: SINCE,
        bound: Bound::At { at: later, then: AtBound::PassUp },
        hears: Some(REQUESTER),
    };
    for (deadline, nearest) in [
        (SOON, Nearest { at: SOON, then: AtBound::Hold, kind: WaitKind::Dependency }),
        (Wall::from_nanos(later.as_nanos() + 1), Nearest { at: later, then: AtBound::PassUp, kind: WaitKind::Party }),
    ] {
        goal.authority.budget.deadline = Some(deadline);
        let document = document_matching_line(&core, &limits, &goal);
        assert_eq!(document.wait.as_ref(), Some(&question));
        assert_eq!(document.nearest, Some(nearest));
        assert_eq!(document.waits.get(0).unwrap().kind, WaitKind::Dependency);
    }
}

#[test]
fn a_working_kept_tasks_dependency_counts_toward_its_nearest_bound() {
    let (core, limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.attempt = 1;
    row.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    row.waiting_on = Box::new([1]);
    row.authority.budget.deadline = Some(SOON);
    let doc = document_matching_line(&core, &limits, &row);
    assert_eq!((doc.wait, doc.waits_on), (None, WaitsOn::Nothing));
    assert_eq!(doc.nearest, Some(Nearest { at: SOON, then: AtBound::Hold, kind: WaitKind::Dependency }));
    assert_eq!(doc.waits.get(0).unwrap().object, WaitObject::Task(1));
}

#[test]
fn working_line_has_no_first_but_its_document_lists_party_and_question_waits() {
    let (mut core, limits) = pair();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.attempt = 1;
    row.phase = tasks::Phase::Active(tasks::Active::Running { attempt: 1 });
    row.questions = Box::new([own_credit(&row, 7)]);
    row.proposal = Some(Box::new(tasks::Proposal {
        number: 8,
        proposer: 2,
        project: 1,
        action: tasks::ProposalAction::Release { task: 2 },
        reason: Box::new([]),
        as_holder: false,
        state: tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(1), since: SINCE },
    }));
    remember(&mut core, row.number, row.activity, 7);
    let doc = document_matching_line(&core, &limits, &row);
    assert_eq!(doc.wait, None);
    assert_eq!(doc.waits_on, WaitsOn::Nothing);
    assert_eq!(doc.wait_count, 2);
    assert_eq!(doc.waits.get(0).unwrap().kind, WaitKind::Party);
    assert_eq!(doc.waits.get(1).unwrap().kind, WaitKind::Question);
}
