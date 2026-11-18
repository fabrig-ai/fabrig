use super::*;
use jig_core_world::fixtures::{finish_chat, second_party};
use jig_test_profile::fixtures::{people_member, person_unchanged, unchanged};

const PROJECT: u32 = 1;
const OTHER: u64 = 3;
const ORIGINAL: tasks::Funder = tasks::Funder::Pool { project: PROJECT, person: OWNER, period: 1 };
const GIVER: tasks::Funder = tasks::Funder::Pool { project: PROJECT, person: OTHER, period: 1 };

fn ledger(world: &World, funder: tasks::Funder) -> tasks::Numbers {
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Ledger(funder)));
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))) = world.store.rows.get(&key) else {
        panic!("funding ledger {funder:?}")
    };
    row.numbers
}

fn world(seed: u64) -> World {
    let (configuration, mut limits) = fixture(seed, false);
    limits.core.people.amendment_bytes = 256;
    World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() })
}

fn create(world: &mut World, budget: u64) -> u64 {
    ask(world, 10, people::Ask::Create { project: PROJECT, batch: Box::new([people_member(b"work", budget)]) });
    let Some(people::Reply::Outcome(people::Outcome::Created { first, .. })) = world.people_answers.last() else {
        panic!("created task: {:?}", world.people_answers.last())
    };
    *first
}

fn owner(world: &mut World, open: bool) {
    let sign_in = second_party(world, b"widening-owner", people::Role::Owner);
    world.sign_in = Some(sign_in);
    if open {
        ask(world, 11, people::Ask::SetPool { project: PROJECT, person: OTHER, budget: 500 });
        assert_eq!(
            world.people_answers.last(),
            Some(&people::Reply::Outcome(people::Outcome::PoolSet { project: PROJECT, person: OTHER }))
        );
    }
}

fn edit(budget: u64) -> people::Amendment {
    let mut change = person_unchanged(b"authority");
    change.authority = Some(people_member(b"", budget).authority);
    change
}

fn amend(world: &mut World, task: u64, key: u8, amendment: people::Amendment, compound: bool) {
    let revision = live_row(world, task).revisions.spec;
    let request = if compound {
        let mut operations = List::with_capacity(1);
        operations
            .push(people::Operation::Amend { task, revision, amendment: Box::new(amendment), release: None })
            .expect("one operation");
        people::Ask::Compound { project: PROJECT, operations }
    } else {
        people::Ask::Amend { project: PROJECT, task, revision, amendment, release: None }
    };
    ask(world, key, request);
    if compound {
        assert!(
            matches!(
                world.people_answers.last(),
                Some(people::Reply::Outcome(people::Outcome::Compound { operations: 1, .. }))
            ),
            "{:?}",
            world.people_answers.last()
        );
    } else {
        assert_eq!(
            world.people_answers.last(),
            Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: false }))
        );
    }
}

fn spent_task(seed: u64) -> (World, u64) {
    let mut world = world(seed);
    finish_chat(&mut world, 0, b"done");
    let task = create(&mut world, 100);
    assert_eq!(world.assigned.as_ref().expect("target assigned").task, task);
    world.end(tasks::End::Exceeded(tasks::Exceeded { bound: tasks::Bound::Reasoning, value: 2048 }), 30);
    assert_eq!(live_row(&world, task).numbers.spent, 30);
    (world, task)
}

#[test]
fn an_owners_widening_refunds_the_old_pool_and_commits_the_new_funder_alone_or_compound() {
    for compound in [false, true] {
        let (mut world, task) = spent_task(671);
        owner(&mut world, true);
        let before = ledger(&world, ORIGINAL);
        amend(&mut world, task, 12, edit(150), compound);
        let row = live_row(&world, task);
        assert_eq!(
            (row.requester, row.funder, row.numbers.budget, row.numbers.spent, row.numbers.spent_below),
            (tasks::Party::Person(OWNER), GIVER, 120, 0, 0)
        );
        assert_eq!(row.authority.budget.spend, 120);
        assert_eq!(ledger(&world, ORIGINAL).reserved, before.reserved - 100);
        assert_eq!(ledger(&world, ORIGINAL).spent_below, before.spent_below + 30);
        assert_eq!(ledger(&world, GIVER).reserved, 120);
        world.close();
    }
}

#[test]
fn an_owners_budget_and_deadline_narrowing_keep_the_original_pool() {
    let (mut world, task) = spent_task(672);
    owner(&mut world, false);
    let before = ledger(&world, ORIGINAL);
    let mut change = edit(80);
    change.authority.as_mut().expect("authority").deadline = Some(skein_lib::Wall::from_nanos(1_000_000_000));
    amend(&mut world, task, 12, change, false);
    let row = live_row(&world, task);
    assert_eq!((row.funder, row.numbers.budget, row.numbers.spent), (ORIGINAL, 80, 30));
    assert_eq!(ledger(&world, ORIGINAL).reserved, before.reserved - 20);
    amend(&mut world, task, 13, person_unchanged(b"words only"), false);
    assert_eq!(live_row(&world, task).funder, ORIGINAL);
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ledger(GIVER)))));
    world.close();
}

#[test]
fn an_authority_only_widening_opens_the_owners_pool_in_its_decision() {
    for compound in [false, true] {
        let mut world = world(673);
        finish_chat(&mut world, 0, b"done");
        let task = create(&mut world, 0);
        owner(&mut world, false);
        let mut change = edit(0);
        change.authority.as_mut().expect("authority").tools = 1;
        let answers = world.people_answers.len();
        world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
        let revision = live_row(&world, task).revisions.spec;
        let request = if compound {
            let mut operations = List::with_capacity(1);
            operations
                .push(people::Operation::Amend { task, revision, amendment: Box::new(change), release: None })
                .expect("one operation");
            people::Ask::Compound { project: PROJECT, operations }
        } else {
            people::Ask::Amend { project: PROJECT, task, revision, amendment: change, release: None }
        };
        ask(&mut world, 12, request);
        assert_eq!(world.people_answers.len(), answers, "no reply before the opening decision commits");
        world.release_commits();
        assert_eq!(live_row(&world, task).funder, GIVER);
        assert_eq!(live_row(&world, task).authority.tools, tasks::Tools(1));
        assert_eq!(ledger(&world, GIVER).budget, 500);
        assert_eq!(ledger(&world, GIVER).reserved, 0);
        world.close();
    }
}

#[test]
fn a_run_widening_reclaims_a_partys_funded_delegate_and_then_resizes() {
    let mut world = plan_world(674);
    let parent = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let child = delegate(&mut world, parent, 1, child);
    owner(&mut world, true);
    amend(&mut world, child, 12, edit(100), false);
    assert_eq!(live_row(&world, child).funder, GIVER);
    for (completion, budget) in [(2, 150), (3, 160)] {
        let mut change = unchanged(b"widen delegate");
        let mut after = live_row(&world, child).authority.clone();
        after.budget.spend = budget;
        change.authority = Some(after);
        let revision = live_row(&world, child).revisions.spec;
        world.send(root::Event::Core(core::Event::NamedAction {
            to: ReplyTo::new(Token::new(9200 + u64::from(completion))),
            key: core::CallKey { task: parent.0, attempt: parent.1, completion, position: 0 },
            action: core::NamedAction::Amend { target: child, revision, keep_parameters: false, amendment: change },
        }));
        let row = live_row(&world, child);
        assert_eq!((row.funder, row.numbers.budget), (tasks::Funder::Task(parent.0), budget));
        assert_eq!(ledger(&world, GIVER).reserved, 0);
    }
    world.close();
}
