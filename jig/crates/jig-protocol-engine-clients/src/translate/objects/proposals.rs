//! Whole single-read owner documents (protocol/client.md, 3 and 6).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
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

    w::AmendmentState::new(
        limits,
        w::AmendmentStateParts {
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
        },
    )
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
pub fn holder(value: t::ProposalHolder, limits: &w::Limits) -> Result<w::HistoryHolder, w::Problem> {
    Ok(match value {
        t::ProposalHolder::Task(value) => {
            w::HistoryHolder::Task(w::HistoryHolderTask::new(limits, w::HistoryHolderTaskParts { value })?)
        }
        t::ProposalHolder::Person(value) => {
            w::HistoryHolder::Person(w::HistoryHolderPerson::new(limits, w::HistoryHolderPersonParts { value })?)
        }
        t::ProposalHolder::Policy { project, kind: value } => w::HistoryHolder::Policy(w::HistoryHolderPolicy::new(
            limits,
            w::HistoryHolderPolicyParts { project, kind: kind(value) },
        )?),
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
    let resources = part_pieces(&resources, limits.proposed_task_resources, w::Path::ProposedTaskResources)?;

    w::ProposedTask::new(
        limits,
        w::ProposedTaskParts {
            deadline: match value.deadline {
                jig_domain_tasks::Deadline::Named => w::NewDeadline::Named,
                jig_domain_tasks::Deadline::Span(value) => {
                    w::NewDeadline::Span(w::DeadlineSpan::new(limits, w::DeadlineSpanParts { value })?)
                }
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
        },
    )
}
pub fn action(
    models: &[crate::Model],
    value: &d::ProposalAction,
    limits: &w::Limits,
) -> Result<w::ProposalAction, w::Problem> {
    Ok(match value {
        d::ProposalAction::Settings { against, items, review, made } => {
            w::ProposalAction::Settings(Box::new(w::ProposalActionSettings::new(
                limits,
                w::ProposalActionSettingsParts {
                    against: *against,
                    items: super::super::settings::item_heads(items, limits)?,
                    made: *made,
                    review: match review {
                        Some(review) => Some(super::super::settings::review(review, models, limits)?),
                        None => None,
                    },
                },
            )?))
        }
        d::ProposalAction::Batch(value) => {
            let mut rows = List::with_capacity(value.len());
            for row in value {
                rows.push(proposed(models, row, limits)?).expect("supplied rows fit");
            }
            let rows =
                proposed_task_pieces(&rows, limits.proposal_action_batch_value, w::Path::ProposalActionBatchValue)?;
            w::ProposalAction::Batch(w::ProposalActionBatch::new(limits, w::ProposalActionBatchParts { value: rows })?)
        }
        d::ProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::ProposalAction::Effect(w::ProposalActionEffect::new(
                limits,
                w::ProposalActionEffectParts {
                    part: owner::part(part, limits)?,
                    authority: owner::authority(models, authority, limits)?,
                    attempt: *attempt,
                    completion: *completion,
                    position: *position,
                    purpose: *purpose,
                },
            )?)
        }
        d::ProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::ProposalAction::ProcedureEffect(w::ProposalActionProcedureEffect::new(
                limits,
                w::ProposalActionProcedureEffectParts {
                    part: owner::part(part, limits)?,
                    authority: owner::authority(models, authority, limits)?,
                    step: *step,
                    purpose: *purpose,
                },
            )?)
        }
        d::ProposalAction::Amend { task: target, addressed, before, after, reason, release } => {
            w::ProposalAction::Amend(Box::new(w::ProposalActionAmend::new(
                limits,
                w::ProposalActionAmendParts {
                    task: task::link(target, limits)?,
                    addressed: match *addressed {
                        Some(item) => Some(w::AddressChange::new(
                            limits,
                            w::AddressChangeParts {
                                revision: w::Revision::new(limits, w::RevisionParts { value: item.revision })?,
                                address: chips::address(item.address, limits)?,
                            },
                        )?),
                        None => None,
                    },
                    before: amendment(models, before, limits)?,
                    after: amendment(models, after, limits)?,
                    reason: owner::text(reason, limits)?,
                    release: *release,
                },
            )?))
        }
        d::ProposalAction::Widen { task: target, before, after } => {
            w::ProposalAction::Widen(w::ProposalActionWiden::new(
                limits,
                w::ProposalActionWidenParts {
                    task: task::link(target, limits)?,
                    before: owner::authority(models, before, limits)?,
                    after: owner::authority(models, after, limits)?,
                },
            )?)
        }
        d::ProposalAction::Release { task: target, hold } => w::ProposalAction::Release(w::ProposalActionRelease::new(
            limits,
            w::ProposalActionReleaseParts {
                task: task::link(target, limits)?,
                hold: match hold {
                    Some(item) => Some(chips::held(item, limits)?),
                    None => None,
                },
            },
        )?),
    })
}
pub fn proposal(models: &[crate::Model], value: &d::Proposal, limits: &w::Limits) -> Result<w::Proposal, w::Problem> {
    if let d::Proposer::Party(_) = value.proposer
        && let d::ProposalAction::Amend { .. } = value.action
    {
        return Err(super::super::problem(skein_codec::Reason::Malformed));
    }
    let mut funding = List::with_capacity(value.funding.len());
    for row in &value.funding {
        funding
            .push(w::FundingOffer::new(
                limits,
                w::FundingOfferParts {
                    source: owner::funder(row.source, limits)?,
                    needed: row.needed,
                    left: row.left,
                    after: row.after,
                },
            )?)
            .expect("supplied rows fit");
    }

    w::Proposal::new(
        limits,
        w::ProposalParts {
            key: task::proposal(&d::ProposalRef { proposer: value.proposer, number: value.number }, limits)?,
            project: value.project,
            reason: owner::text(&value.reason, limits)?,
            action: action(models, &value.action, limits)?,
            as_holder: value.as_holder,
            waiting: w::ProposalWaiting::new(
                limits,
                w::ProposalWaitingParts {
                    holder: holder(value.waiting.holder, limits)?,
                    since: value.waiting.since.as_nanos(),
                },
            )?,
            funding,
            acts: task::offered(models, &value.acts, limits, limits.proposal_acts, w::Path::ProposalActs)?,
        },
    )
}
pub(super) fn proposed_task_pieces(
    rows: &List<w::ProposedTask>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn part_pieces(rows: &List<w::Part>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
