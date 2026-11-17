//! Task row parts (temper/docs/design/client/forge/parts.md, 5).
use jig_client::contract::{self as c, RowPart};
use skein_lib::Reader;
use temper_client_protocol_wire::{
    self as wire,
    v1::{ChatPart, GoalPart},
};

#[must_use]
pub const fn asks(kind: u32) -> c::PartAsk {
    c::PartAsk { owner: wire::FORGE, kind, version: wire::CLIENT_VERSION }
}

// Each list asks for its one part in the first slot; a row carries the query's order.
fn value(line: &c::TaskLine) -> Option<&[u8]> {
    match line.parts.as_slice().first()? {
        RowPart::Value(family) => {
            if family.family == wire::FORGE && family.version == wire::CLIENT_VERSION {
                Some(&family.bytes)
            } else {
                None
            }
        }
        RowPart::Unknown | RowPart::NotServed(_) | RowPart::Failed(_) => None,
    }
}

#[must_use]
pub fn goal(line: &c::TaskLine) -> Option<GoalPart> {
    let mut reader = Reader::new(value(line)?);
    let part = GoalPart::decode(&wire::v1::CEILINGS, &mut reader).ok()?;
    if reader.is_empty() { Some(part) } else { None }
}

#[must_use]
pub fn chat(line: &c::TaskLine) -> Option<ChatPart> {
    let mut reader = Reader::new(value(line)?);
    let part = ChatPart::decode(&wire::v1::CEILINGS, &mut reader).ok()?;
    if reader.is_empty() { Some(part) } else { None }
}
