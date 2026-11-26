//! Changes rows join live ancestry or bounded ended loads, without storing provenance
//! (client/forge/records.md, Where each row came from).
use super::{projection_settled::live_task, resume_fixture as fixture};
use crate::{
    Decision, Domain, Limits, Record, Work,
    boundary::{EndedRead, Read},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token, Wall};
use temper_engine_domain_forge as forge;

fn lineage() -> [tasks::TaskRecord; 3] {
    let (root, _, task) = live_task(Some(1), tasks::Executor::Procedure { connector: 0, code: 1 });
    let mut milestone = root.engine.core().tasks.task(task).unwrap().clone();
    milestone.number = 1;
    milestone.root = 1;
    milestone.spec.words = b"Store shape\nMilestone words".as_slice().into();
    milestone.executor = tasks::Executor::Procedure { connector: core::RECURRING, code: core::GATHERING };
    milestone.flags.ended_by = tasks::EndedBy::Requester;
    milestone.delegates = Box::new([2]);
    let mut goal = milestone.clone();
    goal.number = 2;
    goal.requester = tasks::Party::Task(1);
    goal.depth = 1;
    goal.executor = tasks::Executor::Procedure { connector: 0, code: 1 };
    goal.flags = tasks::Flags::PLAIN;
    goal.spec.words = b"Ship the store\nGoal words".as_slice().into();
    goal.delegates = Box::new([3]);
    let mut change = goal.clone();
    change.number = 3;
    change.requester = tasks::Party::Task(2);
    change.depth = 2;
    change.tracked = None;
    change.spec.words = b"Change".as_slice().into();
    change.delegates = Box::new([]);
    [milestone, goal, change]
}

fn ledger() -> Record {
    let (root, _, number) = live_task(None, tasks::Executor::Procedure { connector: 0, code: 1 });
    let row = root.engine.core().tasks.task(number).unwrap();
    Record::Core(core::Record::Tasks(tasks::Stored::Ledger(*root.engine.core().tasks.funding(row.funder).unwrap())))
}

fn root(rows: &[tasks::TaskRecord]) -> (Domain, Env<Limits>) {
    let mut limits = fixture::limits();
    limits.tasks.tasks = 4;
    limits.people.inbox_tasks = 8;
    limits.tasks.project_tasks = 4;
    limits.tasks.project_goals = 4;
    limits.tasks.tree_tasks = 4;
    limits.tasks.inbox_bytes = 256;
    limits.tasks.depth = 3;
    let mut records = List::with_capacity(5);
    if !rows.is_empty() {
        records.push(ledger()).unwrap();
    }
    for row in rows {
        records.push(Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(row.clone()))))).unwrap();
    }
    fixture::root_with_records(fixture::config(7), &limits, records.as_slice())
}

fn document(task: u64) -> forge::ChangesView {
    let landed = forge::ChangesLanded {
        goal: None,
        chat: None,
        milestone: None,
        signed: Box::new([]),
        task,
        title: b"Change".as_slice().into(),
        at: Wall::EPOCH,
        commit: [7; 32],
        default_branch: true,
    };
    forge::ChangesView {
        catching_up: None,
        project: 1,
        landing: b"main".as_slice().into(),
        more: 0,
        landed_more: 0,
        landed_default_more: 0,
        landed: Box::new([landed.clone()]),
        landed_default: Box::new([landed]),
        queues: Box::new([forge::ChangesQueue {
            repository: b"owner/repo".as_slice().into(),
            branch: b"main".as_slice().into(),
            red: false,
            way: temper_engine_domain_forge_change::Way::Turn,
            changes: Box::new([forge::ChangesQueued {
                task,
                title: b"Change".as_slice().into(),
                ready_since: Wall::EPOCH,
                checks_since: None,
                goal: None,
                chat: None,
                milestone: None,
            }]),
            in_flight: Some(forge::TrainLanding {
                task: 99,
                member: task,
                title: b"Change".as_slice().into(),
                round: 1,
                step: forge::LandingStep::Preparing,
                probe: None,
                ready_since: Wall::EPOCH,
                goal: None,
                chat: None,
                milestone: None,
            }),
        }]),
    }
}

