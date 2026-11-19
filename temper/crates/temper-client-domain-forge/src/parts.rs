//! Task row parts (temper/docs/design/client/forge/parts.md, 5).
use jig_client_protocol_wire::Record as _;
use alloc::boxed::Box;
use jig_client::contract::{self as c, RowPart};
use jig_client_protocol_link as link;
use jig_client_protocol_wire as jig;
use skein_lib::{List, Reader, Wall};
use temper_client_protocol_wire::{
    self as wire,
    v1::{ChatPart, GoalPart, MilestonePart, MilestoneTotals},
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

#[derive(Debug)]
pub struct MilestoneRow {
    pub whole: Option<Wall>,
    pub totals: MilestoneTotals,
    pub goals: List<(c::TaskLine, Option<Box<[u8]>>)>,
    pub more: u32,
}

#[must_use]
pub fn milestone(line: &c::TaskLine, limits: &link::Limits) -> Option<MilestoneRow> {
    let mut reader = Reader::new(value(line)?);
    let part = MilestonePart::decode(&wire::v1::CEILINGS, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let part = part.into_parts();
    let mut goals = List::with_capacity(part.goals.len());
    let mut more = part.more;
    for goal in part.goals.into_boxed() {
        let goal = goal.into_parts();
        if let Some(line) = milestone_line(&goal.line, limits) {
            goals.push((line, goal.headline)).expect("same goal count");
        } else {
            more = more.saturating_add(1);
        }
    }
    Some(MilestoneRow {
        whole: match part.whole {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        totals: part.totals,
        goals,
        more,
    })
}

fn milestone_line(bytes: &[u8], limits: &link::Limits) -> Option<c::TaskLine> {
    let mut reader = Reader::new(bytes);
    let row = jig::TaskLine::decode(limits.wire(), &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    link::translate::queries::line(&row, limits).ok()
}
