use crate::joined_world as joined;
mod joined_story {
    use super::joined::{Joined, application, credential, received, sent, values};
    use jig_domain_host as host;
    use jig_protocol_engine_workers as engine;
    use jig_protocol_worker_link::{Credential, Loss, Request};
    use jig_worker_link_world::fixtures::{self, ATTEMPT_TWO, RUN_SEVEN};
    use skein_channel::Closed;
    use skein_lib::{Duration, Secret, Time, Token};
    fn world(seed: u64) -> Joined {
        let mut world = Joined::new(&fixtures::joined_limits(), seed);
        world.run();
        world
    }
    fn assign(world: &mut Joined) {
        world.engine_down(
            host::LinkDown::Assign { assignment: fixtures::assignment() },
            fixtures::assignment_parts(),
            values(),
        );
        world.run();
    }
    fn downs(world: &Joined) -> Vec<&host::LinkDown> {
        world
            .worker_root
            .iter()
            .filter_map(|request| if let Request::Down { down, .. } = request { Some(down) } else { None })
            .collect()
    }
    fn records(world: &Joined) -> Vec<Vec<u16>> {
        world.frames().iter().map(|frames| frames.iter().copied().filter(|kind| *kind >= 256).collect()).collect()
    }
    fn value(request: &Request, expected: &engine::GrantValue) {
        let Request::Value { account, generation, expires, value } = request else { panic!("a layer value") };
        assert_eq!((*account, *generation, *expires), (1, 3, Time::ZERO.saturating_add(Duration::from_secs(60))));
        assert_eq!(received(value), sent(expected));
    }
    #[test]
    fn a_worker_is_admitted_and_its_hello_reaches_the_engine_root() {
        let mut world = world(1);
        let engine::Request::Hello { channel, hello, worker: _ } = &world.engine_root[0] else { panic!("hello") };
        assert_eq!((*channel, hello), (Token::new(1), &fixtures::hello()));
        assert_eq!(world.dials, [Time::ZERO]);
        world.finish();
    }
    #[test]
    fn an_assignments_grant_values_reach_the_worker_layer() {
        let mut world = world(1);
        assign(&mut world);
        assert_eq!(world.worker_order, ["value", "value", "down"]);
        assert_eq!(world.values.len(), 2);
        value(&world.values[0], &credential(engine::Scheme::Bearer));
        value(&world.values[1], &application());
        let host::LinkDown::Assign { assignment } = downs(&world)[0] else { panic!("assignment") };
        assert_eq!(assignment.assignment.grants, fixtures::assignment().assignment.grants);
        world.finish();
    }
    #[test]
    fn a_turn_crosses_and_is_kept_after_a_cut_and_a_redial() {
        for seed in 0..8 {
            let mut world = world(seed);
            assign(&mut world);
            world.acknowledge(false);
            world.worker_up(fixtures::turn(), Box::new([]));
            world.run();
            world.cut();
            world.acknowledge(true);
            world.advance(Duration::from_millis(200));
            assert_eq!(world.engine_turns(), [1]);
            assert_eq!(world.engine_channels(), [Token::new(1), Token::new(2)]);
            let turns = world
                .engine_root
                .iter()
                .filter(|request| matches!(request, engine::Request::Up { up: host::LinkUp::Turn { .. }, .. }))
                .count();
            assert_eq!(turns, 2, "the unacknowledged turn crossed both channels");
            assert_eq!(
                downs(&world).last(),
                Some(&&host::LinkDown::TurnKept { run: RUN_SEVEN, attempt: ATTEMPT_TWO, turn: 1 })
            );
            assert_eq!(records(&world), [vec![0x0100, 0x0101], vec![0x0100, 0x0101]]);
            world.finish();
        }
    }
    #[test]
    fn a_relayed_call_is_answered_across_the_two_sides() {
        let mut world = world(1);
        world.worker_up(fixtures::call(&[0, 2, 7]), Box::new([]));
        world.run();
        assert_eq!(downs(&world), [&fixtures::relay_answer(&[0, 2, 7])]);
        world.finish();
    }
    #[test]
    fn relays_go_again_after_the_hello_before_the_hubs_retained_turns() {
        let mut world = world(3);
        world.acknowledge(false);
        world.worker_up(fixtures::call(&[0, 2, 7]), Box::new([]));
        world.worker_up(fixtures::turn(), Box::new([]));
        world.run();
        world.cut();
        world.acknowledge(true);
        world.advance(Duration::from_millis(200));
        assert_eq!(
            downs(&world),
            [
                &fixtures::relay_answer(&[0, 2, 7]),
                &host::LinkDown::TurnKept { run: RUN_SEVEN, attempt: ATTEMPT_TWO, turn: 1 },
            ]
        );
        assert_eq!(records(&world), [vec![0x0100, 0x0102, 0x0101], vec![0x0100, 0x0102, 0x0101]]);
        world.finish();
    }
    #[test]
    fn an_answer_is_acknowledged_once() {
        let mut world = world(1);
        world.worker_up(fixtures::answer(), fixtures::work_parts());
        world.run();
        assert_eq!(downs(&world), [&host::LinkDown::Acknowledge { run: RUN_SEVEN, attempt: ATTEMPT_TWO }]);
        world.finish();
    }
    fn numbered_turn(number: u32) -> host::LinkUp {
        let host::LinkUp::Turn { run, attempt, mut turn } = fixtures::turn() else { panic!("turn fixture") };
        turn.turn = number;
        host::LinkUp::Turn { run, attempt, turn }
    }
    fn activity(number: u32) -> host::LinkUp {
        host::LinkUp::Activity {
            run: RUN_SEVEN,
            attempt: ATTEMPT_TWO,
            activity: host::AgentActivity::Long {
                operation: jig_domain_link::Operation::Command,
                span: Duration::from_secs(u64::from(number)),
            },
        }
    }
    fn message(number: u32) -> host::LinkDown {
        host::LinkDown::Message {
            run: RUN_SEVEN,
            attempt: ATTEMPT_TWO,
            name: Token::new(u64::from(number)),
            sender: b"sender".as_slice().into(),
            words: b"later".as_slice().into(),
            finish_requested: false,
        }
    }
    fn healthy(world: &Joined, row: &str, second: u32) {
        assert!(
            !world.worker_root.iter().any(|r| matches!(r, Request::Lost | Request::Fact { .. })),
            "{row} at {second} s: worker {:?}",
            world.worker_root
        );
        assert!(
            !world.engine_root.iter().any(|r| matches!(r, engine::Request::Lost { .. })),
            "{row} at {second} s: engine {:?}",
            world.engine_root
        );
    }
    #[test]
    fn a_link_busy_both_ways_for_minutes_is_never_lost() {
        struct Load {
            name: &'static str,
            turns: bool,
            facts: u32,
            messages: u32,
            every: u32,
            grants: [Option<usize>; 2],
        }
        // Ten simulated minutes, one second per step; forty records per minute in a burst.
        const SECONDS: u32 = 600;
        const BURST: u32 = 40;
        let rows = [
            Load { name: "steady", turns: true, facts: 0, messages: 0, every: 1, grants: [None, None] },
            Load { name: "trickle", turns: true, facts: 2, messages: 1, every: 1, grants: [Some(3), Some(2)] },
            Load { name: "engine burst", turns: false, facts: 0, messages: BURST, every: 60, grants: [None, Some(1)] },
        ];
        for row in rows {
            // Reuse the shipped Small profile's sizes: its cap admits the burst.
            let sizes = jig_test_service::limits::profile_limits(jig_protocol_config::Profile::Small).workers.sizes;
            let limits = jig_protocol_worker_link::Limits { sizes, ..fixtures::joined_limits() };
            let mut world = Joined::new(&limits, 1);
            world.run();
            world.grants(row.grants[0].map(|_| 0), row.grants[1].map(|_| 0));
            let mut expected_up = Vec::new();
            let mut expected_down = Vec::new();
            let mut next_message = 1;
            for second in 0..SECONDS {
                if row.turns {
                    let number = second + 1;
                    world.worker_up(numbered_turn(number), Box::new([]));
                    expected_up.push(numbered_turn(number));
                }
                for fact in 0..row.facts {
                    let number = second * row.facts + fact + 1;
                    world.worker_up(activity(number), Box::new([]));
                    expected_up.push(activity(number));
                }
                if second % row.every == 0 {
                    for _ in 0..row.messages {
                        world.engine_down(message(next_message), Box::new([]), Box::new([]));
                        expected_down.push(message(next_message));
                        next_message += 1;
                    }
                }
                world.run();
                // Leave the initial trickle queued; thereafter each old batch drains
                // while the next waits, for longer than either liveness deadline.
                let grants = if row.turns && row.facts > 0 && second == 0 { [Some(0), Some(0)] } else { row.grants };
                // Separate each frame's grant from the next one.
                let rounds = grants.into_iter().flatten().max().unwrap_or(1);
                for frame in 0..rounds {
                    world.grants(grants[0].map(|n| usize::from(frame < n)), grants[1].map(|n| usize::from(frame < n)));
                    world.run();
                }
                healthy(&world, row.name, second);
                if row.turns && row.facts > 0 {
                    assert_eq!(world.waiting(), [true, true], "{} at {second} s keeps output queued", row.name);
                }
                world.advance(Duration::from_secs(1));
                healthy(&world, row.name, second + 1);
            }
            world.grants(None, None);
            world.run();
            healthy(&world, row.name, SECONDS);
            let up: Vec<_> = world
                .engine_root
                .iter()
                .filter_map(|r| if let engine::Request::Up { up, .. } = r { Some(up) } else { None })
                .collect();
            assert_eq!(up, expected_up.iter().collect::<Vec<_>>(), "{}: every worker record in order", row.name);
            let received_messages: Vec<_> =
                downs(&world).into_iter().filter(|d| matches!(d, host::LinkDown::Message { .. })).collect();
            assert_eq!(
                received_messages,
                expected_down.iter().collect::<Vec<_>>(),
                "{}: every engine message in order",
                row.name
            );
            let kept: Vec<_> = downs(&world)
                .into_iter()
                .filter_map(|d| if let host::LinkDown::TurnKept { turn, .. } = d { Some(*turn) } else { None })
                .collect();
            let expected_kept: Vec<_> = (1..=if row.turns { SECONDS } else { 0 }).collect();
            assert_eq!(kept, expected_kept, "{}: every acknowledgement in order", row.name);
            world.finish();
        }
    }
    fn refusal(credential: Credential, reason: u16) {
        let mut world = Joined::with_credential(&fixtures::joined_limits(), 1, credential);
        world.run();
        assert!(world.engine_root.is_empty());
        assert!(world.worker_root.iter().any(|request| matches!(request,
            Request::Fact { why: Loss::Closed(Closed::RefusedPeer(found)) } if *found == reason)));
        world.advance(Duration::from_millis(200));
        assert!(world.dials.len() >= 2);
        assert!(world.dials[1] >= Time::ZERO.saturating_add(Duration::from_millis(50)));
        assert!(world.dials[1] <= Time::ZERO.saturating_add(Duration::from_millis(100)));
        world.finish();
    }
    #[test]
    fn a_wrong_secret_worker_redials_after_unauthorized() {
        refusal(Credential { secret: Secret::new(b"wrong".as_slice().into()), ..fixtures::credential() }, 256);
    }
    #[test]
    fn another_payload_version_worker_redials_after_application() {
        refusal(Credential { payload: 4, ..fixtures::credential() }, 257);
    }
}
