//! A direct giver replaces the financial chain without changing the requester tree.
use super::{ask_now, sign_in};
use crate::tests::*;
use jig_domain_contract as contract;

const OTHER: u64 = 2;
const TARGET: u64 = 10;
const PARENT: u64 = 11;
const OLD: tasks::Funder = tasks::Funder::Pool { project: 1, person: OTHER, period: 1 };
const OWNER: tasks::Funder = tasks::Funder::Pool { project: 1, person: 1, period: 1 };

fn family(open_owner: bool, procedure: bool) -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    sign_in(&mut core, &limits);
    for person in [OTHER, 1] {
        if person == 1 && !open_owner {
            continue;
        }
        send_task(
            &mut core,
            &limits,
            tasks::Event::CarvePool {
                reply_to: ReplyTo::new(Token::new(1)),
                project: 1,
                person,
                period: 1,
                budget: 400,
            },
        );
    }
    let mut parent = new_row(&core, PARENT, OLD, None);
    parent.numbers.budget = 200;
    parent.authority.budget.spend = 200;
    if procedure {
        parent.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    }
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(OTHER), Box::new([parent]));
    let mut target = new_row(&core, TARGET, tasks::Funder::Task(PARENT), None);
    target.numbers.budget = 100;
    target.authority = minimal_authority(100);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(PARENT), Box::new([target]));
    // The other party has already given the target a widening, leaving its tree intact.
    send_task(
        &mut core,
        &limits,
        tasks::Event::Amend {
            reads_project: false,
            reply_to: ReplyTo::new(Token::new(2)),
            by: tasks::Party::Person(OTHER),
            places: Box::new([]),
            operation: tasks::Amend {
                task: TARGET,
                message: 1,
                stop_run: false,
                funder: Some(OLD),
                amendment: Box::new(fixtures::unchanged(b"fund")),
            },
        },
    );
    send_task(&mut core, &limits, tasks::Event::ChargeEffect { funder: tasks::Funder::Task(TARGET), maximum: 30 });
    assert_eq!(core.tasks.task(TARGET).expect("target").numbers.spent, 30);
    core.counters.deployment.messages = 1;
    (core, limits)
}

fn minimal_authority(spend: u64) -> tasks::Authority {
    tasks::Authority {
        tools: tasks::Tools(0),
        models: tasks::Models::ALL,
        grants: Box::new([]),
        delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
        budget: tasks::Budget { spend, deadline: None },
        notes: tasks::Scopes(0),
        note_resources: Box::new([]),
    }
}

fn change(spend: u64, deadline: Option<Wall>) -> people::Amendment {
    let mut change = fixtures::person_unchanged(b"authority");
    change.authority = Some(people::Authority {
        tools: 0,
        models: None,
        grants: Box::new([]),
        delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
        spend,
        deadline,
        notes: 0,
        note_resources: Box::new([]),
    });
    change
}

fn amend(core: &mut Core, limits: &Limits, key: u8, change: people::Amendment) -> people::Outcome {
    let revision = core.tasks.task(TARGET).expect("target").revisions.spec;
    ask_now(
        core,
        limits,
        key,
        people::Ask::Amend { project: 1, task: TARGET, revision, amendment: change, release: None },
    )
}

fn balances(core: &Core, funder: tasks::Funder) -> tasks::Numbers {
    core.tasks.funding(funder).expect("pool").numbers
}

#[test]
fn an_owners_direct_widening_refunds_spend_to_the_givers_pool() {
    let (mut core, limits) = family(true, false);
    let before = balances(&core, OLD);
    assert_eq!(
        amend(&mut core, &limits, 10, change(150, None)),
        people::Outcome::Amended { task: TARGET, released: false }
    );
    let task = core.tasks.task(TARGET).expect("widened task");
    assert_eq!((task.requester, task.root), (tasks::Party::Task(PARENT), PARENT));
    assert_eq!(task.funder, OWNER);
    assert_eq!(task.numbers, tasks::Numbers { budget: 120, spent: 0, spent_below: 0, reserved: 0 });
    assert_eq!(task.authority.budget.spend, 120);
    assert_eq!(balances(&core, OLD).reserved, before.reserved.checked_sub(100).expect("reservation"));
    assert_eq!(balances(&core, OLD).spent_below, before.spent_below.checked_add(30).expect("spend"));
    assert_eq!(balances(&core, OWNER).reserved, 120);
}

