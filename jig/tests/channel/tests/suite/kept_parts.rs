//! The cut turn returns on W2's worker path, across a cold engine start too.
use jig_core_kit::{
    fixtures,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_link::{Hello, Hosting, Part, PartKind, Phase};
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

fn hello(world: &mut World, channel: Token, hosting: Box<[Hosting]>) {
    world.send(root::Event::Link {
        channel,
        link: root::link::Event::Hello {
            worker: 0,
            hello: Hello {
                fresh: false,
                reads_usage: true,
                slots: 1,
                stop_bound: Duration::ZERO,
                bounds: jig_test_profile::fixtures::bounds(),
                jobs: Box::new([]),
                workstreams: Box::new([]),
                hosting,
            },
        },
    });
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Logins { channel, logins: Box::new([]) })));
}

#[test]
fn a_suspended_workers_parts_return_after_restart_but_not_on_a_fresh_run() {
    for (restart, resume) in [(false, true), (true, true), (false, false)] {
        let (mut config, limits) = jig_core_kit::peers::fixture(479, 0);
        config.core.defaults.charters[0].policy.resume = resume;
        let mut world = World::new(Setup { seed: 479, configuration: Some(config), limits, ..Setup::default() });
        let (task, attempt) = (world.task(), world.attempt());
        world.turn(1, 3, None, b"committed");
        let part = Part {
            number: 1,
            turn: 2,
            thread: 0,
            call: 3,
            kind: PartKind::Answer,
            body: b"cut turn".as_slice().into(),
        };
        world.send(root::Event::Link {
            channel: Token::new(7),
            link: root::link::Event::Up(host::LinkUp::Part {
                run: Token::new(task),
                attempt: Token::new(attempt),
                part: part.clone(),
            }),
        });
        let key = root::Key::Core(core::Key::Core(core::CoreKey::Part(core::PartKey { task, attempt, number: 1 })));
        assert!(world.store.rows.contains_key(&key), "the part's write committed");
        if restart {
            let (mut config, _) = jig_core_kit::peers::fixture(479, 0);
            config.core.defaults.charters[0].policy.resume = resume;
            world.restart_with(config);
            hello(
                &mut world,
                Token::new(7),
                Box::new([Hosting { run: Token::new(task), attempt: Token::new(attempt), phase: Phase::Active }]),
            );
        }
        world.send(root::Event::Link { channel: Token::new(7), link: root::link::Event::Up(host::LinkUp::Draining) });
        world.send(root::Event::Link {
            channel: Token::new(7),
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
        let suspended = world.domain.core().tasks.suspension(task).expect("shutdown suspends");
        assert_eq!(
            world.domain.core().run_choice(task),
            Err(core::RunWait::Worker { worker: 1, until: suspended.until })
        );
        assert_eq!(fixtures::live_row(&world, task).tries, tasks::Tries::NONE);
        world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: Token::new(7) })));
        hello(&mut world, Token::new(8), Box::new([]));
        let assigned = world.assigned.as_ref().expect("resumed assignment");
        assert!(assigned.attempt > attempt);
        assert_eq!(fixtures::live_row(&world, task).tries, tasks::Tries::NONE);
        if resume {
            assert_eq!(assigned.transcript.as_ref(), [Box::<[u8]>::from(b"committed".as_slice())]);
            assert_eq!(
                assigned.parts.as_ref(),
                [jig_domain_link::KeptPart { attempt, turn: 2, thread: 0, call: 3, kind: part.kind, body: part.body }]
            );
        } else {
            assert!(assigned.parts.is_empty());
            assert!(assigned.transcript.is_empty());
        }
        let mut ending = world.end_event(
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                cancel_delegates: false,
            },
            0,
        );
        if let root::Event::Answer { channel, .. } = &mut ending {
            *channel = Token::new(8);
        } else {
            panic!("answer fixture")
        }
        world.send(ending);
        assert!(!world.store.rows.contains_key(&key), "the task's end retires the remaining part");
    }
}
