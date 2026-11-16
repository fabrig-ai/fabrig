//! Temper's channel parts (temper/docs/design/protocol/wires.md, 6).
use crate::{patches, v1, wire};
use jig_protocol_worker_wire::{Application, Carried, Kept, Root, Spoken};
use skein_codec::bridge::BridgeError;
use skein_lib::List;

/// The worker wire's layouts, from this build's floor.
#[must_use]
pub fn spoken() -> Spoken {
    Spoken::kept(wire::FAMILY, wire::FLOOR, patches::kept::temper_worker_wire::LAYOUTS)
}

/// The same kept wire and roots for the engine's workers and the worker's link.
pub fn application(limits: &v1::Limits) -> Result<Application, BridgeError> {
    v1::check_limits(limits).or(Err(BridgeError::Room))?;
    let mut spoken_wires = List::with_capacity(1);
    spoken_wires.push(spoken()).expect("one application wire");
    let mut values = List::uncapped();
    for value in limits.values() {
        values.push(value).expect("uncapped limit values");
    }
    let mut kept = List::with_capacity(1);
    kept.push(Kept {
        family: wire::FAMILY,
        texts: patches::kept::temper_worker_wire::TEXTS,
        patches: patches::PATCHES,
        floor: wire::FLOOR,
        sites: patches::sites,
        names: &v1::LIMIT_NAMES,
        values,
    })
    .expect("one application kept wire");
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
    Ok(Application { spoken: spoken_wires, kept, roots })
}
