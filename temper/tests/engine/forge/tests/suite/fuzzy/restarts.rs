//! Restart faults over the connector's existing steps (testing/restarts.md, 9).
use std::convert::Infallible;

use skein_lib::{Duration, Wall};
use skein_world::{
    streams::{Stream, Streams},
    sweep::{Sweep, cases, gate},
};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, landings,
    restarts::{GOAL, Injector},
};
use temper_faults::restarts::FORGE;

fn heard() -> top::LandingHeard {
    top::LandingHeard {
        job: None,
        outcomes: Box::new([]),
        dependencies: Box::new([]),
        held: Box::new([]),
        ended: Box::new([]),
        failure: Box::new([]),
        preparation: Some(landing::Preparation::Idle),
        released: false,
        cancelled: false,
        kept: Box::new([]),
    }
}

fn run(seed: u64, sweep: &Sweep) -> (Injector, usize) {
    let mut workload = Streams::new(seed).rng(Stream::Workload, "forge-restarts");
    let mut world = landings::turn_in(World::batch_turn(seed, None));
    let count = workload.between(1, 2);
    for member in 0..count {
        let branch = format!("temper/c{}", 51 + member);
        let file = format!("file{member}");
        world.produce_file(branch.as_bytes(), file.as_bytes(), &workload.next_u64().to_be_bytes());
        landings::ready_change(&mut world, 51 + member, branch.as_bytes());
    }
    let row = landings::landing_row(world.change(51));
    landings::register(&mut world, row);
    world.event(top::Event::Names {
        task: 42,
        resources: Box::new([top::Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: top::What::Branch(Box::new([b"main".as_slice().into()])),
        }]),
    });
    // Three seconds include a limited read's retry and a finite outage's end.
    // Never-ending faults need no further idle retries to check the world's rules.
    let end = world.now().saturating_add(Duration::from_secs(3));
    let mut injector = Injector::new(seed);
    // The first eight driver iterations interleave the restart with the workload.
    injector.draw(&FORGE, 8, Wall::from_nanos(end.as_nanos()));
    if injector.needs_uncertain_write() {
        world.lose_made_write_answers();
    }
    super::super::restarts::project(&mut world, 1);
    if injector.needs_uncertain_write() {
        // Establish the script's precondition, not a drain: the issue was made
        // but its caller still awaits the answer when the driver begins. Setup
        // has at most two members and no outage or timeout; the fake answers
        // calls in at most 50 ms. Only the successful creation answer is lost.
        for _ in 0..100 {
            if world.made_without_answer() > 0 {
                break;
            }
            assert!(world.step_until(end), "seed {seed}: issue must reach the fake");
        }
        assert_eq!(world.made_without_answer(), 1);
    }
    world.take_seen();
    let mut next_entry = 10_000;
    let mut pending = false;
    let mut reconciling = false;
    let mut finished = false;
    let mut reached = false;
    for _ in 0..20_000 {
        for request in world.take_seen() {
            match request {
                top::Request::LandingDecision { task, decision, entry, .. } if task == landings::LANDING => {
                    pending = false;
                    if let Some(entry) = entry {
                        world.event(top::Event::Committed { entry });
                    }
                    if matches!(decision, landing::Decision::Prepare { .. }) {
                        world.prepare_landing(task, workload.chance(500));
                    }
                    finished = decision == landing::Decision::Finish;
                }
                top::Request::RestartDone { stage: top::RestartStage::Restored } => {
                    reconciling = true;
                    pending = false;
                }
                top::Request::RestartDone { stage: top::RestartStage::Settled } => reconciling = false,
                top::Request::ContinueGoalBranch { task: GOAL } => {
                    world.event(top::Event::ContinueGoalBranch { task: GOAL, entry: next_entry });
                    next_entry += 1;
                }
                _ => {}
            }
        }
        if !pending && !reconciling && !finished {
            world.event(top::Event::StepLanding { task: landings::LANDING, entry: next_entry, heard: heard() });
            next_entry += 1;
            pending = true;
        }
        if !injector.step(&mut world, sweep, end) {
            reached = true;
            break;
        }
    }
    assert!(reached, "seed {seed}: world reaches the horizon");
    assert_eq!(world.now(), end);
    // A never-ending fault promises no goal's completion. The world's fake
    // and connector still enforce their call/row contracts on every step.
    super::history::check(seed, 51, &world.history(51));
    let dispatches = world
        .take_calls()
        .iter()
        .filter(|op| matches!(op, skein_forge::api::Op::Write(skein_forge::api::Write::Dispatch { .. })))
        .count();
    eprintln!(
        "seed {seed}: {:?}; {dispatches} gate dispatches, {} scripted answers",
        injector.ledger().fell(),
        world.restart_answers().len()
    );
    (injector, dispatches)
}

#[test]
fn seeded_forge_restarts_cover_every_kind() {
    let mut sweep = Sweep::new("forge restarts");
    for kind in FORGE {
        sweep.configure(kind);
    }
    // One extra kind per seed, plus its crash: six seeds are the minimum.
    let at_gate = gate();
    let mut dispatches = 0;
    cases(skein_world::this_test!("seeded_forge_restarts_cover_every_kind"), [0, 1, 2, 5, 12, 21], |seed| {
        eprintln!("forge restarts seed {seed}");
        let (run, sent) = run(seed, &sweep);
        sweep.ran(run.ledger());
        if at_gate {
            dispatches += sent;
        }
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    sweep.check_fell().expect("every configured restart fault fell");
    if at_gate {
        assert!(dispatches > 0, "the drawn landings exercise gate dispatches");
    }
}
