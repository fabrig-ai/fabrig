//! Open failures and their intended outcomes (jig/docs/design/domain/testing-fakes.md, 1.3).
use jig_core_world::{
    fixtures::{ask, ask_as, delegate, finish_chat, goal, live_row, member, second_party},
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use jig_test_profile::fixtures::{Goal, people_member, person_unchanged};
use skein_lib::{Duration, ReplyTo, Token, Wall};

const SEED: u64 = 43_431_577;
const PROJECT: u32 = 1;
const QUESTION: &[u8] = b"May I proceed?\nMore detail.";

fn running(world: &World, task: u64) -> (u64, u64) {
    *world.assignments.iter().rev().find(|(number, _)| *number == task).expect("task assigned")
}

fn park(world: &mut World, run: (u64, u64), read: Option<u64>) {
    if read.is_some() {
        world.send(root::Event::Turn {
            channel: Token::new(7),
            task: run.0,
            attempt: run.1,
            turn: 1,
            cumulative: 0,
            read,
            transcript: b"read the question".as_slice().into(),
            opens_window: true,
            refused: Box::new([]),
        });
    }
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Parked,
        read,
        refused: Box::new([]),
    });
}

fn made(world: &World) -> u64 {
    let Some(people::Reply::Outcome(people::Outcome::Created { first, last, .. })) = world.people_answers.last() else {
        panic!("goal created: {:?}", world.people_answers.last())
    };
    assert_eq!(first, last);
    *first
}

/// P43: a watched child awaiting this coordinator's answer cannot wake it by
/// finishing. Each unanswered park must spend a run try and eventually hold
/// with the question's words; today the live task watch leaves it idle.
#[test]
#[ignore = "P43, open: bounded waits (jig/docs/design/domain/testing-fakes.md, 1.3)"]
fn a_coordinator_watching_its_asker_is_woken() {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(SEED);
    configuration.core.defaults.charters[0].policy.resume = false;
    limits.core.tasks.inbox_bytes = 2048;
    limits.core.tasks.retries.run.retries = 2;
    let attempts = limits.core.tasks.retries.run.retries + 1;
    let mut world = World::new(Setup { seed: SEED, configuration: Some(configuration), limits, ..Setup::default() });
    finish_chat(&mut world, 0, b"done");
    let create = goal(
        &world,
        PROJECT,
        b"coordinate the child".as_slice().into(),
        people::TaskExecutor::Agent { charter: 1 },
        Goal { budget: 100, ..Goal::default() },
    );
    ask(&mut world, 80, create);
    let coordinator = made(&world);
    let first = running(&world, coordinator);
    assert!(!world.assigned.as_ref().expect("coordinator assigned").run.policy.resume);
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"ask before proceeding");
    let child = delegate(&mut world, first, 1, child);
    let subscription = tasks::Subscription {
        number: 980,
        kind: tasks::SubscriptionKind::Task { target: child, held: false, result: true },
    };
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Subscribe {
        reply_to: ReplyTo::new(Token::new(980)),
        task: coordinator,
        subscription,
    })));
    assert!(live_row(&world, coordinator).subscriptions.contains(&subscription));
    park(&mut world, first, None);
    assert_eq!(live_row(&world, coordinator).phase, tasks::Phase::Active(tasks::Active::Idle));
    let child_run = running(&world, child);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(981)),
        key: core::CallKey { task: child, attempt: child_run.1, completion: 1, position: 0 },
        action: core::NamedAction::Message {
            target: coordinator,
            kind: tasks::MessageKind::Question,
            words: QUESTION.into(),
        },
    }));
    assert!(!live_row(&world, child).questions.is_empty(), "child question admitted: {:?}", world.answers);
    let question = live_row(&world, child).questions.first().expect("child asked its requester").number;
    park(&mut world, child_run, None);
    assert_eq!(live_row(&world, child).phase, tasks::Phase::Active(tasks::Active::Idle));
    let woken = running(&world, coordinator);
    assert_ne!(woken, first, "the question wakes the coordinator once");
    let mut tries = Vec::new();
    for spent in 1..=attempts {
        let run = running(&world, coordinator);
        park(&mut world, run, (spent == 1).then_some(question));
        let row = live_row(&world, coordinator);
        tries.push(row.tries.run);
        if spent < attempts {
            assert!(
                matches!(row.phase, tasks::Phase::Active(tasks::Active::Running { .. })),
                "due again: {:?}, retry: {:?}",
                row.phase,
                row.retry_notice
            );
            assert_ne!(running(&world, coordinator), run, "another run is assigned at each park");
        }
    }
    let row = live_row(&world, coordinator);
    let tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Run), source: Some(source), .. } = &row.phase
    else {
        panic!("unanswered question must escalate past run tries: {:?}", row.phase)
    };
    assert_eq!(tries, (1..=attempts).collect::<Vec<_>>(), "each unanswered park spends a run try");
    let words = format!("Task {child}'s question {question} is still open: May I proceed?");
    assert_eq!(source.as_ref(), &tasks::HoldSource::Preparation { words: words.as_bytes().into() });
    assert_eq!(live_row(&world, child).questions.first().expect("question remains open").number, question);
    world.close();
}

