//! Each derived carrier meets the existing start check (configuration/derived.md, 6).
use super::fixtures;
use crate::{
    limits,
    startup::{Problems, checks, lines},
};
use jig_protocol_config as config;
use temper_service_core::{
    agent::Declaration,
    derived::{self, STOCK, Sources},
};

fn case(scale: u32) -> (Sources, Declaration, config::Limits, limits::Limits) {
    let sources = Sources {
        turn: STOCK.turn.checked_mul(scale).unwrap(),
        payload: STOCK.payload.checked_mul(scale).unwrap(),
        ..STOCK
    };
    let mut agent = fixtures::agent().declaration();
    agent.turn_body = sources.turn;
    agent.tool_payload = sources.payload;
    agent.outcome_bytes = u64::from(sources.payload);
    agent.bounds.message_bytes = sources.payload.checked_sub(2).unwrap();
    agent.bounds.inbox = sources.inbox;
    agent.bounds.unacknowledged = sources.unacknowledged;
    let mut jig = config::stock();
    if sources.turn > jig_protocol_worker_wire::CEILINGS.turn_body {
        let found = derived::engine(&mut jig, sources).unwrap_err();
        assert_eq!(found.path, config::Path::root().member(b"limits").member(b"overrides").member(b"io.output"));
    } else {
        derived::engine(&mut jig, sources).unwrap();
    }
    let mut own = limits::stock();
    limits::derive(&mut own, sources).unwrap();
    (sources, agent, jig, own)
}
fn turns(jig: &config::Limits, own: &limits::Limits, agent: &Declaration) -> Box<[u8]> {
    let mut problems = Problems::empty();
    checks::turns(Some(jig), &[], own, Some(agent), &mut problems);
    lines(&problems)
}
#[expect(clippy::disallowed_methods, reason = "test diagnostics decode startup's UTF-8 text")]
fn names(found: &[u8], figure: &[u8], source: &[u8]) {
    for text in [figure, source] {
        let mut present = false;
        for window in found.windows(text.len()) {
            present |= window == text;
        }
        assert!(present, "{}", core::str::from_utf8(found).unwrap());
    }
}
#[test]
fn stock_engine_link_cap_holds_the_fleet_ceiling() {
    let (_, _, jig, own) = case(1);
    let mut settings = fixtures::engine();
    settings.jig.limits = jig;
    settings.limits = own;
    let limits = crate::workers::limits(&settings).expect("stock engine link");
    let sizes = limits.sizes;
    let cap = jig_protocol_worker_wire::sizes::engine_cap(&sizes, limits.room).unwrap();
    assert!(cap.bytes >= limits.room, "room for a fitted assignment and its notices");
}

