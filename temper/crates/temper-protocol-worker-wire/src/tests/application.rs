//! Temper's declarations are held against its encoders (protocol/wires.md, 6).
use super::fixtures;
use crate::v1::Record;
use crate::{v1, wire};
use jig_protocol_worker_wire::Carried;
use skein_lib::List;

#[test]
fn the_application_speaks_its_current_wire() {
    let application = crate::application(&fixtures::limits()).expect("application");
    let spoken = application.spoken.get(0).expect("the application's wire");
    assert_eq!(application.spoken.len(), 1);
    assert_eq!(spoken, &crate::spoken());
    assert_eq!(
        (spoken.name, spoken.version, spoken.layout, spoken.text),
        (wire::FAMILY, 12, wire::layout(), wire::TEXT)
    );
}

fn payloads() -> [(Carried, &'static str, Box<[u8]>); 8] {
    let limits = fixtures::limits();
    let items = fixtures::items(1);
    let mut landed = List::with_capacity(1);
    landed.push(v1::Landed { tag: 17, commit: [3; 32], checked: true }).unwrap();
    let left = v1::Left { landed };
    let mut repositories = List::with_capacity(1);
    repositories.push(v1::SavedRepository { tag: 19, from_saved: true, landing: v1::Landing::Moved }).unwrap();
    let saved = v1::Saved { repositories };
    let preparation = v1::Preparation::Refused(v1::Tag { tag: 23 });
    let identity = crate::identity(&fixtures::identity(b"token"));
    [
        (Carried::Items, "Items", crate::items(&items)),
        (Carried::Left, "Left", crate::left(&left)),
        (Carried::Saved, "Saved", crate::saved(&saved)),
        (Carried::Preparation, "Preparation", crate::preparation(&preparation)),
        (
            Carried::JobSpec,
            "JobSpec",
            v1::JobSpec { command: b"tool".as_slice().into(), request: b"{}".as_slice().into(), refusal: false }
                .encode(&limits)
                .unwrap(),
        ),
        (Carried::Grant, "Identity", identity.expose().into()),
        (Carried::Ask, "Ask", crate::ask(&v1::Ask::Landed(v1::Head { head: [11; 32] }))),
        (Carried::Answer, "Answer", crate::answer(&v1::Answer::Reds(v1::Reds { reds: List::with_capacity(0) }))),
    ]
}

#[test]
fn the_roots_name_the_declarations_temper_encodes() {
    let application = crate::application(&fixtures::limits()).unwrap();
    let schema = skein_codec::schema::parse(wire::TEXT.as_bytes()).unwrap();
    for (carried, declaration, bytes) in payloads() {
        let mut found = None;
        for root in &application.roots {
            if root.carried == carried {
                found = Some(root);
            }
        }
        let root = found.expect("translated part");
        assert_eq!(root.wire, wire::FAMILY);
        assert_eq!(root.declaration, declaration);
        let mut value = skein_codec::value::Value::with_room(256);
        value.decode(&schema, schema.find(root.declaration.as_bytes()).unwrap(), bytes).expect("encoder's declaration");
    }
    assert_eq!(application.roots.len(), 8);
    for root in &application.roots {
        assert_ne!(root.carried, Carried::JobResult, "tooling's result is opaque");
    }
}

#[test]
fn application_refuses_limits_past_the_wire_ceiling() {
    let mut limits = fixtures::limits();
    limits.red_head = v1::CEILINGS.red_head;
    crate::application(&limits).unwrap();
    limits.red_head = v1::CEILINGS.red_head.checked_add(1).unwrap();
    assert!(v1::check_limits(&limits).is_err());
    assert_eq!(crate::application(&limits).err(), v1::check_limits(&limits).err());
}

fn peer(worker: &skein_codec::schema::Schema) -> jig_protocol_worker_wire::layouts::Peer {
    use jig_protocol_worker_wire::layouts::{Plans, Theirs};
    let app = crate::application(&fixtures::limits()).unwrap();
    Plans::new(&[wire::TEXT], &app.roots)
        .unwrap()
        .judge(&[Theirs { name: wire::FAMILY, layout: worker.fingerprints().family().digest(), text: worker.text() }])
        .unwrap()
}

fn transcode(
    mode: skein_codec::layout::Mode,
    source: &skein_codec::schema::Schema,
    target: &skein_codec::schema::Schema,
    declaration: &str,
    bytes: &[u8],
) -> Box<[u8]> {
    use skein_codec::layout::{Plan, Workspace};
    let from = source.root(declaration.as_bytes(), None).unwrap();
    let to = target.root(declaration.as_bytes(), None).unwrap();
    let plan = Plan::between(mode, source, from, &[], target, to, &[]).unwrap();
    let mut workspace = Workspace::new();
    assert!(workspace.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    plan.convert(bytes, &mut out, &mut workspace).unwrap();
    out.to_boxed()
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the fixture derives a text by editing schema phrases")]
fn temper_roots_round_trip_in_their_carried_directions() {
    use jig_protocol_worker_wire::layouts::Nested;
    use skein_codec::{layout::Mode, schema::parse};
    // One new field in the reach of each root, and a removed field with a
    // default on the way back. The payloads use the real temper encoders.
    let mut text = wire::TEXT.replace("account: option u64 save:", "dropped account: option u64 = none save:");
    for end in [
        "rerun: option u32 = none }",
        "landed: list 64 Landed }",
        "repositories: list 64 SavedRepository }",
        "record Tag { tag: u64 }",
        "refusal: bool = false }",
        "email: text 256 }",
        "record Head { head: fixed 32 }",
        "reds: list 256 Red }",
    ] {
        assert_eq!(text.matches(end).count(), 1, "{end}");
        text = text.replace(end, &format!("{} worker_mark: u32 = 17 }}", end.strip_suffix(" }").unwrap()));
    }
    let engine = parse(wire::TEXT.as_bytes()).unwrap();
    let worker = parse(text.as_bytes()).unwrap();
    let mut peer = peer(&worker);
    assert!(!peer.same());
    let mut out = skein_lib::Buffer::with_capacity(1024);
    for (carried, declaration, bytes) in payloads() {
        let root = Nested::Carried(carried);
        match carried {
            Carried::Items | Carried::JobSpec | Carried::Grant | Carried::Answer => {
                peer.down(root, &bytes, &mut out).unwrap();
                assert_ne!(out.as_slice(), &*bytes, "{declaration} must cross layouts");
                assert_eq!(transcode(Mode::Read, &worker, &engine, declaration, out.as_slice()), bytes);
            }
            Carried::Left | Carried::Saved | Carried::Preparation | Carried::Ask => {
                let sent = transcode(Mode::Keep, &engine, &worker, declaration, &bytes);
                assert_ne!(sent, bytes, "{declaration} must cross layouts");
                peer.up(root, &sent, &mut out).unwrap();
                assert_eq!(out.as_slice(), &*bytes, "{declaration}");
            }
            Carried::JobResult => unreachable!("opaque, not a temper root"),
        }
    }
}

/// temper/docs/design/protocol/wires.md, 8: an older worker can only keep a job that doesn't ask.
#[test]
#[expect(clippy::disallowed_macros, reason = "the fixture derives an older schema text")]
fn an_older_worker_cannot_keep_a_jobs_refusal_ask() {
    use jig_protocol_worker_wire::layouts::{Cause, Nested};
    use skein_codec::{layout::Mode, schema::parse};
    let engine = parse(wire::TEXT.as_bytes()).unwrap();
    let (before, field) = wire::TEXT.split_once(" refusal:").expect("the appended JobSpec field");
    let (_, after) = field.split_once(" }").expect("the end of JobSpec");
    let text = format!("{before} }}{after}");
    let worker = parse(text.as_bytes()).unwrap();
    let mut peer = peer(&worker);
    let mut value = v1::JobSpec { command: b"tool".as_slice().into(), request: b"{}".as_slice().into(), refusal: true };
    let limits = fixtures::limits();
    let root = Nested::Carried(Carried::JobSpec);
    let mut out = skein_lib::Buffer::with_capacity(1024);

    let asked = value.encode(&limits).unwrap();
    let problem = peer.down(root, &asked, &mut out).expect_err("old workers cannot keep the ask");
    assert_eq!(problem.cause, Cause::Field);
    assert_eq!(&*problem.kept(), b"the worker's layout cannot keep temper-worker-wire JobSpec.refusal: field");
    assert_eq!(out.as_slice(), []);

    value.refusal = false;
    let unasked = value.encode(&limits).unwrap();
    peer.down(root, &unasked, &mut out).expect("old workers keep the default");
    assert_eq!(transcode(Mode::Read, &worker, &engine, "JobSpec", out.as_slice()), unasked);
}

#[test]
fn an_older_worker_cannot_keep_a_projects_rerun_bound() {
    use jig_protocol_worker_wire::layouts::{Cause, Nested};
    use skein_codec::{layout::Mode, schema::parse};
    let engine = parse(wire::TEXT.as_bytes()).unwrap();
    let text = wire::TEXT.replace(" rerun: option u32 = none", "");
    let worker = parse(text.as_bytes()).unwrap();
    let mut peer = peer(&worker);
    let root = Nested::Carried(Carried::Items);
    let mut out = skein_lib::Buffer::with_capacity(1024);
    let mut value = fixtures::items(1);
    let limits = fixtures::limits();

    for bound in [0, 3] {
        value.rerun = Some(bound);
        let bytes = crate::items(&value);
        assert_eq!(v1::Items::decode(&limits, &mut skein_lib::Reader::new(&bytes)).unwrap(), value);
        let problem = peer.down(root, &bytes, &mut out).expect_err("untranslated for this worker");
        assert_eq!(problem.cause, Cause::Field);
        assert_eq!(&*problem.kept(), b"the worker's layout cannot keep temper-worker-wire Items.rerun: field");
        assert_eq!(out.as_slice(), []);
    }

    value.rerun = None;
    let bytes = crate::items(&value);
    peer.down(root, &bytes, &mut out).expect("old workers keep the default");
    assert_ne!(out.as_slice(), &*bytes, "the previous layout has no field for rerun");
    let converted = transcode(Mode::Read, &worker, &engine, "Items", out.as_slice());
    assert_eq!(v1::Items::decode(&limits, &mut skein_lib::Reader::new(&converted)).unwrap(), value);
}

#[test]
fn git_identity_with_a_field_the_worker_lacks_is_refused() {
    use jig_protocol_worker_wire::layouts::{Cause, Nested};
    let text = wire::TEXT.replace("email: text 256", "worker_mark: u32 = 17");
    let worker = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let value = crate::identity(&fixtures::identity(b"token"));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    let problem = peer(&worker).down(Nested::Carried(Carried::Grant), value.expose(), &mut out).unwrap_err();
    assert_eq!(problem.cause, Cause::Field);
    assert_eq!(&*problem.kept(), b"the worker's layout cannot keep temper-worker-wire Identity.email: field");
    assert_eq!(out.as_slice(), []);
}
