//! Turn parts cross only channels that have their kinds (channel/records.md, 5).
use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;

#[test]
fn parts_are_requested_sent_and_acknowledged_where_terms_hold_them() {
    let mut world = crate::joined_world::Joined::new(&fixtures::limits(), 1);
    world.run();
    world.acknowledge(false);
    let mut assignment = fixtures::assignment();
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
    assert!(assignment.tell_parts);
    assert!(assignment.parts.is_empty());
    let before = world.worker_records().len();
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
                body: b"body".as_slice().into(),
            },
        },
        Box::new([]),
    );
    world.run();
    assert_eq!(world.worker_records().get(before..), Some([wire::Kind::Part.number()].as_slice()));
    world.engine_down(
        host::LinkDown::PartKept { run: fixtures::RUN_SEVEN, attempt: fixtures::ATTEMPT_TWO, part: 3 },
        Box::new([]),
        Box::new([]),
    );
    world.run();
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Down {
        down: host::LinkDown::PartKept { run, attempt, part: 3 }, ..
    } if *run == fixtures::RUN_SEVEN && *attempt == fixtures::ATTEMPT_TWO)));
    world.finish();
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
