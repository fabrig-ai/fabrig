//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::Limits;
use crate::translate::{piece_problem, too_large};
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v22 as w;
use skein_lib::Map;
use skein_lib::Reader;
mod chips;
mod holds;
mod parties;
mod parts;
mod runs;
mod wake;
use chips::{
    chip_details_fields, chip_ending_fields, chip_fields, chip_phase_fields, closing_fields, ending_fields,
    result_value_fields,
};
#[expect(unused_imports, reason = "The facts root keeps its translation names reachable.")]
pub(crate) use chips::{chip_phase, closing_stage};
pub(crate) use holds::{chip_hold, failure_class, hold_category, retry_notice};
use holds::{held_runtime_fields, hold_reason_fields};
pub(crate) use parties::{party, person_address, person_wait, recipient, requester};
pub(crate) use parts::{
    budget, dependencies, document_text, executor, funder, numbers, part, proposal_key, resource_line,
};
pub(crate) use runs::{active_run, call_identity, run_stage};
pub(crate) use wake::wake_policy;

#[derive(Debug)]
struct Child {
    parent: Option<u32>,
    value: Node,
}
#[derive(Debug)]
pub(crate) enum Node {
    Chip(Box<c::Chip>),
    ChipPhaseFacts(Box<c::ChipPhaseFacts>),
    Closing(Box<c::Closing>),
    Ending(Box<c::Ending>),
    HeldRuntime(Box<c::HeldRuntime>),
    HoldReason(Box<c::HoldReason>),
    ChipDetails(Box<c::ChipDetails>),
    ChipEnding(Box<c::ChipEnding>),
    ResultValue(Box<c::ResultValue>),
}
fn take_chip(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::Chip, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::Chip(value) => Ok(*value),
        Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_chip_phase(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::ChipPhaseFacts, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::ChipPhaseFacts(value) => Ok(*value),
        Node::Chip(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_closing(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::Closing, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::Closing(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_ending(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::Ending, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::Ending(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
pub(crate) fn ending(value: &w::Ending, limits: &Limits) -> Result<c::Ending, w::Problem> {
    let root = decode(value.rows(), limits)?;
    match root {
        Node::Ending(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
pub(crate) fn hold_reason(value: &w::HoldReason, limits: &Limits) -> Result<c::HoldReason, w::Problem> {
    let root = decode(value.rows(), limits)?;
    match root {
        Node::HoldReason(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
fn take_owner_chip_details(
    index: u32,
    parent: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipDetails, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::ChipDetails(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_owner_chip_ending(
    index: u32,
    parent: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipEnding, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::ChipEnding(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_result_value(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::ResultValue, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::ResultValue(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhaseFacts(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::ChipDetails(_)
        | Node::ChipEnding(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn decode(bytes: &[u8], limits: &Limits) -> Result<Node, w::Problem> {
    let mut reader = Reader::new(bytes);
    let bound = limits.fact_rows();
    let mut rows = Map::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = super::next_piece(&mut reader, &mut sum, limits)? else { break };
        let row = w::Fact::decode(limits.receiving(), &mut Reader::new(&item))?;
        if rows.insert(rows.len(), row).is_err() {
            return Err(too_large());
        }
    }
    if !reader.is_empty() || rows.is_empty() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    let mut children = Map::with_capacity(rows.len());
    let mut root = None;
    for _ in 0..rows.len() {
        let index = rows.len().checked_sub(1).expect("fact row exists");
        let row = rows.remove(&index).expect("fact row exists");
        let (parent, value) = match row {
            w::Fact::Chip(row) => {
                (*row.parent(), Node::Chip(Box::new(chip_fields(row.value(), limits, index, &mut children)?)))
            }
            w::Fact::ChipPhase(row) => {
                (*row.parent(), Node::ChipPhaseFacts(Box::new(chip_phase_fields(row.value(), index, &mut children)?)))
            }
            w::Fact::Closing(row) => {
                (*row.parent(), Node::Closing(Box::new(closing_fields(row.value(), index, &mut children)?)))
            }
            w::Fact::Ending(row) => {
                (*row.parent(), Node::Ending(Box::new(ending_fields(row.value(), limits, index, &mut children)?)))
            }
            w::Fact::HeldRuntime(row) => (
                *row.parent(),
                Node::HeldRuntime(Box::new(held_runtime_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::HoldReason(row) => (
                *row.parent(),
                Node::HoldReason(Box::new(hold_reason_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::ChipDetails(row) => (
                *row.parent(),
                Node::ChipDetails(Box::new(chip_details_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::ChipEnding(row) => (
                *row.parent(),
                Node::ChipEnding(Box::new(chip_ending_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::ResultValue(row) => (
                *row.parent(),
                Node::ResultValue(Box::new(result_value_fields(row.value(), limits, index, &mut children)?)),
            ),
        };
        if index == 0 {
            if parent.is_some() {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            root = Some(value);
        } else {
            if parent.is_none() || parent >= Some(index) {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            children.insert(index, Child { parent, value }).expect("fact capacity checked");
        }
    }
    if !children.is_empty() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    root.ok_or(piece_problem(skein_codec::Reason::Malformed))
}

pub(crate) use chips::prior_phase;
pub(crate) use chips::result_field;
pub(crate) use holds::failure_tries;