// A goal can retain more per-task spend than its maker's present role. Create
// it as a maintainer, then narrow that party to member, without altering the
// existing goal. The owner has a smaller per-task bound too (40 versus 60).
fn retained_goal() -> (World, u64, u64, u64) {
    let (configuration, limits) = jig_core_world::peers::fixture(SEED, 0);
    let mut world = World::new(Setup { seed: SEED, configuration: Some(configuration), limits, ..Setup::default() });
    let owner = world.sign_in.expect("owner signed in");
    finish_chat(&mut world, 0, b"done");
    let maker = second_party(&mut world, b"goal-maker", people::Role::Maintainer);
    let mut member = people_member(b"finish before deadline", 60);
    member.authority.deadline = Some(Wall::from_nanos(1_000_000_000));
    member.label = jig_client_test_records::GOAL;
    member.tracked = Some(1);
    member.flags.listed = true;
    ask_as(&mut world, maker, 81, people::Ask::Create { project: PROJECT, batch: Box::new([member]) });
    let task = made(&world);
    let tasks::Party::Person(person) = live_row(&world, task).requester else { panic!("party's goal") };
    ask_as(
        &mut world,
        owner,
        82,
        people::Ask::SetRoles {
            project: PROJECT,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person, role: people::Role::Member },
            ]),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::RolesSet { project: PROJECT }))
    );
    assert_eq!(live_row(&world, task).numbers.budget, 60);
    (world, owner, maker, task)
}

fn deadline_change(world: &World, task: u64, deadline: Wall) -> people::Amendment {
    let mut change = person_unchanged(b"more time");
    let mut authority = people_member(b"", live_row(world, task).authority.budget.spend).authority;
    authority.deadline = Some(deadline);
    change.authority = Some(authority);
    change
}

/// S15: the party that started a goal can extend its passed deadline. The goal
/// holds authority beyond its party's role because the role was narrowed after
/// the goal started. Today whole-authority admission refuses the unchanged
/// per-task spend, rather than judging the later deadline.
#[test]
#[ignore = "S15, open: provider failures (jig/docs/design/domain/testing-fakes.md, 1.3)"]
fn a_party_extends_its_goals_deadline() {
    let (mut world, _, maker, task) = retained_goal();
    let run = running(&world, task);
    world.await_timer(task);
    park(&mut world, run, None);
    world.advance(Duration::from_secs(2));
    assert!(matches!(live_row(&world, task).phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }));
    let deadline = Wall::from_nanos(10_000_000_000);
    let amendment = deadline_change(&world, task, deadline);
    let row = live_row(&world, task);
    let (revision, hold) = (row.revisions.spec, row.revisions.hold);
    ask_as(
        &mut world,
        maker,
        83,
        people::Ask::Amend { project: PROJECT, task, revision, amendment, release: Some(hold) },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: true })),
        "only the deadline widens; retained spend is not a new grant"
    );
    assert_eq!(live_row(&world, task).authority.budget.deadline, Some(deadline));
    assert!(matches!(live_row(&world, task).phase, tasks::Phase::Active(_)), "the goal goes on");
    assert_ne!(running(&world, task), run);
    world.close();
}

/// P77: an owner's deadline widening applies without a self-proposal. The goal
/// holds authority beyond its party's role because the role was narrowed after
/// the goal started, and beyond role 0's per-task spend too. A proposal reaching
/// that owner must be acceptable; today admission refuses the unchanged spend.
#[test]
#[ignore = "P77, open: provider failures (jig/docs/design/domain/testing-fakes.md, 1.3)"]
fn an_owners_widening_is_never_proposed_to_itself() {
    let (mut world, owner, maker, task) = retained_goal();
    let deadline = Wall::from_nanos(10_000_000_000);
    let amendment = deadline_change(&world, task, deadline);
    let revision = live_row(&world, task).revisions.spec;
    ask_as(&mut world, owner, 83, people::Ask::Amend { project: PROJECT, task, revision, amendment, release: None });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: false })),
        "an owner's widening must apply without proposing it to that same owner"
    );
    assert_eq!(live_row(&world, task).authority.budget.deadline, Some(deadline));
    let amendment = deadline_change(&world, task, Wall::from_nanos(20_000_000_000));
    let revision = live_row(&world, task).revisions.spec;
    ask_as(
        &mut world,
        maker,
        84,
        people::Ask::Propose {
            project: PROJECT,
            action: people::Action::Amend { task, revision, amendment: Box::new(amendment) },
            reason: b"please extend".as_slice().into(),
        },
    );
    let Some(people::Reply::Outcome(people::Outcome::Proposed { proposal, to: 0 })) =
        world.people_answers.last().cloned()
    else {
        panic!("proposal reaches the owner: {:?}", world.people_answers.last())
    };
    let tasks::Party::Person(person) = live_row(&world, task).requester else { panic!("party's goal") };
    ask_as(
        &mut world,
        owner,
        85,
        people::Ask::DecideProposal {
            project: PROJECT,
            proposer: people::Proposer::Party(person),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: None,
        },
    );
    assert!(
        matches!(
            world.people_answers.last(),
            Some(people::Reply::Outcome(people::Outcome::ProposalDecided {
                choice: people::ProposalChoice::Accepted,
                ..
            }))
        ),
        "a proposal reaches a role whose acceptance applies it: {:?}",
        world.people_answers.last()
    );
    assert_eq!(live_row(&world, task).authority.budget.deadline, Some(Wall::from_nanos(20_000_000_000)));
    world.close();
}
