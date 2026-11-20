//! Pause notices retain the channel's ordering and version gate (protocol/channel/pause.md, 4).
use jig_domain_host as host;
use jig_domain_link::{PauseMode, PauseState};
use jig_protocol_worker_link::Request;
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};

#[test]
fn every_pause_mode_reaches_the_owner_between_the_records_around_it() {
    let mut world = World::new(&fixtures::limits(), 1);
    world.script_root();
    world.engine(fixtures::engine_accepting());
    world.run();
    let mut expected = Vec::new();
    for (mode, value) in [
        (wire::PauseMode::Resume, PauseMode::Resume),
        (wire::PauseMode::Pause, PauseMode::Pause),
        (wire::PauseMode::Drain, PauseMode::Drain),
    ] {
        world.play(super::restarting::sources());
        world.play(fixtures::frame(wire::Kind::Pause.number(), &fixtures::encoded(&wire::Pause { mode })));
        world.play(super::restarting::sources());
        expected.push(host::LinkDown::Sources { sources: Box::new([]) });
        expected.push(host::LinkDown::Pause { mode: value });
        expected.push(host::LinkDown::Sources { sources: Box::new([]) });
    }
    let got: Vec<_> = world
        .root()
        .iter()
        .filter_map(|request| match request {
            Request::Down { down, parts } => {
                assert!(parts.is_empty());
                Some(down)
            }
            _ => None,
        })
        .collect();
    assert_eq!(got, expected.iter().collect::<Vec<_>>());
    assert_eq!(world.facts(), []);
}

#[test]
fn paused_states_are_sent_only_where_the_agreed_version_has_the_kind() {
    for version in [18, 19] {
        let mut world = crate::joined_world::Joined::with_wires(
            &super::translation::limits(),
            1,
            super::translation::capped(version, smith_charter::wire::VERSION),
            wire::spoken(),
        );
        world.run();
        world.acknowledge(false);
        assert_eq!(world.engine_channels().len(), 1, "range-stating worker admits the older engine");
        for state in [PauseState::Taking, PauseState::Paused, PauseState::Draining, PauseState::Drained] {
            let before = world.worker_records().len();
            world.worker_up(host::LinkUp::Paused { state }, Box::new([]));
            world.run();
            if version == 19 {
                assert_eq!(world.worker_records().get(before..), Some([wire::Kind::Paused.number()].as_slice()));
                assert!(matches!(world.engine_root.last(),
                    Some(jig_protocol_engine_workers::Request::Up { up: host::LinkUp::Paused { state: got }, .. }) if *got == state));
            } else {
                assert_eq!(world.worker_records().len(), before);
            }
        }
        assert!(
            !world
                .worker_root
                .iter()
                .any(|request| matches!(request, Request::Untranslated { .. } | Request::Fact { .. } | Request::Lost))
        );
        world.worker_up(host::LinkUp::Draining, Box::new([]));
        world.run();
        assert_eq!(world.worker_records().last(), Some(&wire::Kind::Draining.number()));
        assert!(matches!(
            world.engine_root.last(),
            Some(jig_protocol_engine_workers::Request::Up { up: host::LinkUp::Draining, .. })
        ));
        world.finish();
    }
}
