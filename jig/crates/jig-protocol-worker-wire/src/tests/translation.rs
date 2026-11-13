//! The shared component bridge machine (jig/docs/design/protocol/channel/bridges.md, 5 and 9).
use crate::translation::Translation;
use crate::{self as channel, Agreed, Application, Carried, Root, Spoken, v13 as wire};
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
    let mut bridges = List::with_capacity(1);
    bridges
        .push(
            Bridge::new(
                Capacity { versions: 2, patches: 2, text: 4096, bytes: 128, nodes: 32 },
                &APP,
                &["patch 1\nowner application 1\nfamily application 1 to 2\nwiden Value.n u8 u16\n"],
                1,
                no_sites,
            )
            .unwrap(),
        )
        .unwrap();
    let mut roots = List::with_capacity(16);
    for carried in
        [Carried::Items, Carried::Left, Carried::Saved, Carried::Preparation, Carried::JobSpec, Carried::Grant]
    {
        roots.push(Root { carried, wire: "application", declaration: "Value" }).unwrap();
    }
    let mut spoken = List::with_capacity(1);
    spoken.push(Spoken::of("application", &APP, 1).unwrap()).unwrap();
    Application { spoken, bridges, roots }
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
    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    frame(0x180 + u16::from(bytes[0]), &bytes[1..])
}
fn worker(record: &wire::FromWorker) -> Frame {
    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut writer).unwrap();
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
fn golden(text: &str, name: &str) -> Box<[u8]> {
    for golden in skein_codegen::generate(&skein_codegen::parse(text).unwrap()).goldens {
        if golden.name.strip_suffix(".bin") == Some(name) {
            return golden.bytes.into_boxed_slice();
        }
    }
    panic!("missing golden {name}")
}
fn charter(contract: u16) -> Box<[u8]> {
    let bytes = golden(smith_charter::patches::kept::smith_charter::TEXTS.last().unwrap(), "record_charter_smallest");
    let mut parts =
        smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts();
    let mut tools = parts.tools.into_parts();
    tools.contract = contract;
    parts.tools = smith_charter::Tools::new(&smith_charter::CEILINGS, tools).unwrap();
    let record = smith_charter::Charter::new(&smith_charter::CEILINGS, parts).unwrap();
    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut writer).unwrap();
    writer.finish()
}
fn assignment(charter: Box<[u8]>) -> wire::Assignment {
    let mut parts = super::fixtures::assignment_resumed().into_parts();
    parts.charter = charter;
    parts.transcript = Box::new([]);
    parts.workspace = None;
    parts.grants = List::with_capacity(16);
    wire::Assignment::new(&wire::CEILINGS, parts).unwrap()
}

