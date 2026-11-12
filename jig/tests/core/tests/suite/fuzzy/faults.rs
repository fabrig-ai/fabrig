//! Seeded actor, store and crash interleavings, with outside observations checked
//! after every root iteration. Each failing seed is printed for direct replay.

use jig_core_world::Fault as StoreFault;
use jig_core_world::faults;
use jig_core_world::world::Cut;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_core_world::world::fixture;
use jig_fake_workers::Fault as WorkerFault;
use jig_test_connector as connector;
use skein_lib::{Duration, Rng};

fn replay(seed: u64, scenario: impl FnOnce()) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(scenario));
    assert!(result.is_ok(), "fault scenario seed {seed}");
}

#[test]
fn random_host_worlds_draw_link_store_and_cold_restart_faults() {
    for seed in 1..=64 {
        replay(seed, || {
            let mut rng = Rng::new(seed);
            let engine = rng.below(2) == 0;
            let count = 1 + u32::try_from(rng.below(3)).expect("three workers");
            let mut world = faults::workers(seed, count, engine);
            let owner = world
                .peers
                .as_ref()
                .expect("peers")
                .workers
                .iter()
                .position(|worker| !worker.seen.is_empty())
                .expect("host assignment");
            let channel = world.peers.as_ref().expect("peers").workers[owner].channel;
            match rng.below(7) {
                0 => {
                    faults::worker_fault(&mut world, owner, WorkerFault::Slow { by: Duration::from_secs(1) });
                    faults::say(&mut world, 2);
                    world.advance(Duration::from_secs(1));
                }
                1 | 2 if channel != 0 => {
                    let by = if rng.below(2) == 0 { 1 } else { 7 };
                    faults::worker_fault(&mut world, owner, WorkerFault::DropChannel { for_: Duration::from_secs(by) });
                    world.advance(Duration::from_secs(by));
                    world.advance(Duration::from_secs(2));
                }
                3 if channel != 0 => {
                    faults::worker_fault(&mut world, owner, WorkerFault::Vanish);
                    world.advance(Duration::from_secs(6));
                    world.advance(Duration::from_secs(2));
                }
                4 => {
                    world.store.fault(StoreFault::Hold { commits: 1 });
                    faults::say(&mut world, 2);
                    world.release_commits();
                }
                5 => {
                    world.store.fault(StoreFault::Fail { commit: world.store.applied + 1 });
                    faults::say(&mut world, 2);
                    assert!(world.stopped);
                    faults::restart(&mut world, seed, engine);
                }
                _ => {
                    let next = world.store.applied + 1;
                    let cut = if rng.below(2) == 0 { Cut::Submitted(next) } else { Cut::Durable(next) };
                    world.cut = Some(cut);
                    faults::say(&mut world, 2);
                    assert_eq!(world.reached_cut, Some(cut));
                    world
                        .store
                        .fault(StoreFault::SlowPages { by: u32::try_from(rng.below(4)).expect("small page delay") });
                    faults::restart(&mut world, seed, engine);
                    faults::restart(&mut world, seed, engine);
                }
            }
            assert!(world.domain.ready());
            assert!(!world.stopped);
            let peers = world.peers.as_ref().expect("peers");
            assert!(peers.workers.iter().all(|worker| worker.expenses.values().all(|cost| cost.spent <= 10)));
            assert!(peers.parties[0].quiescent());
            if engine {
                // Engine 6 step 3 settles live engine claims (seed 18); their phase, reserve and activity change durably.
                world.broken();
            }
        });
    }
}