#[test]
fn turn_carrier() {
    for scale in [1, 2] {
        let (s, agent, mut jig, own) = case(scale);
        assert_eq!(jig.hub.turn_bytes, u64::from(s.turn));
        assert!(turns(&jig, &own, &agent).is_empty());
        jig.hub.turn_bytes -= 1;
        names(&turns(&jig, &own, &agent), b"hub.turn_bytes", b"the agent's largest turn");
    }
}
#[test]
fn resume_carrier() {
    for scale in [1, 2] {
        let (s, agent, mut jig, own) = case(scale);
        assert_eq!(jig.core.resume_bytes, (s.turn + 4).max(4_194_304));
        assert!(turns(&jig, &own, &agent).is_empty());
        jig.core.resume_bytes = s.turn + 3;
        names(&turns(&jig, &own, &agent), b"core.resume_bytes", b"the agent's largest turn with its length");
    }
}
#[test]
fn transcript_carrier() {
    for scale in [1, 2] {
        let (s, agent, mut jig, own) = case(scale);
        let resume = u64::from(s.turn + 4);
        assert_eq!(jig.hub.transcript_bytes, resume + 12 * (resume / 4));
        assert!(turns(&jig, &own, &agent).is_empty());
        jig.hub.transcript_bytes -= 1;
        names(&turns(&jig, &own, &agent), b"hub.transcript_bytes", b"the engine's resume window");
    }
}
#[test]
fn loaded_transcript_carrier() {
    for scale in [1, 2] {
        let (s, agent, jig, mut own) = case(scale);
        assert_eq!(own.root.loads.transcript_bytes, s.turn);
        assert!(turns(&jig, &own, &agent).is_empty());
        own.root.loads.transcript_bytes -= 1;
        names(&turns(&jig, &own, &agent), b"root.loads.transcript_bytes", b"the agent's largest turn");
    }
}
#[test]
fn loaded_row_carrier() {
    for scale in [1, 2] {
        let (s, agent, jig, mut own) = case(scale);
        assert_eq!(own.root.loads.bytes, s.turn + u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap());
        assert!(turns(&jig, &own, &agent).is_empty());
        own.root.loads.bytes -= 1;
        names(&turns(&jig, &own, &agent), b"root.loads.bytes", b"the agent's largest turn with a loaded row");
    }
}
#[test]
fn loaded_reply_carrier() {
    for scale in [1, 2] {
        let (s, _, _, mut own) = case(scale);
        assert_eq!(
            own.root.loads.reply_bytes,
            s.turn + u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap()
        );
        assert!(temper_engine_domain::loads::valid(&own.root.loads));
        own.root.loads.reply_bytes -= 1;
        assert!(
            !temper_engine_domain::loads::valid(&own.root.loads),
            "root.loads.reply_bytes must cover root.loads.bytes"
        );
    }
}
#[test]
fn outcome_carrier() {
    for scale in [1, 2] {
        let (s, agent, mut jig, own) = case(scale);
        assert_eq!(jig.hub.outcome_bytes, config::stock().hub.outcome_bytes.max(u64::from(s.payload)));
        assert!(turns(&jig, &own, &agent).is_empty());
        jig.hub.outcome_bytes = u64::from(s.payload - 1);
        names(&turns(&jig, &own, &agent), b"hub.outcome_bytes", b"the agent's outcome");
    }
}
#[test]
fn call_answer_carrier() {
    use temper_service_core::startup::CallAnswer;
    for scale in [1, 2] {
        let (s, _, mut jig, _) = case(scale);
        assert_eq!(jig.core.call_answer_bytes, config::stock().core.call_answer_bytes.max(s.payload));
        CallAnswer::Explicit.compose(&mut jig, s.payload, s.payload, s.payload).unwrap();
        jig.core.call_answer_bytes = s.payload - 1;
        let refusal = CallAnswer::Explicit.compose(&mut jig, s.payload, s.payload, s.payload).unwrap_err();
        let mut problems = Problems::empty();
        problems.push(
            temper_service_core::startup::call_input_path(config::Path::root().member(b"engine")),
            refusal.extend(),
        );
        names(&lines(&problems), b"core.call_answer_bytes", b"the agent's call input");
    }
}
#[test]
fn forge_operation_carrier() {
    for scale in [1, 2] {
        let (s, agent, jig, mut own) = case(scale);
        assert_eq!(own.forge.client.op_bytes, s.payload - 2);
        assert!(turns(&jig, &own, &agent).is_empty());
        own.forge.client.op_bytes -= 1;
        names(&turns(&jig, &own, &agent), b"forge.client.op_bytes", b"tasks.result_bytes");
    }
}
#[test]
fn journal_carrier() {
    for scale in [1, 2] {
        let (s, _, _, own) = case(scale);
        assert_eq!(own.root.journal.transcript_bytes, (s.turn + 4).max(4_194_304));
        let mut engine = fixtures::guard_engine();
        engine.jig.limits.core.resume_bytes = own.root.journal.transcript_bytes;
        let mut root = limits::engine(&engine.jig.limits, &engine.limits).unwrap();
        root.journal.transcript_bytes = own.root.journal.transcript_bytes;
        let seeds = fixtures::seeds();
        let mut problems = Problems::empty();
        checks::root(
            config::core(&engine.jig, seeds.deployment, seeds.seed),
            &engine,
            &seeds,
            &[],
            &root,
            &mut problems,
        );
        assert!(problems.is_empty(), "{problems:?}");
        root.journal.transcript_bytes -= 1;
        let mut problems = Problems::empty();
        checks::root(
            config::core(&engine.jig, seeds.deployment, seeds.seed),
            &engine,
            &seeds,
            &[],
            &root,
            &mut problems,
        );
        names(&lines(&problems), b"root.journal.transcript_bytes", b"core.resume_bytes");
    }
}
#[test]
fn message_carrier() {
    for scale in [1, 2] {
        let (s, mut agent, jig, _) = case(scale);
        assert_eq!(jig.core.tasks.message_bytes, s.payload - 2);
        let mut engine = fixtures::guard_engine().jig;
        engine.limits = jig;
        // Isolate receiving admission from the charters' endpoint advice.
        engine.charters = Box::new([]);
        let core = config::core(&engine, fixtures::seeds().deployment, 7);
        let mut problems = Problems::empty();
        checks::admission(&core, &engine, &agent, &mut problems);
        assert!(problems.is_empty(), "{problems:?}");
        agent.bounds.message_bytes -= 1;
        let mut problems = Problems::empty();
        checks::admission(&core, &engine, &agent, &mut problems);
        names(&lines(&problems), b"tasks.message_bytes", b"tool_payload less 2");
    }
}
#[test]
fn result_carrier() {
    for scale in [1, 2] {
        let (s, agent, jig, mut own) = case(scale);
        assert_eq!(jig.core.tasks.result_bytes, s.payload - 2);
        assert!(turns(&jig, &own, &agent).is_empty());
        // Today's check bounds the result by the receiving forge operation.
        own.forge.client.op_bytes = jig.core.tasks.result_bytes - 1;
        names(&turns(&jig, &own, &agent), b"tasks.result_bytes", b"forge.client.op_bytes");
    }
}
#[test]
fn inbox_carrier() {
    for scale in [1, 2] {
        let (s, _, jig, _) = case(scale);
        assert_eq!(jig.core.tasks.inbox_bytes, s.inbox * (s.payload - 2));
        let mut given = jig_domain_core::settings::Override {
            name: b"tasks.inbox_bytes".as_slice().into(),
            value: u64::from(jig.core.tasks.inbox_bytes),
            reason: b"inbox carrier".as_slice().into(),
        };
        config::overrides::with_overrides(&jig, core::slice::from_ref(&given)).unwrap();
        given.value = u64::from(jig.core.tasks.message_bytes - 1);
        let problems = Problems::jig(
            config::overrides::with_overrides(&jig, core::slice::from_ref(&given)).unwrap_err(),
            &config::Path::root(),
        );
        names(&lines(&problems), b"tasks.inbox_bytes", b"tasks.message_bytes");
    }
}
