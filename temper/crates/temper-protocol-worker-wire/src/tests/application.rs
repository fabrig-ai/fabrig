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

#[test]
fn the_roots_name_the_declarations_temper_encodes() {
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
    let application = crate::application(&limits).unwrap();
    let schema = skein_codec::schema::parse(wire::TEXT.as_bytes()).unwrap();
    for (carried, declaration, bytes) in [
        (Carried::Items, "Items", crate::items(&items)),
        (Carried::Left, "Left", crate::left(&left)),
        (Carried::Saved, "Saved", crate::saved(&saved)),
        (Carried::Preparation, "Preparation", crate::preparation(&preparation)),
        (
            Carried::JobSpec,
            "JobSpec",
            v1::JobSpec { command: b"tool".as_slice().into(), request: b"{}".as_slice().into() }
                .encode(&limits)
                .unwrap(),
        ),
        (Carried::Grant, "Identity", identity.expose().into()),
        (Carried::Ask, "Ask", crate::ask(&v1::Ask::Reds)),
        (Carried::Answer, "Answer", crate::answer(&v1::Answer::Reds(v1::Reds { reds: List::with_capacity(0) }))),
    ] {
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
