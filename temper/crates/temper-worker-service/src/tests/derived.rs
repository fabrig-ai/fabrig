//! Worker carriers and places (configuration/derived.md, 5–6).
use super::fixtures;
use crate::{
    settings,
    startup::{Problems, lines, worker},
    worker_settings,
};
use jig_protocol_config as config;
use jig_protocol_worker_wire::Record;
use temper_service_core::derived::{self, STOCK};

fn root(payload: u32, turn: u32) -> temper_worker_domain::Limits {
    let mut root = worker_settings::root_limits(
        &fixtures::worker_settings(),
        &fixtures::startup_agent(),
        &worker_settings::Limits::shared(temper_service_core::limits::stock()),
    );
    root.agent.message_bytes = u64::from(payload);
    root.agent.call_bytes = u64::from(payload);
    root.bounds.message_bytes = payload.checked_sub(2).unwrap();
    root.agent.turns = STOCK.unacknowledged;
    root.bounds.unacknowledged = STOCK.unacknowledged;
    root.host.turns = STOCK.unacknowledged;
    root.host.turn_bytes = u64::from(turn);
    root.host.event_bytes = u64::from(derived::message(payload).unwrap());
    root.host.turn_queue_bytes = derived::turn_queue(turn, STOCK.unacknowledged).unwrap();
    root.checkout.message_bytes = payload;
    root
}
fn checked(root: &temper_worker_domain::Limits) -> Box<[u8]> {
    let mut problems = Problems::empty();
    worker::worker(root, &mut problems);
    lines(&problems)
}
fn slots_input(slots: u32) -> Box<[u8]> {
    fixtures::with(
        &fixtures::worker_alone_document(),
        b"workers[0].slots",
        skein_lib::Decimal::of(u64::from(slots)).as_bytes(),
    )
}
fn names(found: &[u8], figure: &[u8], source: &[u8]) {
    for text in [figure, source] {
        let mut present = false;
        for window in found.windows(text.len()) {
            present |= window == text;
        }
        assert!(present, "{found:?}");
    }
}
#[test]
fn turn_queue_carrier() {
    for turn in [STOCK.turn, STOCK.turn * 2] {
        let mut root = root(STOCK.payload, turn);
        assert_eq!(root.host.turn_queue_bytes, u64::from(STOCK.unacknowledged) * u64::from(turn));
        assert!(checked(&root).is_empty(), "{:?}", checked(&root));
        root.host.turn_queue_bytes -= 1;
        names(
            &checked(&root),
            b"hub.turn_queue_bytes",
            b"the agent's turns * workers[0].limits.overrides.hub.turn_bytes",
        );
    }
}
#[test]
fn event_carrier() {
    for payload in [STOCK.payload, STOCK.payload * 2] {
        let mut root = root(payload, STOCK.turn);
        assert_eq!(root.host.event_bytes, u64::from(payload - 2));
        assert!(checked(&root).is_empty(), "{:?}", checked(&root));
        root.host.event_bytes -= 1;
        names(&checked(&root), b"hub.event_bytes", b"tool_payload less the 2-byte separator");
    }
}
#[test]
fn checkout_message_carrier() {
    for payload in [STOCK.payload, STOCK.payload * 2] {
        let input = br#"{"profile":"standard","overrides":{}}"#;
        let document = config::document(input, &config::DocumentLimits { bytes: 1024, tokens: 64, depth: 8 }).unwrap();
        let mut problems = Problems::empty();
        let own = temper_service_core::startup::configured_limits_over(
            &document,
            0,
            &config::Path::root(),
            derived::Sources { payload, ..STOCK },
            &mut problems,
        )
        .unwrap();
        assert!(problems.is_empty());
        assert_eq!(own.checkout.message_bytes, payload);
        let mut root = root(payload, STOCK.turn);
        root.checkout.message_bytes = own.checkout.message_bytes;
        assert!(checked(&root).is_empty(), "{:?}", checked(&root));
        root.checkout.message_bytes -= 1;
        names(&checked(&root), b"checkout.message_bytes", b"the agent's call_bytes");
    }
}
#[test]
fn workspaces_follow_slots() {
    let stock = config::stock().hub.slots;
    for slots in [stock, stock * 2, 12, 24] {
        let input = slots_input(slots);
        let part = crate::startup::worker_part(&input, &config::Directories::working(), None, &[]).unwrap();
        assert_eq!(part.limits.checkout.workspaces, slots);
        assert_eq!(part.worker.jig.limits.as_ref().unwrap().hub.slots, slots);
        let mut root = root(STOCK.payload, STOCK.turn);
        root.host.slots = slots;
        root.checkout.workspaces = part.limits.checkout.workspaces;
        assert!(checked(&root).is_empty(), "{:?}", checked(&root));
        let mut overridden: serde_json::Value = serde_json::from_slice(&input).unwrap();
        overridden["application"]["limits"] = serde_json::json!({"overrides":{"checkout.workspaces":slots-1}});
        let found = crate::startup::worker_part(
            &serde_json::to_vec(&overridden).unwrap(),
            &config::Directories::working(),
            None,
            &[],
        )
        .unwrap_err();
        assert_eq!(
            found.problems.get(0).unwrap().what,
            crate::startup::What::Jig(config::What::Because(worker::MOVED_LIMITS))
        );
    }
}
#[test]
fn git_operations_follow_workspaces() {
    let stock = config::stock().hub.slots;
    for slots in [stock, stock * 2, 12, 24] {
        let input = slots_input(slots);
        let part = crate::startup::worker_part(&input, &config::Directories::working(), None, &[]).unwrap();
        assert_eq!(part.limits.git_component.operations, slots);
        let mut problems = Problems::empty();
        worker::git_limits(&part.limits, &part.limits.git_component, &mut problems);
        assert!(problems.is_empty());
        let mut overridden: serde_json::Value = serde_json::from_slice(&input).unwrap();
        overridden["application"]["limits"] = serde_json::json!({"overrides":{"git_component.operations":slots-1}});
        let problems = crate::startup::worker_part(
            &serde_json::to_vec(&overridden).unwrap(),
            &config::Directories::working(),
            None,
            &[],
        )
        .unwrap_err();
        names(&lines(&problems), b"application.limits", worker::MOVED_LIMITS);
    }
}
#[test]
fn output_carrier() {
    use jig_protocol_worker_wire as wire;
    let mut worker = fixtures::worker_settings();
    worker.jig.limits = Some(config::stock());
    let mut sources = STOCK;
    let zero = wire::v21::Limits {
        turn_body: 0,
        turn_refused: worker.jig.limits.as_ref().expect("worker limits").core.tasks.run_inbox,
        ..wire::CEILINGS
    };
    let overhead = u32::try_from(wire::Turn::encoded_bound(&zero).unwrap()).unwrap() + 8;
    assert_eq!(overhead + 2 * STOCK.turn, 36_864_476);
    for turn in [STOCK.turn, wire::CEILINGS.assignment_transcript / 4 - 4] {
        sources.turn = turn;
        derived::worker(worker.jig.limits.as_mut().expect("worker limits"), sources).unwrap();
        assert_eq!(
            worker.jig.limits.as_ref().expect("worker limits").io.output,
            (overhead + turn).max(derived::OUTPUT_FLOOR)
        );
        let own = fixtures::worker_limits();
        let payload = worker_settings::payload_limits(&own).unwrap();
        let mut link = worker_settings::link_limits(&worker.jig, &payload, own.peer.forges).unwrap();
        let schema = wire::schema(&wire::sizes::limits(&link.sizes).unwrap()).unwrap();
        assert!(jig_protocol_worker_link::valid_with_schema(&link, &schema));
        let mut engine = jig_protocol_engine_workers::Limits {
            sizes: link.sizes,
            names: worker.jig.limits.as_ref().expect("worker limits").link.names,
            workers: 1,
            opening: 1,
            chunk: link.chunk,
            room: link.room,
            handshake: link.handshake,
            hello: link.handshake,
            ping: link.ping,
            silence: link.silence,
            stall: link.stall,
        };
        assert!(jig_protocol_engine_workers::limits::valid_with_schema(&engine, &schema));
        link.room = overhead + turn - 1;
        assert!(!jig_protocol_worker_link::valid_with_schema(&link, &schema));
        let mut least = wire::sizes::smallest_resumed(
            engine.sizes.answered,
            engine.sizes.call_name,
            engine.sizes.tool_name,
            engine.sizes.answer,
        )
        .unwrap()
        .max(270);
        for rule in wire::KINDS {
            if rule.direction == skein_channel::Direction::FromResponder
                && rule.body != wire::Kind::Assignment
                && rule.body != wire::Kind::Job
                && rule.body != wire::Kind::Figures
            {
                least = least.max(wire::sizes::largest_body(&engine.sizes, rule.body).unwrap() + 8);
            }
        }
        engine.room = least;
        assert!(jig_protocol_engine_workers::limits::valid_with_schema(&engine, &schema));
        engine.room -= 1;
        assert!(
            !jig_protocol_engine_workers::limits::valid_with_schema(&engine, &schema),
            "io.output must hold the engine's largest responder frame and smallest resumed start"
        );
        let mut problems = Problems::empty();
        worker::checkout::capacity(None, core::slice::from_ref(&worker.jig), &own, None, &mut problems);
        assert!(problems.is_empty(), "{problems:?}");
    }
    sources.turn = STOCK.turn * 2;
    let found = derived::worker(worker.jig.limits.as_mut().expect("worker limits"), sources).unwrap_err();
    assert_eq!(found.path, config::Path::root().member(b"limits").member(b"overrides").member(b"io.output"));
    assert_eq!(
        found.what,
        config::What::Because(b"cannot derive from the agent's largest turn (T) within the worker wire's ceilings")
    );
}
#[test]
fn worker_start_refuses_a_turn_past_the_wire_ceiling() {
    let input = fixtures::worker_alone_document();
    let mut part = crate::startup::worker_part(&input, &config::Directories::working(), None, &[]).unwrap();
    let mut agent: settings::Agent = fixtures::startup_agent();
    agent.limits.bodies.turn_body = STOCK.turn * 2;
    let found = worker::check_worker_part(&mut part, &fixtures::files(), &agent).unwrap_err();
    names(&lines(&found), b"workers[0].limits.overrides.io.output", b"the agent's largest turn (T)");
}
