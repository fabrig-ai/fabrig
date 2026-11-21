//! Funding acceptance for the open provider failures (domain/testing.md, 7.3).
use jig_core_world::fixtures::{PARTY, PROJECT, ask, ask_as, live_row, second_party};
use jig_core_world::world::{Setup, World};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use jig_test_profile::fixtures::people_member;
use skein_lib::{ReplyTo, Token};

const SEED: u64 = 93_098_082;
const PERIOD: u64 = 1;
const ROOM: u64 = 100;

fn ledger(world: &World, funder: tasks::Funder) -> &tasks::FundingRecord {
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))) =
        world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ledger(funder))))
    else {
        panic!("funding ledger {funder:?}")
    };
    row
}

fn created(world: &World) -> u64 {
    let Some(people::Reply::Outcome(people::Outcome::Created { first, last, .. })) = world.people_answers.last() else {
        panic!("one funded start: {:?}", world.people_answers.last());
    };
    assert_eq!(first, last);
    *first
}

fn as_allowed() -> people::Member {
    let mut member = people_member(b"start without a budget", 0);
    member.budget = people::Budget::AsAllowed { cap: None };
    member
}

/// P93: an uncapped `AsAllowed` start must leave room for another start from the
/// same pool. Today it takes all R, and the second is refused for `Lack::Budget`.
#[test]
#[ignore = "P93, open: provider failures (jig/docs/design/domain/testing.md, 7.3)"]
fn a_start_without_a_budget_leaves_room() {
    let config = jig_test_profile::config(SEED);
    assert!(config.core.seed_projects[0].policy.policy.roles[0].authority.budget.spend >= ROOM);
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        initial: Some(people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } }),
        require_assignment: false,
        ..Setup::default()
    });
    ask(&mut world, 40, people::Ask::SetPool { project: PROJECT, person: PARTY, budget: ROOM });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PoolSet { project: PROJECT, person: PARTY }))
    );
    let pool = tasks::Funder::Pool { project: PROJECT, person: PARTY, period: PERIOD };
    assert_eq!(ledger(&world, pool).numbers.budget - ledger(&world, pool).numbers.reserved, ROOM);
    ask(&mut world, 41, people::Ask::Create { project: PROJECT, batch: Box::new([as_allowed()]) });
    let first = created(&world);
    ask(&mut world, 42, people::Ask::Create { project: PROJECT, batch: Box::new([as_allowed()]) });
    assert!(
        live_row(&world, first).numbers.budget < ROOM,
        "P93: the first start must take less than {ROOM}; got {}, second answer: {:?}",
        live_row(&world, first).numbers.budget,
        world.people_answers.last()
    );
    let second = created(&world);
    assert_ne!(first, second);
    assert_eq!(live_row(&world, first).funder, pool);
    assert_eq!(live_row(&world, second).funder, pool);
    assert!(live_row(&world, second).numbers.budget > 0, "P93: the second start must be funded");
}

/// No refusal has period-room fields yet. Check carried words where present;
/// a new typed funding refusal must instead check its period and room here.
fn told_its_funding(refusal: &people::Refusal) -> Result<(), String> {
    match refusal {
        people::Refusal::Authority { .. } => {
            Err(format!("P98: told its authority, not its period's funding: {refusal:?}"))
        }
        people::Refusal::Judged { why, .. } => {
            let text = String::from_utf8_lossy(why).to_lowercase();
            let words: Vec<_> = text.split_whitespace().collect();
            let period = PERIOD.to_string();
            let names_period = words.windows(2).any(|pair| pair == ["period", period.as_str()]);
            let names_room = words
                .windows(2)
                .any(|pair| pair == ["0", "left"] || pair == ["0", "remaining"] || pair == ["fully", "allocated"]);
            if names_period && names_room {
                Ok(())
            } else {
                Err(format!("P98: refusal must name period {PERIOD} and its zero room: {text}"))
            }
        }
        _ => Err(format!("P98: no period or remaining room in refusal: {refusal:?}")),
    }
}

