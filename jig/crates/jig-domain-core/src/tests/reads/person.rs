//! Person reads admit one project and recheck retained ownership (domain/engine/executors.md, 3).
use crate::reads::{
    self,
    person::{Admission, admitted},
};
use crate::tests::*;
use jig_domain_people::{ApplicationScope, Proposer};

const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const TASK: u64 = 1;
const RETAINED: u64 = 90;
const PERSON: u64 = 1;
const KEY: CallKey = CallKey { task: TASK, attempt: 7, completion: 500, position: 0 };
const QUERY: TaskQuery = TaskQuery {
    project: Some(PROJECT),
    requester: None,
    root: None,
    parent: None,
    label: None,
    phase: None,
    executor: None,
    tracked: None,
    order: QueryOrder::Number,
};

#[test]
fn admission_names_exactly_one_project_or_a_task_to_check() {
    for project in [PROJECT, OTHER_PROJECT] {
        let named = if project == PROJECT { Admission::Project } else { Admission::Refused };
        for read in [
            reads::Read::Project { project },
            reads::Read::Spend { project, from: None },
            reads::Read::Tasks { query: TaskQuery { project: Some(project), ..QUERY }, live: None, ended: None },
            reads::Read::Notes { project, from: None },
            reads::Read::ScopedNotes { scope: notes::Scope::Project { project }, description: None, from: None },
            reads::Read::ScopedNotes {
                scope: notes::Scope::Goal { project, goal: TASK },
                description: None,
                from: None,
            },
            reads::Read::Inbox {
                group: reads::Group::Needs,
                filter: reads::Filter { project: Some(project), goal: None },
                from: None,
            },
            reads::Read::Application {
                scope: ApplicationScope::Project(project),
                connector: 1,
                read: Token::new(1),
                window: window(),
            },
        ] {
            assert_eq!(admitted(&read, PROJECT), named, "{read:?}");
        }
        let task = u64::from(project);
        for read in [
            reads::Read::Task { task },
            reads::Read::TaskText { task, field: reads::TaskTextField::Specification, from: 0 },
            reads::Read::Activity { task, before: None },
            reads::Read::ActivityBeside { task, position: 1, side: reads::Side::Earlier },
            reads::Read::Turn { task, attempt: 1, turn: 1 },
            reads::Read::EndedDelegates { task, from: None },
            reads::Read::History { task, subtree: true, from: None },
            reads::Read::Result { task },
            reads::Read::Reply { task, activity: 1 },
            reads::Read::Question { task, message: 1 },
            reads::Read::Escalation { task },
            reads::Read::PersonTask { task },
            reads::Read::Proposal { proposer: Proposer::Task(task), proposal: 1 },
        ] {
            assert_eq!(admitted(&read, PROJECT), Admission::Task(task), "{read:?}");
        }
    }
    for read in [
        reads::Read::Frame,
        reads::Read::Deployment,
        reads::Read::Settings,
        reads::Read::Setting { setting: crate::settings::Setting::Charter(1), revision: 0, proposal: None },
        reads::Read::Revision { number: 1 },
        reads::Read::Revisions { before: None },
        reads::Read::Proposal { proposer: Proposer::Party(PERSON), proposal: 1 },
        reads::Read::Tasks { query: TaskQuery { project: None, ..QUERY }, live: None, ended: None },
        reads::Read::Inbox {
            group: reads::Group::Needs,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        },
        reads::Read::Spend { project: people::DEPLOYMENT, from: None },
        reads::Read::ScopedNotes { scope: notes::Scope::Deployment, description: None, from: None },
        reads::Read::ScopedNotes {
            scope: notes::Scope::Resources {
                project: PROJECT,
                connector: 1,
                pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
            },
            description: None,
            from: None,
        },
        reads::Read::Application { scope: ApplicationScope::Own, connector: 1, read: Token::new(1), window: window() },
        reads::Read::Application {
            scope: ApplicationScope::Deployment,
            connector: 1,
            read: Token::new(1),
            window: window(),
        },
    ] {
        assert_eq!(admitted(&read, PROJECT), Admission::Refused, "{read:?}");
    }
    assert_eq!(admitted(&reads::Read::Project { project: 0 }, 0), Admission::Refused);
}

fn retained_row(core: &Core) -> reads::Row {
    let mut owner = core.tasks.task(TASK).expect("person's run").clone();
    owner.number = RETAINED;
    owner.phase =
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: b"retained".as_slice().into() }));
    stored_row(Record::Tasks(tasks::Stored::Ended(Box::new(owner))), None)
}

fn stored_load(core: &mut Core, limits: &Limits) -> (Token, reads::Row) {
    let row = retained_row(core);
    core.settings.tools.ask = authority::Tools(1);
    let mut out = reads::person::run(
        core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        ReplyTo::new(Token::new(500)),
        KEY,
        reads::Read::Task { task: RETAINED },
    );
    let Some(Request::Held(held)) = out.pop() else { panic!("retained load") };
    let Held::ReadLoad { load, .. } = *held else { panic!("retained load") };
    assert!(out.is_empty());
    (load, row)
}

