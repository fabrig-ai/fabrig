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
    fn another_wire_version_worker_redials_after_wires() {
        refusal(
            Credential {
                wires: {
                    let mut wires = fixtures::application();
                    wires.get_mut(0).expect("application wire").lowest = 4;
                    wires
                },
                ..fixtures::credential()
            },
            257,
        );
    }
}

fn largest_worker_frame(sizes: &jig_protocol_worker_wire::sizes::Sizes) -> u32 {
    jig_protocol_worker_wire::KINDS
        .iter()
        .filter(|rule| rule.direction == skein_channel::Direction::FromInitiator)
        .map(|rule| jig_protocol_worker_wire::sizes::largest_body(sizes, rule.body).expect("worker body bound") + 8)
        .max()
        .expect("worker records")
}

#[test]
fn an_assignment_past_the_engines_room_keeps_the_link_and_fails_its_run() {
    use jig_domain_host as host;
    use jig_protocol_engine_workers as engine;
    use jig_protocol_worker_link::Request;
    use jig_worker_link_world::fixtures;
    use skein_lib::{Duration, Token};
    // Both sides' equal room must hold every worker frame. A charter of that
    // size makes the assignment larger than the room by its fixed fields.
    let charter = largest_worker_frame(&fixtures::SIZES);
    let limits = jig_protocol_worker_link::Limits {
        sizes: jig_protocol_worker_wire::sizes::Sizes { charter, detail: 128, ..fixtures::SIZES },
        ..fixtures::joined_limits()
    };
    let bytes = fixtures::assignment_record(fixtures::window()).measure() + 8 + charter - 7
        + u32::try_from(fixtures::TOKEN_TWO.len() - fixtures::TOKEN_ONE.len()).expect("fixture grant length");
    let room = bytes - 1;
    let mut world = joined::Joined::with_engine_room(&jig_protocol_worker_link::Limits { room, ..limits }, 1, room);
    world.run();
    let mut assignment = fixtures::assignment();
    assignment.assignment.charter = vec![b'x'; usize::try_from(charter).expect("charter size")].into();
    world.engine_down(host::LinkDown::Assign { assignment }, fixtures::assignment_parts(), joined::values());
    world.run();
    let mut pings = Vec::new();
    for second in 1..=120 {
        world.advance(Duration::from_secs(1));
        assert!(
            !world.worker_root.iter().any(|r| matches!(r, Request::Lost | Request::Fact { .. })),
            "worker lost at {second} s: {:?}; engine {:?}",
            world.worker_root,
            world.engine_root
        );
        pings.push(world.ping_counts());
    }
    let mut small = fixtures::assignment();
    small.assignment.run = Token::new(8);
    world.engine_down(host::LinkDown::Assign { assignment: small }, fixtures::assignment_parts(), joined::values());
    world.run();
    assert!(
        !world.engine_root.iter().any(|r| matches!(r, engine::Request::Lost { .. } | engine::Request::Ended { .. }))
    );
    assert!(world.engine_ended.is_empty(), "{:?}", world.engine_ended);
    for (index, counts) in pings.iter().enumerate() {
        let expected = (index + 1) / 5;
        assert_eq!(*counts, [expected, expected], "pings at {} s", index + 1);
    }
    let downs: Vec<_> = world
        .worker_root
        .iter()
        .filter_map(|r| {
            if let Request::Down { down: host::LinkDown::Assign { assignment }, .. } = r {
                Some(assignment.assignment.run)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(downs, [Token::new(8)]);
    let answers: Vec<_> = world
        .engine_root
        .iter()
        .filter_map(|r| {
            if let engine::Request::Up { up: host::LinkUp::Answer { run, attempt, answer }, parts, .. } = r {
                Some((run, attempt, answer, parts))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(answers.len(), 1);
    let (run, attempt, answer, parts) = answers[0];
    assert_eq!((*run, *attempt), (Token::new(7), Token::new(2)));
    assert!(parts.is_empty());
    let host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start { cause: host::StartCause::Other, why })) =
        &answer.ending
    else {
        panic!("invalid start: {answer:?}")
    };
    let words = format!("assignment of {bytes} bytes past the link's room of {room} bytes");
    assert_eq!(why.as_ref(), words.as_bytes());
    let unsent: Vec<_> = world
        .engine_root
        .iter()
        .filter_map(|r| {
            if let engine::Request::Unsent { worker, channel, unsent } = r {
                Some((*worker, *channel, *unsent))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        unsent,
        [(
            0,
            Token::new(1),
            engine::Unsent {
                kind: jig_protocol_worker_wire::Kind::Assignment,
                bytes,
                bound: engine::Bound::Room(room)
            }
        )]
    );
    world.finish();
}

fn resumed_call(index: u32, bytes: u32, delivery: bool, cut: bool) -> jig_domain_host::AnsweredCall {
    use jig_domain_host::{AnsweredCall, DeliveryOutcome, SettledAnswer};
    let answer = if delivery && index == 0 {
        SettledAnswer::Delivery { outcome: DeliveryOutcome::Delivered, evidence: b"delivery".as_slice().into() }
    } else {
        let body = if cut {
            jig_protocol_worker_wire::fit::cut_words(bytes)
        } else {
            vec![u8::try_from(index).expect("few calls"); usize::try_from(bytes).expect("answer bytes")].into()
        };
        SettledAnswer::Host { error: index.is_multiple_of(2), body }
    };
    AnsweredCall { name: format!("call-{index:03}").into_bytes().into(), tool: b"read".as_slice().into(), answer }
}

fn resumed_assignment(calls: u32, answer: u32, delivery: bool) -> jig_domain_host::Assignment {
    use jig_worker_link_world::fixtures;
    let mut assignment = fixtures::assignment();
    assignment.assignment.workspace = None;
    assignment.assignment.grants = Box::new([]);
    assignment.messages = Box::new([]);
    assignment.answered = (0..calls).map(|index| resumed_call(index, answer, delivery, false)).collect();
    assignment
}

fn resumed_answer() -> jig_domain_host::LinkUp {
    use jig_domain_host as host;
    use jig_worker_link_world::fixtures;
    host::LinkUp::Answer {
        run: fixtures::RUN_SEVEN,
        attempt: fixtures::ATTEMPT_TWO,
        answer: host::RunAnswer {
            read: None,
            refused: Box::new([]),
            turns: 1,
            spent: 3,
            ending: host::Ending::Ended {
                outcome: b"result".as_slice().into(),
                work: host::Work { left: None, saved: None },
            },
        },
    }
}

fn resumed_continues(world: &mut joined::Joined) {
    use jig_domain_host as host;
    use jig_protocol_engine_workers as engine;
    use jig_protocol_worker_link::Request;
    use jig_worker_link_world::fixtures;
    world.worker_up(fixtures::turn(), Box::new([]));
    world.run();
    world.worker_up(resumed_answer(), Box::new([]));
    world.run();
    assert_eq!(world.engine_turns(), [1]);
    assert!(world.engine_root.iter().any(|r| matches!(r, engine::Request::Up { up, .. } if *up == fixtures::turn())));
    assert!(world.engine_root.iter().any(|r| matches!(r, engine::Request::Up { up, .. } if *up == resumed_answer())));
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Down { down: host::LinkDown::TurnKept { run, attempt, turn: 1 }, .. } if (*run, *attempt) == (fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO))));
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Down { down: host::LinkDown::Acknowledge { run, attempt }, .. } if (*run, *attempt) == (fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO))));
}

