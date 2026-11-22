//! The shared component bridge machine (jig/docs/design/protocol/channel/bridges.md, 5 and 9).
use crate::translation::Translation;
use crate::v21::Record;
use crate::{self as channel, Agreed, Application, Carried, Kept, Root, Spoken, v21 as wire};
use skein_channel::{Frame, frame_writer};
use skein_codec::{
    bridge::Bridge,
    patch::{Build, Capacity, Halt, Old, Site},
    value::Node,
};
use skein_lib::{List, Reader, Writer};

const APP: [&str; 2] = [
    "family application 1\nversioned record Value { n: u8 }",
    "family application 2\nversioned record Value { n: u16 }",
];
const VALUE: &[u8] = &[0, 2, 0, 42];
const OLDER: &[u8] = &[0, 1, 42];
fn no_sites(site: Site, _: &Old, _: &mut Build, _: Node) -> Result<(), Halt> {
    Err(Halt::NoSite(site))
}
fn application() -> Application {
    let mut kept = List::with_capacity(1);
    kept.push(Kept {
        family: "application",
        texts: &APP,
        patches: &["patch 1\nowner application 1\nfamily application 1 to 2\nwiden Value.n u8 u16\n"],
        floor: 1,
        sites: no_sites,
        names: &[],
        values: List::uncapped(),
    })
    .unwrap();
    let mut roots = List::with_capacity(16);
    for carried in [
        Carried::Items,
        Carried::Left,
        Carried::Saved,
        Carried::Preparation,
        Carried::JobSpec,
        Carried::Grant,
        Carried::Ask,
        Carried::Answer,
    ] {
        roots.push(Root { carried, wire: "application", declaration: "Value" }).unwrap();
    }
    let mut spoken = List::with_capacity(1);
    spoken.push(Spoken::of("application", &APP, 1).unwrap()).unwrap();
    Application { spoken, kept, roots }
}
fn opaque() -> Translation {
    Translation::new(Application::opaque(List::with_capacity(16))).unwrap()
}
fn agreed(jig: u16, charter: u16) -> [Agreed; 4] {
    [
        Agreed { name: "jig-worker-wire", version: jig },
        Agreed { name: "smith-charter", version: charter },
        Agreed { name: "smith-transcript", version: 7 },
        Agreed { name: "application", version: 1 },
    ]
}
fn frame(kind: u16, body: &[u8]) -> Frame {
    let mut frame = frame_writer(kind, u32::try_from(body.len()).unwrap()).unwrap();
    frame.put(body).unwrap();
    frame.finish().unwrap()
}
fn body(frame: &Frame) -> &[u8] {
    &frame.bytes()[8..]
}
fn engine(record: &wire::FromEngine) -> Frame {
    let mut writer =
        Writer::new(skein_lib::length::usize_of(record.measure(&crate::CEILINGS).expect("record within limits")));
    record.write(&crate::CEILINGS, &mut writer).unwrap();
    let bytes = writer.finish();
    frame(0x180 + u16::from(bytes[0]), &bytes[1..])
}
fn worker(record: &wire::FromWorker) -> Frame {
    let mut writer =
        Writer::new(skein_lib::length::usize_of(record.measure(&crate::CEILINGS).expect("record within limits")));
    record.write(&crate::CEILINGS, &mut writer).unwrap();
    let bytes = writer.finish();
    frame(0x100 + u16::from(bytes[0]), &bytes[1..])
}
fn engine_value(frame: &Frame) -> wire::FromEngine {
    let mut writer = Writer::new(body(frame).len().checked_add(1).unwrap());
    writer.put(&[u8::try_from(frame.kind().checked_sub(0x180).unwrap()).unwrap()]).unwrap();
    writer.put(body(frame)).unwrap();
    let bytes = writer.finish();
    wire::FromEngine::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).unwrap()
}
fn worker_value(frame: &Frame) -> wire::FromWorker {
    let mut writer = Writer::new(body(frame).len().checked_add(1).unwrap());
    writer.put(&[u8::try_from(frame.kind().checked_sub(0x100).unwrap()).unwrap()]).unwrap();
    writer.put(body(frame)).unwrap();
    let bytes = writer.finish();
    wire::FromWorker::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).unwrap()
}
fn fixed_value(text: &'static str, name: &str) -> Box<[u8]> {
    fixture_fixed_value(&skein_codegen::values(text).fixed, name)
}
fn fixture_fixed_value(fixed_values: &[skein_codegen::FixedValue], name: &str) -> Box<[u8]> {
    for fixed_value in fixed_values {
        if fixed_value.name.strip_suffix(".bin") == Some(name) {
            return fixed_value.bytes.clone().into_boxed_slice();
        }
    }
    panic!("missing fixed_value {name}")
}
#[test]
fn a_hello_before_twenty_one_drops_freshness_and_reads_back_not_fresh() {
    for version in [18, 19, 20] {
        let mut hello = super::fixtures::hello_smallest();
        hello.fresh = true;
        let current = worker(&wire::FromWorker::Hello(hello));
        let current_len = current.bytes().len();
        let mut bridge = opaque();
        let old = bridge
            .send(current, version, &agreed(version, smith_charter::wire::VERSION), &wire::CEILINGS)
            .expect("hello goes down to an older worker wire");
        assert_eq!(old.bytes().len().checked_add(1), Some(current_len), "fresh is dropped");
        let restored = bridge
            .receive(old.kind(), body(&old), version, &agreed(version, smith_charter::wire::VERSION), &wire::CEILINGS)
            .expect("an older hello goes up");
        let wire::FromWorker::Hello(hello) = worker_value(&restored) else { panic!("hello") };
        assert!(!hello.fresh, "an older channel cannot prove a new process");
    }
}