#[test]
fn an_authority_only_widening_opens_the_givers_pool_alone_or_in_a_compound() {
    for compound in [false, true] {
        let (mut core, limits) = tree_core();
        sign_in(&mut core, &limits);
        send_task(
            &mut core,
            &limits,
            tasks::Event::CarvePool {
                reply_to: ReplyTo::new(Token::new(1)),
                project: 1,
                person: OTHER,
                period: 1,
                budget: 400,
            },
        );
        let mut target = new_row(&core, TARGET, OLD, None);
        target.numbers.budget = 0;
        target.authority = minimal_authority(0);
        fixtures::make_batch(&mut core, &limits, tasks::Party::Person(OTHER), Box::new([target]));
        assert!(core.tasks.funding(OWNER).is_none());
        let mut edit = change(0, None);
        edit.authority.as_mut().expect("authority").tools = 1;
        if compound {
            let mut operations = List::with_capacity(1);
            operations
                .push(people::Operation::Amend { task: TARGET, revision: 0, amendment: Box::new(edit), release: None })
                .expect("one amendment");
            assert_eq!(
                ask_now(&mut core, &limits, 10, people::Ask::Compound { project: 1, operations }),
                people::Outcome::Compound { project: 1, operations: 1, first_message: 1 }
            );
        } else {
            assert_eq!(amend(&mut core, &limits, 10, edit), people::Outcome::Amended { task: TARGET, released: false });
        }
        let task = core.tasks.task(TARGET).expect("widened task");
        assert_eq!((task.funder, task.authority.tools, task.numbers.budget), (OWNER, tasks::Tools(1), 0));
        assert_eq!(balances(&core, OWNER), tasks::Numbers { budget: 500, spent: 0, spent_below: 0, reserved: 0 });
    }
}

#[test]
fn a_partys_widening_of_its_earlier_period_task_funds_it_from_the_open_period() {
    let (mut core, limits) = tree_core();
    sign_in(&mut core, &limits);
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(1)),
            project: 1,
            person: 1,
            period: 1,
            budget: 500,
        },
    );
    let mut target = new_row(&core, TARGET, OWNER, None);
    target.authority = minimal_authority(100);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([target]));
    send_task(&mut core, &limits, tasks::Event::ChargeEffect { funder: tasks::Funder::Task(TARGET), maximum: 30 });
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 2, budget: 1000 },
    );
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(1)),
            project: 1,
            person: 1,
            period: 2,
            budget: 500,
        },
    );
    assert_eq!(
        amend(&mut core, &limits, 10, change(150, None)),
        people::Outcome::Amended { task: TARGET, released: false }
    );
    let giver = tasks::Funder::Pool { project: 1, person: 1, period: 2 };
    let task = core.tasks.task(TARGET).expect("widened task");
    assert_eq!((task.funder, task.numbers.budget, task.numbers.spent), (giver, 120, 0));
    assert_eq!((balances(&core, OWNER).reserved, balances(&core, OWNER).spent_below), (0, 30));
    assert_eq!(balances(&core, giver).reserved, 120);
}

#[test]
fn an_owners_narrowing_and_word_only_edit_keep_the_old_funder() {
    let (mut core, limits) = family(false, false);
    let before = balances(&core, OLD);
    assert_eq!(
        amend(&mut core, &limits, 10, change(80, Some(Wall::from_nanos(1000)))),
        people::Outcome::Amended { task: TARGET, released: false }
    );
    let task = core.tasks.task(TARGET).expect("narrowed task");
    assert_eq!(task.funder, OLD);
    assert_eq!((task.numbers.budget, task.numbers.spent), (80, 30));
    assert_eq!(balances(&core, OLD).reserved, before.reserved.checked_sub(20).expect("difference"));
    let mut words = fixtures::person_unchanged(b"words");
    words.instructions = Some(b"new instructions".as_slice().into());
    assert_eq!(amend(&mut core, &limits, 11, words), people::Outcome::Amended { task: TARGET, released: false });
    assert_eq!(core.tasks.task(TARGET).expect("edited task").funder, OLD);
    assert!(core.tasks.funding(OWNER).is_none());
}

