//! Engine statements and worker ranges (jig/docs/design/protocol/channel/bridges.md, 3 and 9).
use crate::joined_world::Joined;
use jig_protocol_worker_link::{Loss, Request};
use jig_protocol_worker_wire::{self as wire, Spoken};
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, List};

fn singles(version: u16) -> List<Spoken> {
    let mut wires = wire::spoken();
    for index in 0..wires.len() {
        let wire = wires.get_mut(index).expect("wire index");
        let chosen = if wire.name == "jig-worker-wire" { version } else { wire.highest() };
        *wire = Spoken::current(wire.name, chosen, wire.layout(chosen).expect("chosen layout"));
    }
    wires
}

#[test]
fn the_engine_states_thirteen_first_and_the_worker_uses_it() {
    let mut world = Joined::new(&fixtures::joined_limits(), 3);
    world.run();
    assert_eq!(world.engine_records().first(), Some(&0x018d));
    let jig_protocol_engine_workers::Request::Opened { wires, .. } = &world.engine_openings[0] else {
        panic!("opened")
    };
    assert_eq!(
        wires.iter().map(|wire| (wire.name, wire.version)).collect::<Vec<_>>(),
        [
            ("jig-worker-wire", 13),
            ("smith-pieces", 1),
            ("smith-charter", 5),
            ("smith-transcript", 7),
            ("test-worker-wire", 3),
        ]
    );
    assert_eq!(world.engine_root.len(), 1, "the worker sends its hello after the statement");
    world.engine_down(jig_domain_host::LinkDown::Sources { sources: Box::new([]) }, Box::new([]), Box::new([]));
    world.run();
    assert!(
        world
            .worker_root
            .iter()
            .any(|event| matches!(event, Request::Down { down: jig_domain_host::LinkDown::Sources { .. }, .. })),
        "the kept statement permits subsequent records"
    );
    world.finish();
}

#[test]
fn an_engine_without_statements_closes_ranged_workers_and_they_redial_after_the_pause() {
    let mut world = Joined::with_wires(&fixtures::joined_limits(), 3, singles(12), wire::spoken());
    world.run();
    assert!(world.engine_root.is_empty(), "no worker hello at the old engine");
    assert!(world.worker_root.iter().any(|event| matches!(event, Request::Fact { why: Loss::EngineStatesNoWires })));
    assert!(!world.worker_root.iter().any(|event| matches!(event, Request::Ready)));
    assert_eq!(world.dials.len(), 1);
    world.advance(Duration::from_millis(1));
    assert_eq!(world.dials.len(), 1, "the refusal's pause is not skipped");
    world.advance(Duration::from_millis(1600));
    assert!(world.dials.len() >= 2, "the worker redials");
    world.finish();
}

#[test]
fn a_single_version_worker_needs_no_statement_and_accepts_one_too() {
    for version in [12, 13] {
        let mut world = Joined::with_wires(&fixtures::joined_limits(), 3, wire::spoken(), singles(version));
        world.run();
        assert_eq!(world.engine_root.len(), 1, "hello at version {version}");
        assert_eq!(world.engine_records().contains(&0x018d), version == 13);
        world.finish();
    }
}

#[test]
fn repeated_unknown_and_out_of_range_statements_fault_the_link_and_redial() {
    for fault in ["repeated", "unknown", "range"] {
        let mut world = World::new(&fixtures::limits(), 1);
        let mut engine =
            if fault == "repeated" { fixtures::engine_accepting() } else { fixtures::engine_without_statement() };
        if fault == "repeated" {
            engine.play(fixtures::agreement());
        } else {
            let mut wires = List::with_capacity(1);
            let name = if fault == "unknown" { b"unknown-wire".as_slice() } else { b"jig-worker-wire" };
            wires
                .push(
                    wire::AgreedWire::new(
                        &fixtures::codec(),
                        wire::AgreedWireParts { name: name.into(), version: if fault == "unknown" { 3 } else { 14 } },
                    )
                    .expect("wire"),
                )
                .expect("wire slot");
            engine.play(fixtures::agreement_frame(wires));
        }
        world.engine(engine);
        world.run();
        assert!(world.facts().contains(&Loss::Fault), "{fault}");
        world.engine(fixtures::engine_accepting());
        world.advance(Duration::from_millis(1600));
        assert_eq!(world.dials().len(), 2, "{fault}");
        world.down(jig_protocol_worker_link::Event::Stop);
    }
}
