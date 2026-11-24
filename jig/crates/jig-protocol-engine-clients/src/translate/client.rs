//! Client map adapters for owner projections and piece-list refusal paths (skein/docs/design/data/codec/maps.md, 9).
use crate::generated::client as mapped;
use jig_client_protocol_wire as w;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;

pub(crate) fn dependency_added_to_record(value: &t::Added) -> w::DependencyAdded {
    w::DependencyAdded { at: skein_codec::convert::wall_to_record(&value.at), by: super::owner::requester(value.by) }
}

pub(crate) fn hold_escalation_to_record(value: &d::EscalationState) -> w::HoldEscalation {
    super::chips::escalation(value)
}

pub(crate) fn circle_to_record(value: &t::Circle) -> w::Circle {
    let mut tasks = skein_lib::List::with_capacity(u32::try_from(value.tasks.len()).expect("bounded circle"));
    for task in &value.tasks {
        tasks.push(*task).expect("same circle");
    }
    w::Circle { tasks, cut: value.cut }
}

// The owner reports a piece-list count at the enclosing document, not its field.
pub(super) fn verdict_choice(value: &t::Verdict) -> Result<w::VerdictChoice, w::Problem> {
    match mapped::verdict_choice_to_record(value) {
        Ok(record) => Ok(record),
        Err(problem) => Err(super::problem(problem.reason)),
    }
}

pub(super) fn grant(value: &t::Grant) -> Result<w::Grant, w::Problem> {
    match mapped::grant_to_record(value) {
        Ok(record) => Ok(record),
        Err(problem) => Err(super::problem(problem.reason)),
    }
}

pub(super) fn resource_scope(value: &t::ResourceScope) -> Result<w::ResourceScope, w::Problem> {
    match mapped::resource_scope_to_record(value) {
        Ok(record) => Ok(record),
        Err(problem) => Err(super::problem(problem.reason)),
    }
}