fn charter(contract: u16) -> Box<[u8]> {
    let bytes =
        fixed_value(smith_charter::patches::kept::smith_charter::TEXTS.last().unwrap(), "record_charter_smallest");
    let mut parts = smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    parts.tools.contract = contract;
    let record = parts;
    record.encode(&smith_charter::CEILINGS).expect("bounded charter")
}
fn assignment(charter: Box<[u8]>) -> wire::Assignment {
    let mut parts = super::fixtures::assignment_resumed();
    parts.charter = charter;
    parts.transcript = Box::new([]);
    parts.workspace = None;
    parts.grants = List::with_capacity(16);
    parts
}

fn failed_answer(failure: wire::RunFailure) -> wire::FromWorker {
    let mut answer = super::fixtures::answer_failed();
    let wire::Ending::Failed(failed) = answer.ending else { panic!("failed answer") };
    let mut failed = failed;
    failed.failure = wire::Failure::Run(wire::RunFailureValue { value: failure });
    answer.ending = wire::Ending::Failed(failed);
    wire::FromWorker::Answer(answer)
}

#[test]
fn pause_notices_are_unheld_below_twenty() {
    let mut translation = opaque();
    for version in crate::wire::FLOOR..20 {
        for record in [
            wire::FromEngine::Pause(wire::Pause { mode: wire::PauseMode::Resume }),
            wire::FromEngine::Pause(wire::Pause { mode: wire::PauseMode::Pause }),
            wire::FromEngine::Pause(wire::Pause { mode: wire::PauseMode::Drain }),
        ] {
            let refusal = translation
                .send(engine(&record), version, &agreed(version, 5), &wire::CEILINGS)
                .expect_err("pause not held");
            let Halt::Unheld(_) = refusal.halt else { panic!("new notice is unheld: {refusal:?}") };
        }
        for state in
            [wire::PauseState::Taking, wire::PauseState::Paused, wire::PauseState::Draining, wire::PauseState::Drained]
        {
            let refusal = translation
                .send(
                    worker(&wire::FromWorker::Paused(wire::Paused { state })),
                    version,
                    &agreed(version, 5),
                    &wire::CEILINGS,
                )
                .expect_err("paused not held");
            let Halt::Unheld(_) = refusal.halt else { panic!("new notice is unheld: {refusal:?}") };
        }
    }
}

#[test]
fn an_untooled_answer_is_unheld_below_seventeen() {
    let record = failed_answer(wire::RunFailure::Untooled);
    let refusal = opaque().send(worker(&record), 16, &agreed(16, 5), &wire::CEILINGS).unwrap_err();
    assert_eq!(refusal.wire.as_ref(), b"jig-worker-wire");
    assert_eq!(refusal.version, crate::VERSION);
    let Halt::Unheld(_) = refusal.halt else { panic!("untooled is unheld: {refusal:?}") };
}

#[test]
fn older_run_failures_round_trip_through_sixteen_without_changing() {
    let mut sender = opaque();
    let mut receiver = opaque();
    for failure in [
        wire::RunFailure::Model,
        wire::RunFailure::Budget,
        wire::RunFailure::Policy,
        wire::RunFailure::Cancelled,
        wire::RunFailure::Stale,
        wire::RunFailure::Limited(wire::LimitedAccount { account: 7, reset: Some(skein_lib::Duration::from_secs(9)) }),
        wire::RunFailure::Refused(wire::RefusedAccount { account: 7 }),
        wire::RunFailure::Exceeded(wire::Exceeded { bound: wire::Bound::Window, value: 8192 }),
        wire::RunFailure::Transcript,
    ] {
        let record = failed_answer(failure);
        let down = sender.send(worker(&record), 16, &agreed(16, 5), &wire::CEILINGS).unwrap();
        let up = receiver.receive(down.kind(), body(&down), 16, &agreed(16, 5), &wire::CEILINGS).unwrap();
        assert_eq!(worker_value(&up), record);
    }
}

