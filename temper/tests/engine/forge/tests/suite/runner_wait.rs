//! A runner wait keeps a queue turn; running alone spends the stall
//! (domain/forge/change/stalls.md, 2.1–2.4).
use skein_fake_forge_domain::api as raw;
use skein_forge::api as client;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{LIMITS, World, changes, translate};

const DECLARATION: &[u8] = b"ci/build";
const FIRST: &[u8] = b"temper/51";

fn checking_after_update(declaration: &[u8]) -> (World, client::Commit) {
    let mut world = World::with_check_cue(17, raw::Cue::Declared { path: DECLARATION.into() });
    world.adopt();
    world.produce_file(FIRST, DECLARATION, declaration);
    let pull = world.open_pull(FIRST, b"main");
    world.status(FIRST, raw::Check::Passed);
    let head = translate::commit(world.branch(FIRST));
    let mut row = changes::change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.since = world.wall();
    row.change.state = change::State::Queued { head, ready_since: world.wall() };
    world.event(top::Event::Change { row });
    world.push_file(b"main", b"base", b"base moved");
    assert!(matches!(
        changes::change_step(&mut world, 1, change::Status::Passed, Box::new([])),
        change::Decision::Effect(change::Effect::Update { .. })
    ));
    let updated = translate::commit(world.branch(FIRST));
    assert_ne!(updated, head);
    assert!(matches!(
        changes::change_step(&mut world, 2, change::Status::Passed, Box::new([])),
        change::Decision::Wait { .. }
    ));
    assert_checking_with_turn(&world, updated);
    (world, updated)
}

// The scripted root polls the checking procedure within the client's poll_max.
fn poll_checking_for(world: &mut World, seconds: u64) {
    assert_eq!(seconds % 10, 0);
    for _ in 0..seconds / 10 {
        if matches!(world.change(51).change.state, change::State::Checking { .. }) {
            world.event(top::Event::StepChange {
                task: 51,
                entry: world.now().as_nanos(),
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
                gates: Box::new([]),
                queue_repair_active: false,
                kept: false,
                merge_job: false,
                pending: Box::new([]),
            });
        }
        world.run_for(10);
    }
}

fn assert_checking_with_turn(world: &World, head: client::Commit) {
    let change = &world.change(51).change;
    assert_eq!(change.state, change::State::Checking { head });
    assert!(change.owns_turn);
}

fn assert_never_held(world: &World) {
    assert!(!world.history(51).iter().any(|event| matches!(event.what, top::history::What::Held { .. })));
}

fn read_run(world: &World, run: u64) -> client::Run {
    let op = client::Op::Read(client::Read::Run { run });
    let answer = world.inspect(&raw::Read::Run { run });
    let client::Answer::Run(run) = translate::answer(&op, answer, &LIMITS.client).expect("run translation") else {
        panic!("a run read returns a run")
    };
    run
}

#[test]
fn waiting_for_a_runner_keeps_the_turn_then_passes() {
    let (mut world, head) = checking_after_update(b"wait 2m run 10s pass");
    poll_checking_for(&mut world, 90);
    assert_checking_with_turn(&world, head);
    let clock = world.change(51).change.clock.expect("waiting execution clock");
    assert_eq!(clock.head, head);
    assert_eq!(clock.seen, change::Execution::Waiting);
    assert_eq!(clock.ran, Duration::ZERO);
    assert!(clock.waited > LIMITS.change_policy.stall);
    assert_eq!(read_run(&world, clock.run).state, client::RunState::Waiting);
    assert_never_held(&world);

    poll_checking_for(&mut world, 50);
    let run = read_run(&world, clock.run);
    assert_eq!(run.state, client::RunState::Completed);
    assert_eq!(run.conclusion, Some(client::Conclusion::Success));
    assert!(
        matches!(world.change(51).change.state, change::State::First { head: checked, .. } if checked == head),
        "passed checking: {:?}",
        world.change(51).change.state
    );
    assert!(world.change(51).change.owns_turn);
    assert_never_held(&world);
}

#[test]
fn running_past_the_stall_after_a_runner_wait_passes_the_turn() {
    let (mut world, head) = checking_after_update(b"wait 1m run 2m pass");
    world.produce_file(b"temper/52", b"second", b"second change");
    let pull = world.open_pull(b"temper/52", b"main");
    let second_head = translate::commit(world.branch(b"temper/52"));
    let mut second = changes::change_row();
    second.task = 52;
    second.change.task = 52;
    second.branch = b"temper/52".as_slice().into();
    second.pull = Some(pull);
    second.change.last_head = Some(second_head);
    second.change.since = world.wall();
    second.change.state = change::State::Queued { head: second_head, ready_since: world.wall() };
    world.event(top::Event::Change { row: second });
    assert_eq!(
        changes::change_step_task(&mut world, 52, 3, change::Status::Passed, Box::new([]), false, false),
        change::Decision::Wait { until: None }
    );
    poll_checking_for(&mut world, 40);
    assert_checking_with_turn(&world, head);
    let clock = world.change(51).change.clock.expect("waiting execution clock");
    assert_eq!(clock.seen, change::Execution::Waiting);
    assert_eq!(clock.ran, Duration::ZERO);
    assert_never_held(&world);
    assert!(!world.change(52).change.owns_turn);

    poll_checking_for(&mut world, 80);
    assert!(matches!(
        &world.change(51).change.state,
        change::State::Held { was, why: change::Hold::Stalled } if **was == change::State::Checking { head }
    ));
    let clock = world.change(51).change.clock.expect("stalled execution clock");
    assert_eq!(clock.seen, change::Execution::Running);
    assert!(clock.ran >= LIMITS.change_policy.stall);
    assert!(!world.change(51).change.owns_turn);
    assert!(world.seen().iter().any(|request| matches!(request, top::Request::WakeProcedure { task: 52 })));
    assert!(matches!(
        changes::change_step_task(&mut world, 52, 4, change::Status::Passed, Box::new([]), false, false),
        change::Decision::Effect(change::Effect::Update { .. })
    ));
    assert!(world.change(52).change.owns_turn);
}
