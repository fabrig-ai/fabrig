//! A process replacement and a same-process reconnect at the core/fleet boundary
//! (jig/docs/design/domain/engine/executors.md, 6.5).
use jig_core_kit::{
    fixtures,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_link::Hello;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

fn ended(world: &World, task: u64, attempt: u64) -> Option<tasks::EndKind> {
    world.store.rows.values().find_map(|row| match row {
        root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row))) if row.task == task => match row.line {
            tasks::Line::RunEnded { attempt: of, end, .. } if of == attempt => Some(end),
            _ => None,
        },
        _ => None,
    })
}

#[test]
fn a_new_worker_process_suspends_two_attempts_but_a_reconnect_keeps_their_grace() {
    for fresh in [false, true] {
        let (config, mut limits) = jig_core_kit::peers::fixture(470, 0);
        limits.core.fleet.slots = 2;
        let grace = limits.core.fleet.grace;
        let window = limits.core.fleet.restart_wait;
        let mut world = World::new(Setup { seed: 470, configuration: Some(config), limits, ..Setup::default() });
        let sign_in = world.sign_in.expect("signed in");
        let ask = fixtures::chat(&world, 1, b"a second run".as_slice().into());
        fixtures::ask_as(&mut world, sign_in, 61, ask);
        let runs = world.assignments.clone();
        assert_eq!(runs.len(), 2, "two runs share the worker process");
        for &(task, _) in &runs {
            let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
            let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &world.store.rows[&key]
            else {
                panic!("durable claim")
            };
            assert_eq!(proof.worker, Some(1), "preparation records the worker setting number");
            assert_eq!(proof.placement, Some(Token::new(7)));
        }
        let (now, wall) = world.clocks();
        world.set_clock(now, wall);
        world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: Token::new(7) })));
        // The new channel authenticates under the same worker name (position zero).
        world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
            channel: Token::new(8),
            worker: 0,
            hello: Hello {
                fresh,
                slots: 2,
                stop_bound: Duration::ZERO,
                bounds: jig_test_profile::fixtures::bounds(),
                jobs: Box::new([]),
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        })));
        for &(task, attempt) in &runs {
            if fresh {
                assert_eq!(world.clocks(), (now, wall), "suspension waits for no grace timer");
                assert_eq!(
                    ended(&world, task, attempt),
                    Some(tasks::EndKind::Suspended { worker: 1, why: tasks::SuspendWhy::Restarted })
                );
                assert_eq!(fixtures::live_row(&world, task).tries, tasks::Tries::NONE);
                assert!(
                    world.store.rows.values().any(|row| matches!(row,
                        root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(terminal)))
                            if terminal.task == task && terminal.attempt == attempt
                                && terminal.end == tasks::End::Suspended(tasks::Suspended {
                                    worker: 1, why: tasks::SuspendWhy::Restarted, window,
                                })
                    )),
                    "the core records the fleet's restart window"
                );
            } else {
                assert_eq!(ended(&world, task, attempt), None, "a reconnect proves no run gone");
            }
        }
        if !fresh {
            world.advance(Duration::from_nanos(grace.as_nanos().saturating_sub(Duration::from_millis(1).as_nanos())));
            for &(task, attempt) in &runs {
                assert_eq!(ended(&world, task, attempt), None);
            }
            world.advance(Duration::from_millis(1));
            let (now, wall) = world.clocks();
            world.fire_due(now, wall);
            for &(task, attempt) in &runs {
                assert!(ended(&world, task, attempt).is_some(), "the adrift grace eventually ends it");
                assert_ne!(fixtures::live_row(&world, task).tries, tasks::Tries::NONE);
            }
        }
    }
}
