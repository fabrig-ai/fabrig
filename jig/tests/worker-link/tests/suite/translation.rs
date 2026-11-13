//! Both components, capped below their current values (jig/docs/design/protocol/channel/bridges.md, 9).
use crate::joined_world::Joined;
use jig_domain_host as host;
use jig_protocol_engine_workers as engine;
use jig_protocol_worker_link::Request;
use jig_protocol_worker_wire::{self as wire, Application, Spoken};
use jig_worker_link_world::fixtures;
use skein_lib::{List, Reader, Writer};

pub(super) fn capped(jig: u16, charter: u16) -> List<Spoken> {
    let mut wires = wire::spoken();
    for wire in wires.iter_mut() {
        let version = match wire.name {
            "jig-worker-wire" => jig,
            "smith-charter" => charter,
            _ => wire.highest(),
        };
        *wire = Spoken::current(wire.name, version, wire.layout(version).expect("bounded translation fixture"));
    }
    wires
}
pub(super) fn limits() -> jig_protocol_worker_link::Limits {
    let mut limits = fixtures::joined_limits();
    limits.sizes.charter = 1024;
    limits.sizes.result = 1024;
    limits.sizes.transcript = 1024;
    limits.sizes.turn = 1024;
    limits
}
pub(super) fn golden(text: &str, name: &str) -> Box<[u8]> {
    skein_codegen::generate(&skein_codegen::parse(text).expect("bounded translation fixture"))
        .goldens
        .into_iter()
        .find(|g| g.name == format!("{name}.bin"))
        .expect("bounded translation fixture")
        .bytes
        .into_boxed_slice()
}
pub(super) fn charter(contract: u16) -> Box<[u8]> {
    let bytes = golden(
        smith_charter::patches::kept::smith_charter::TEXTS.last().expect("bounded translation fixture"),
        "record_charter_smallest",
    );
    let mut parts = smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(&bytes))
        .expect("bounded translation fixture")
        .into_parts();
    let mut tools = parts.tools.into_parts();
    tools.contract = contract;
    parts.tools = smith_charter::Tools::new(&smith_charter::CEILINGS, tools).expect("bounded translation fixture");
    let record = smith_charter::Charter::new(&smith_charter::CEILINGS, parts).expect("bounded translation fixture");
    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut writer).expect("bounded translation fixture");
    writer.finish()
}
pub(super) fn world(version: u16, charter: u16, seed: u64) -> Joined {
    let mut world = Joined::with_wires(&limits(), seed, wire::spoken(), capped(version, charter));
    world.run();
    world.acknowledge(false);
    world
}

#[test]
fn capped_components_translate_smith_payloads_and_refuse_an_unheld_charter_without_a_frame() {
    let mut world = world(12, 4, 7);
    let mut assignment = fixtures::assignment();
    assignment.assignment.charter = charter(1);
    assignment.turns = Box::new([]);
    let expected = assignment.assignment.charter.clone();
    world.engine_down(
        host::LinkDown::Assign { assignment },
        fixtures::assignment_parts(),
        crate::joined_world::values(),
    );
    world.run();
    let Request::Down { down: host::LinkDown::Assign { assignment }, .. } =
        world.worker_root.last().expect("bounded translation fixture")
    else {
        panic!("assignment")
    };
    assert_eq!(assignment.assignment.charter, expected);
    assert_eq!(&expected[..2], &[0, 5]);
    assert!(!world.engine_records().contains(&wire::Kind::Wires.number()));
    let outcome = golden(
        smith_charter::patches::kept::smith_charter::TEXTS.last().expect("bounded translation fixture"),
        "record_run_result_smallest",
    );
    let mut answer = fixtures::answer();
    let host::LinkUp::Answer {
        answer: host::RunAnswer { ending: host::Ending::Ended { outcome: bytes, .. }, .. }, ..
    } = &mut answer
    else {
        panic!("answer")
    };
    *bytes = outcome.clone();
    world.worker_up(answer, fixtures::work_parts());
    world.run();
    let engine::Request::Up { up: host::LinkUp::Answer { answer, .. }, .. } =
        world.engine_root.last().expect("bounded translation fixture")
    else {
        panic!("answer")
    };
    let host::Ending::Ended { outcome: bytes, .. } = &answer.ending else { panic!("ended") };
    assert_eq!(*bytes, outcome);
    let before = world.assignment_bytes().len();
    let down_before = world.worker_root.len();
    let mut assignment = fixtures::assignment();
    assignment.assignment.charter = charter(2);
    assignment.turns = Box::new([]);
    world.engine_down(
        host::LinkDown::Assign { assignment },
        fixtures::assignment_parts(),
        crate::joined_world::values(),
    );
    world.run();
    assert_eq!(world.assignment_bytes().len(), before, "no untranslatable assignment frame");
    assert_eq!(world.worker_root.len(), down_before);
    let engine::Request::Up { up: host::LinkUp::Answer { answer, .. }, .. } =
        world.engine_root.last().expect("bounded translation fixture")
    else {
        panic!("refusal")
    };
    let host::Ending::Refused(host::Refusal::Untranslated(mismatch)) = &answer.ending else { panic!("untranslated") };
    assert_eq!(mismatch.reason().as_ref(), b"version mismatch: smith-charter 5, the worker reads 4..4");
    world.finish();
}