#[test]
fn a_compound_widening_uses_the_projected_funder_and_rebased_budget() {
    let (mut core, limits) = family(true, false);
    let revision = core.tasks.task(TARGET).expect("target").revisions.spec;
    let mut operations = List::with_capacity(2);
    for (offset, spend) in [(0, 150), (1, 140)] {
        operations
            .push(people::Operation::Amend {
                task: TARGET,
                revision: revision.checked_add(offset).expect("revision"),
                amendment: Box::new(change(spend, None)),
                release: None,
            })
            .expect("two amendments");
    }
    assert_eq!(
        ask_now(&mut core, &limits, 10, people::Ask::Compound { project: 1, operations }),
        people::Outcome::Compound { project: 1, operations: 2, first_message: 2 }
    );
    let task = core.tasks.task(TARGET).expect("target");
    assert_eq!(task.funder, OWNER);
    assert_eq!((task.numbers.budget, task.numbers.spent, task.authority.budget.spend), (140, 0, 140));
    assert_eq!(balances(&core, OWNER).reserved, 140);
    assert_eq!(balances(&core, OLD).spent_below, 30);
}

#[test]
fn a_run_widening_takes_back_a_delegate_then_resizes_its_own_reservation() {
    let (mut core, limits) = family(true, false);
    core.proofs.insert(PARENT, RunProof::claimed(PARENT, 7, 7, None, fleet::HostKind::Worker)).expect("proof");
    for (completion, spend, budget) in [(1, 150, 120), (2, 140, 140)] {
        let mut change = fixtures::unchanged(b"widen delegate");
        change.authority = Some(minimal_authority(spend));
        let revision = core.tasks.task(TARGET).expect("target").revisions.spec;
        drop(watch_outputs(
            &mut core,
            &limits,
            Event::NamedAction {
                to: ReplyTo::new(Token::new(90 + u64::from(completion))),
                key: CallKey { task: PARENT, attempt: 7, completion, position: 0 },
                action: NamedAction::Amend { target: TARGET, revision, keep_parameters: false, amendment: change },
            },
        ));
        let task = core.tasks.task(TARGET).expect("target");
        assert_eq!((task.funder, task.numbers.budget, task.numbers.spent), (tasks::Funder::Task(PARENT), budget, 0));
        assert_eq!(core.tasks.task(PARENT).expect("giver").numbers.reserved, budget);
    }
}

#[test]
fn a_procedure_steps_widening_gives_from_the_procedure_task() {
    let (mut core, limits) = family(false, true);
    let mut change = fixtures::unchanged(b"widen delegate");
    change.authority = Some(minimal_authority(150));
    let revision = core.tasks.task(TARGET).expect("target").revisions.spec;
    let (_, _, step_number) = core.tasks.procedure_due(PARENT).expect("due procedure");
    let decision = ProcedureStep {
        amendments: Box::new([StepAmendment { task: TARGET, revision, amendment: change }]),
        ..ProcedureStep::new(StepEnd::Wait)
    };
    drop(watch_connector_outputs(
        &mut core,
        &limits,
        1,
        contract::Up::Procedures(contract::procedures::Up::Step { task: PARENT, step: step_number, code: 1, decision }),
    ));
    let task = core.tasks.task(TARGET).expect("target");
    assert_eq!((task.funder, task.numbers.budget, task.numbers.spent), (tasks::Funder::Task(PARENT), 120, 0));
    assert_eq!(core.tasks.task(PARENT).expect("procedure").numbers.reserved, 120);
}