/// P98: a new party with an allotment must be told that its period is fully
/// allocated, not that it lacks authority. Today the pool carve becomes
/// `Refusal::Authority { lacking: Lack::Funding, .. }`, without period-room facts.
#[test]
#[ignore = "P98, open: provider failures (jig/docs/design/domain/testing.md, 7.3)"]
fn a_full_period_is_named_as_funding() {
    let config = jig_test_profile::config(SEED);
    assert!(config.core.seed_projects[0].policy.policy.roles[0].period_spend > 0);
    let mut limits = jig_test_profile::limits();
    limits.core.people.people = 4;
    limits.core.people.holdings = 4;
    limits.core.people.sign_ins = 4;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::new(Setup { seed: SEED, configuration: Some(config), limits, ..Setup::default() });
    second_party(&mut world, b"second pool", people::Role::Owner);
    let second = match world.people_answers.last() {
        Some(people::Reply::SignedIn { person, .. }) => *person,
        answer => panic!("second party: {answer:?}"),
    };
    ask(&mut world, 40, people::Ask::SetPool { project: PROJECT, person: second, budget: 500 });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PoolSet { project: PROJECT, person: second }))
    );
    let period = tasks::Funder::Period { project: PROJECT, period: PERIOD };
    assert_eq!(
        ledger(&world, period).numbers.budget - ledger(&world, period).numbers.reserved,
        0,
        "pools allocate the entire period"
    );
    let newcomer = second_party(&mut world, b"new party", people::Role::Owner);
    ask_as(
        &mut world,
        newcomer,
        41,
        people::Ask::Create { project: PROJECT, batch: Box::new([people_member(b"fund me", 10)]) },
    );
    let Some(people::Reply::Outcome(people::Outcome::Refused(refusal))) = world.people_answers.last() else {
        panic!("full period refuses new funding: {:?}", world.people_answers.last());
    };
    assert_eq!(told_its_funding(refusal), Ok(()));
}

/// P82's residual: a delegating task with budget 2M must fund one `AsAllowed`
/// delegate of at least M and claim its run. Today `run_spend` reserves all 2M
/// when half equals the minimum, and the delegate is refused for funding.
#[test]
#[ignore = "P82's residual, open: provider failures (jig/docs/design/domain/testing.md, 7.3)"]
fn a_small_delegating_task_funds_its_delegates() {
    let config = jig_test_profile::config(SEED);
    let minimum = config.core.defaults.rules.minimum_run_spend;
    let mut parent = people_member(b"small delegating task", 2 * minimum);
    parent.authority.tools = core::ToolFamilies::JIG.delegate.0;
    parent.authority.delegation =
        people::Delegation { kinds: Box::new([people::Executor::Charter(1)]), tasks: 1, depth: 1 };
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        initial: Some(people::Ask::Create { project: PROJECT, batch: Box::new([parent]) }),
        ..Setup::default()
    });
    let assignment = world.assigned.as_ref().expect("the small parent run starts");
    let key = core::CallKey { task: assignment.task, attempt: assignment.attempt, completion: 1, position: 0 };
    assert_eq!(live_row(&world, key.task).numbers.budget, 2 * minimum);
    let mut child = jig_core_world::fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    child.authority.budget.spend = 0;
    child.authority.tools = tasks::Tools(0);
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    child.budget = people::Budget::AsAllowed { cap: None };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9001)),
        key,
        batch: Box::new([child]),
        stubs: Box::new([]),
    }));
    assert!(
        matches!(world.answers.last(), Some((_, _, core::CallPart::Delegated(_)))),
        "P82's residual: a 2M task must fund its delegate while its run lives; got {:?}",
        world.answers.last()
    );
    let core::CallPart::Delegated(made) = &world.answers.last().expect("delegate answered").2 else { unreachable!() };
    let child = made.first().expect("one delegate made").task;
    assert!(live_row(&world, child).numbers.budget >= minimum, "P82's residual: delegate gets at least M");
    jig_core_world::fixtures::finish_chat(&mut world, 0, b"delegated");
    assert!(
        world.assignments.iter().any(|(task, _)| *task == child),
        "P82's residual: the delegate run is claimed, not held for budget"
    );
}
