//! Temper's channel parts (temper/docs/design/protocol/wires.md, 6).
use crate::{patches, v1, wire};
use jig_protocol_worker_wire::{Application, Carried, Root, Spoken};
use skein_codec::{bridge::BridgeError, patch::Capacity, schema::DeclIx};
use skein_lib::List;

/// The worker wire's layouts, from this build's floor.
#[must_use]
pub fn spoken() -> Spoken {
    Spoken::of(wire::FAMILY, patches::kept::temper_worker_wire::TEXTS, wire::FLOOR).expect("temper's kept worker wire")
}

/// The same bridge and roots for the engine's workers and the worker's link.
pub fn application(limits: &v1::Limits) -> Result<Application, BridgeError> {
    let mut spoken_wires = List::with_capacity(1);
    spoken_wires.push(spoken()).expect("one application wire");
    let mut bridges = List::with_capacity(1);
    bridges.push(patches::kept::temper_worker_wire::bridge(capacity(limits)?)?).expect("one application bridge");
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

fn capacity(limits: &v1::Limits) -> Result<Capacity, BridgeError> {
    let texts = patches::kept::temper_worker_wire::TEXTS;
    let mut bytes = 0_u32;
    for text in texts {
        let schema = skein_codec::schema::parse(text.as_bytes()).expect("temper's kept worker schema");
        for declaration in 0..schema.declarations() {
            let bound = schema.most_bytes_within(DeclIx::new(declaration), &v1::LIMIT_NAMES, &limits.values());
            bytes = bytes.max(u32::try_from(bound.ok_or(BridgeError::Room)?).or(Err(BridgeError::Room))?);
        }
    }
    // As in jig's translation, eight nodes per encoded byte covers records,
    // fields, and list entries, including the unencoded enclosing nodes.
    Ok(Capacity {
        versions: u32::try_from(texts.len()).or(Err(BridgeError::Room))?,
        patches: u32::try_from(patches::PATCHES.len())
            .or(Err(BridgeError::Room))?
            .checked_mul(2)
            .ok_or(BridgeError::Room)?,
        text: skein_codec::schema::MAX_TEXT,
        bytes,
        nodes: bytes.checked_mul(8).ok_or(BridgeError::Room)?,
    })
}
