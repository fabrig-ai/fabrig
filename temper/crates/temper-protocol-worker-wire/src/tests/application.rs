//! Temper's declarations are held against its encoders (protocol/wires.md, 6).
use super::fixtures;
use crate::{patches::kept::temper_worker_wire as kept, v1, wire};
use jig_protocol_worker_wire::{Carried, Spoken};
use skein_lib::List;

#[test]
fn the_application_speaks_from_the_floor_with_its_bridge() {
    let application = crate::application(&fixtures::limits()).expect("application");
    let description = application.kept.get(0).unwrap();
    assert_eq!(description.family, wire::FAMILY);
    assert_eq!(description.texts, kept::TEXTS);
    assert_eq!(description.patches, crate::patches::PATCHES);
    assert_eq!(description.floor, kept::FLOOR);
    assert_eq!(description.names, v1::LIMIT_NAMES);
    assert_eq!(description.values.as_slice(), fixtures::limits().values());
    assert_eq!(application.spoken.get(0), Some(&crate::spoken()));
    assert_eq!(crate::spoken(), Spoken::of(wire::FAMILY, kept::TEXTS, wire::FLOOR).expect("parsed kept range"));
    assert_eq!(wire::FLOOR, kept::FLOOR);
    assert_eq!(application.spoken.len(), 1);
    assert_eq!(application.kept.len(), 1);
    let spoken = application.spoken.get(0).unwrap();
    let bridge = application.kept.get(0).unwrap().bridge().unwrap();
    assert_eq!(spoken.name, wire::FAMILY);
    assert_eq!(spoken.lowest, wire::FLOOR);
    assert_eq!(spoken.highest(), wire::VERSION);
    assert_eq!(bridge.family(), wire::FAMILY.as_bytes());
    assert_eq!(bridge.floor(), wire::FLOOR);
    assert_eq!(bridge.current(), wire::VERSION);
    for text in kept::TEXTS {
        let schema = skein_codec::schema::parse(text.as_bytes()).unwrap();
        if schema.version() >= wire::FLOOR {
            let digest = schema.fingerprints().family().digest();
            assert_eq!(spoken.layout(schema.version()), Some(digest));
            assert_eq!(bridge.layout(schema.version()), Some(digest));
        }
    }
}

#[test]
fn the_roots_name_the_declarations_temper_encodes() {
    let limits = fixtures::limits();
    let items = v1::Items::new(&limits, fixtures::items(1)).unwrap();
    let mut landed = List::with_capacity(1);
    landed
        .push(v1::Landed::new(&limits, v1::LandedParts { tag: 17, commit: [3; 32], checked: true }).unwrap())
        .unwrap();
    let left = v1::Left::new(&limits, v1::LeftParts { landed }).unwrap();
    let mut repositories = List::with_capacity(1);
    repositories
        .push(
            v1::SavedRepository::new(
                &limits,
                v1::SavedRepositoryParts { tag: 19, from_saved: true, landing: v1::Landing::Moved },
            )
            .unwrap(),
        )
        .unwrap();
    let saved = v1::Saved::new(&limits, v1::SavedParts { repositories }).unwrap();
    let preparation = v1::Preparation::Refused(v1::Tag::new(&limits, v1::TagParts { tag: 23 }).unwrap());
    let identity = crate::identity(&fixtures::identity(b"token"));
    let application = crate::application(&limits).unwrap();
    let schema = skein_codec::schema::parse(kept::TEXTS.last().unwrap().as_bytes()).unwrap();
    for (carried, declaration, bytes) in [
        (Carried::Items, "Items", crate::items(&items)),
        (Carried::Left, "Left", crate::left(&left)),
        (Carried::Saved, "Saved", crate::saved(&saved)),
        (Carried::Preparation, "Preparation", crate::preparation(&preparation)),
        (Carried::Grant, "Identity", identity.expose().into()),
        (Carried::Ask, "Ask", crate::ask(&v1::Ask::Reds)),
        (
            Carried::Answer,
            "Answer",
            crate::answer(&v1::Answer::Reds(
                v1::Reds::new(&limits, v1::RedsParts { reds: List::with_capacity(0) }).unwrap(),
            )),
        ),
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
fn left_carries_checked_deliveries_across_six_and_seven() {
    let limits = fixtures::limits();
    let application = crate::application(&limits).unwrap();
    let mut bridge = application.kept.get(0).unwrap().bridge().unwrap();
    let mut old = skein_lib::Writer::new(44);
    old.put(&1_u32.to_be_bytes()).unwrap();
    old.put(&17_u64.to_be_bytes()).unwrap();
    old.put(&[3; 32]).unwrap();
    let old = old.finish();
    let up = bridge.translate(6, 7, b"Left", old.clone()).unwrap();
    let left = v1::Left::decode(&limits, &mut skein_lib::Reader::new(&up)).unwrap();
    let head = left.landed().get(0).unwrap();
    assert_eq!((head.tag(), head.commit(), head.checked()), (17, &[3; 32], true));
    assert_eq!(bridge.translate(7, 6, b"Left", up).unwrap().as_ref(), old.as_ref());
    let mut landed = List::with_capacity(1);
    landed
        .push(v1::Landed::new(&limits, v1::LandedParts { tag: 17, commit: [3; 32], checked: false }).unwrap())
        .unwrap();
    let unchecked = v1::Left::new(&limits, v1::LeftParts { landed }).unwrap();
    let Err(skein_codec::patch::Halt::Unheld(_)) = bridge.translate(7, 6, b"Left", crate::left(&unchecked)) else {
        panic!("unchecked delivery cannot be read as checked")
    };
}

#[test]
fn identity_goes_down_whole_to_every_spoken_version() {
    let application = crate::application(&fixtures::limits()).unwrap();
    let bridge = application.kept.get(0).unwrap().bridge().unwrap();
    // Also check the current root, so a misspelled declaration cannot pass
    // when no older version is spoken.
    assert_eq!(bridge.total(wire::VERSION, b"Identity"), Some(true));
    for version in bridge.floor()..bridge.current() {
        assert_eq!(bridge.total(version, b"Identity"), Some(true), "Identity to {version}");
    }
}

#[test]
fn application_refuses_limits_past_the_wire_ceiling() {
    let mut limits = fixtures::limits();
    limits.red_head = v1::CEILINGS.red_head;
    crate::application(&limits).unwrap();
    limits.red_head = v1::CEILINGS.red_head.checked_add(1).unwrap();
    assert!(v1::check_limits(&limits).is_err());
    assert_eq!(crate::application(&limits).err(), Some(skein_codec::bridge::BridgeError::Room));
}