// Both real sides keep a resumed producer going within io's room
// (jig/docs/design/protocol/channel/README.md, 8, 10 and 11).
#[test]
fn a_resumed_start_fits_the_room_on_each_stock_profile() {
    use jig_domain_host as host;
    use jig_protocol_config::{Profile, profile};
    use jig_protocol_engine_workers as engine;
    use jig_protocol_worker_link::{Limits, Request};
    use jig_protocol_worker_wire::fit::cut_words;
    use jig_worker_link_world::fixtures;
    use skein_lib::Duration;
    // Fixed65 + charter7 + a two-turn window16; a call's fields14,
    // name8 and tool4; delivery evidence8. The oldest call is a delivery.
    const BASE: u32 = 88;
    const CALL: u32 = 26;
    const DELIVERY: u32 = CALL + 8;
    const FEW: u32 = 3;
    const EXCESS: u32 = 2;
    let mut failures = Vec::new();
    for p in [Profile::Small, Profile::Standard] {
        let config = profile(p);
        let engine = jig_test_service::limits::for_profile(p, &config).workers;
        // The world's worker fixtures keep their deadlines and entrances;
        // its wire sizes are the profile's, with io's room capping its pipe.
        let limits =
            Limits { sizes: engine.sizes, chunk: engine.chunk, room: engine.room, ..fixtures::joined_limits() };
        let answer = config.core.call_answer_bytes;
        let room = config.io.largest_room();
        let words = cut_words(answer);
        let word_bytes = u32::try_from(words.len()).expect("cut words length");
        let past = room / (answer + CALL) + EXCESS;
        for (name, calls, delivery, whole) in [
            ("few calls", FEW, false, FEW),
            (
                "past the room by its calls",
                past,
                true,
                (room - BASE - DELIVERY - (past - 1) * (CALL + word_bytes)) / (answer - word_bytes),
            ),
        ] {
            let mut world = joined::Joined::with_limits(&limits, engine, 1);
            world.run();
            let assignment = resumed_assignment(calls, answer, delivery);
            world.engine_down(host::LinkDown::Assign { assignment }, Box::new([]), Box::new([]));
            world.run();
            let assignments: Vec<_> = world
                .worker_root
                .iter()
                .filter_map(|r| {
                    if let Request::Down { down: host::LinkDown::Assign { assignment }, .. } = r {
                        Some(assignment)
                    } else {
                        None
                    }
                })
                .collect();
            if assignments.len() != 1 {
                failures.push(format!("{p:?}, {name}, {calls} calls: no assignment; engine {:?}", world.engine_root));
                world.finish();
                continue;
            }
            let assignment = assignments[0];
            assert_eq!(assignment.turns, fixtures::assignment().turns, "{p:?}, {name}: window");
            assert_eq!(assignment.answered.len(), usize::try_from(calls).expect("calls"));
            for (index, call) in assignment.answered.iter().enumerate() {
                let index = u32::try_from(index).expect("few calls");
                let expected = resumed_call(index, answer, delivery, index < calls - whole);
                assert_eq!(*call, expected, "{p:?}, {name}, call {index}");
            }
            let bytes = BASE
                + u32::from(delivery) * DELIVERY
                + (calls - u32::from(delivery) - whole) * (CALL + word_bytes)
                + whole * (CALL + answer);
            assert_eq!(world.assignment_bytes(), [bytes], "{p:?}, {name}: frame");
            assert!(bytes <= room);
            eprintln!("{p:?}, {name}: {calls} calls, {whole} host answers whole, frame {bytes}/{room}");
            resumed_continues(&mut world);
            for second in 1..=120 {
                world.advance(Duration::from_secs(1));
                assert!(
                    !world.worker_root.iter().any(|r| matches!(r, Request::Lost | Request::Fact { .. })),
                    "{p:?}, {name}: worker lost at {second}"
                );
                assert!(
                    !world
                        .engine_root
                        .iter()
                        .any(|r| matches!(r, engine::Request::Lost { .. } | engine::Request::Unsent { .. })),
                    "{p:?}, {name}: engine failed at {second}"
                );
                assert!(world.engine_ended.is_empty());
            }
            assert!(world.ping_counts().into_iter().all(|count| count > 0), "{p:?}, {name}: pings both ways");
            world.finish();
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
