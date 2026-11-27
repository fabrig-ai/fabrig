//! Change reads through the write record (domain/forge/change/drift.md, 1).
use skein_fake_forge_domain::{self as forge, Lag, api as raw};
use skein_forge::api::{self, Commit};
use skein_lib::{Duration, Wall};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{LIMITS, REPO, World, changes::change_row, translate};

const BRANCH: &[u8] = b"temper/51";
const TASK: u64 = 51;
const PRODUCER: u64 = 412;

fn name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(BRANCH.split(|byte| *byte == b'/').map(Box::from).collect()),
    }
}

fn step(world: &mut World, entry: u64) -> (change::Decision, Option<top::ChangeEvidence>) {
    world.take_seen();
    world.event(top::Event::StepChange {
        task: TASK,
        entry,
        heard: change::Heard {
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            review_request: None,
            closer_waiting: false,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Passed,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        merge_job: false,
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: false,
    });
    world.run_for(1);
    world
        .take_seen()
        .into_iter()
        .find_map(|request| match request {
            top::Request::ChangeDecision { task: TASK, decision, evidence, .. } => Some((decision, evidence)),
            _ => None,
        })
        .expect("change decided")
}

fn pushed_change(seed: u64, pull_base: &[u8]) -> (World, u64, u64) {
    let mut world = World::new(seed);
    world.adopt();
    world.landing_ci(b"main", true);
    world.produce(BRANCH);
    let old = world.branch(BRANCH);
    let head = translate::commit(old);
    // Start with a confirmed push from the previous attempt, then admit the
    // next lease through the scripted producer as it writes to the forge.
    world.event(top::Event::Restore {
        record: top::Stored::Writes(top::Writes {
            name: name(),
            head: Some(head),
            next: 2,
            entries: Box::new([top::Written {
                order: 1,
                operation: top::writes::Operation::Push {
                    before: Some(translate::commit(world.branch(b"main"))),
                    after: head,
                },
                sender: top::Sender::Attempt { task: PRODUCER, attempt: 1 },
                state: top::WriteState::Confirmed,
            }]),
        }),
    });
    if pull_base != b"main" {
        world.external(raw::Op::Git(raw::Git::Create { branch: pull_base.into(), commit: 1 }), 2);
    }
    let pull = world.open_pull(BRANCH, pull_base);
    let mut row = change_row();
    row.change.state = change::State::Checking { head };
    row.change.last_head = Some(head);
    row.change.since = world.wall();
    row.pull = Some(pull);
    world.event(top::Event::Change { row });
    (world, old, pull)
}

fn producer_push(world: &mut World) -> Commit {
    let head = world.push_attempt(&name(), (PRODUCER, 2), BRANCH, b"second", b"new work");
    world.status(BRANCH, raw::Check::Pending);
    head
}

#[test]
fn a_lagging_view_of_a_producer_push_waits_then_checks_that_push() {
    let (mut world, _, _) = pushed_change(407, b"main");
    // Scripted pushes run at the fake's epoch; keep its pull view behind
    // until four seconds after the world's current clock.
    let span = Duration::from_nanos(world.wall().as_nanos()).saturating_add(Duration::from_secs(4));
    forge::lag(world.forge_mut(), Lag { chance: 1000, min: span, max: span });
    let head = producer_push(&mut world);
    world.take_calls();
    assert!(matches!(step(&mut world, 10).0, change::Decision::Wait { .. }));
    assert_ne!(world.change(TASK).change.last_head, Some(head));
    assert!(!world.take_calls().contains(&api::Op::Read(api::Read::Checks { commit: head })));
    world.run_for(4);
    let caught_up = step(&mut world, 11).0;
    assert!(matches!(caught_up, change::Decision::Wait { .. }), "{caught_up:?}");
    let calls = world.take_calls();
    // The forge can supply CI on the pull read itself.
    assert!(
        calls.iter().any(|op| matches!(op,
            api::Op::Read(api::Read::Checks { commit }) if *commit == head
        )) || calls.contains(&api::Op::Read(api::Read::Pull { number: world.change(TASK).pull.unwrap(), ci: true })),
        "{calls:?}"
    );
    assert_eq!(world.change(TASK).change.state, change::State::Checking { head });
}

fn reset(world: &mut World, old: u64) -> Commit {
    let head = producer_push(world);
    assert!(matches!(step(world, 10).0, change::Decision::Wait { .. }));
    assert_eq!(world.change(TASK).change.last_head, Some(head));
    // The fake's lease push only fast-forwards. Recreate the ref at the
    // earlier commit without a delivery to the connector.
    world.delete_branch(BRANCH);
    assert_eq!(
        world.external(raw::Op::Git(raw::Git::Create { branch: BRANCH.into(), commit: old }), 2),
        raw::Answer::Branch(raw::Created::Created)
    );
    head
}

#[test]
fn a_reset_to_an_older_write_without_an_actor_waits_until_the_stall() {
    let (mut world, old, _) = pushed_change(408, b"main");
    let head = reset(&mut world, old);
    let deadline = Wall::from_nanos(world.wall().as_nanos() + LIMITS.change_policy.stall.as_nanos());
    assert!(matches!(step(&mut world, 11).0, change::Decision::Wait { until: Some(_) }));
    world.run_for(20);
    assert!(matches!(step(&mut world, 12).0, change::Decision::Wait { until: Some(_) }));
    world.run_to(deadline);
    let evidence = world
        .take_seen()
        .into_iter()
        .find_map(|request| match request {
            top::Request::ChangeDecision {
                decision: change::Decision::Hold(change::Cause::Stalled), evidence, ..
            } => evidence,
            _ => None,
        })
        .expect("stall holds at the original deadline");
    assert_eq!(
        evidence.behind,
        Some(top::ReadBehind {
            read: top::BehindRead::Branch,
            expected: Some(head),
            found: Some(translate::commit(old)),
        })
    );
}

#[test]
fn a_reset_to_an_older_write_by_another_actor_is_drift_immediately() {
    let (mut world, old, _) = pushed_change(409, b"main");
    let head = reset(&mut world, old);
    let found = translate::commit(old);
    world.event(top::Event::Hint {
        hint: api::Hint {
            repository: REPO,
            change: api::Change::BranchMoved { branch: BRANCH.into(), head: found, actor: 2 },
            key: None,
        },
    });
    let (decision, evidence) = step(&mut world, 11);
    assert_eq!(decision, change::Decision::Hold(change::Cause::BranchMoved));
    let evidence = evidence.expect("the read that held");
    assert_eq!(evidence.prior.last_head, Some(head));
    assert_eq!(evidence.head, Some(found));
    assert_eq!(evidence.nearest_write, Some(2));
}

#[test]
fn another_hands_close_holds_the_change_for_its_own_pull() {
    let (mut world, _, pull) = pushed_change(410, b"main");
    world.close(pull);
    assert_eq!(step(&mut world, 10).0, change::Decision::Hold(change::Cause::PullClosed));
    assert_eq!(world.change(TASK).pull, Some(pull));
    assert_eq!(world.writes(), 0);
}

#[test]
fn a_numbered_pull_read_on_another_base_holds_as_retargeted() {
    // The fake need not implement editing the base: a numbered read of the
    // same pull on a different branch is the observation the change judges.
    let (mut world, _, pull) = pushed_change(411, b"other");
    assert_eq!(step(&mut world, 10).0, change::Decision::Hold(change::Cause::Retargeted));
    assert_eq!(world.change(TASK).pull, Some(pull));
    assert_eq!(world.writes(), 0);
}
