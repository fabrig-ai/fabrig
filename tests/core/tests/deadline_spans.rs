use skein_world::domain::heap;

#[global_allocator]
static HEAP: heap::Counting = heap::Counting;

use jig_core_world::{
    fixtures::{
        ask_as,
        deadlines::{self as f, CREATED},
        live_row, refusal,
    },
    stories::offers::{member_configuration, world_with, world_with_policy},
};
use jig_domain_people as p;
use jig_domain_tasks as tasks;
use skein_lib::Wall;

#[test]
fn a_span_starts_at_create_admission() {
    let mut world = world_with(301, Some(p::Role::Member), |_| {});
    let sign_in = world.sign_in.expect("signed in");
    f::clock(&mut world, 10);
    f::create(&mut world, sign_in, 31);
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(15_000_000_000)));
}
#[test]
fn a_pending_span_survives_restart_until_acceptance() {
    let (mut world, _, owner, proposal) = f::pending(302);
    assert_eq!(f::person_proposal(&world, proposal).batch()[0].deadline, tasks::Deadline::Span(f::SPAN));
    world.restart_with(member_configuration(302));
    assert_eq!(f::person_proposal(&world, proposal).batch()[0].deadline, tasks::Deadline::Span(f::SPAN));
    f::clock(&mut world, 30);
    f::decide(&mut world, owner, 33, proposal, p::ProposalDecision::Accept);
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(35_000_000_000)));
}
#[test]
fn a_delayed_commit_does_not_move_the_deadline() {
    let mut world = world_with(303, Some(p::Role::Member), |_| {});
    let sign_in = world.sign_in.expect("signed in");
    f::clock(&mut world, 10);
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    f::create(&mut world, sign_in, 31);
    assert!(!world.store.held.is_empty());
    f::clock(&mut world, 30);
    world.release_commits();
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(15_000_000_000)));
}
#[test]
fn a_span_past_the_ceiling_is_refused_when_proposed() {
    let mut world = world_with_policy(304, Some(p::Role::Member), |_, _, policy| {
        policy.ceiling.budget.deadline = Some(Wall::from_nanos(14_000_000_000));
        for role in &mut policy.roles {
            role.authority.budget.deadline = policy.ceiling.budget.deadline;
        }
    });
    let sign_in = world.sign_in.expect("signed in");
    f::clock(&mut world, 10);
    ask_as(
        &mut world,
        sign_in,
        32,
        p::Ask::Propose {
            project: 1,
            reason: b"span".as_slice().into(),
            action: p::Action::Batch(Box::new([f::span_member()])),
        },
    );
    refusal(&world, p::Refusal::Authority { lacking: p::Lack::Shape, proposable: Some(p::Accepter::Nobody) });
}
#[test]
fn a_resolved_span_beyond_the_creator_is_refused() {
    for (seconds, expected) in [(15, None), (16, Some(p::Accepter::Role(0)))] {
        let (mut world, member, _) = f::finite_world(305);
        f::clock(&mut world, seconds);
        f::create(&mut world, member, 31);
        if let Some(proposable) = expected {
            refusal(&world, p::Refusal::Authority { lacking: p::Lack::Deadline, proposable: Some(proposable) });
        } else {
            assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(20_000_000_000)));
        }
    }
}
#[test]
fn an_aged_span_is_beyond_its_accepter_and_passed_up() {
    let (mut world, member, owner) = f::finite_world(306);
    let proposal = f::task_proposal(&mut world, member);
    assert_eq!(f::holder(&world), tasks::ProposalHolder::Person(1));
    assert_eq!(f::acceptance(&mut world, proposal), jig_domain_core::acts::Admission::Allowed);
    let before = live_row(&world, 1).numbers;
    f::clock(&mut world, 30);
    assert_eq!(f::holder(&world), tasks::ProposalHolder::Person(1));
    let why = p::Refusal::Authority { lacking: p::Lack::Deadline, proposable: Some(p::Accepter::Role(0)) };
    assert_eq!(f::acceptance(&mut world, proposal), jig_domain_core::acts::Admission::Refused(why.clone()));
    f::task_decide(&mut world, member, 33, proposal, p::ProposalDecision::Accept);
    refusal(&world, why);
    assert_eq!(live_row(&world, 1).numbers, before);
    assert!(
        !world
            .store
            .rows
            .contains_key(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(CREATED))))
    );
    f::task_decide(&mut world, member, 34, proposal, p::ProposalDecision::Pass);
    assert_eq!(f::holder(&world), tasks::ProposalHolder::Policy { project: 1, kind: tasks::ProposalKind::Batch });
    f::clock(&mut world, 40);
    f::task_decide(&mut world, owner, 35, proposal, p::ProposalDecision::Accept);
    assert_eq!(live_row(&world, CREATED).authority.budget.deadline, Some(Wall::from_nanos(45_000_000_000)));
}
#[test]
fn an_aged_span_past_the_ceiling_makes_no_task() {
    let mut world = world_with_policy(307, Some(p::Role::Member), |_, _, policy| {
        policy.ceiling.budget.deadline = Some(Wall::from_nanos(20_000_000_000));
        for role in &mut policy.roles {
            role.authority.budget.deadline = policy.ceiling.budget.deadline;
        }
    });
    let member = world.sign_in.expect("member");
    let owner = jig_core_world::fixtures::second_party(&mut world, b"second-party", p::Role::Owner);
    f::clock(&mut world, 10);
    let proposal = f::propose(&mut world, member, 32, Box::new([f::span_member()]));
    f::clock(&mut world, 30);
    let before = live_row(&world, 1).numbers;
    f::decide(&mut world, owner, 33, proposal, p::ProposalDecision::Accept);
    refusal(&world, p::Refusal::Authority { lacking: p::Lack::Deadline, proposable: Some(p::Accepter::Nobody) });
    assert_eq!(live_row(&world, 1).numbers, before);
    assert!(
        !world
            .store
            .rows
            .contains_key(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(CREATED))))
    );
}