#[test]
fn random_effect_worlds_recover_lost_answers_and_late_copies_by_their_declared_class() {
    for seed in 65..=128 {
        replay(seed, || {
            let mut rng = Rng::new(seed);
            let recovery = match rng.below(4) {
                0 => connector::Recovery::Keyed,
                1 => connector::Recovery::Conditional,
                2 => connector::Recovery::Idempotent,
                _ => connector::Recovery::Unrecoverable,
            };
            let guarded = rng.below(2) == 0;
            let configuration = || {
                let (mut config, limits) = fixture(seed, guarded);
                config.first.kinds[0].recovery = recovery;
                config.first.kinds[0].form = match recovery {
                    connector::Recovery::Keyed => connector::Form::Creation,
                    connector::Recovery::Conditional | connector::Recovery::Unrecoverable => {
                        connector::Form::Transition
                    }
                    connector::Recovery::Idempotent => connector::Form::Set,
                };
                (config, limits)
            };
            let (config, limits) = configuration();
            let mut world = World::new(Setup {
                seed,
                requirement: guarded,
                configuration: Some(config),
                limits,
                ..Setup::default()
            });
            if guarded {
                world.observed(13);
            }
            // Retain the immutable request at the independent system, apply it,
            // and lose the reply. The core learns no result before the restart.
            world.hold_writes = true;
            let mut effect = World::effect();
            if recovery == connector::Recovery::Conditional {
                world.systems[0].other_hand(&effect.resources[0], 9);
                effect.condition = Some(9);
            }
            world.send(jig_test_domain::Event::EffectCall {
                to: skein_lib::ReplyTo::new(skein_lib::Token::new(200)),
                key: world.call_key(1),
                number: 1,
                effect: effect.clone(),
                deadline: skein_lib::Wall::from_nanos(world.wall_time().as_nanos() + 1_000_000_000),
                proposal: None,
            });
            if world.pending_writes.is_empty() && guarded {
                world.observed(13);
                world.send(jig_test_domain::Event::EffectCall {
                    to: skein_lib::ReplyTo::new(skein_lib::Token::new(201)),
                    key: world.call_key(1),
                    number: 1,
                    effect,
                    deadline: skein_lib::Wall::from_nanos(world.wall_time().as_nanos() + 1_000_000_000),
                    proposal: None,
                });
            }
            let (_, request) = world.pending_writes.pop().unwrap_or_else(|| {
                panic!(
                    "no decided effect: {recovery:?}, guarded {guarded}, answers {:?}, reads {:?}, trace {:?}",
                    world.answers,
                    world.pending_reads,
                    world.trace.iter().rev().take(12).collect::<Vec<_>>()
                )
            });
            let later = request.clone();
            let _lost = world.systems[0].answer(request, jig_test_system::Fault::AfterApply);
            world.hold_writes = false;
            for _ in 0..=rng.below(3) {
                world.store.fault(StoreFault::SlowPages { by: u32::try_from(rng.below(3)).expect("small delay") });
                world.restart_with(configuration().0);
            }
            // A transport's already-sent copy may arrive after lookup settled it.
            // The fake independently implements each promised recovery class.
            if recovery != connector::Recovery::Unrecoverable {
                let late = world.systems[0].answer(later, jig_test_system::Fault::None);
                world.send(jig_test_domain::Event::Connector { number: 1, event: connector::Event::System(late) });
            }
            let made = world.systems[0].observed().iter().filter(|row| row.applied).count();
            if matches!(
                recovery,
                connector::Recovery::Keyed | connector::Recovery::Conditional | connector::Recovery::Unrecoverable
            ) {
                assert_eq!(made, 1);
            } else {
                assert!(made >= 1);
                assert!(world.systems[0].observed().iter().filter(|row| row.applied).all(|row| row.target == 13));
            }
        });
    }
}

#[test]
fn random_drawn_commit_cuts_preserve_whole_turns_and_atomic_effect_decisions() {
    for seed in 129..=160 {
        replay(seed, || {
            let mut rng = Rng::new(seed);
            let mut world = World::new(Setup { seed, ..Setup::default() });
            let next = world.store.applied + 1;
            let cut = if rng.below(2) == 0 { Cut::Submitted(next) } else { Cut::Durable(next) };
            world.cut = Some(cut);
            world.turn(1, 1 + rng.below(5), None, b"one whole turn at the cut");
            assert_eq!(world.reached_cut, Some(cut));
            world.restart(seed, false);
            let turns: Vec<_> = world
                .store
                .rows
                .values()
                .filter_map(|row| match row {
                    jig_test_domain::Record::Core(jig_domain_core::Record::Core(
                        jig_domain_core::CoreRecord::Turn(turn),
                    )) => Some(turn),
                    jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
                })
                .collect();
            match cut {
                Cut::Submitted(_) => assert_eq!(turns.len(), 0),
                Cut::Durable(_) => assert_eq!(turns.len(), 1),
                Cut::Disk { .. } => panic!("this table selects transaction cuts"),
            }
            assert!(turns.iter().all(|turn| turn.transcript.as_ref() == b"one whole turn at the cut"));
            assert!(world.domain.ready());
        });
    }
}

#[test]
fn every_commit_of_a_turn_effect_and_terminal_survives_both_crash_cuts() {
    let mut reference = World::new(Setup { seed: 300, ..Setup::default() });
    let first = reference.store.applied + 1;
    crate::faults::effect_scenario(&mut reference);
    let last = reference.store.applied;
    assert!(last >= first + 2);
    for number in first..=last {
        for cut in [Cut::Submitted(number), Cut::Durable(number)] {
            crate::faults::an_effect_cut_recovers(cut);
        }
    }
}

#[test]
fn a_failed_commit_at_every_scenario_boundary_stops_before_any_later_output() {
    let mut reference = World::new(Setup { seed: 301, ..Setup::default() });
    let first = reference.store.applied + 1;
    crate::faults::effect_scenario(&mut reference);
    for number in first..=reference.store.applied {
        crate::faults::a_failed_commit_stops(number);
    }
}

#[test]
fn every_startup_sign_in_chat_and_assignment_commit_restarts_on_either_side_of_durability() {
    for engine in [false, true] {
        let mut reference = faults::workers(307, 1, engine);
        // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
        if engine {
            reference.broken();
        }
        for number in 1..=reference.store.applied {
            for cut in [Cut::Submitted(number), Cut::Durable(number)] {
                crate::faults::a_startup_cut_restarts(engine, cut);
            }
        }
    }
}

#[test]
fn every_operation_of_a_restart_after_a_torn_decision_can_be_killed_again() {
    crate::faults::restart_operation_sweep();
}
