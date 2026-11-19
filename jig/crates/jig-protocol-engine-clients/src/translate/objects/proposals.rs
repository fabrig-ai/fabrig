//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Writer};
pub fn amendment(
    models: &[crate::Model],
    value: &d::AmendmentState,
    limits: &w::Limits,
) -> Result<w::AmendmentState, w::Problem> {
    let mut dependencies = List::with_capacity(value.dependencies.len());
    for row in &value.dependencies {
        dependencies.push(*row).expect("supplied rows fit");
    }

    Ok(w::AmendmentState {
        addressed: match value.addressed {
            Some(item) => Some(chips::address(item, limits)?),
            None => None,
        },
        taken_by: value.taken_by,
        instructions: match &value.instructions {
            Some(item) => Some(owner::text(item, limits)?),
            None => None,
        },
        spec: owner::specification(&value.spec, limits)?,
        authority: owner::authority(models, &value.authority, limits)?,
        wake: owner::wake(value.wake, limits)?,
        dependencies,
        cut: value.cut,
    })
}
pub fn kind(value: t::ProposalKind) -> w::MessageProposalKind {
    match value {
        t::ProposalKind::Settings => w::MessageProposalKind::Settings,
        t::ProposalKind::Batch => w::MessageProposalKind::Batch,
        t::ProposalKind::Effect => w::MessageProposalKind::Effect,
        t::ProposalKind::Amend => w::MessageProposalKind::Amend,
        t::ProposalKind::Widen => w::MessageProposalKind::Widen,
        t::ProposalKind::Release => w::MessageProposalKind::Release,
    }
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub fn holder(value: t::ProposalHolder, _limits: &w::Limits) -> Result<w::HistoryHolder, w::Problem> {
    Ok(match value {
        t::ProposalHolder::Task(value) => w::HistoryHolder::Task(w::HistoryHolderTask { value }),
        t::ProposalHolder::Person(value) => w::HistoryHolder::Person(w::HistoryHolderPerson { value }),
        t::ProposalHolder::Policy { project, kind: value } => {
            w::HistoryHolder::Policy(w::HistoryHolderPolicy { project, kind: kind(value) })
        }
    })
}
pub fn proposed(
    models: &[crate::Model],
    value: &d::ProposedTask,
    limits: &w::Limits,
) -> Result<w::ProposedTask, w::Problem> {
    let mut dependencies = List::with_capacity(value.dependencies.len());
    for row in &value.dependencies {
        dependencies.push(*row).expect("supplied rows fit");
    }
    let mut resources = List::with_capacity(value.resources.len());
    for row in &value.resources {
        resources.push(owner::part(row, limits)?).expect("supplied rows fit");
    }
    let resources = part_pieces(&resources, limits.proposed_task_resources, w::paths::PROPOSED_TASK_RESOURCES, limits)?;

    Ok(w::ProposedTask {
        deadline: match value.deadline {
            jig_domain_tasks::Deadline::Named => w::NewDeadline::Named,
            jig_domain_tasks::Deadline::Span(value) => w::NewDeadline::Span(w::DeadlineSpan { value }),
        },
        number: value.number,
        spec: owner::specification(&value.spec, limits)?,
        executor: owner::executor(value.executor, limits)?,
        contract: owner::contract(&value.contract, limits)?,
        authority: owner::authority(models, &value.authority, limits)?,
        budget: value.budget,
        funder: owner::funder(value.funder, limits)?,
        dependencies,
        resources,
        wake: owner::wake(value.wake, limits)?,
        priority: value.priority,
        cut: value.cut,
    })
}
pub fn action(
    models: &[crate::Model],
    value: &d::ProposalAction,
    limits: &w::Limits,
) -> Result<w::ProposalAction, w::Problem> {
    Ok(match value {
        d::ProposalAction::Settings { against, items, review, made } => {
            w::ProposalAction::Settings(Box::new(w::ProposalActionSettings {
                against: *against,
                items: super::super::settings::item_heads(items, limits)?,
                made: *made,
                review: match review {
                    Some(review) => Some(super::super::settings::review(review, models, limits)?),
                    None => None,
                },
            }))
        }
        d::ProposalAction::Batch(value) => {
            let mut rows = List::with_capacity(value.len());
            for row in value {
                rows.push(proposed(models, row, limits)?).expect("supplied rows fit");
            }
            let rows = proposed_task_pieces(
                &rows,
                limits.proposal_action_batch_value,
                w::paths::PROPOSAL_ACTION_BATCH_VALUE,
                limits,
            )?;
            w::ProposalAction::Batch(w::ProposalActionBatch { value: rows })
        }
        d::ProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::ProposalAction::Effect(w::ProposalActionEffect {
                part: owner::part(part, limits)?,
                authority: owner::authority(models, authority, limits)?,
                attempt: *attempt,
                completion: *completion,
                position: *position,
                purpose: *purpose,
            })
        }
        d::ProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::ProposalAction::ProcedureEffect(w::ProposalActionProcedureEffect {
                part: owner::part(part, limits)?,
                authority: owner::authority(models, authority, limits)?,
                step: *step,
                purpose: *purpose,
            })
        }
        d::ProposalAction::Amend { task: target, addressed, before, after, reason, release } => {
            w::ProposalAction::Amend(Box::new(w::ProposalActionAmend {
                task: task::link(target, limits)?,
                addressed: match *addressed {
                    Some(item) => Some(w::AddressChange {
                        revision: w::Revision { value: item.revision },
                        address: chips::address(item.address, limits)?,
                    }),
                    None => None,
                },
                before: amendment(models, before, limits)?,
                after: amendment(models, after, limits)?,
                reason: owner::text(reason, limits)?,
                release: *release,
            }))
        }
        d::ProposalAction::Widen { task: target, before, after } => w::ProposalAction::Widen(w::ProposalActionWiden {
            task: task::link(target, limits)?,
            before: owner::authority(models, before, limits)?,
            after: owner::authority(models, after, limits)?,
        }),
        d::ProposalAction::Release { task: target, hold } => w::ProposalAction::Release(w::ProposalActionRelease {
            task: task::link(target, limits)?,
            hold: match hold {
                Some(item) => Some(chips::held(item, limits)?),
                None => None,
            },
        }),
    })
}
pub fn proposal(models: &[crate::Model], value: &d::Proposal, limits: &w::Limits) -> Result<w::Proposal, w::Problem> {
    let mut funding = List::with_capacity(value.funding.len());
    for row in &value.funding {
        funding
            .push(w::FundingOffer {
                source: owner::funder(row.source, limits)?,
                needed: row.needed,
                left: row.left,
                after: row.after,
            })
            .expect("supplied rows fit");
    }

    Ok(w::Proposal {
        projects: value.projects.clone(),
        key: task::proposal(&d::ProposalRef { proposer: value.proposer, number: value.number }, limits)?,
        project: value.project,
        reason: owner::text(&value.reason, limits)?,
        action: action(models, &value.action, limits)?,
        as_holder: value.as_holder,
        waiting: w::ProposalWaiting {
            holder: holder(value.waiting.holder, limits)?,
            since: value.waiting.since.as_nanos(),
        },
        funding,
        acts: task::offered(models, &value.acts, limits, limits.proposal_acts, w::paths::PROPOSAL_ACTS)?,
    })
}
pub(super) fn proposed_task_pieces(
    rows: &List<w::ProposedTask>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn part_pieces(
    rows: &List<w::Part>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
