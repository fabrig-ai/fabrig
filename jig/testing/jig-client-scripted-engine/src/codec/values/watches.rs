//! Wire projections of watches facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, pieces, problem, w};
use super::{
    wire_accounts, wire_current_call, wire_deployment, wire_inbox, wire_live_call, wire_me, wire_project,
    wire_proposal, wire_settings_document, wire_task, wire_tasks, wire_tree,
};
pub(in crate::codec) fn wire_delivery_value(
    value: &c::Delivery,
    limits: &w::Limits,
) -> Result<w::DeliveryValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Delivery::Settings(value) => w::DeliveryValue::Settings(wire_settings_document(value, limits)?),
        c::Delivery::Project(value) => w::DeliveryValue::Project(wire_project(value, limits)?),
        c::Delivery::Me(value) => w::DeliveryValue::Me(wire_me(value, limits)?),
        c::Delivery::Task(value) => w::DeliveryValue::Task(Box::new(wire_task(value, limits)?)),
        c::Delivery::Proposal(value) => w::DeliveryValue::Proposal(wire_proposal(value, limits)?),
        c::Delivery::Tree(value) => w::DeliveryValue::Tree(wire_tree(value, limits)?),
        c::Delivery::Chips(value) => w::DeliveryValue::Chips(w::Chips::new(
            limits,
            w::ChipsParts {
                rows: pieces((value).iter().map(|item| facts::chip(item, limits)), w::Chip::measure, w::Chip::encode)?,
            },
        )?),
        c::Delivery::Inbox(value) => w::DeliveryValue::Inbox(wire_inbox(value, limits)?),
        c::Delivery::Tasks(value) => w::DeliveryValue::Tasks(wire_tasks(value, limits)?),
        c::Delivery::Run(_) => return Err(problem()),
        c::Delivery::RunState(value) => w::DeliveryValue::RunState(wire_run_state(value, limits)?),
        c::Delivery::Deployment(value) => w::DeliveryValue::Deployment(wire_deployment(value, limits)?),
        c::Delivery::Accounts(value) => w::DeliveryValue::Accounts(wire_accounts(value, limits)?),
    })
}

pub(in crate::codec) fn wire_run_state(value: &c::RunState, limits: &w::Limits) -> Result<w::RunState, w::Problem> {
    w::RunState::new(
        limits,
        w::RunStateParts {
            task: value.task,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            committed_turn: value.committed_turn,
            writing: value.writing.as_ref().map(|item| wire_run_text(item, limits)).transpose()?,
            call: value.call.as_ref().map(|item| wire_current_call(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_run_text(value: &c::RunText, limits: &w::Limits) -> Result<w::RunText, w::Problem> {
    w::RunText::new(
        limits,
        w::RunTextParts {
            attempt: value.attempt,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            turn: wire_provisional_turn_name(&value.turn, limits)?,
            from: value.from,
            through: value.through,
            text: bytes::copy_of(value.text.bytes()),
            cut: value.cut,
            call: value.call.as_ref().map(|item| wire_live_call(item, limits)).transpose()?,
            writing: value.writing.as_ref().map(|item| wire_writing_call(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_provisional_turn_name(
    value: &c::ProvisionalTurnName,
    limits: &w::Limits,
) -> Result<w::ProvisionalTurnName, w::Problem> {
    w::ProvisionalTurnName::new(
        limits,
        w::ProvisionalTurnNameParts { attempt: value.attempt, number: value.number, retry: value.retry },
    )
}

pub(in crate::codec) fn wire_writing_call(
    value: &c::WritingCall,
    limits: &w::Limits,
) -> Result<w::WritingCall, w::Problem> {
    w::WritingCall::new(
        limits,
        w::WritingCallParts {
            id: bytes::copy_of(value.id.bytes()),
            tool: bytes::copy_of(value.tool.bytes()),
            from: value.from,
            through: value.through,
            input: bytes::copy_of(value.input.bytes()),
            cut: value.cut,
        },
    )
}
