//! Turn parts cross only channels that have their kinds (channel/records.md, 5).
use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;

#[test]
fn parts_are_requested_and_sent_only_at_twenty_two_and_acknowledged_back() {
    for version in [21, 22] {
        let mut world = super::translation::world(version, smith_charter::wire::VERSION, 1);
        world.acknowledge(false);
        let mut assignment = fixtures::assignment();
        assignment.assignment.charter = super::translation::charter(1);
        assignment.turns = Box::new([]);
        world.engine_down(
            host::LinkDown::Assign { assignment },
            fixtures::assignment_parts(),
            crate::joined_world::values(),
        );
        world.run();
        let Request::Down { down: host::LinkDown::Assign { assignment }, .. } =
            world.worker_root.last().expect("assignment")
        else {
            panic!("assignment")
        };
        assert_eq!(assignment.tell_parts, version == 22);
        assert!(assignment.parts.is_empty());
        let before = world.worker_records().len();
        let body = super::translation::golden(
            smith_transcript::patches::kept::smith_transcript::TEXTS.last().unwrap(),
            "record_turn_smallest",
        );
        world.worker_up(
            host::LinkUp::Part {
                run: fixtures::RUN_SEVEN,
                attempt: fixtures::ATTEMPT_TWO,
                part: jig_domain_link::Part {
                    number: 3,
                    turn: 1,
                    thread: 0,
                    call: 4,
                    kind: jig_domain_link::PartKind::Answer,
                    body,
                },
            },
            Box::new([]),
        );
        world.run();
        if version == 22 {
            assert_eq!(world.worker_records().get(before..), Some([wire::Kind::Part.number()].as_slice()));
            assert!(world.worker_root.iter().any(|r| matches!(r, Request::Down {
                down: host::LinkDown::PartKept { run, attempt, part: 3 }, ..
            } if *run == fixtures::RUN_SEVEN && *attempt == fixtures::ATTEMPT_TWO)));
        } else {
            assert_eq!(world.worker_records().len(), before);
            assert!(!world.engine_records().contains(&wire::Kind::PartKept.number()));
        }
        world.finish();
    }
}

#[test]
fn busy_parts_wait_for_backoff_and_cumulative_kept_cancels_them() {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    let busy = wire::PartBusy { task: 7, attempt: 2, number: 1 };
    let before = world.root().len();
    world.play(fixtures::frame(wire::Kind::PartBusy.number(), &fixtures::encoded(&busy)));
    assert_eq!(world.root().len(), before, "busy stays below the hub");
    world.advance(Duration::from_millis(99));
    assert!(!world.root().iter().any(|r| matches!(r, Request::ResendPart { .. })));
    world.advance(Duration::from_millis(101));
    assert!(world.root().iter().any(|r| matches!(r, Request::ResendPart { run, attempt, part: 1 }
        if *run == fixtures::RUN_SEVEN && *attempt == fixtures::ATTEMPT_TWO)));
    world.play(fixtures::frame(wire::Kind::PartBusy.number(), &fixtures::encoded(&busy)));
    let kept = wire::PartKept { task: 7, attempt: 2, number: 2 };
    world.play(fixtures::frame(wire::Kind::PartKept.number(), &fixtures::encoded(&kept)));
    world.advance(Duration::from_millis(200));
    assert_eq!(world.root().iter().filter(|r| matches!(r, Request::ResendPart { .. })).count(), 1);
}
