//! Client map adapters for owner projections and piece-list refusal paths (skein/docs/design/data/codec/maps.md, 9).
use crate::generated::client as mapped;
use alloc::boxed::Box;
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

pub(crate) use super::activity::{activity_to_record, run_end_to_record};
pub(crate) use super::live::writing as run_text_to_record;
pub(crate) use super::owner::specification as specification_document_to_record;

// Context supplies read-only catalogue and limits to owner projections, not to
// the surrounding structural translations.
pub(crate) fn chip_to_record(
    value: &d::Chip,
    (_, limits): &(&[crate::Model], &w::Limits),
) -> Result<w::Chip, w::Problem> {
    super::chips::chip(value, limits)
}

pub(crate) fn chip_ending_to_record(
    value: &d::Ending,
    (_, limits): &(&[crate::Model], &w::Limits),
) -> Result<w::ChipEnding, w::Problem> {
    super::chips::ending(value, limits)
}

pub(crate) fn authority_document_to_record(
    value: &d::AuthorityLine,
    (models, limits): &(&[crate::Model], &w::Limits),
) -> Result<w::AuthorityDocument, w::Problem> {
    super::owner::authority(models, value, limits)
}

pub(crate) fn result_document_to_record(
    value: &d::ResultDoc,
    (models, limits): &(&[crate::Model], &w::Limits),
) -> Result<w::ResultDocument, w::Problem> {
    super::objects::result(models, value, limits)
}

pub(crate) fn retained_action_to_record(
    value: &d::RetainedAction,
    (models, limits): &(&[crate::Model], &w::Limits),
) -> Result<w::RetainedAction, w::Problem> {
    super::objects::ended::retained_action(models, value, limits)
}

pub(crate) fn history_amendment_to_record(
    (before, after): &(Box<d::AmendmentState>, Box<d::AmendmentState>),
    context: &(&[crate::Model], &w::Limits),
) -> Result<w::HistoryAmendment, w::Problem> {
    Ok(w::HistoryAmendment {
        before: mapped::amendment_state_to_record(before, context)?,
        after: mapped::amendment_state_to_record(after, context)?,
    })
}

pub(crate) fn task_line_to_record(
    value: &d::TaskLine,
    (_, limits): &(&[crate::Model], &w::Limits),
) -> Result<w::TaskLine, w::Problem> {
    // Inbox replies have no application part slots.
    super::queries::line(value, limits)
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "client maps pass domain values by reference")]
pub(crate) fn count_to_record(value: &d::Count) -> w::Count {
    super::live::count(*value)
}

pub(crate) fn impact_counts_to_record(value: &d::PhaseCounts) -> w::ImpactCounts {
    w::ImpactCounts {
        ended: 0,
        waiting: value.waiting,
        running: value
            .idle
            .checked_add(value.starting)
            .expect("task bound")
            .checked_add(value.running)
            .expect("task bound")
            .checked_add(value.backing_off)
            .expect("task bound"),
        closing: value.closing,
        held: value.held,
        done: value.done,
        failed: value.failed,
        cancelled: value.cancelled,
    }
}

pub(crate) fn waited_for_to_record(value: &jig_domain_people::EntryKind) -> Result<w::WaitedFor, w::Problem> {
    use jig_domain_people::EntryKind;
    match value {
        EntryKind::Question { .. } => Ok(w::WaitedFor::Question),
        EntryKind::Proposal { .. } => Ok(w::WaitedFor::Proposal),
        EntryKind::Escalation { .. } => Ok(w::WaitedFor::Escalation),
        EntryKind::PersonTask => Ok(w::WaitedFor::PersonTask),
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => Err(super::problem(skein_codec::Reason::Tag)),
    }
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "client maps pass domain values by reference")]
pub(crate) fn task_wait_invalid_to_record(value: &t::AccountWhy) -> w::TaskWaitInvalid {
    // The refusal map for AccountWhy takes people's vocabulary, not tasks'.
    w::TaskWaitInvalid {
        why: match value {
            t::AccountWhy::Unknown => w::AccountWhy::Unknown,
            t::AccountWhy::Provider => w::AccountWhy::Provider,
            t::AccountWhy::Uses => w::AccountWhy::Uses,
            t::AccountWhy::Limited => w::AccountWhy::Limited,
        },
    }
}

pub(crate) fn proposal_decision_created_to_record((first, last): &(u64, u64)) -> w::ProposalDecisionCreated {
    w::ProposalDecisionCreated { first: *first, last: *last }
}