fn completed(core: &mut Core, limits: &Limits, load: Token, result: reads::Loaded) -> Option<reads::Answer> {
    let mut out = reads::loaded(
        core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        &mut Queue::uncapped(),
        load,
        result,
    );
    let answer = match out.pop() {
        Some(Request::Now(now)) => {
            let Now::PersonAnswer { key, answer, .. } = *now else { panic!("person answer") };
            assert_eq!(key, KEY);
            Some(*answer)
        }
        Some(_) => panic!("unrecorded answer"),
        None => None,
    };
    assert!(out.is_empty(), "no writes or call records");
    answer
}

#[derive(Debug, Clone, Copy)]
enum Change {
    Attempt,
    Requester(tasks::Party),
    Project,
    Role,
    Closed,
}

#[test]
fn stored_person_reads_recheck_the_attempt_requester_and_role() {
    for (change, expected) in [
        (Change::Attempt, None),
        (Change::Requester(tasks::Party::Person(2)), Some(reads::Answer::None(reads::Why::Unknown))),
        (Change::Requester(tasks::Party::Task(2)), Some(reads::Answer::None(reads::Why::Unknown))),
        (
            Change::Requester(tasks::Party::Deployment { project: PROJECT }),
            Some(reads::Answer::None(reads::Why::Unknown)),
        ),
        (Change::Project, Some(reads::Answer::None(reads::Why::Unknown))),
        (Change::Role, Some(reads::Answer::None(reads::Why::Unknown))),
        (Change::Closed, Some(reads::Answer::None(reads::Why::Unknown))),
    ] {
        let (mut core, limits) = watch_core();
        let (load, row) = stored_load(&mut core, &limits);
        let mut caller = core.tasks.task(TASK).expect("caller").clone();
        match change {
            Change::Attempt => core.proofs.get_mut(&TASK).expect("current attempt").attempt = 8,
            Change::Requester(party) => caller.requester = party,
            Change::Project => caller.project = OTHER_PROJECT,
            Change::Role => {
                people::step(
                    &mut core.people,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
                    people::Event::Roles { project: PROJECT, holdings: Box::new([]) },
                    &mut Queue::uncapped(),
                );
            }
            Change::Closed => core.projects.clear(),
        }
        fixtures::restore(&mut core, &limits, Box::new([caller]));
        assert_eq!(
            completed(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None }),
            expected,
            "{change:?}"
        );
    }
}

#[test]
fn stored_person_reads_refuse_foreign_owners_and_linked_rows() {
    for linked in [false, true] {
        let (mut core, limits) = watch_core();
        let (load, mut row) = stored_load(&mut core, &limits);
        let Record::Tasks(tasks::Stored::Ended(owner)) = &mut row.value else { panic!("owner") };
        if linked {
            let mut foreign = owner.clone();
            foreign.project = OTHER_PROJECT;
            row.linked = Box::new([foreign]);
        } else {
            owner.project = OTHER_PROJECT;
        }
        assert_eq!(
            completed(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None }),
            Some(reads::Answer::None(reads::Why::Unknown)),
            "linked {linked}"
        );
    }
}

#[test]
fn retained_pages_prove_their_owner_even_when_empty() {
    for read in [
        reads::Read::Activity { task: RETAINED, before: None },
        reads::Read::ActivityBeside { task: RETAINED, position: 1, side: reads::Side::Earlier },
        reads::Read::History { task: RETAINED, subtree: true, from: None },
        reads::Read::EndedDelegates { task: RETAINED, from: None },
    ] {
        let (mut core, mut limits) = watch_core();
        limits.load_slots = 1;
        let row = retained_row(&core);
        core.settings.tools.ask = authority::Tools(1);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut out = reads::person::run(&mut core, &env, ReplyTo::new(Token::new(500)), KEY, read.clone());
        let Some(Request::Held(held)) = out.pop() else { panic!("owner load: {read:?}") };
        let Held::ReadLoad { load, range } = *held else { panic!("owner load") };
        assert_eq!(
            range.scope,
            reads::Scope::TaskText { task: RETAINED, field: reads::TaskTextField::Specification, from: 0 }
        );
        let mut next = reads::loaded(
            &mut core,
            &env,
            &mut Queue::uncapped(),
            load,
            reads::Loaded::Page { rows: Box::new([row]), next: None },
        );
        let Some(Request::Held(held)) = next.pop() else { panic!("page load: {read:?}") };
        let Held::ReadLoad { load, range } = *held else { panic!("page load reuses its admission") };
        assert_eq!(Some(range.scope), reads::scope(&read));
        let answer = completed(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([]), next: None })
            .expect("empty page");
        let reads::Answer::Page(page) = answer else { panic!("empty retained page: {read:?}") };
        assert!(page.rows.is_empty());
        assert!(page.next.is_none());
    }
}