const APP: [&str; 2] =
    ["family test-worker-wire 1\nrecord Value { n: u8 }", "family test-worker-wire 2\nrecord Value { n: u16 }"];
fn no_sites(
    site: skein_codec::patch::Site,
    _: &skein_codec::patch::Old,
    _: &mut skein_codec::patch::Build,
    _: skein_codec::value::Node,
) -> Result<(), skein_codec::patch::Halt> {
    Err(skein_codec::patch::Halt::NoSite(site))
}
fn application(single: bool) -> Application {
    let mut spoken = List::with_capacity(1);
    let wire = Spoken::of("test-worker-wire", &APP, 1).expect("bounded translation fixture");
    spoken
        .push(if single {
            Spoken::current(wire.name, 1, wire.layout(1).expect("bounded translation fixture"))
        } else {
            wire
        })
        .expect("bounded translation fixture");
    let mut bridges = List::with_capacity(1);
    bridges
        .push(
            skein_codec::bridge::Bridge::new(
                skein_codec::patch::Capacity { versions: 2, patches: 2, text: 4096, bytes: 128, nodes: 32 },
                &APP,
                &["patch 1\nowner application 1\nfamily test-worker-wire 1 to 2\nwiden Value.n u8 u16\n"],
                1,
                no_sites,
            )
            .expect("bounded translation fixture"),
        )
        .expect("bounded translation fixture");
    let mut roots = List::with_capacity(1);
    roots
        .push(wire::Root { carried: wire::Carried::Left, wire: "test-worker-wire", declaration: "Value" })
        .expect("bounded translation fixture");
    Application { spoken, bridges, roots }
}
#[test]
fn a_newer_worker_checks_total_sends_after_the_statement_but_a_capped_worker_does_not() {
    for cap in [false, true] {
        let worker_wires = if cap { capped(13, 5) } else { wire::spoken() };
        let mut world =
            Joined::with_applications(&limits(), 3, capped(13, 5), worker_wires, application(true), application(cap));
        world.run();
        if cap {
            assert!(world.worker_root.iter().any(|e| matches!(e, Request::Ready)));
        } else {
            assert!(
                world
                    .worker_root
                    .iter()
                    .any(|e| matches!(e, Request::CannotSendWhole { wire: "test-worker-wire", version: 1 }))
            );
            assert!(!world.worker_root.iter().any(|e| matches!(e, Request::Ready)));
            assert!(world.engine_root.is_empty());
            world.advance(skein_lib::Duration::from_millis(1600));
            assert!(world.dials.len() >= 2);
        }
        world.finish();
    }
}

fn rooted(single: bool) -> Application {
    let mut application = application(single);
    application.roots = List::with_capacity(6);
    for carried in [
        wire::Carried::Items,
        wire::Carried::Left,
        wire::Carried::Saved,
        wire::Carried::Preparation,
        wire::Carried::JobSpec,
        wire::Carried::Grant,
    ] {
        application
            .roots
            .push(wire::Root { carried, wire: "test-worker-wire", declaration: "Value" })
            .expect("six roots");
    }
    application
}
fn rooted_world() -> Joined {
    let mut world = Joined::with_applications(&limits(), 9, wire::spoken(), capped(12, 5), rooted(false), rooted(true));
    world.run();
    world.acknowledge(false);
    world
}
const APP_VALUE: &[u8] = &[0, 42];
fn rooted_values() -> Box<[Option<engine::GrantValue>]> {
    Box::new([
        Some(crate::joined_world::credential(engine::Scheme::Bearer)),
        Some(engine::GrantValue::Application { secret: skein_lib::Secret::new(APP_VALUE.into()) }),
    ])
}
fn rooted_parts() -> Box<[wire::Part]> {
    Box::new([wire::Part { token: skein_lib::Token::new(10), bytes: APP_VALUE.into() }])
}