#[test]
fn holder_rechecks_use_their_own_wall() {
    type Stimulus = fn(&mut jig_core_world::effects::World, u64, u64, u64);
    for (name, stimulate) in
        [("pass", f::pass as Stimulus), ("reroute", f::reroute as Stimulus), ("stall", f::stall as Stimulus)]
    {
        for (seconds, expected) in [
            (30, tasks::ProposalHolder::Policy { project: 1, kind: tasks::ProposalKind::Batch }),
            (100, tasks::ProposalHolder::Person(1)),
        ] {
            let (mut world, member, owner) = f::finite_world(308);
            let proposal = f::task_proposal(&mut world, member);
            f::clock(&mut world, seconds);
            stimulate(&mut world, member, proposal, owner);
            assert_eq!(f::holder(&world), expected, "{name} at {seconds}");
        }
    }
}

const CREATOR_WALL: Wall = Wall::from_nanos(20_000_000_000);

#[test]
fn a_delegate_span_starts_when_the_delegate_is_made() {
    use jig_core_world::fixtures::{delegate, member, run_with_deadline};
    let mut world = run_with_deadline(309, Some(CREATOR_WALL));
    let run = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"span delegate");
    child.deadline = tasks::Deadline::Span(f::SPAN);
    f::clock(&mut world, 10);
    let child = delegate(&mut world, run, 1, child);
    assert_eq!(live_row(&world, child).authority.budget.deadline, Some(Wall::from_nanos(15_000_000_000)));
}

#[test]
fn a_delegate_span_past_its_creator_is_refused() {
    use jig_core_world::fixtures::{delegate, member, run_with_deadline};
    use jig_domain_core as core;
    use jig_test_domain as root;
    use skein_lib::{ReplyTo, Token};
    for seconds in [16, 15] {
        let mut world = run_with_deadline(310, Some(CREATOR_WALL));
        let run = world.assignments[0];
        let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"bounded span");
        child.deadline = tasks::Deadline::Span(f::SPAN);
        f::clock(&mut world, seconds);
        if seconds == 15 {
            let child = delegate(&mut world, run, 1, child);
            assert_eq!(live_row(&world, child).authority.budget.deadline, Some(CREATOR_WALL));
        } else {
            world.send(root::Event::Core(core::Event::DelegateValidated {
                to: ReplyTo::new(Token::new(9001)),
                key: core::CallKey { task: run.0, attempt: run.1, completion: 1, position: 0 },
                batch: Box::new([child]),
                stubs: Box::new([]),
            }));
            let core::CallPart::DelegationDenied { findings, .. } = &world.answers.last().expect("refusal").2 else {
                panic!("delegate denied: {:?}", world.answers)
            };
            assert!(findings.iter().any(|finding| matches!(
                finding,
                jig_domain_authority::Finding::Authority {
                    lacks: jig_domain_authority::Lacks { deadline: true, .. },
                    ..
                }
            )));
            assert!(live_row(&world, run.0).delegates.is_empty());
            assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(CREATED)))));
        }
    }
}

