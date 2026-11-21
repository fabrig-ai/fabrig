//! Suspended placement through the worker link (domain/engine/executors.md, 6.4).
use jig_core_world::fixtures::{live_row, sign_in};
use jig_core_world::world::{Boot, Setup, World};
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_link::{Form, Hello, Scheme};
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Time, Token, Wall};

const FIRST: Token = Token::new(7);
const SECOND: Token = Token::new(8);
const RETURNED: Token = Token::new(9);
const KEPT: &[u8] = b"work before the worker stopped";

fn worker(world: &mut World, position: u32, channel: Token) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: position,
        channel,
        hello: Hello {
            fresh: false,
            reads_usage: true,
            slots: 1,
            stop_bound: Duration::from_millis(100),
            bounds: jig_test_profile::fixtures::bounds(),
            jobs: Box::new([]),
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}

fn placement(world: &World, task: u64) -> Option<Token> {
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let Some(root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof)))) = world.store.rows.get(&key)
    else {
        panic!("durable claim proof")
    };
    proof.placement
}

#[test]
fn a_shutdown_resumes_on_its_worker_or_elsewhere_at_the_window_end() {
    for returns in [true, false] {
        let seed = 8491;
        let (configuration, mut limits) = jig_core_world::world::fixture(seed, false);
        limits.core.fleet.workers = 2;
        let budget = configuration.core.seed_projects[0].policy.policy.period_spend;
        let mut world = World::new(Setup {
            seed,
            configuration: Some(configuration),
            limits,
            boot: Boot::Empty,
            clock: Some((Time::ZERO, Wall::EPOCH)),
            ..Setup::default()
        });
        world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget, next: None }));
        world.send(root::Event::Core(core::Event::Account(accounts::Event::Component(
            accounts::FromComponent::Granted {
                form: Form::Credential(Scheme::Bearer),
                account: 1,
                generation: 1,
                valid: Duration::from_secs(60),
            },
        ))));
        worker(&mut world, 1, SECOND);
        world.send(sign_in(Token::new(101), 0, &7_u64.to_be_bytes(), b"person", b"Person"));
        let task = world.task();
        let attempt = world.attempt();
        assert_eq!(placement(&world, task), Some(SECOND));
        world.send(root::Event::Turn {
            channel: SECOND,
            task,
            attempt,
            turn: 1,
            cumulative: 3,
            read: None,
            refused: Box::new([]),
            opens_window: true,
            transcript: KEPT.into(),
        });
        worker(&mut world, 0, FIRST);
        world.send(root::Event::Link { channel: SECOND, link: root::link::Event::Up(host::LinkUp::Draining) });
        world.send(root::Event::Link {
            channel: SECOND,
            link: root::link::Event::Up(host::LinkUp::Answer {
                run: Token::new(task),
                attempt: Token::new(attempt),
                answer: host::RunAnswer {
                    turns: 1,
                    spent: 3,
                    read: None,
                    refused: Box::new([]),
                    ending: host::Ending::Failed {
                        failure: host::Failure::Cancelled(host::Reason::Shutdown),
                        detail: Box::new([]),
                        work: host::Work { left: None, saved: None },
                        model: None,
                    },
                },
            }),
        });
        assert_eq!(world.assignments.len(), 1, "the free first worker cannot steal the suspended task");
        let suspension = world.domain.core().tasks.suspension(task).expect("worker shutdown suspends");
        assert_eq!(suspension.worker, 2);
        assert_eq!(live_row(&world, task).tries, tasks::Tries::NONE);
        assert_eq!(
            world.domain.core().run_choice(task),
            Err(core::RunWait::Worker { worker: 2, until: suspension.until })
        );
        assert_eq!(
            super::runs::funding_wait(&mut world, task),
            Some(core::documents::TaskRunWait::Worker { worker: 2, until: suspension.until })
        );
        world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: SECOND })));
        let next = if returns {
            world.advance(Duration::from_secs(1));
            world.hold_logins = true;
            worker(&mut world, 1, RETURNED);
            assert_eq!(world.assignments.len(), 1, "the new hello must hear its source inventory");
            world.hold_logins = false;
            world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Logins {
                channel: RETURNED,
                logins: Box::new([]),
            })));
            RETURNED
        } else {
            let before = suspension.until.as_nanos() - world.clocks().1.as_nanos() - 1;
            world.advance(Duration::from_nanos(before));
            assert_eq!(world.assignments.len(), 1, "the window is still open one nanosecond before its end");
            world.advance(Duration::from_nanos(1));
            assert_eq!(world.clocks().1, suspension.until);
            FIRST
        };
        assert_eq!(world.assignments.len(), 2);
        assert!(world.attempt() > attempt);
        assert_eq!(placement(&world, task), Some(next));
        assert_eq!(live_row(&world, task).tries, tasks::Tries::NONE);
        assert_eq!(live_row(&world, task).suspension, None);
        world.send(sign_in(Token::new(103), 0, &7_u64.to_be_bytes(), b"person", b"Person"));
        assert_eq!(super::runs::funding_wait(&mut world, task), None);
        assert_eq!(world.assigned.as_ref().unwrap().transcript.as_ref(), [Box::<[u8]>::from(KEPT)]);
    }
}