#[test]
fn a_worker_of_twenty_one_sends_whole_at_nineteen_and_twenty_but_not_below() {
    for version in [16, 17, 18] {
        let agreed = [Agreed { name: "jig-worker-wire", version }];
        assert_eq!(opaque().incomplete(&agreed).as_slice(), &agreed);
    }
    for version in [19, 20] {
        assert!(opaque().incomplete(&[Agreed { name: "jig-worker-wire", version }]).is_empty());
    }
}

fn model_answer(refusal: Option<Box<[u8]>>) -> wire::FromWorker {
    let mut answer = super::fixtures::answer_failed();
    let mut failed = match answer.ending {
        wire::Ending::Failed(failed) => failed,
        wire::Ending::Refused(_) | wire::Ending::Ended(_) | wire::Ending::Parked(_) => unreachable!("failed fixture"),
    };
    let model = wire::ModelFailure {
        endpoint: b"anthropic-api".as_slice().into(),
        model: b"claude-opus-5-5".as_slice().into(),
        kind: wire::ModelFailureKind::Invalid,
        evidence: wire::ModelEvidence::Unsent,
        status: None,
        refusal,
    };
    model.measure(&wire::CEILINGS).expect("bounded refusal");
    failed.model = Some(model);
    failed.measure(&wire::CEILINGS).expect("failed fixture");
    answer.ending = wire::Ending::Failed(failed);
    answer.measure(&wire::CEILINGS).expect("answer fixture");
    wire::FromWorker::Answer(answer)
}

#[test]
fn model_refusals_round_trip_at_eighteen_and_are_unheld_below_it() {
    use skein_codec::{
        schema::{FieldIx, Shape, parse},
        value::{Part, Path},
    };
    // The refusal is the reverse from 18: its path names that plan's source layout.
    let mut source = None;
    for text in channel::patches::kept::jig_worker_wire::TEXTS {
        if text.starts_with("family jig-worker-wire 18\n") {
            source = Some(text);
        }
    }
    let schema = parse(source.expect("wire 18").as_bytes()).expect("schema");
    let decl = schema.find(b"ModelFailure").expect("model failure record");
    let first = match schema.declaration(decl).shape {
        Shape::Record { first, .. } => first,
        Shape::Enum { .. } => unreachable!("model failure is a record"),
    };
    let field = FieldIx::new(first.index().checked_add(5).expect("refusal's field"));
    assert_eq!(schema.spelled(schema.field(field).name), b"refusal");
    for refusal in [Some(b"empty-message".as_slice().into()), None] {
        let record = model_answer(refusal.clone());
        for version in [16, 17, 18, 19] {
            let sent = opaque().send(worker(&record), version, &agreed(version, 5), &wire::CEILINGS);
            if version < 18 && refusal.is_some() {
                let refused = sent.expect_err("an older engine cannot hold the refusal");
                assert_eq!(refused.wire.as_ref(), b"jig-worker-wire");
                assert_eq!(refused.version, channel::VERSION);
                assert_eq!(refused.halt, Halt::Unheld(Path { decl, part: Part::Field(field) }));
            } else {
                let down = sent.expect("the engine holds this failure");
                let up = opaque()
                    .receive(down.kind(), body(&down), version, &agreed(version, 5), &wire::CEILINGS)
                    .expect("failure comes back up");
                assert_eq!(worker_value(&up), record, "version {version}");
            }
        }
    }
}

#[test]
fn enumeration_bridges_refuse_versions_below_the_breaking_floor() {
    let record = wire::FromEngine::Cancel(wire::Cancel { task: 7, attempt: 2 });
    let mut sender = opaque();
    sender.send(engine(&record), 14, &agreed(14, 5), &wire::CEILINGS).expect_err("below breaking floor");
    assert_eq!(
        engine_value(
            &sender.send(engine(&record), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS).unwrap()
        ),
        record
    );
}