fn fill(root: &mut Domain, env: &Env<Limits>, view: &mut forge::ChangesView) {
    let mut decision = Decision::decide(&mut root.journal, &env.limits.journal);
    crate::changes_from::fill(root, env, &mut decision, view);
    assert!(decision.write_free(), "provenance and signer names are memory-only");
    release(root, env, decision);
}

fn release(root: &mut Domain, env: &Env<Limits>, decision: Decision) {
    crate::decision::accept_routed(
        &mut root.journal,
        &mut root.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
    crate::decision::resume(&mut root.journal, &mut Queue::uncapped());
    root.archive_reads.reclaim();
}

fn facts(view: &forge::ChangesView) -> [&forge::ChangesLanded; 2] {
    [&view.landed[0], &view.landed_default[0]]
}

fn named(view: &forge::ChangesView) {
    let queued = &view.queues[0].changes[0];
    let flight = view.queues[0].in_flight.as_ref().unwrap();
    for (goal, chat, milestone) in [
        (&queued.goal, &queued.chat, &queued.milestone),
        (&flight.goal, &flight.chat, &flight.milestone),
        (&view.landed[0].goal, &view.landed[0].chat, &view.landed[0].milestone),
        (&view.landed_default[0].goal, &view.landed_default[0].chat, &view.landed_default[0].milestone),
    ] {
        assert_eq!(goal, &Some(forge::ChangeFrom { task: 2, title: b"Ship the store".as_slice().into() }));
        assert_eq!(chat, &None);
        assert_eq!(milestone, &Some(forge::ChangeFrom { task: 1, title: b"Store shape".as_slice().into() }));
    }
}

#[test]
fn every_change_row_names_its_live_goal_and_milestone_by_chip_title() {
    let (mut root, env) = root(&lineage());
    let mut view = document(3);
    fill(&mut root, &env, &mut view);
    named(&view);
    assert!(root.archive_reads.is_empty());
}

fn waiter(root: &Domain) -> (Token, u64) {
    let mut reads = root.archive_reads.iter();
    let (id, Some(Read::Ended(EndedRead::ChangeFrom { task }))) = reads.next().expect("one pending ancestor") else {
        panic!("provenance load")
    };
    assert!(reads.next().is_none(), "queue, flight and both histories share a task's load");
    (id.token(), *task)
}

fn answer(root: &mut Domain, env: &Env<Limits>, row: Option<tasks::TaskRecord>, failed: bool) {
    let (waiter, task) = waiter(root);
    if failed {
        crate::archive::ended_failed(root, waiter);
    } else {
        let rows: Box<[Record]> = match row {
            Some(row) => Box::new([Record::Core(core::Record::Tasks(tasks::Stored::Ended(Box::new(row))))]),
            None => Box::new([]),
        };
        crate::archive::ended_loaded(root, waiter, rows);
    }
    let Some(Work::ChangeFromLoaded { task: loaded, row }) = root.work.pop() else {
        panic!("ended load routed to provenance")
    };
    assert_eq!(loaded, task);
    let mut decision = Decision::decide(&mut root.journal, &env.limits.journal);
    crate::changes_from::loaded(root, env, &mut decision, task, row);
    release(root, env, decision);
    let Some(Work::Core(core::Event::Marked { connector, subjects })) = root.work.pop() else {
        panic!("arrival marks changes")
    };
    assert_eq!(connector, root.config.forge_connector);
    assert_eq!(subjects.as_slice(), &[Token::new(forge::changes_subject(1))]);
    assert!(root.work.is_empty());
}

#[test]
fn cold_landed_rows_load_each_ended_ancestor_once_then_mark_the_project() {
    let (mut root, env) = root(&[]);
    let mut view = document(3);
    fill(&mut root, &env, &mut view);
    assert_eq!(waiter(&root).1, 3);
    for row in facts(&view) {
        assert_eq!((&row.goal, &row.chat, &row.milestone), (&None, &None, &None));
    }
    fill(&mut root, &env, &mut view);
    for mut row in lineage().into_iter().rev() {
        assert_eq!(waiter(&root).1, row.number);
        row.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
        row.ended_at = Some(Wall::EPOCH);
        answer(&mut root, &env, Some(row), false);
    }
    fill(&mut root, &env, &mut view);
    named(&view);
    assert!(root.archive_reads.is_empty());
}

#[test]
fn missing_or_failed_ended_ancestors_leave_no_partial_names() {
    for failed in [false, true] {
        let (mut root, env) = root(&[]);
        let mut view = document(3);
        fill(&mut root, &env, &mut view);
        answer(&mut root, &env, Some(lineage()[2].clone()), false);
        answer(&mut root, &env, Some(lineage()[1].clone()), false);
        answer(&mut root, &env, None, failed);
        fill(&mut root, &env, &mut view);
        for row in facts(&view) {
            assert_eq!((&row.goal, &row.chat, &row.milestone), (&None, &None, &None));
        }
        assert!(root.archive_reads.is_empty(), "failed provenance is not retried on every document");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test constructs a title cut inside a UTF-8 character")]
fn only_a_lineage_root_on_the_projects_chat_charter_names_a_chat() {
    for executor in [people::TemplateExecutor::Charter(1), people::TemplateExecutor::Chosen] {
        let mut config = fixture::config(7);
        config.core.defaults.templates = Box::new([core::Template {
            number: 1,
            name: b"Chat".as_slice().into(),
            executor,
            contract: tasks::Contract::Report { words: 32 },
            shallower: 1,
            budget: people::TemplateBudget::Given,
            deadline: people::TemplateDeadline::Given,
            tracked: None,
            flags: tasks::Flags::PLAIN,
            label: 1,
            grace: None,
            llm: None,
            resources: Box::new([]),
        }]);
        let mut row = lineage()[2].clone();
        row.number = 2;
        row.root = 2;
        row.depth = 0;
        row.requester = tasks::Party::Deployment { project: 1 };
        row.executor = tasks::Executor::Agent { charter: 1 };
        row.spec.words = format!("{}€\nChat words", "é".repeat(127)).into_bytes().into();
        row.delegates = Box::new([3]);
        let mut child = lineage()[2].clone();
        child.root = 2;
        child.requester = tasks::Party::Task(2);
        child.depth = 1;
        let mut limits = fixture::limits();
        limits.tasks.spec_bytes = 1024;
        limits.tasks.inbox_bytes = 256;
        let (mut root, env) = fixture::root_with_records(
            config,
            &limits,
            &[
                ledger(),
                Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(row)))),
                Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(child)))),
            ],
        );
        let mut view = document(3);
        fill(&mut root, &env, &mut view);
        let row = &view.landed[0];
        assert_eq!((&row.goal, &row.milestone), (&None, &None));
        match executor {
            people::TemplateExecutor::Charter(_) => {
                assert_eq!(row.chat, Some(forge::ChangeFrom { task: 2, title: "é".repeat(127).into_bytes().into() }));
            }
            people::TemplateExecutor::Chosen => assert_eq!(row.chat, None),
        }
    }
}

