//! Pause notices retain the channel's ordering and terms gate (protocol/channel/pause.md, 4).
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
fn paused_states_are_sent_only_where_the_engines_terms_list_the_kind() {
    for has_pause in [true, false] {
        let mut world = World::new(&fixtures::limits(), 1);
        world.script_root();
        let peer =
            if has_pause { fixtures::engine_accepting() } else { fixtures::engine_without_term(wire::Kind::Paused) };
        world.engine(peer);
        world.run();
        assert!(world.root().iter().any(|r| matches!(r, Request::Ready)));
        for state in [PauseState::Taking, PauseState::Paused, PauseState::Draining, PauseState::Drained] {
            world.down(jig_protocol_worker_link::Event::Up { up: host::LinkUp::Paused { state }, parts: Box::new([]) });
            world.run();
        }
        let records = world.records();
        let noticed = records[0].iter().filter(|record| record.kind == wire::Kind::Paused.number()).count();
        assert_eq!(noticed, if has_pause { 4 } else { 0 });
        assert!(!world.root().iter().any(|r| matches!(r, Request::Lost | Request::Untranslated { .. })));
        world.down(jig_protocol_worker_link::Event::Up { up: host::LinkUp::Draining, parts: Box::new([]) });
        world.run();
        assert_eq!(world.records()[0].last().map(|record| record.kind), Some(wire::Kind::Draining.number()));
    }
}
