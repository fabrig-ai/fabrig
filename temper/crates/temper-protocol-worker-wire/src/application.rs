//! Temper's channel parts (temper/docs/design/protocol/wires.md, 6).
use crate::{patches, v1, wire};
use jig_protocol_worker_wire::{Application, Carried, Root, Spoken};
use skein_codec::{
    bridge::{Bridge, BridgeError},
    patch::Capacity,
    schema::{DeclIx, Schema},
};
use skein_lib::List;

/// The worker wire's layouts, from this build's floor.
#[must_use]
pub fn spoken() -> Spoken {
    Spoken::of(wire::FAMILY, patches::kept::temper_worker_wire::TEXTS, wire::FLOOR).expect("temper's kept worker wire")
}

/// The same bridge and roots for the engine's workers and the worker's link.
pub fn application(limits: &v1::Limits) -> Result<Application, BridgeError> {
    let (capacity, schemas) = capacity(limits)?;
    let bridge = Bridge::from_schemas(capacity, schemas, patches::PATCHES, wire::FLOOR, patches::sites)?;
    let mut layouts = List::with_capacity(capacity.versions);
    for version in bridge.floor()..=bridge.current() {
        layouts.push(bridge.layout(version).expect("kept application layout")).expect("one layout per version");
    }
    let mut spoken_wires = List::with_capacity(1);
    spoken_wires.push(Spoken { name: wire::FAMILY, lowest: wire::FLOOR, layouts }).expect("one application wire");
    let mut bridges = List::with_capacity(1);
    bridges.push(bridge).expect("one application bridge");
    let mut roots = List::with_capacity(8);
    for (carried, declaration) in [
        (Carried::Items, "Items"),
        (Carried::Left, "Left"),
        (Carried::Saved, "Saved"),
        (Carried::Preparation, "Preparation"),
        (Carried::JobSpec, "JobSpec"),
        (Carried::Grant, "Identity"),
        (Carried::Ask, "Ask"),
        (Carried::Answer, "Answer"),
    ] {
        roots.push(Root { carried, wire: wire::FAMILY, declaration }).expect("eight translated parts");
    }
    Ok(Application { spoken: spoken_wires, bridges, roots })
}

fn capacity(limits: &v1::Limits) -> Result<(Capacity, List<Schema>), BridgeError> {
    let texts = patches::kept::temper_worker_wire::TEXTS;
    let versions = u32::try_from(texts.len()).or(Err(BridgeError::Room))?;
    let mut schemas = List::with_capacity(versions);
    let values = limits.values();
    let mut bytes = 0_u32;
    for text in texts {
        let schema = skein_codec::schema::parse(text.as_bytes()).expect("temper's kept worker schema");
        let bounds = schema.bounds_within(&v1::LIMIT_NAMES, &values);
        for declaration in 0..schema.declarations() {
            let bound = bounds.at(DeclIx::new(declaration));
            bytes = bytes.max(u32::try_from(bound.ok_or(BridgeError::Room)?).or(Err(BridgeError::Room))?);
        }
        schemas.push(schema).expect("one schema per kept text");
    }
    // As in jig's translation, eight nodes per encoded byte covers records,
    // fields, and list entries, including the unencoded enclosing nodes.
    Ok((
        Capacity {
            versions,
            patches: u32::try_from(patches::PATCHES.len())
                .or(Err(BridgeError::Room))?
                .checked_mul(2)
                .ok_or(BridgeError::Room)?,
            text: skein_codec::schema::MAX_TEXT,
            bytes,
            nodes: bytes.checked_mul(8).ok_or(BridgeError::Room)?,
        },
        schemas,
    ))
}