#[test]
fn a_delegate_naming_no_deadline_ends_with_its_creators() {
    use jig_core_world::fixtures::{delegate, member, run_with_deadline};
    for deadline in [Some(CREATOR_WALL), None] {
        let mut world = run_with_deadline(311, deadline);
        let run = world.assignments[0];
        let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"no deadline");
        child.authority.budget.deadline = None;
        let child = delegate(&mut world, run, 1, child);
        assert_eq!(live_row(&world, child).authority.budget.deadline, deadline);
    }
}

#[test]
fn a_proposed_delegate_keeps_its_span() {
    use jig_core_world::fixtures::{member, run_with_deadline};
    use jig_domain_core as core;
    use jig_test_domain as root;
    use skein_lib::{ReplyTo, Token};
    let mut world = run_with_deadline(312, Some(CREATOR_WALL));
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"proposed span");
    child.deadline = tasks::Deadline::Span(f::SPAN);
    f::clock(&mut world, 10);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9001)),
        key: world.call_key(1),
        action: core::NamedAction::ProposeBatch {
            batch: Box::new([child]),
            reason: b"span proposal".as_slice().into(),
            as_holder: true,
        },
    }));
    assert!(matches!(world.answers.last().expect("proposal answer").2, core::CallPart::Proposed { .. }));
    let proposal = live_row(&world, 1).proposal.as_ref().expect("retained proposal");
    let tasks::ProposalAction::Batch(batch) = &proposal.action else { panic!("proposed batch") };
    assert_eq!(batch[0].deadline, tasks::Deadline::Span(f::SPAN));
}

#[test]
fn a_parked_delegate_past_its_wall_is_held_and_its_creator_told() {
    use jig_core_world::fixtures::{delegate, member, run_with_deadline};
    use jig_domain_core as core;
    use jig_test_domain as root;
    use skein_lib::Time;
    let mut world = run_with_deadline(313, Some(CREATOR_WALL));
    let creator = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"parked span");
    child.deadline = tasks::Deadline::Span(f::SPAN);
    f::clock(&mut world, 10);
    let child = delegate(&mut world, creator, 1, child);
    assert_eq!(world.assigned.as_ref().expect("delegate running").task, child);
    world.end(tasks::End::Parked, 0);
    assert_eq!(live_row(&world, child).phase, tasks::Phase::Active(tasks::Active::Idle));
    let deadline = live_row(&world, child).authority.budget.deadline.expect("span wall");
    world.fire_due(Time::from_nanos(deadline.as_nanos()), deadline);
    assert_eq!(live_row(&world, child).phase, tasks::Phase::Active(tasks::Active::Idle));
    let past = Wall::from_nanos(deadline.as_nanos() + 1);
    world.fire_due(Time::from_nanos(past.as_nanos()), past);
    assert!(matches!(live_row(&world, child).phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. }));
    world.fire_due(Time::from_nanos(past.as_nanos() + 1), past);
    assert!(matches!(live_row(&world, child).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(task), .. } if task == creator.0));
    let notices = world
        .inbound
        .iter()
        .filter(|word| matches!(word.kind, tasks::MessageKind::Escalation { task, .. } if task == child))
        .count();
    assert_eq!(notices, 1, "the requester hears the deadline hold once");
    let activity: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row))) if row.task == child => Some(row),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect();
    assert!(!activity.iter().any(|row| matches!(row.line, tasks::Line::Turn { .. })));
    assert_eq!(activity.iter().filter(|row| matches!(row.line, tasks::Line::RunBegan { .. })).count(), 1);
    assert_eq!(world.assignments.iter().filter(|(task, _)| *task == child).count(), 1);
}
