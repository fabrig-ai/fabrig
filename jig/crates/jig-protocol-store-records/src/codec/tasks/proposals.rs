//! Proposals and immutable task history (protocol/store/records.md, 1, None).
use super::Context;
use super::value;
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::List;

/// Translate `Amendment` to its store record.
pub fn amendment(
    source: &domain::Amendment,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Amendment, wire::Problem> {
    mapped::amendment_to_record(source, &Context::new(limits, moved))
}
/// Read `Amendment` from its store record.
pub fn read_amendment(source: wire::Amendment, limits: &wire::Limits) -> Result<domain::Amendment, wire::Problem> {
    mapped::amendment_to_domain(source, &Context::new(limits, None))
}

/// Translate `ProposalAction` to its store value.
pub fn action(
    source: &domain::ProposalAction,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::ProposalAction, wire::Problem> {
    Ok(match source {
        domain::ProposalAction::Settings { against, items, projects } => {
            wire::ProposalAction::Settings(setting_heads(*against, items, projects, limits)?)
        }
        domain::ProposalAction::Batch(value) => {
            wire::ProposalAction::Batch(wire::ProposedBatch { members: batch(value, limits, moved)? })
        }
        domain::ProposalAction::Effect { connector, authority, attempt, completion, position, purpose } => {
            wire::ProposalAction::Effect(wire::ProposedEffect {
                connector: *connector,
                authority: value::authority(authority, limits, moved)?,
                attempt: *attempt,
                completion: *completion,
                position: *position,
                purpose: *purpose,
            })
        }
        domain::ProposalAction::ProcedureEffect { connector, authority, step, purpose } => {
            wire::ProposalAction::ProcedureEffect(wire::ProposedProcedureEffect {
                connector: *connector,
                authority: value::authority(authority, limits, moved)?,
                step: *step,
                purpose: *purpose,
            })
        }
        domain::ProposalAction::Amend { task, revision, amendment } => {
            wire::ProposalAction::Amend(Box::new(wire::ProposedAmendment {
                task: *task,
                revision: *revision,
                amendment: mapped::amendment_to_record(amendment, &Context::new(limits, moved))?,
            }))
        }
        domain::ProposalAction::Widen { task, authority } => wire::ProposalAction::Widen(wire::ProposedWidening {
            task: *task,
            authority: value::authority(authority, limits, moved)?,
        }),
        domain::ProposalAction::ReleasePlaced { task, hold, revision } => {
            wire::ProposalAction::ReleasePlaced(wire::ReleasePlaced { task: *task, hold: *hold, revision: *revision })
        }
        domain::ProposalAction::Release { task } => wire::ProposalAction::Release(wire::Task { task: *task }),
    })
}
/// Read `ProposalAction` from its store value.
pub fn read_action(
    source: wire::ProposalAction,
    limits: &wire::Limits,
) -> Result<domain::ProposalAction, wire::Problem> {
    let context = &Context::new(limits, None);
    Ok(match source {
        wire::ProposalAction::Settings(value) => {
            let mut items = List::with_capacity(value.items.len());
            for item in value.items.into_boxed() {
                items.push(mapped::setting_head_to_domain(item)).expect("decoded settings heads");
            }
            domain::ProposalAction::Settings {
                against: value.against,
                items: items.into_boxed(),
                projects: value.projects.into_boxed(),
            }
        }
        wire::ProposalAction::Batch(value) => {
            let wire::ProposedBatch { members } = value;
            domain::ProposalAction::Batch(read_batch(members, limits)?)
        }
        wire::ProposalAction::Effect(value) => {
            let wire::ProposedEffect { connector, authority, attempt, completion, position, purpose } = value;
            domain::ProposalAction::Effect {
                connector,
                authority: value::read_authority(authority, limits)?,
                attempt,
                completion,
                position,
                purpose,
            }
        }
        wire::ProposalAction::ProcedureEffect(value) => {
            let wire::ProposedProcedureEffect { connector, authority, step, purpose } = value;
            domain::ProposalAction::ProcedureEffect {
                connector,
                authority: value::read_authority(authority, limits)?,
                step,
                purpose,
            }
        }
        wire::ProposalAction::Amend(value) => {
            let wire::ProposedAmendment { task, revision, amendment } = *value;
            domain::ProposalAction::Amend {
                task,
                revision,
                amendment: mapped::amendment_to_domain(amendment, context)?,
            }
        }
        wire::ProposalAction::Widen(value) => {
            let wire::ProposedWidening { task, authority } = value;
            domain::ProposalAction::Widen { task, authority: value::read_authority(authority, limits)? }
        }
        wire::ProposalAction::ReleasePlaced(value) => {
            let wire::ReleasePlaced { task, hold, revision } = value;
            domain::ProposalAction::ReleasePlaced { task, hold, revision }
        }
        wire::ProposalAction::Release(value) => {
            let wire::Task { task } = value;
            domain::ProposalAction::Release { task }
        }
    })
}

/// Translate `Proposal` to its store record.
pub fn proposal(
    source: &domain::Proposal,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Proposal, wire::Problem> {
    mapped::proposal_to_record(source, &Context::new(limits, moved))
}
/// Read `Proposal` from its store record.
pub fn read_proposal(source: wire::Proposal, limits: &wire::Limits) -> Result<domain::Proposal, wire::Problem> {
    mapped::proposal_to_domain(source, &Context::new(limits, None))
}

/// Translate `PersonProposal` to its store record.
pub fn person_proposal(
    source: &domain::PersonProposal,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::PersonProposal, wire::Problem> {
    mapped::person_proposal_to_record(source, &Context::new(limits, moved))
}

fn batch(source: &[domain::New], limits: &wire::Limits, moved: Option<&[u64; 64]>) -> Result<Box<[u8]>, wire::Problem> {
    super::pieces::creations(source, limits, limits.proposed_batch_members, wire::paths::PROPOSED_BATCH_MEMBERS, moved)
}
fn read_batch(source: Box<[u8]>, limits: &wire::Limits) -> Result<Box<[domain::New]>, wire::Problem> {
    super::pieces::read_creations(source, limits, wire::paths::PROPOSED_BATCH_MEMBERS)
}
fn setting_heads(
    against: u32,
    items: &[domain::SettingHead],
    projects: &[u32],
    limits: &wire::Limits,
) -> Result<wire::ProposedSettings, wire::Problem> {
    if items.len() > skein_lib::length::usize_of(limits.proposed_settings_items) {
        return Err(wire::Problem { path: wire::paths::PROPOSED_SETTINGS_ITEMS, reason: skein_codec::Reason::Bound });
    }
    let mut heads = List::with_capacity(domain::SETTINGS_ITEMS);
    for item in items {
        heads.push(mapped::setting_head_to_record(item)).expect("bounded heads");
    }
    if projects.len()
        > skein_lib::length::usize_of(limits.proposed_settings_projects.min(wire::CEILINGS.proposed_settings_projects))
    {
        return Err(wire::Problem {
            path: wire::paths::PROPOSED_SETTINGS_PROJECTS,
            reason: skein_codec::Reason::Bound,
        });
    }
    let mut copied = List::with_capacity(wire::CEILINGS.proposed_settings_projects);
    for project in projects {
        copied.push(*project).expect("bounded proposal projects");
    }
    Ok(wire::ProposedSettings { against, items: heads, projects: copied })
}

pub fn person_action_to_record(
    value: &domain::PersonAction,
    context: &Context,
) -> Result<wire::PersonAction, wire::Problem> {
    let limits = &context.limits;
    let moved = context.moved.as_ref();
    Ok(match value {
        domain::PersonAction::Settings { against, items } => {
            wire::PersonAction::Settings(setting_heads(*against, items, &[], limits)?)
        }
        domain::PersonAction::Batch(members) => {
            wire::PersonAction::Batch(wire::ProposedBatch { members: batch(members, limits, moved)? })
        }
        domain::PersonAction::Amend { task, revision, amendment } => {
            wire::PersonAction::Amend(Box::new(wire::ProposedAmendment {
                task: *task,
                revision: *revision,
                amendment: mapped::amendment_to_record(amendment, &Context::new(limits, moved))?,
            }))
        }
    })
}
pub fn person_action_to_domain(
    value: wire::PersonAction,
    context: &Context,
) -> Result<domain::PersonAction, wire::Problem> {
    let limits = &context.limits;
    Ok(match value {
        wire::PersonAction::Settings(row) => {
            let mut items = List::with_capacity(domain::SETTINGS_ITEMS);
            for item in row.items.into_boxed() {
                items.push(mapped::setting_head_to_domain(item)).expect("decoded heads");
            }
            domain::PersonAction::Settings { against: row.against, items: items.into_boxed() }
        }
        wire::PersonAction::Batch(batch) => domain::PersonAction::Batch(read_batch(batch.members, limits)?),
        wire::PersonAction::Amend(value) => {
            let wire::ProposedAmendment { task, revision, amendment } = *value;
            domain::PersonAction::Amend {
                task,
                revision,
                amendment: Box::new(mapped::amendment_to_domain(amendment, &Context::new(limits, None))?),
            }
        }
    })
}