#[test]
fn enumeration_bridges_cover_every_kind_and_refuse_unheld_variants() {
    let text = channel::patches::kept::jig_worker_wire::TEXTS.last().unwrap();
    let goldens = skein_codegen::generate(&skein_codegen::parse(text).unwrap()).goldens;
    let mut sender = opaque();
    let mut receiver = opaque();
    let mut covered = skein_lib::Set::with_capacity(32);
    for golden in goldens {
        if !(golden.name.starts_with("enum_from_engine_") || golden.name.starts_with("enum_from_worker_")) {
            continue;
        }
        let base = if golden.name.starts_with("enum_from_engine_") { 0x180 } else { 0x100 };
        let kind = base + u16::from(golden.bytes[0]);
        // Calls at the older default, not the codec's all-zero/all-max golden.
        let mut original = frame(kind, &golden.bytes[1..]);
        if kind == channel::Kind::Call.number() {
            let mut call = wire::Call::decode(&wire::CEILINGS, &mut Reader::new(body(&original))).unwrap().into_parts();
            call.contract = 1;
            original = worker(&wire::FromWorker::Call(wire::Call::new(&wire::CEILINGS, call).unwrap()));
        }
        let expected = skein_lib::bytes::copy_of(original.bytes());
        let down = sender.send(original, 12, &agreed(12, 5), &wire::CEILINGS);
        if kind == channel::Kind::Wires.number() {
            assert!(down.is_err(), "no wires kind at twelve");
            continue;
        }
        let down = match down {
            Ok(down) => down,
            Err(error) => panic!("{}: {error:?}", golden.name),
        };
        if kind == channel::Kind::Call.number() {
            assert_eq!(down.body_len() + 2, u32::try_from(expected.len() - 8).unwrap());
        }
        let up = receiver.receive(down.kind(), body(&down), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
        assert_eq!(up.bytes(), expected.as_ref(), "{}", golden.name);
        covered.insert(kind).unwrap();
    }
    assert_eq!(covered.len(), u32::try_from(channel::KINDS.len().checked_sub(1).unwrap()).unwrap());
}

#[test]
fn smith_charters_results_and_old_window_turns_take_their_own_bridges() {
    let mut engine_side = opaque();
    let mut worker_side = opaque();
    let current = charter(1);
    let record = wire::FromEngine::Assignment(assignment(current.clone()));
    let down = engine_side.send(engine(&record), 12, &agreed(12, 4), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Assignment(at_worker) = engine_value(&down) else { panic!("assignment") };
    assert_eq!(&at_worker.charter()[..2], &[0, 4]);
    let up = worker_side.receive(down.kind(), body(&down), 12, &agreed(12, 4), &wire::CEILINGS).unwrap();
    assert_eq!(engine_value(&up), record);
    let refusal = engine_side
        .send(engine(&wire::FromEngine::Assignment(assignment(charter(2)))), 12, &agreed(12, 4), &wire::CEILINGS)
        .unwrap_err();
    assert_eq!(refusal.wire.as_ref(), b"smith-charter");
    assert_eq!(refusal.version, 5);
    let Halt::Unheld(_) = refusal.halt else { panic!("unheld") };

    let result =
        golden(smith_charter::patches::kept::smith_charter::TEXTS.last().unwrap(), "record_run_result_smallest");
    let work = wire::Work::new(&wire::CEILINGS, wire::WorkParts { left: None, saved: None }).unwrap();
    let ended = wire::Ended::new(&wire::CEILINGS, wire::EndedParts { outcome: result, work }).unwrap();
    let answer = wire::FromWorker::Answer(
        wire::Answer::new(
            &wire::CEILINGS,
            wire::AnswerParts {
                task: 7,
                attempt: 2,
                turns: 1,
                spent: 2,
                read: None,
                refused: List::with_capacity(16),
                ending: wire::Ending::Ended(ended),
            },
        )
        .unwrap(),
    );
    let down = worker_side.send(worker(&answer), 12, &agreed(12, 4), &wire::CEILINGS).unwrap();
    let wire::FromWorker::Answer(at_engine) = worker_value(&down) else { panic!("answer") };
    let wire::Ending::Ended(end) = at_engine.ending() else { panic!("ended") };
    assert_eq!(&end.outcome()[..2], &[0, 4]);
    let up = engine_side.receive(down.kind(), body(&down), 12, &agreed(12, 4), &wire::CEILINGS).unwrap();
    assert_eq!(worker_value(&up), answer);

    let old_turn =
        golden(smith_transcript::patches::kept::smith_transcript::TEXTS.first().unwrap(), "record_turn_smallest");
    let mut window = Writer::new(old_turn.len() + 4);
    skein_codec::piece::write(&mut window, &old_turn).unwrap();
    let window = window.finish();
    let mut parts = assignment(current).into_parts();
    parts.transcript = window.clone();
    let down = engine_side
        .send(
            engine(&wire::FromEngine::Assignment(wire::Assignment::new(&wire::CEILINGS, parts).unwrap())),
            12,
            &agreed(12, 4),
            &wire::CEILINGS,
        )
        .unwrap();
    let wire::FromEngine::Assignment(at_worker) = engine_value(&down) else { panic!("assignment") };
    assert_eq!(at_worker.transcript(), window.as_ref(), "old turns cross byte for byte");
    let up = worker_side.receive(down.kind(), body(&down), 12, &agreed(12, 4), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Assignment(at_worker) = engine_value(&up) else { panic!("assignment") };
    let mut reader = Reader::new(at_worker.transcript());
    let turn = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    assert_eq!(&turn[..2], &[0, 7]);
}

#[test]
fn application_roots_translate_parts_and_each_grant_without_touching_its_envelope() {
    let mut sender = Translation::new(application()).unwrap();
    let mut receiver = Translation::new(application()).unwrap();
    let mut parts = assignment(Box::new([])).into_parts();
    parts.workspace = Some(
        wire::Workspace::new(&wire::CEILINGS, wire::WorkspaceParts { workstream: 8, items: VALUE.into() }).unwrap(),
    );
    parts.grants.push(grant()).unwrap();
    let original = wire::FromEngine::Assignment(wire::Assignment::new(&wire::CEILINGS, parts).unwrap());
    let down = sender.send(engine(&original), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Assignment(record) = engine_value(&down) else { panic!("assignment") };
    assert_eq!(record.workspace().as_ref().unwrap().items(), OLDER);
    check_grant(record.grants().get(0).unwrap());
    let up = receiver.receive(down.kind(), body(&down), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    assert_eq!(engine_value(&up), original);
    let original = wire::FromEngine::Grant(
        wire::Granted::new(&wire::CEILINGS, wire::GrantedParts { task: 7, attempt: 2, grant: grant() }).unwrap(),
    );
    let down = sender.send(engine(&original), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Grant(record) = engine_value(&down) else { panic!("grant") };
    check_grant(record.grant());
    let up = receiver.receive(down.kind(), body(&down), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    assert_eq!(engine_value(&up), original);
    let job = wire::Job::new(
        &wire::CEILINGS,
        wire::JobParts {
            task: 7,
            attempt: 2,
            kind: 3,
            spec: VALUE.into(),
            deadline: skein_lib::Duration::from_secs(2),
            workspace: None,
            grants: List::with_capacity(16),
        },
    )
    .unwrap();
    let down = sender.send(engine(&wire::FromEngine::Job(job)), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    let wire::FromEngine::Job(record) = engine_value(&down) else { panic!("job") };
    assert_eq!(record.spec(), OLDER);
    for ending in [
        wire::JobEnding::Finished(
            wire::JobFinished::new(
                &wire::CEILINGS,
                wire::JobFinishedParts { result: b"no root".as_slice().into(), work: work() },
            )
            .unwrap(),
        ),
        wire::JobEnding::Failed(
            wire::JobFailed::new(
                &wire::CEILINGS,
                wire::JobFailedParts {
                    failure: wire::JobFailure::Unprepared(
                        wire::PreparationValue::new(
                            &wire::CEILINGS,
                            wire::PreparationValueParts {
                                value: wire::Preparation::Permanent(
                                    wire::Resource::new(
                                        &wire::CEILINGS,
                                        wire::ResourceParts { resource: Some(VALUE.into()) },
                                    )
                                    .unwrap(),
                                ),
                            },
                        )
                        .unwrap(),
                    ),
                    detail: Box::new([]),
                    work: work(),
                },
            )
            .unwrap(),
        ),
    ] {
        let original = wire::FromWorker::JobAnswer(
            wire::JobAnswer::new(&wire::CEILINGS, wire::JobAnswerParts { task: 7, attempt: 2, ending }).unwrap(),
        );
        let down = sender.send(worker(&original), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
        let wire::FromWorker::JobAnswer(record) = worker_value(&down) else { panic!("job answer") };
        match record.ending() {
            wire::JobEnding::Finished(end) => {
                assert_eq!(end.result(), b"no root");
                assert_eq!(end.work().left().as_deref(), Some(OLDER));
                assert_eq!(end.work().saved().as_deref(), Some(OLDER));
            }
            wire::JobEnding::Failed(end) => {
                let wire::JobFailure::Unprepared(value) = end.failure() else { panic!("unprepared") };
                let wire::Preparation::Permanent(resource) = value.value() else { panic!("permanent") };
                assert_eq!(resource.resource().as_deref(), Some(OLDER));
            }
            wire::JobEnding::Refused(_) => panic!("not refused"),
        }
        let up = receiver.receive(down.kind(), body(&down), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
        assert_eq!(worker_value(&up), original);
    }
}
fn grant() -> wire::Grant {
    wire::Grant::new(
        &wire::CEILINGS,
        wire::GrantParts {
            number: 9,
            generation: 42,
            valid: skein_lib::Duration::from_secs(3),
            value: wire::GrantValue::Application(
                wire::ApplicationValue::new(
                    &wire::CEILINGS,
                    wire::ApplicationValueParts { secret: skein_lib::Secret::new(VALUE.into()) },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap()
}
fn check_grant(grant: &wire::Grant) {
    assert_eq!((grant.number(), grant.generation(), grant.valid()), (9, 42, skein_lib::Duration::from_secs(3)));
    let wire::GrantValue::Application(value) = grant.value() else { panic!("application") };
    assert_eq!(value.secret().expose(), OLDER);
}
fn work() -> wire::Work {
    wire::Work::new(&wire::CEILINGS, wire::WorkParts { left: Some(VALUE.into()), saved: Some(VALUE.into()) }).unwrap()
}

#[test]
fn construction_rejects_missing_bridges_and_declarations_and_total_checks_worker_sends() {
    let mut missing = application();
    missing.bridges = List::with_capacity(16);
    Translation::new(missing).unwrap_err();
    let mut missing = application();
    missing.roots.get_mut(0).unwrap().declaration = "Absent";
    Translation::new(missing).unwrap_err();
    let mut machine = Translation::new(application()).unwrap();
    let incomplete = machine.incomplete(&agreed(12, 4));
    assert_eq!(
        incomplete.as_slice(),
        &[Agreed { name: "jig-worker-wire", version: 12 }, Agreed { name: "application", version: 1 }]
    );
    assert!(opaque().incomplete(&agreed(13, 5)).is_empty());
}

#[test]
fn current_links_keep_no_plans_and_only_an_older_family_materializes() {
    let mut sender = opaque();
    let mut receiver = opaque();
    let call = wire::FromWorker::Call(
        wire::Call::new(
            &wire::CEILINGS,
            wire::CallParts {
                task: 7,
                attempt: 2,
                name: b"name".as_slice().into(),
                tool: b"tool".as_slice().into(),
                writes: false,
                input: b"{}".as_slice().into(),
                deadline: skein_lib::Duration::from_secs(1),
                contract: 1,
            },
        )
        .unwrap(),
    );
    let original = worker(&call);
    let current = sender.send(original, 13, &agreed(13, 5), &wire::CEILINGS).unwrap();
    let up = receiver.receive(current.kind(), body(&current), 13, &agreed(13, 5), &wire::CEILINGS).unwrap();
    assert_eq!(worker_value(&up), call);
    assert_eq!((sender.planned(), receiver.planned()), (0, 0));
    assert!(sender.incomplete(&agreed(13, 5)).is_empty());
    assert_eq!(sender.planned(), 0);
    let old = sender.send(worker(&call), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    let up = receiver.receive(old.kind(), body(&old), 12, &agreed(12, 5), &wire::CEILINGS).unwrap();
    assert_eq!(worker_value(&up), call);
    assert_eq!((sender.planned(), receiver.planned()), (1, 1));
}
