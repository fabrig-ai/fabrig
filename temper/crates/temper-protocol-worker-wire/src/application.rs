//! Temper's channel parts (temper/docs/design/protocol/wires.md, 6).
use crate::{v1, wire};
use alloc::boxed::Box;
use jig_protocol_worker_wire::{Application, Carried, Root, Spoken};
use skein_codec::layout::{Mode, Refusal, Side};
use skein_lib::{Buffer, List};

/// The worker wire at its current layout.
#[must_use]
pub fn spoken() -> Spoken {
    Spoken { name: wire::FAMILY, version: wire::VERSION, layout: wire::layout(), text: wire::TEXT }
}

/// The same current wire and roots for the engine's workers and the worker's link.
pub fn application(limits: &v1::Limits) -> Result<Application, v1::Problem> {
    v1::check_limits(limits)?;
    let mut spoken_wires = List::with_capacity(1);
    spoken_wires.push(spoken()).expect("one application wire");
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
    Ok(Application { spoken: spoken_wires, roots, written: List::uncapped() })
}

/// Temper's written converters (jig/docs/design/protocol/channel/layouts.md, 7).
#[expect(clippy::disallowed_methods, reason = "the protocol validates parsed schema identifiers")]
pub fn convert(_mode: Mode, from: &Side<'_>, _to: &Side<'_>, _value: &[u8], out: &mut Buffer) -> Result<(), Refusal> {
    out.clear();
    let name = from.text.spelled(from.text.declaration(from.root.decl).name);
    Err(Refusal {
        declaration: Box::from(core::str::from_utf8(name).expect("schema declaration names are ASCII")),
        part: Box::from(""),
        why: Box::from("temper has no written converter for this layout pair"),
    })
}