#[test]
fn smith_charters_results_and_old_window_turns_take_their_own_bridges() {
    let mut engine_side = opaque();
    let mut worker_side = opaque();
    let current = charter(1);
    let record = wire::FromEngine::Assignment(assignment(current.clone()));
    let down =
        engine_side.send(engine(&record), channel::VERSION, &agreed(channel::VERSION, 4), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Assignment(at_worker) = engine_value(&down) else { panic!("assignment") };
    assert_eq!(&at_worker.charter[..2], &[0, 4]);
    let up = worker_side
        .receive(down.kind(), body(&down), channel::VERSION, &agreed(channel::VERSION, 4), &wire::CEILINGS)
        .unwrap();
    assert_eq!(engine_value(&up), record);
    let refusal = engine_side
        .send(
            engine(&wire::FromEngine::Assignment(assignment(charter(2)))),
            channel::VERSION,
            &agreed(channel::VERSION, 4),
            &wire::CEILINGS,
        )
        .unwrap_err();
    assert_eq!(refusal.wire.as_ref(), b"smith-charter");
    assert_eq!(refusal.version, 5);
    let Halt::Unheld(_) = refusal.halt else { panic!("unheld") };

    let result =
        fixed_value(smith_charter::patches::kept::smith_charter::TEXTS.last().unwrap(), "record_run_result_smallest");
    let work = wire::Work { left: None, saved: None };
    let ended = wire::Ended { outcome: result, work };
    let answer = wire::FromWorker::Answer(wire::Answer {
        task: 7,
        attempt: 2,
        turns: 1,
        spent: 2,
        read: None,
        refused: List::with_capacity(16),
        ending: wire::Ending::Ended(ended),
    });
    let down =
        worker_side.send(worker(&answer), channel::VERSION, &agreed(channel::VERSION, 4), &wire::CEILINGS).unwrap();
    let wire::FromWorker::Answer(at_engine) = worker_value(&down) else { panic!("answer") };
    let wire::Ending::Ended(end) = at_engine.ending else { panic!("ended") };
    assert_eq!(&end.outcome[..2], &[0, 4]);
    let up = engine_side
        .receive(down.kind(), body(&down), channel::VERSION, &agreed(channel::VERSION, 4), &wire::CEILINGS)
        .unwrap();
    assert_eq!(worker_value(&up), answer);

    let old_turn =
        fixed_value(smith_transcript::patches::kept::smith_transcript::TEXTS.first().unwrap(), "record_turn_smallest");
    let mut window = Writer::new(old_turn.len() + 4);
    skein_codec::piece::write(&mut window, &old_turn).unwrap();
    let window = window.finish();
    let mut parts = assignment(current);
    parts.transcript = window.clone();
    let down = engine_side
        .send(
            engine(&wire::FromEngine::Assignment(parts)),
            channel::VERSION,
            &agreed(channel::VERSION, 4),
            &wire::CEILINGS,
        )
        .unwrap();
    let wire::FromEngine::Assignment(at_worker) = engine_value(&down) else { panic!("assignment") };
    assert_eq!(at_worker.transcript, window, "old turns cross byte for byte");
    let up = worker_side
        .receive(down.kind(), body(&down), channel::VERSION, &agreed(channel::VERSION, 4), &wire::CEILINGS)
        .unwrap();
    let wire::FromEngine::Assignment(at_worker) = engine_value(&up) else { panic!("assignment") };
    let mut reader = Reader::new(&at_worker.transcript);
    let turn = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    assert_eq!(&turn[..2], &[0, 7]);
}

#[test]
fn application_roots_translate_parts_and_each_grant_without_touching_its_envelope() {
    let mut sender = Translation::new(application()).unwrap();
    let mut receiver = Translation::new(application()).unwrap();
    let mut parts = assignment(Box::new([]));
    parts.workspace = Some(wire::Workspace { workstream: 8, items: VALUE.into() });
    parts.grants.push(grant()).unwrap();
    let original = wire::FromEngine::Assignment(parts);
    let down = sender.send(engine(&original), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Assignment(record) = engine_value(&down) else { panic!("assignment") };
    assert_eq!(record.workspace.as_ref().unwrap().items.as_ref(), OLDER);
    check_grant(record.grants.get(0).unwrap());
    let up = receiver
        .receive(down.kind(), body(&down), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS)
        .unwrap();
    assert_eq!(engine_value(&up), original);
    let original = wire::FromEngine::Grant(wire::Granted { task: 7, attempt: 2, grant: grant() });
    let down = sender.send(engine(&original), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Grant(record) = engine_value(&down) else { panic!("grant") };
    check_grant(&record.grant);
    let up = receiver
        .receive(down.kind(), body(&down), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS)
        .unwrap();
    assert_eq!(engine_value(&up), original);
    let job = wire::Job {
        task: 7,
        attempt: 2,
        kind: 3,
        spec: VALUE.into(),
        deadline: skein_lib::Duration::from_secs(2),
        workspace: None,
        grants: List::with_capacity(16),
    };
    let down = sender
        .send(engine(&wire::FromEngine::Job(job)), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS)
        .unwrap();
    let wire::FromEngine::Job(record) = engine_value(&down) else { panic!("job") };
    assert_eq!(record.spec.as_ref(), OLDER);
    for ending in [
        wire::JobEnding::Finished(wire::JobFinished { result: b"no root".as_slice().into(), work: work() }),
        wire::JobEnding::Failed(wire::JobFailed {
            failure: wire::JobFailure::Unprepared(wire::PreparationValue {
                value: wire::Preparation::Permanent(wire::Resource { resource: Some(VALUE.into()) }),
            }),
            detail: Box::new([]),
            work: work(),
        }),
    ] {
        let original = wire::FromWorker::JobAnswer(wire::JobAnswer { task: 7, attempt: 2, ending });
        let down =
            sender.send(worker(&original), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS).unwrap();
        let wire::FromWorker::JobAnswer(record) = worker_value(&down) else { panic!("job answer") };
        match record.ending {
            wire::JobEnding::Finished(end) => {
                assert_eq!(end.result.as_ref(), b"no root");
                assert_eq!(end.work.left.as_deref(), Some(OLDER));
                assert_eq!(end.work.saved.as_deref(), Some(OLDER));
            }
            wire::JobEnding::Failed(end) => {
                let wire::JobFailure::Unprepared(value) = end.failure else { panic!("unprepared") };
                let wire::Preparation::Permanent(resource) = value.value else { panic!("permanent") };
                assert_eq!(resource.resource.as_deref(), Some(OLDER));
            }
            wire::JobEnding::Refused(_) => panic!("not refused"),
        }
        let up = receiver
            .receive(down.kind(), body(&down), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS)
            .unwrap();
        assert_eq!(worker_value(&up), original);
    }
}
fn grant() -> wire::Grant {
    wire::Grant {
        number: 9,
        generation: 42,
        valid: skein_lib::Duration::from_secs(3),
        value: wire::GrantValue::Application(wire::ApplicationValue { secret: skein_lib::Secret::new(VALUE.into()) }),
    }
}
fn check_grant(grant: &wire::Grant) {
    assert_eq!((grant.number, grant.generation, grant.valid), (9, 42, skein_lib::Duration::from_secs(3)));
    let wire::GrantValue::Application(ref value) = grant.value else { panic!("application") };
    assert_eq!(value.secret.expose(), OLDER);
}
fn work() -> wire::Work {
    wire::Work { left: Some(VALUE.into()), saved: Some(VALUE.into()) }
}

#[test]
fn construction_rejects_missing_bridges_and_declarations_and_total_checks_worker_sends() {
    let mut missing = application();
    missing.kept = List::with_capacity(16);
    Translation::new(missing).unwrap_err();
    let mut missing = application();
    missing.roots.get_mut(0).unwrap().declaration = "Absent";
    Translation::new(missing).unwrap_err();
    let mut wrong = application();
    wrong.kept.get_mut(0).unwrap().family = "another-family";
    wrong.roots = List::uncapped();
    assert_eq!(Translation::new(wrong).unwrap_err(), skein_codec::bridge::BridgeError::Floor);
    for texts in [&[][..], &["not a schema"][..]] {
        let mut invalid = application();
        invalid.kept.get_mut(0).unwrap().texts = texts;
        assert_eq!(Translation::new(invalid).unwrap_err(), skein_codec::bridge::BridgeError::Floor);
    }
    let mut machine = Translation::new(application()).unwrap();
    let incomplete = machine.incomplete(&agreed(12, 4));
    assert_eq!(
        incomplete.as_slice(),
        &[Agreed { name: "jig-worker-wire", version: 12 }, Agreed { name: "application", version: 1 }]
    );
    assert!(opaque().incomplete(&agreed(channel::VERSION, 5)).is_empty());
}

#[test]
fn current_links_keep_no_plans_and_only_an_older_family_materializes() {
    let mut sender = opaque();
    let mut receiver = opaque();
    let call = wire::FromWorker::Call(wire::Call {
        task: 7,
        attempt: 2,
        name: b"name".as_slice().into(),
        tool: b"tool".as_slice().into(),
        writes: false,
        input: b"{}".as_slice().into(),
        deadline: skein_lib::Duration::from_secs(1),
        contract: 1,
    });
    let original = worker(&call);
    let current = sender.send(original, channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS).unwrap();
    let up = receiver
        .receive(current.kind(), body(&current), channel::VERSION, &agreed(channel::VERSION, 5), &wire::CEILINGS)
        .unwrap();
    assert_eq!(worker_value(&up), call);
    assert_eq!((sender.planned(), receiver.planned()), (0, 0));
    assert!(sender.incomplete(&agreed(channel::VERSION, 5)).is_empty());
    assert_eq!(sender.planned(), 0);
    sender.send(worker(&call), 14, &agreed(14, 5), &wire::CEILINGS).expect_err("below breaking floor");
    assert_eq!((sender.planned(), receiver.planned()), (1, 0));
}

#[test]
fn application_ask_and_answer_roots_translate_and_unrooted_bytes_cross_as_is() {
    let agreed = agreed(channel::VERSION, 5);
    let mut sender = Translation::new(application()).unwrap();
    let mut receiver = Translation::new(application()).unwrap();
    let ask = wire::FromWorker::Ask(wire::Ask { run: 7, attempt: 2, number: 1, bytes: VALUE.into() });
    let sent = sender.send(worker(&ask), channel::VERSION, &agreed, &wire::CEILINGS).unwrap();
    let wire::FromWorker::Ask(value) = worker_value(&sent) else { panic!("ask") };
    assert_eq!((value.run, value.attempt, value.number, value.bytes.as_ref()), (7, 2, 1, OLDER));
    let up = receiver.receive(sent.kind(), body(&sent), channel::VERSION, &agreed, &wire::CEILINGS).unwrap();
    assert_eq!(worker_value(&up), ask);
    for answer in [wire::AskAnswer::Bytes(wire::AskBytes { bytes: VALUE.into() }), wire::AskAnswer::Refused] {
        let record = wire::FromEngine::Asked(wire::Asked { run: 7, attempt: 2, number: 1, answer });
        let sent = sender.send(engine(&record), channel::VERSION, &agreed, &wire::CEILINGS).unwrap();
        let wire::FromEngine::Asked(value) = engine_value(&sent) else { panic!("asked") };
        if let wire::AskAnswer::Bytes(bytes) = value.answer {
            assert_eq!(bytes.bytes.as_ref(), OLDER);
        }
        let up = receiver.receive(sent.kind(), body(&sent), channel::VERSION, &agreed, &wire::CEILINGS).unwrap();
        assert_eq!(engine_value(&up), record);
    }
    let unrooted = opaque().send(worker(&ask), channel::VERSION, &agreed, &wire::CEILINGS).unwrap();
    assert_eq!(worker_value(&unrooted), ask);
}

#[test]
fn kept_bridge_matches_the_eager_bridge() {
    let application = application();
    let kept = application.kept.get(0).unwrap();
    let mut lazy = kept.bridge().unwrap();
    let mut eager = Bridge::new(
        Capacity { versions: 2, patches: 2, text: 4096, bytes: 128, nodes: 32 },
        kept.texts,
        kept.patches,
        kept.floor,
        kept.sites,
    )
    .unwrap();
    assert_eq!(lazy.floor(), eager.floor());
    assert_eq!(lazy.current(), eager.current());
    for version in eager.floor()..=eager.current() {
        assert_eq!(lazy.layout(version), eager.layout(version));
    }
    let old = lazy.translate(2, 1, b"Value", VALUE.into()).unwrap();
    assert_eq!(old, eager.translate(2, 1, b"Value", VALUE.into()).unwrap());
    assert_eq!(old.as_ref(), OLDER);
    assert_eq!(lazy.translate(1, 2, b"Value", old.clone()).unwrap().as_ref(), VALUE);
    assert_eq!(eager.translate(1, 2, b"Value", old).unwrap().as_ref(), VALUE);
}

#[test]
fn current_application_links_keep_no_plans_and_older_parts_reuse_one_bridge() {
    let mut app = application();
    app.roots.push(Root { carried: Carried::JobResult, wire: "application", declaration: "Value" }).unwrap();
    let mut sender = Translation::new(app).unwrap();
    let mut app = application();
    app.roots.push(Root { carried: Carried::JobResult, wire: "application", declaration: "Value" }).unwrap();
    let mut receiver = Translation::new(app).unwrap();
    let mut current = agreed(channel::VERSION, smith_charter::wire::VERSION);
    current[3].version = 2;
    assert_eq!((sender.planned(), receiver.planned()), (0, 0));
    assert!(sender.incomplete(&current).is_empty());
    assert!(receiver.incomplete(&current).is_empty());
    assert_eq!((sender.planned(), receiver.planned()), (0, 0));
    let mut parts = assignment(Box::new([]));
    parts.workspace = Some(wire::Workspace { workstream: 8, items: VALUE.into() });
    parts.grants.push(grant()).unwrap();
    let assignment = engine(&wire::FromEngine::Assignment(parts));
    let job = engine(&wire::FromEngine::Job(wire::Job {
        task: 7,
        attempt: 2,
        kind: 3,
        spec: VALUE.into(),
        deadline: skein_lib::Duration::from_secs(2),
        workspace: None,
        grants: List::with_capacity(16),
    }));
    let finished = worker(&wire::FromWorker::JobAnswer(wire::JobAnswer {
        task: 7,
        attempt: 2,
        ending: wire::JobEnding::Finished(wire::JobFinished { result: VALUE.into(), work: work() }),
    }));
    let unprepared = worker(&wire::FromWorker::JobAnswer(wire::JobAnswer {
        task: 7,
        attempt: 2,
        ending: wire::JobEnding::Failed(wire::JobFailed {
            failure: wire::JobFailure::Unprepared(wire::PreparationValue {
                value: wire::Preparation::Permanent(wire::Resource { resource: Some(VALUE.into()) }),
            }),
            detail: Box::new([]),
            work: work(),
        }),
    }));
    let ask = worker(&wire::FromWorker::Ask(wire::Ask { run: 7, attempt: 2, number: 1, bytes: VALUE.into() }));
    let answer = engine(&wire::FromEngine::Asked(wire::Asked {
        run: 7,
        attempt: 2,
        number: 1,
        answer: wire::AskAnswer::Bytes(wire::AskBytes { bytes: VALUE.into() }),
    }));
    for original in [assignment, job, finished, unprepared, ask, answer] {
        let expected = skein_lib::bytes::copy_of(original.bytes());
        let sent = sender.send(original, channel::VERSION, &current, &wire::CEILINGS).unwrap();
        assert_eq!(sent.bytes(), expected.as_ref());
        let up = receiver.receive(sent.kind(), body(&sent), channel::VERSION, &current, &wire::CEILINGS).unwrap();
        assert_eq!(up.bytes(), expected.as_ref());
        assert_eq!((sender.planned(), receiver.planned()), (0, 0));
    }
    let older = agreed(channel::VERSION, smith_charter::wire::VERSION);
    for _ in 0_u8..2 {
        let original = wire::FromWorker::Ask(wire::Ask { run: 7, attempt: 2, number: 1, bytes: VALUE.into() });
        let sent = sender.send(worker(&original), channel::VERSION, &older, &wire::CEILINGS).unwrap();
        let wire::FromWorker::Ask(value) = worker_value(&sent) else { panic!("ask") };
        assert_eq!(value.bytes.as_ref(), OLDER);
        let up = receiver.receive(sent.kind(), body(&sent), channel::VERSION, &older, &wire::CEILINGS).unwrap();
        assert_eq!(worker_value(&up), original);
        assert_eq!((sender.planned(), receiver.planned()), (1, 1));
    }
}

fn usage_records() -> [wire::FromWorker; 5] {
    let window = wire::UsageWindow {
        label: b"five_hour".as_slice().into(),
        length: Some(skein_lib::Duration::from_secs(18_000)),
        used: Some(2800),
        resets: Some(123_456),
        state: 2,
    };
    let mut windows = List::with_capacity(8);
    windows.push(window.clone()).unwrap();
    let notice = wire::Usage {
        task: 7,
        attempt: 2,
        account: 3,
        label: window.label.clone(),
        length: window.length,
        used: window.used,
        resets: window.resets,
        state: window.state,
    };
    let read = |outcome| wire::FromWorker::UsageRead(wire::UsageRead { read: 42, outcome });
    [
        wire::FromWorker::Usage(notice),
        read(wire::UsageOutcome::Read(wire::UsageWindows { windows })),
        read(wire::UsageOutcome::NotSignedIn),
        read(wire::UsageOutcome::Spent),
        read(wire::UsageOutcome::Failed(wire::SourceFailed {
            reason: wire::SourceFailure::Unconfigured,
            detail: Box::new([]),
        })),
    ]
}
#[test]
fn usage_records_round_trip_at_nineteen() {
    for record in usage_records() {
        let frame = opaque().send(worker(&record), 19, &agreed(19, 5), &wire::CEILINGS).unwrap();
        let up = opaque().receive(frame.kind(), body(&frame), 19, &agreed(19, 5), &wire::CEILINGS).unwrap();
        assert_eq!(worker_value(&up), record);
    }
    let record = wire::FromEngine::ReadUsage(wire::ReadUsage { read: 42, source: b"personal".as_slice().into() });
    let frame = opaque().send(engine(&record), 19, &agreed(19, 5), &wire::CEILINGS).unwrap();
    let up = opaque().receive(frame.kind(), body(&frame), 19, &agreed(19, 5), &wire::CEILINGS).unwrap();
    assert_eq!(engine_value(&up), record);
}
#[test]
fn usage_and_usage_read_are_unheld_below_nineteen() {
    let mut translation = opaque();
    for version in [16, 17, 18] {
        // One representative per new up kind; outcomes round-trip above.
        for record in usage_records().into_iter().take(2) {
            let refusal = translation.send(worker(&record), version, &agreed(version, 5), &wire::CEILINGS).unwrap_err();
            assert_eq!(refusal.wire.as_ref(), b"jig-worker-wire");
            assert_eq!(refusal.version, crate::VERSION);
            let Halt::Unheld(_) = refusal.halt else { panic!("usage is unheld: {refusal:?}") };
        }
    }
}
#[test]
fn older_notices_and_calls_round_trip_through_seventeen_unchanged() {
    let mut sender = opaque();
    let mut receiver = opaque();
    let notice = failed_answer(wire::RunFailure::Untooled);
    let frame = sender.send(worker(&notice), 17, &agreed(17, 5), &wire::CEILINGS).unwrap();
    let up = receiver.receive(frame.kind(), body(&frame), 17, &agreed(17, 5), &wire::CEILINGS).unwrap();
    assert_eq!(worker_value(&up), notice);
    let call = wire::FromEngine::Cancel(wire::Cancel { task: 7, attempt: 2 });
    let frame = sender.send(engine(&call), 17, &agreed(17, 5), &wire::CEILINGS).unwrap();
    let up = receiver.receive(frame.kind(), body(&frame), 17, &agreed(17, 5), &wire::CEILINGS).unwrap();
    assert_eq!(engine_value(&up), call);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the bridge test formats generated fixture names")]
fn seventeens_kept_fixed_values_go_up_to_eighteen_with_no_refusal_and_back() {
    use crate::patches::kept::jig_worker_wire as kept;
    use skein_codegen::{Declaration, snake, values};
    let mut seventeen = None;
    let mut eighteen = None;
    for text in kept::TEXTS {
        if text.starts_with("family jig-worker-wire 17\n") {
            seventeen = Some(*text);
        } else if text.starts_with("family jig-worker-wire 18\n") {
            eighteen = Some(*text);
        }
    }
    let old = values(seventeen.expect("kept 17"));
    let older = &old.fixed;
    let newer = &values(eighteen.expect("kept 18")).fixed;
    let mut bridge = kept::bridge(Capacity {
        versions: u32::try_from(kept::TEXTS.len()).unwrap(),
        patches: u32::try_from(crate::patches::PATCHES.len() * 2).unwrap(),
        text: skein_codec::schema::MAX_TEXT,
        bytes: 65_536,
        nodes: 65_536,
    })
    .unwrap();
    for declaration in &old.schema.declarations {
        let mut names = List::uncapped();
        let root = match declaration {
            Declaration::Record(record) => {
                names.push(format!("record_{}", snake(&record.name))).unwrap();
                &record.name
            }
            Declaration::Enum(value) => {
                for variant in &value.variants {
                    names.push(format!("enum_{}_{}", snake(&value.name), variant.name)).unwrap();
                }
                &value.name
            }
        };
        for name in names.into_boxed() {
            for size in ["smallest", "full"] {
                let name = format!("{name}_{size}");
                let before = fixture_fixed_value(older, &name);
                let after = fixture_fixed_value(newer, &name);
                let up = bridge.translate(17, 18, root.as_bytes(), before.clone()).unwrap();
                match name.as_str() {
                    "record_model_failure_full" | "record_failed_full" | "enum_ending_failed_full" => {
                        // The full v18 fixture ends with a present refusal, which the v17 fixture cannot carry.
                        let present = b"\x01\x00\x00\x00\x03abc";
                        assert_eq!(after.strip_suffix(present.as_slice()), Some(before.as_ref()), "{name}.bin");
                        assert_eq!(up.strip_suffix(&[0]), Some(before.as_ref()), "{name}.bin");
                        let halt = bridge.translate(18, 17, root.as_bytes(), after).unwrap_err();
                        let Halt::Unheld(_) = halt else { panic!("{name}.bin: {halt:?}") };
                    }
                    _ => assert_eq!(up, after, "{name}.bin"),
                }
                let down = bridge.translate(18, 17, root.as_bytes(), up).unwrap();
                assert_eq!(down, before, "{name}.bin");
            }
        }
    }
}