#[test]
fn application_parts_and_grant_renewals_cross_components_and_unheld_jobs_send_nothing() {
    let mut world = rooted_world();
    world.engine_down(host::LinkDown::Assign { assignment: fixtures::assignment() }, rooted_parts(), rooted_values());
    world.run();
    let Request::Down { down: host::LinkDown::Assign { assignment }, parts } =
        world.worker_root.last().expect("assignment")
    else {
        panic!("assignment")
    };
    assert_eq!(parts[0].bytes.as_ref(), APP_VALUE);
    assert_eq!(assignment.assignment.grants.as_ref(), fixtures::assignment().assignment.grants.as_ref());
    let Request::Value {
        account,
        generation,
        expires,
        value: jig_protocol_worker_link::GrantValue::Application { secret },
    } = world.values.last().expect("value")
    else {
        panic!("application grant")
    };
    assert_eq!(secret.expose(), APP_VALUE);
    assert_eq!(
        (*account, *generation, *expires),
        (1, 3, skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(60)))
    );
    let grant = fixtures::grant(jig_domain_link::Form::Application);
    world.engine_down(
        host::LinkDown::Grant { run: fixtures::RUN_SEVEN, attempt: fixtures::ATTEMPT_TWO, grant },
        Box::new([]),
        Box::new([Some(engine::GrantValue::Application { secret: skein_lib::Secret::new(APP_VALUE.into()) })]),
    );
    world.run();
    let Request::Value { value: jig_protocol_worker_link::GrantValue::Application { secret }, .. } =
        world.values.last().expect("renewal")
    else {
        panic!("grant")
    };
    assert_eq!(secret.expose(), APP_VALUE);
    let mut answer = fixtures::answer();
    let host::LinkUp::Answer { answer: host::RunAnswer { ending: host::Ending::Ended { work, .. }, .. }, .. } =
        &mut answer
    else {
        panic!("answer")
    };
    work.saved = None;
    world.worker_up(answer, rooted_parts());
    world.run();
    let engine::Request::Up { parts, .. } = world.engine_root.last().expect("answer") else { panic!("answer") };
    assert_eq!(parts[0].bytes.as_ref(), APP_VALUE);
    let job = || host::JobAssignment {
        run: fixtures::RUN_SEVEN,
        attempt: fixtures::ATTEMPT_TWO,
        kind: jig_domain_link::JobKind(1),
        spec: APP_VALUE.into(),
        deadline: skein_lib::Duration::from_secs(1),
        workspace: None,
        grants: Box::new([]),
    };
    world.engine_down(host::LinkDown::AssignJob { job: job() }, Box::new([]), Box::new([]));
    world.run();
    let Request::Down { down: host::LinkDown::AssignJob { job: received }, .. } =
        world.worker_root.last().expect("job")
    else {
        panic!("job")
    };
    assert_eq!(received.spec.as_ref(), APP_VALUE);
    let frames = world.engine_records().len();
    let mut invalid = job();
    invalid.spec = Box::from([1, 0]);
    world.engine_down(host::LinkDown::AssignJob { job: invalid }, Box::new([]), Box::new([]));
    world.run();
    assert_eq!(world.engine_records().len(), frames);
    let engine::Request::Up {
        up: host::LinkUp::JobAnswer { answer: host::JobEnding::Refused(host::JobRefusal::Untranslated(mismatch)), .. },
        ..
    } = world.engine_root.last().expect("refusal")
    else {
        panic!("untranslated job")
    };
    assert_eq!(mismatch.reason().as_ref(), b"version mismatch: test-worker-wire 2, the worker reads 1..1");
    let count = world.engine_root.len();
    world.worker_up(
        host::LinkUp::JobAnswer {
            run: fixtures::RUN_SEVEN,
            attempt: fixtures::ATTEMPT_TWO,
            answer: host::JobEnding::Finished {
                result: b"opaque".as_slice().into(),
                work: host::Work { left: Some(skein_lib::Token::new(10)), saved: None },
            },
        },
        Box::new([wire::Part { token: skein_lib::Token::new(10), bytes: Box::from([1, 0]) }]),
    );
    world.run();
    assert_eq!(world.engine_root.len(), count, "a capped worker discards an untranslatable send");
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Untranslated { kind: 0x010c, .. })));
    world.finish();
}