#[test]
fn signers_use_the_project_members_name_then_login_and_no_name_for_an_outsider() {
    let mut limits = fixture::limits();
    limits.people.people = 3;
    limits.people.holdings = 3;
    let mut rows = List::with_capacity(4);
    for (person, login, name) in
        [(1u64, b"one".as_slice(), b"First".as_slice()), (2, b"two", b""), (3, b"outside", b"Outside")]
    {
        rows.push(Record::Core(core::Record::People(people::Stored::Person {
            number: person,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: person.to_be_bytes().into() },
                login: login.into(),
                name: name.into(),
            },
        })))
        .unwrap();
    }
    rows.push(Record::Core(core::Record::People(people::Stored::Roles {
        project: 1,
        holdings: Box::new([
            people::Holding { person: 1, role: people::Role::Owner },
            people::Holding { person: 2, role: people::Role::Member },
        ]),
    })))
    .unwrap();
    let (mut root, env) = fixture::root_with_records(fixture::config(7), &limits, rows.as_slice());
    let mut view = document(999);
    let mut signed = List::with_capacity(3);
    for person in 1..=3 {
        signed.push(forge::ChangesSigned { person, head: [7; 32], at: Wall::EPOCH, name: None }).unwrap();
    }
    view.landed[0].signed = signed.into_boxed();
    fill(&mut root, &env, &mut view);
    for (sign, expected) in view.landed[0].signed.iter().zip([Some(b"First".as_slice()), Some(b"two".as_slice()), None])
    {
        assert_eq!(sign.name.as_deref(), expected);
    }
}
