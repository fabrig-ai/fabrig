//! Durable people refusal facts (protocol/store/records.md, 1).
use super::{domain, wire};
use skein_lib::Wall;
pub(super) fn encode_again(value: domain::Again) -> wire::Again {
    match value {
        domain::Again::Busy => wire::Again::Busy,
        // Admission-only facts are never retained in the store.
        domain::Again::Full { .. } => unreachable!("answers-full is never a stored outcome"),
        domain::Again::NotReady => wire::Again::NotReady,
    }
}
pub(super) fn decode_again(value: wire::Again) -> domain::Again {
    match value {
        wire::Again::Busy => domain::Again::Busy,
        wire::Again::NotReady => domain::Again::NotReady,
    }
}
pub(super) fn encode_failure_class(value: domain::FailureClass) -> wire::FailureClass {
    match value {
        domain::FailureClass::Transient => wire::FailureClass::Transient,
        domain::FailureClass::Permanent => wire::FailureClass::Permanent,
        domain::FailureClass::Run => wire::FailureClass::Run,
        domain::FailureClass::Agent => wire::FailureClass::Agent,
        domain::FailureClass::Lost => wire::FailureClass::Lost,
        domain::FailureClass::Invalid => wire::FailureClass::Invalid,
        domain::FailureClass::Infrastructure => wire::FailureClass::Infrastructure,
    }
}
pub(super) fn decode_failure_class(value: wire::FailureClass) -> domain::FailureClass {
    match value {
        wire::FailureClass::Transient => domain::FailureClass::Transient,
        wire::FailureClass::Permanent => domain::FailureClass::Permanent,
        wire::FailureClass::Run => domain::FailureClass::Run,
        wire::FailureClass::Agent => domain::FailureClass::Agent,
        wire::FailureClass::Lost => domain::FailureClass::Lost,
        wire::FailureClass::Invalid => domain::FailureClass::Invalid,
        wire::FailureClass::Infrastructure => domain::FailureClass::Infrastructure,
    }
}
pub(super) fn encode_hold_reason(value: &domain::HoldReason, _limits: &wire::Limits) -> wire::HoldReason {
    match value {
        domain::HoldReason::Exceeded(value) => wire::HoldReason::Exceeded(wire::Exceeded {
            bound: match value.bound {
                domain::Bound::Reasoning => wire::Bound::Reasoning,
                domain::Bound::Metadata => wire::Bound::Metadata,
                domain::Bound::Items => wire::Bound::Items,
                domain::Bound::Answer => wire::Bound::Answer,
                domain::Bound::Tools => wire::Bound::Tools,
                domain::Bound::HistoryItems => wire::Bound::HistoryItems,
                domain::Bound::History => wire::Bound::History,
                domain::Bound::Window => wire::Bound::Window,
                domain::Bound::Output => wire::Bound::Output,
            },
            value: value.value,
        }),
        domain::HoldReason::Failures { class } => {
            wire::HoldReason::Failures(wire::HoldReasonFailures { class: encode_failure_class(*class) })
        }
        domain::HoldReason::Stalled => wire::HoldReason::Stalled,
        domain::HoldReason::Dependency { task } => {
            wire::HoldReason::Dependency(wire::HoldReasonDependency { task: *task })
        }
        domain::HoldReason::Stopped => wire::HoldReason::Stopped,
        domain::HoldReason::Placed { hold } => wire::HoldReason::Placed(wire::HoldReasonPlaced { hold: *hold }),
        domain::HoldReason::Drift => wire::HoldReason::Drift,
        domain::HoldReason::Effects => wire::HoldReason::Effects,
        domain::HoldReason::EffectFailed => wire::HoldReason::EffectFailed,
        domain::HoldReason::Uncertain { entry } => {
            wire::HoldReason::Uncertain(wire::HoldReasonUncertain { entry: *entry })
        }
        domain::HoldReason::Budget => wire::HoldReason::Budget,
        domain::HoldReason::Deadline => wire::HoldReason::Deadline,
        domain::HoldReason::HoldsWaited => wire::HoldReason::HoldsWaited,
        domain::HoldReason::ResourceUnavailable => wire::HoldReason::ResourceUnavailable,
        domain::HoldReason::ResourceAccess => wire::HoldReason::ResourceAccess,
        domain::HoldReason::PoolLost { pool } => wire::HoldReason::PoolLost(wire::HoldReasonPoolLost { pool: *pool }),
        domain::HoldReason::Procedure => wire::HoldReason::Procedure,
        domain::HoldReason::Circle => wire::HoldReason::Circle,
    }
}
pub(super) fn decode_hold_reason(value: wire::HoldReason) -> domain::HoldReason {
    match value {
        wire::HoldReason::Exceeded(value) => domain::HoldReason::Exceeded(domain::Exceeded {
            bound: match value.bound {
                wire::Bound::Reasoning => domain::Bound::Reasoning,
                wire::Bound::Metadata => domain::Bound::Metadata,
                wire::Bound::Items => domain::Bound::Items,
                wire::Bound::Answer => domain::Bound::Answer,
                wire::Bound::Tools => domain::Bound::Tools,
                wire::Bound::HistoryItems => domain::Bound::HistoryItems,
                wire::Bound::History => domain::Bound::History,
                wire::Bound::Window => domain::Bound::Window,
                wire::Bound::Output => domain::Bound::Output,
            },
            value: value.value,
        }),
        wire::HoldReason::Failures(value) => {
            let wire::HoldReasonFailures { class } = value;
            domain::HoldReason::Failures { class: decode_failure_class(class) }
        }
        wire::HoldReason::Stalled => domain::HoldReason::Stalled,
        wire::HoldReason::Dependency(value) => {
            let wire::HoldReasonDependency { task } = value;
            domain::HoldReason::Dependency { task }
        }
        wire::HoldReason::Stopped => domain::HoldReason::Stopped,
        wire::HoldReason::Placed(value) => {
            let wire::HoldReasonPlaced { hold } = value;
            domain::HoldReason::Placed { hold }
        }
        wire::HoldReason::Drift => domain::HoldReason::Drift,
        wire::HoldReason::Effects => domain::HoldReason::Effects,
        wire::HoldReason::EffectFailed => domain::HoldReason::EffectFailed,
        wire::HoldReason::Uncertain(value) => {
            let wire::HoldReasonUncertain { entry } = value;
            domain::HoldReason::Uncertain { entry }
        }
        wire::HoldReason::Budget => domain::HoldReason::Budget,
        wire::HoldReason::Deadline => domain::HoldReason::Deadline,
        wire::HoldReason::HoldsWaited => domain::HoldReason::HoldsWaited,
        wire::HoldReason::ResourceUnavailable => domain::HoldReason::ResourceUnavailable,
        wire::HoldReason::ResourceAccess => domain::HoldReason::ResourceAccess,
        wire::HoldReason::PoolLost(value) => {
            let wire::HoldReasonPoolLost { pool } = value;
            domain::HoldReason::PoolLost { pool }
        }
        wire::HoldReason::Procedure => domain::HoldReason::Procedure,
        wire::HoldReason::Circle => domain::HoldReason::Circle,
    }
}
pub(super) fn encode_moved(value: &domain::Moved, limits: &wire::Limits) -> wire::Moved {
    match value {
        domain::Moved::Hold { revision, reason } => {
            wire::Moved::Hold(wire::MovedHold { revision: *revision, reason: encode_hold_reason(reason, limits) })
        }
        domain::Moved::Spec { revision } => wire::Moved::Spec(wire::MovedSpec { revision: *revision }),
        domain::Moved::Person { revision } => wire::Moved::Person(wire::MovedPerson { revision: *revision }),
        domain::Moved::PlacedHold { revision } => {
            wire::Moved::PlacedHold(wire::MovedPlacedHold { revision: *revision })
        }
        domain::Moved::Note { revision } => wire::Moved::Note(wire::MovedNote { revision: *revision }),
    }
}
pub(super) fn decode_moved(value: wire::Moved) -> domain::Moved {
    match value {
        wire::Moved::Hold(value) => {
            let wire::MovedHold { revision, reason } = value;
            domain::Moved::Hold { revision, reason: decode_hold_reason(reason) }
        }
        wire::Moved::Spec(value) => {
            let wire::MovedSpec { revision } = value;
            domain::Moved::Spec { revision }
        }
        wire::Moved::Person(value) => {
            let wire::MovedPerson { revision } = value;
            domain::Moved::Person { revision }
        }
        wire::Moved::PlacedHold(value) => domain::Moved::PlacedHold { revision: value.revision },
        wire::Moved::Note(value) => {
            let wire::MovedNote { revision } = value;
            domain::Moved::Note { revision }
        }
    }
}
pub(super) fn encode_party(value: &domain::Party, _limits: &wire::Limits) -> wire::Party {
    match value {
        domain::Party::Person(value) => wire::Party::Person(wire::PartyPerson { value: *value }),
        domain::Party::Task(value) => wire::Party::Task(wire::PartyTask { value: *value }),
        domain::Party::Deployment { project } => wire::Party::Deployment(wire::PartyDeployment { project: *project }),
    }
}
pub(super) fn decode_party(value: wire::Party) -> domain::Party {
    match value {
        wire::Party::Person(value) => {
            let wire::PartyPerson { value } = value;
            domain::Party::Person(value)
        }
        wire::Party::Task(value) => {
            let wire::PartyTask { value } = value;
            domain::Party::Task(value)
        }
        wire::Party::Deployment(value) => {
            let wire::PartyDeployment { project } = value;
            domain::Party::Deployment { project }
        }
    }
}
pub(super) fn encode_ending(value: &domain::Ending, limits: &wire::Limits) -> wire::Ending {
    match value {
        domain::Ending::Done => wire::Ending::Done,
        domain::Ending::Failed => wire::Ending::Failed,
        domain::Ending::Cancelled { by, at } => {
            wire::Ending::Cancelled(wire::EndingCancelled { by: encode_party(by, limits), at: at.as_nanos() })
        }
        domain::Ending::Decided { by, at } => wire::Ending::Decided(wire::EndingDecided { by: *by, at: at.as_nanos() }),
        domain::Ending::Deleted => wire::Ending::Deleted,
    }
}
pub(super) fn decode_ending(value: wire::Ending) -> domain::Ending {
    match value {
        wire::Ending::Done => domain::Ending::Done,
        wire::Ending::Failed => domain::Ending::Failed,
        wire::Ending::Cancelled(value) => {
            let wire::EndingCancelled { by, at } = value;
            domain::Ending::Cancelled { by: decode_party(by), at: Wall::from_nanos(at) }
        }
        wire::Ending::Decided(value) => {
            let wire::EndingDecided { by, at } = value;
            domain::Ending::Decided { by, at: Wall::from_nanos(at) }
        }
        wire::Ending::Deleted => domain::Ending::Deleted,
    }
}
pub(super) fn encode_lack(value: &domain::Lack, limits: &wire::Limits) -> wire::Lack {
    match value {
        domain::Lack::HoldOwner { hold, owner } => {
            wire::Lack::HoldOwner(wire::LackHoldOwner { hold: *hold, owner: encode_hold_owner(owner, limits) })
        }
        domain::Lack::Ceiling { holder, dimension, value, asked } => wire::Lack::Ceiling(wire::LackCeiling {
            holder: encode_ceiling_holder(*holder, limits),
            dimension: encode_ceiling_dimension(*dimension),
            value: *value,
            asked: *asked,
        }),
        domain::Lack::Approver => wire::Lack::Approver,
        domain::Lack::Budget { needs, left } => wire::Lack::Budget(wire::LackBudget { needs: *needs, left: *left }),
        domain::Lack::Grant { kind } => wire::Lack::Grant(wire::LackGrant { kind: *kind }),
        domain::Lack::Funding => wire::Lack::Funding,
        domain::Lack::TaskKind { executor } => wire::Lack::TaskKind(wire::LackTaskKind {
            executor: match executor {
                Some(executor) => Some(super::super::batch::parts::encode_executor(*executor, limits)),
                None => None,
            },
        }),
        domain::Lack::Depth { most } => wire::Lack::Depth(wire::LackDepth { most: *most }),
        domain::Lack::Deadline => wire::Lack::Deadline,
        domain::Lack::Tasks { left } => wire::Lack::Tasks(wire::LackTasks { left: *left }),
        domain::Lack::Tool => wire::Lack::Tool,
        domain::Lack::Scope => wire::Lack::Scope,
        domain::Lack::Models => wire::Lack::Models,
        domain::Lack::Reference => wire::Lack::Reference,
        domain::Lack::ResourceAccess => wire::Lack::ResourceAccess,
        domain::Lack::ResourceUnavailable => wire::Lack::ResourceUnavailable,
        domain::Lack::Shape => wire::Lack::Shape,
        domain::Lack::Again { required } => wire::Lack::Again(wire::LackAgain { required: *required }),
        domain::Lack::Account => wire::Lack::Account,
        domain::Lack::Writer => wire::Lack::Writer,
        domain::Lack::Required { connector, kind } => {
            wire::Lack::Required(wire::LackRequired { connector: *connector, kind: *kind })
        }
        domain::Lack::Failed { connector, kind } => {
            wire::Lack::Failed(wire::LackFailed { connector: *connector, kind: *kind })
        }
        domain::Lack::Unguarded { connector, kind } => {
            wire::Lack::Unguarded(wire::LackUnguarded { connector: *connector, kind: *kind })
        }
    }
}
pub(super) fn decode_lack(value: wire::Lack) -> domain::Lack {
    match value {
        wire::Lack::HoldOwner(value) => {
            let wire::LackHoldOwner { hold, owner } = value;
            domain::Lack::HoldOwner { hold, owner: decode_hold_owner(owner) }
        }
        wire::Lack::Ceiling(row) => domain::Lack::Ceiling {
            holder: decode_ceiling_holder(&row.holder),
            dimension: decode_ceiling_dimension(&row.dimension),
            value: row.value,
            asked: row.asked,
        },
        wire::Lack::Approver => domain::Lack::Approver,
        wire::Lack::Budget(value) => {
            let wire::LackBudget { needs, left } = value;
            domain::Lack::Budget { needs, left }
        }
        wire::Lack::Grant(value) => {
            let wire::LackGrant { kind } = value;
            domain::Lack::Grant { kind }
        }
        wire::Lack::Funding => domain::Lack::Funding,
        wire::Lack::TaskKind(value) => {
            let mut executor = None;
            if let Some(value) = value.executor {
                executor = Some(super::super::batch::parts::decode_executor(value));
            }
            domain::Lack::TaskKind { executor }
        }
        wire::Lack::Depth(value) => {
            let wire::LackDepth { most } = value;
            domain::Lack::Depth { most }
        }
        wire::Lack::Deadline => domain::Lack::Deadline,
        wire::Lack::Tasks(value) => {
            let wire::LackTasks { left } = value;
            domain::Lack::Tasks { left }
        }
        wire::Lack::Tool => domain::Lack::Tool,
        wire::Lack::Scope => domain::Lack::Scope,
        wire::Lack::Models => domain::Lack::Models,
        wire::Lack::Reference => domain::Lack::Reference,
        wire::Lack::ResourceAccess => domain::Lack::ResourceAccess,
        wire::Lack::ResourceUnavailable => domain::Lack::ResourceUnavailable,
        wire::Lack::Shape => domain::Lack::Shape,
        wire::Lack::Again(value) => {
            let wire::LackAgain { required } = value;
            domain::Lack::Again { required }
        }
        wire::Lack::Account => domain::Lack::Account,
        wire::Lack::Writer => domain::Lack::Writer,
        wire::Lack::Required(value) => {
            let wire::LackRequired { connector, kind } = value;
            domain::Lack::Required { connector, kind }
        }
        wire::Lack::Failed(value) => {
            let wire::LackFailed { connector, kind } = value;
            domain::Lack::Failed { connector, kind }
        }
        wire::Lack::Unguarded(value) => {
            let wire::LackUnguarded { connector, kind } = value;
            domain::Lack::Unguarded { connector, kind }
        }
    }
}
pub(super) fn encode_limit_kind(value: domain::LimitKind) -> wire::LimitKind {
    match value {
        domain::LimitKind::Projects => wire::LimitKind::Projects,
        domain::LimitKind::Words => wire::LimitKind::Words,
        domain::LimitKind::Identity => wire::LimitKind::Identity,
        domain::LimitKind::Roles => wire::LimitKind::Roles,
        domain::LimitKind::Arithmetic => wire::LimitKind::Arithmetic,
        domain::LimitKind::Duplicate => wire::LimitKind::Duplicate,
        domain::LimitKind::Empty => wire::LimitKind::Empty,
        domain::LimitKind::Batch => wire::LimitKind::Batch,
        domain::LimitKind::ProjectLiveTasks => wire::LimitKind::ProjectLiveTasks,
        domain::LimitKind::LiveTasks => wire::LimitKind::LiveTasks,
        domain::LimitKind::Tree => wire::LimitKind::Tree,
        domain::LimitKind::Depth => wire::LimitKind::Depth,
        domain::LimitKind::Delegates => wire::LimitKind::Delegates,
        domain::LimitKind::Subscription => wire::LimitKind::Subscription,
        domain::LimitKind::Dependencies => wire::LimitKind::Dependencies,
        domain::LimitKind::Cycle => wire::LimitKind::Cycle,
        domain::LimitKind::Spec => wire::LimitKind::Spec,
        domain::LimitKind::Contract => wire::LimitKind::Contract,
        domain::LimitKind::Authority => wire::LimitKind::Authority,
        domain::LimitKind::Inputs => wire::LimitKind::Inputs,
        domain::LimitKind::Attempts => wire::LimitKind::Attempts,
        domain::LimitKind::LiveDelegates => wire::LimitKind::LiveDelegates,
        domain::LimitKind::Restore => wire::LimitKind::Restore,
        domain::LimitKind::Read => wire::LimitKind::Read,
        domain::LimitKind::Turn => wire::LimitKind::Turn,
        domain::LimitKind::HoldKind => wire::LimitKind::HoldKind,
        domain::LimitKind::HoldTaken => wire::LimitKind::HoldTaken,
        domain::LimitKind::Holds => wire::LimitKind::Holds,
        domain::LimitKind::InboxFull => wire::LimitKind::InboxFull,
        domain::LimitKind::Notes => wire::LimitKind::Notes,
        domain::LimitKind::Revision => wire::LimitKind::Revision,
        domain::LimitKind::Question => wire::LimitKind::Question,
        domain::LimitKind::Priority => wire::LimitKind::Priority,
        domain::LimitKind::Policy => wire::LimitKind::Policy,
        domain::LimitKind::Resource => wire::LimitKind::Resource,
        domain::LimitKind::Service => wire::LimitKind::Service,
        domain::LimitKind::PlacedHolds => wire::LimitKind::PlacedHolds,
        domain::LimitKind::ProjectPlacedHolds => wire::LimitKind::ProjectPlacedHolds,
        domain::LimitKind::PartyPlacedHolds => wire::LimitKind::PartyPlacedHolds,
        domain::LimitKind::Exemptions => wire::LimitKind::Exemptions,
        domain::LimitKind::Accounts => wire::LimitKind::Accounts,
        domain::LimitKind::HolderAccounts => wire::LimitKind::HolderAccounts,
        domain::LimitKind::Sources => wire::LimitKind::Sources,
        domain::LimitKind::SettingsProposals => wire::LimitKind::SettingsProposals,
        domain::LimitKind::SignIns => wire::LimitKind::SignIns,
        domain::LimitKind::Operations => wire::LimitKind::Operations,
        domain::LimitKind::Grace => wire::LimitKind::Grace,
        domain::LimitKind::Goals => wire::LimitKind::Goals,
    }
}
pub(super) fn decode_limit_kind(value: wire::LimitKind) -> domain::LimitKind {
    match value {
        wire::LimitKind::Projects => domain::LimitKind::Projects,
        wire::LimitKind::Words => domain::LimitKind::Words,
        wire::LimitKind::Identity => domain::LimitKind::Identity,
        wire::LimitKind::Roles => domain::LimitKind::Roles,
        wire::LimitKind::Arithmetic => domain::LimitKind::Arithmetic,
        wire::LimitKind::Duplicate => domain::LimitKind::Duplicate,
        wire::LimitKind::Empty => domain::LimitKind::Empty,
        wire::LimitKind::Batch => domain::LimitKind::Batch,
        wire::LimitKind::ProjectLiveTasks => domain::LimitKind::ProjectLiveTasks,
        wire::LimitKind::LiveTasks => domain::LimitKind::LiveTasks,
        wire::LimitKind::Tree => domain::LimitKind::Tree,
        wire::LimitKind::Depth => domain::LimitKind::Depth,
        wire::LimitKind::Delegates => domain::LimitKind::Delegates,
        wire::LimitKind::Subscription => domain::LimitKind::Subscription,
        wire::LimitKind::Dependencies => domain::LimitKind::Dependencies,
        wire::LimitKind::Cycle => domain::LimitKind::Cycle,
        wire::LimitKind::Spec => domain::LimitKind::Spec,
        wire::LimitKind::Contract => domain::LimitKind::Contract,
        wire::LimitKind::Authority => domain::LimitKind::Authority,
        wire::LimitKind::Inputs => domain::LimitKind::Inputs,
        wire::LimitKind::Attempts => domain::LimitKind::Attempts,
        wire::LimitKind::LiveDelegates => domain::LimitKind::LiveDelegates,
        wire::LimitKind::Restore => domain::LimitKind::Restore,
        wire::LimitKind::Read => domain::LimitKind::Read,
        wire::LimitKind::Turn => domain::LimitKind::Turn,
        wire::LimitKind::HoldKind => domain::LimitKind::HoldKind,
        wire::LimitKind::HoldTaken => domain::LimitKind::HoldTaken,
        wire::LimitKind::Holds => domain::LimitKind::Holds,
        wire::LimitKind::InboxFull => domain::LimitKind::InboxFull,
        wire::LimitKind::Notes => domain::LimitKind::Notes,
        wire::LimitKind::Revision => domain::LimitKind::Revision,
        wire::LimitKind::Question => domain::LimitKind::Question,
        wire::LimitKind::Priority => domain::LimitKind::Priority,
        wire::LimitKind::Policy => domain::LimitKind::Policy,
        wire::LimitKind::Resource => domain::LimitKind::Resource,
        wire::LimitKind::Service => domain::LimitKind::Service,
        wire::LimitKind::PlacedHolds => domain::LimitKind::PlacedHolds,
        wire::LimitKind::ProjectPlacedHolds => domain::LimitKind::ProjectPlacedHolds,
        wire::LimitKind::PartyPlacedHolds => domain::LimitKind::PartyPlacedHolds,
        wire::LimitKind::Exemptions => domain::LimitKind::Exemptions,
        wire::LimitKind::Accounts => domain::LimitKind::Accounts,
        wire::LimitKind::HolderAccounts => domain::LimitKind::HolderAccounts,
        wire::LimitKind::Sources => domain::LimitKind::Sources,
        wire::LimitKind::SettingsProposals => domain::LimitKind::SettingsProposals,
        wire::LimitKind::SignIns => domain::LimitKind::SignIns,
        wire::LimitKind::Operations => domain::LimitKind::Operations,
        wire::LimitKind::Grace => domain::LimitKind::Grace,
        wire::LimitKind::Goals => domain::LimitKind::Goals,
    }
}
#[expect(clippy::too_many_lines, reason = "one exhaustive translation preserves every stored refusal variant")]
pub(crate) fn encode_refusal(value: &domain::Refusal, limits: &wire::Limits) -> Result<wire::Refusal, wire::Problem> {
    Ok(match value {
        domain::Refusal::Forgotten { oldest } => wire::Refusal::Forgotten(wire::RefusalForgotten { oldest: *oldest }),
        domain::Refusal::Judged { at, setting, why } => wire::Refusal::Judged(wire::RefusalJudged {
            at: *at,
            setting: encode_setting(*setting, limits),
            why: why.clone(),
        }),
        domain::Refusal::Stale { newest } => wire::Refusal::Stale(wire::RefusalStale { newest: *newest }),
        domain::Refusal::Full { revision, tasks, more } => wire::Refusal::Full(wire::RefusalFull {
            revision: *revision,
            tasks: names(tasks, limits.refusal_full_tasks, wire::paths::REFUSAL_FULL_TASKS)?,
            more: *more,
        }),
        domain::Refusal::NotWellFormed { at, why } => wire::Refusal::NotWellFormed(wire::RefusalNotWellFormed {
            at: *at,
            why: match why {
                domain::Malformed::Settings { setting, fault, detail } => {
                    encode_malformed_settings(*setting, *fault, *detail, limits)
                }
                domain::Malformed::SettingsUnchanged => wire::Malformed::SettingsUnchanged,
                domain::Malformed::Running => wire::Malformed::Running,
                domain::Malformed::ProposedAsAllowed => wire::Malformed::ProposedAsAllowed,
                domain::Malformed::Cycle => wire::Malformed::Cycle,
                domain::Malformed::Name(rule) => {
                    wire::Malformed::Name(wire::MalformedName { rule: encode_name_rule(*rule) })
                }
                domain::Malformed::AsAllowedTwice => wire::Malformed::AsAllowedTwice,
                domain::Malformed::Conversation => wire::Malformed::Conversation,

                domain::Malformed::EndedBy => wire::Malformed::EndedBy,
            },
        }),
        domain::Refusal::Role { held } => wire::Refusal::Role(wire::RefusalRole {
            held: match held {
                Some(value) => Some(super::super::encode_role(*value, limits)),
                None => None,
            },
        }),
        domain::Refusal::Authority { lacking, proposable } => wire::Refusal::Authority(wire::RefusalAuthority {
            lacking: encode_lack(lacking, limits),
            proposable: match proposable {
                None => None,
                Some(domain::Accepter::Nobody) => Some(wire::Accepter::Nobody),
                Some(domain::Accepter::Role(role)) => Some(wire::Accepter::Role(wire::AccepterRole { role: *role })),
            },
        }),
        domain::Refusal::Limit { which } => {
            wire::Refusal::Limit(wire::RefusalLimit { which: encode_limit_kind(*which) })
        }
        domain::Refusal::Moved { now } => wire::Refusal::Moved(wire::RefusalMoved { now: encode_moved(now, limits) }),
        domain::Refusal::Ended { how } => wire::Refusal::Ended(wire::RefusalEnded { how: encode_ending(how, limits) }),
        domain::Refusal::Taken { by } => wire::Refusal::Taken(wire::RefusalTaken { by: *by }),
        domain::Refusal::LiveTasks { tasks, more } => wire::Refusal::LiveTasks(wire::RefusalLiveTasks {
            tasks: names(tasks, limits.refusal_live_tasks_tasks, wire::paths::REFUSAL_LIVE_TASKS_TASKS)?,
            more: *more,
        }),
        domain::Refusal::Circle { tasks, cut } => wire::Refusal::Circle(wire::RefusalCircle {
            tasks: names(tasks, limits.refusal_circle_tasks, wire::paths::REFUSAL_CIRCLE_TASKS)?,
            cut: *cut,
        }),
        domain::Refusal::Kept { pending } => wire::Refusal::Kept(wire::RefusalKept {
            pending: names(pending, limits.refusal_kept_pending, wire::paths::REFUSAL_KEPT_PENDING)?,
        }),
        domain::Refusal::Unlinked { worker } => wire::Refusal::Unlinked(wire::RefusalUnlinked { worker: *worker }),
        domain::Refusal::Concluded { at } => wire::Refusal::Concluded(wire::RefusalConcluded { at: at.as_nanos() }),
        domain::Refusal::Dependency { dependency, how } => wire::Refusal::Dependency(wire::RefusalDependency {
            dependency: *dependency,
            how: match how {
                domain::DependencyStatus::Done => wire::DependencyStatus::Done,
                domain::DependencyStatus::Failed => wire::DependencyStatus::Failed,
                domain::DependencyStatus::Cancelled => wire::DependencyStatus::Cancelled,
            },
        }),
        domain::Refusal::NotWaiting => wire::Refusal::NotWaiting,
        domain::Refusal::NoFurther => wire::Refusal::NoFurther,
        domain::Refusal::SignInRedirect { why } => wire::Refusal::SignInRedirect(wire::RefusalSignInRedirect {
            why: match why {
                domain::SignInRedirectWhy::NotWaiting => wire::SignInRedirectWhy::NotWaiting,
                domain::SignInRedirectWhy::Address => wire::SignInRedirectWhy::Address,
                domain::SignInRedirectWhy::State => wire::SignInRedirectWhy::State,
                domain::SignInRedirectWhy::TooMany => wire::SignInRedirectWhy::TooMany,
            },
        }),
        domain::Refusal::AccountMove { why } => {
            wire::Refusal::AccountMove(wire::RefusalAccountMove { why: encode_account_move_why(*why) })
        }
        domain::Refusal::NoTerminal => wire::Refusal::NoTerminal,
        domain::Refusal::LoginUnderWay => wire::Refusal::LoginUnderWay,
        domain::Refusal::NotLending => wire::Refusal::NotLending,
        domain::Refusal::NameTaken => wire::Refusal::NameTaken,
        domain::Refusal::Unknown => wire::Refusal::Unknown,
        domain::Refusal::Provider => wire::Refusal::Provider,
        domain::Refusal::NotOffered => wire::Refusal::NotOffered,
        domain::Refusal::ProgramNameReserved => wire::Refusal::ProgramNameReserved,
        domain::Refusal::KeyConflict => wire::Refusal::KeyConflict,
        domain::Refusal::NoEscalationHolder => wire::Refusal::NoEscalationHolder,
        domain::Refusal::Account { account, why } => {
            wire::Refusal::Account(wire::RefusalAccount { account: *account, why: encode_account_why(*why) })
        }
        domain::Refusal::Llm { model, why } => {
            wire::Refusal::Llm(wire::RefusalLlm { model: *model, why: super::super::llm::encode_why(*why) })
        }
    })
}
#[expect(clippy::too_many_lines, reason = "one exhaustive translation preserves every stored refusal variant")]
pub(crate) fn decode_refusal(value: wire::Refusal) -> domain::Refusal {
    match value {
        wire::Refusal::Judged(value) => {
            let wire::RefusalJudged { at, setting, why } = value;
            domain::Refusal::Judged { at, setting: decode_setting(&setting), why }
        }
        wire::Refusal::Forgotten(value) => domain::Refusal::Forgotten { oldest: value.oldest },
        wire::Refusal::Stale(value) => domain::Refusal::Stale { newest: value.newest },
        wire::Refusal::Full(value) => {
            let v = value;
            domain::Refusal::Full { revision: v.revision, tasks: v.tasks.into_boxed(), more: v.more }
        }
        wire::Refusal::NotWellFormed(value) => {
            let wire::RefusalNotWellFormed { at, why } = value;
            domain::Refusal::NotWellFormed {
                at,
                why: match why {
                    wire::Malformed::Settings(value) => decode_malformed_settings(value),
                    wire::Malformed::SettingsUnchanged => domain::Malformed::SettingsUnchanged,
                    wire::Malformed::Running => domain::Malformed::Running,
                    wire::Malformed::ProposedAsAllowed => domain::Malformed::ProposedAsAllowed,
                    wire::Malformed::Cycle => domain::Malformed::Cycle,
                    wire::Malformed::Name(value) => domain::Malformed::Name(decode_name_rule(value.rule)),
                    wire::Malformed::AsAllowedTwice => domain::Malformed::AsAllowedTwice,
                    wire::Malformed::Conversation => domain::Malformed::Conversation,

                    wire::Malformed::EndedBy => domain::Malformed::EndedBy,
                },
            }
        }
        wire::Refusal::Role(value) => {
            let wire::RefusalRole { held } = value;
            domain::Refusal::Role {
                held: match held {
                    Some(value) => {
                        let result = super::super::decode_role(value);
                        Some(result)
                    }
                    None => None,
                },
            }
        }
        wire::Refusal::Authority(value) => {
            let wire::RefusalAuthority { lacking, proposable } = value;
            domain::Refusal::Authority {
                lacking: decode_lack(lacking),
                proposable: match proposable {
                    None => None,
                    Some(wire::Accepter::Nobody) => Some(domain::Accepter::Nobody),
                    Some(wire::Accepter::Role(role)) => Some(domain::Accepter::Role(role.role)),
                },
            }
        }
        wire::Refusal::Limit(value) => {
            let wire::RefusalLimit { which } = value;
            domain::Refusal::Limit { which: decode_limit_kind(which) }
        }
        wire::Refusal::Moved(value) => {
            let wire::RefusalMoved { now } = value;
            domain::Refusal::Moved { now: decode_moved(now) }
        }
        wire::Refusal::Ended(value) => {
            let wire::RefusalEnded { how } = value;
            domain::Refusal::Ended { how: decode_ending(how) }
        }
        wire::Refusal::Taken(value) => {
            let wire::RefusalTaken { by } = value;
            domain::Refusal::Taken { by }
        }
        wire::Refusal::LiveTasks(value) => {
            let wire::RefusalLiveTasks { tasks, more } = value;
            domain::Refusal::LiveTasks { tasks: tasks.into_boxed(), more }
        }
        wire::Refusal::Circle(value) => {
            let wire::RefusalCircle { tasks, cut } = value;
            domain::Refusal::Circle { tasks: tasks.into_boxed(), cut }
        }
        wire::Refusal::Kept(value) => domain::Refusal::Kept { pending: value.pending.into_boxed() },
        wire::Refusal::Unlinked(value) => domain::Refusal::Unlinked { worker: value.worker },
        wire::Refusal::Concluded(value) => domain::Refusal::Concluded { at: Wall::from_nanos(value.at) },
        wire::Refusal::Dependency(value) => {
            let wire::RefusalDependency { dependency, how } = value;
            domain::Refusal::Dependency {
                dependency,
                how: match how {
                    wire::DependencyStatus::Done => domain::DependencyStatus::Done,
                    wire::DependencyStatus::Failed => domain::DependencyStatus::Failed,
                    wire::DependencyStatus::Cancelled => domain::DependencyStatus::Cancelled,
                },
            }
        }
        wire::Refusal::NotWaiting => domain::Refusal::NotWaiting,
        wire::Refusal::NoFurther => domain::Refusal::NoFurther,
        wire::Refusal::SignInRedirect(row) => {
            domain::Refusal::SignInRedirect { why: decode_sign_in_redirect_why(&row.why) }
        }
        wire::Refusal::AccountMove(value) => domain::Refusal::AccountMove { why: decode_account_move_why(value.why) },
        wire::Refusal::NoTerminal => domain::Refusal::NoTerminal,
        wire::Refusal::LoginUnderWay => domain::Refusal::LoginUnderWay,
        wire::Refusal::NotLending => domain::Refusal::NotLending,
        wire::Refusal::NameTaken => domain::Refusal::NameTaken,
        wire::Refusal::Unknown => domain::Refusal::Unknown,
        wire::Refusal::Provider => domain::Refusal::Provider,
        wire::Refusal::NotOffered => domain::Refusal::NotOffered,
        wire::Refusal::ProgramNameReserved => domain::Refusal::ProgramNameReserved,
        wire::Refusal::KeyConflict => domain::Refusal::KeyConflict,
        wire::Refusal::NoEscalationHolder => domain::Refusal::NoEscalationHolder,
        wire::Refusal::Account(value) => {
            let wire::RefusalAccount { account, why } = value;
            domain::Refusal::Account { account, why: decode_account_why(why) }
        }
        wire::Refusal::Llm(value) => {
            let wire::RefusalLlm { model, why } = value;
            domain::Refusal::Llm { model, why: super::super::llm::decode_why(why) }
        }
    }
}
pub(super) fn encode_proposal_choice(value: domain::ProposalChoice) -> wire::ProposalChoice {
    match value {
        domain::ProposalChoice::Accepted => wire::ProposalChoice::Accepted,
        domain::ProposalChoice::Rejected => wire::ProposalChoice::Rejected,
        domain::ProposalChoice::Passed => wire::ProposalChoice::Passed,
        domain::ProposalChoice::Withdrawn => wire::ProposalChoice::Withdrawn,
        domain::ProposalChoice::Stale => wire::ProposalChoice::Stale,
    }
}
pub(super) fn decode_proposal_choice(value: wire::ProposalChoice) -> domain::ProposalChoice {
    match value {
        wire::ProposalChoice::Accepted => domain::ProposalChoice::Accepted,
        wire::ProposalChoice::Rejected => domain::ProposalChoice::Rejected,
        wire::ProposalChoice::Passed => domain::ProposalChoice::Passed,
        wire::ProposalChoice::Withdrawn => domain::ProposalChoice::Withdrawn,
        wire::ProposalChoice::Stale => domain::ProposalChoice::Stale,
    }
}
pub(super) fn encode_escalation_choice(value: domain::EscalationChoice) -> wire::EscalationChoice {
    match value {
        domain::EscalationChoice::Released => wire::EscalationChoice::Released,
        domain::EscalationChoice::Rejected => wire::EscalationChoice::Rejected,
        domain::EscalationChoice::Passed => wire::EscalationChoice::Passed,
    }
}
pub(super) fn decode_escalation_choice(value: wire::EscalationChoice) -> domain::EscalationChoice {
    match value {
        wire::EscalationChoice::Released => domain::EscalationChoice::Released,
        wire::EscalationChoice::Rejected => domain::EscalationChoice::Rejected,
        wire::EscalationChoice::Passed => domain::EscalationChoice::Passed,
    }
}

fn encode_account_why(value: domain::AccountWhy) -> wire::AccountWhy {
    match value {
        domain::AccountWhy::Unknown => wire::AccountWhy::Unknown,
        domain::AccountWhy::Provider => wire::AccountWhy::Provider,
        domain::AccountWhy::Uses => wire::AccountWhy::Uses,
        domain::AccountWhy::Limited => wire::AccountWhy::Limited,
    }
}

fn decode_account_why(value: wire::AccountWhy) -> domain::AccountWhy {
    match value {
        wire::AccountWhy::Unknown => domain::AccountWhy::Unknown,
        wire::AccountWhy::Provider => domain::AccountWhy::Provider,
        wire::AccountWhy::Uses => domain::AccountWhy::Uses,
        wire::AccountWhy::Limited => domain::AccountWhy::Limited,
    }
}

fn names(values: &[u64], bound: u32, path: wire::Path) -> Result<skein_lib::List<u64>, wire::Problem> {
    let mut rows = skein_lib::List::with_capacity(bound);
    for number in values {
        if rows.push(*number).is_err() {
            return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(rows)
}

fn encode_name_rule(rule: domain::NameRule) -> wire::NameRule {
    match rule {
        domain::NameRule::Empty => wire::NameRule::Empty,
        domain::NameRule::Long => wire::NameRule::Long,
        domain::NameRule::Control => wire::NameRule::Control,
        domain::NameRule::Space => wire::NameRule::Space,
        domain::NameRule::Digits => wire::NameRule::Digits,
    }
}

fn decode_name_rule(rule: wire::NameRule) -> domain::NameRule {
    match rule {
        wire::NameRule::Empty => domain::NameRule::Empty,
        wire::NameRule::Long => domain::NameRule::Long,
        wire::NameRule::Control => domain::NameRule::Control,
        wire::NameRule::Space => domain::NameRule::Space,
        wire::NameRule::Digits => domain::NameRule::Digits,
    }
}

const fn encode_account_move_why(value: domain::AccountMoveWhy) -> wire::AccountMoveWhy {
    match value {
        domain::AccountMoveWhy::NotSignedIn => wire::AccountMoveWhy::NotSignedIn,
        domain::AccountMoveWhy::NotOurs => wire::AccountMoveWhy::NotOurs,
        domain::AccountMoveWhy::Held => wire::AccountMoveWhy::Held,
        domain::AccountMoveWhy::Cancelled => wire::AccountMoveWhy::Cancelled,
        domain::AccountMoveWhy::Moving => wire::AccountMoveWhy::Moving,
    }
}

const fn decode_account_move_why(value: wire::AccountMoveWhy) -> domain::AccountMoveWhy {
    match value {
        wire::AccountMoveWhy::NotSignedIn => domain::AccountMoveWhy::NotSignedIn,
        wire::AccountMoveWhy::NotOurs => domain::AccountMoveWhy::NotOurs,
        wire::AccountMoveWhy::Held => domain::AccountMoveWhy::Held,
        wire::AccountMoveWhy::Cancelled => domain::AccountMoveWhy::Cancelled,
        wire::AccountMoveWhy::Moving => domain::AccountMoveWhy::Moving,
    }
}

pub(super) fn encode_setting(value: domain::Setting, _limits: &wire::Limits) -> wire::Setting {
    match value {
        domain::Setting::ProjectPolicy { project } => {
            wire::Setting::ProjectPolicy(wire::ProjectPolicySetting { project })
        }
        domain::Setting::Charter(number) => wire::Setting::Charter(wire::SettingNumber { number }),
        domain::Setting::Template(number) => wire::Setting::Template(wire::SettingNumber { number }),
        domain::Setting::Policy(number) => wire::Setting::Policy(wire::SettingNumber { number }),
        domain::Setting::Model(number) => wire::Setting::Model(wire::SettingNumber { number }),
        domain::Setting::Worker(number) => wire::Setting::Worker(wire::SettingNumber { number }),
        domain::Setting::Provider(number) => wire::Setting::Provider(wire::SettingNumber { number }),
        domain::Setting::Rules => wire::Setting::Rules,
        domain::Setting::Limits => wire::Setting::Limits,
        domain::Setting::Application { kind, scope, number } => {
            wire::Setting::Application(wire::ApplicationSetting { kind, scope, number })
        }
    }
}
fn encode_setting_fault(value: domain::SettingFault) -> wire::SettingFault {
    match value {
        domain::SettingFault::Shape => wire::SettingFault::Shape,
        domain::SettingFault::Count => wire::SettingFault::Count,
        domain::SettingFault::Reference => wire::SettingFault::Reference,
        domain::SettingFault::Rules => wire::SettingFault::Rules,
        domain::SettingFault::Budget => wire::SettingFault::Budget,
        domain::SettingFault::Grace => wire::SettingFault::Grace,
        domain::SettingFault::Owner => wire::SettingFault::Owner,
        domain::SettingFault::Judged => wire::SettingFault::Judged,
        domain::SettingFault::Linked => wire::SettingFault::Linked,
        domain::SettingFault::Accounts => wire::SettingFault::Accounts,
        domain::SettingFault::Range => wire::SettingFault::Range,
    }
}

pub(super) fn decode_setting(value: &wire::Setting) -> domain::Setting {
    match value {
        wire::Setting::Charter(value) => domain::Setting::Charter(value.number),
        wire::Setting::Template(value) => domain::Setting::Template(value.number),
        wire::Setting::Policy(value) => domain::Setting::Policy(value.number),
        wire::Setting::ProjectPolicy(value) => domain::Setting::ProjectPolicy { project: value.project },
        wire::Setting::Model(value) => domain::Setting::Model(value.number),
        wire::Setting::Worker(value) => domain::Setting::Worker(value.number),
        wire::Setting::Provider(value) => domain::Setting::Provider(value.number),
        wire::Setting::Rules => domain::Setting::Rules,
        wire::Setting::Limits => domain::Setting::Limits,
        wire::Setting::Application(value) => {
            domain::Setting::Application { kind: value.kind, scope: value.scope, number: value.number }
        }
    }
}
fn decode_setting_fault(value: &wire::SettingFault) -> domain::SettingFault {
    match value {
        wire::SettingFault::Shape => domain::SettingFault::Shape,
        wire::SettingFault::Count => domain::SettingFault::Count,
        wire::SettingFault::Reference => domain::SettingFault::Reference,
        wire::SettingFault::Rules => domain::SettingFault::Rules,
        wire::SettingFault::Budget => domain::SettingFault::Budget,
        wire::SettingFault::Grace => domain::SettingFault::Grace,
        wire::SettingFault::Owner => domain::SettingFault::Owner,
        wire::SettingFault::Judged => domain::SettingFault::Judged,
        wire::SettingFault::Linked => domain::SettingFault::Linked,
        wire::SettingFault::Accounts => domain::SettingFault::Accounts,
        wire::SettingFault::Range => domain::SettingFault::Range,
    }
}

const fn decode_sign_in_redirect_why(value: &wire::SignInRedirectWhy) -> domain::SignInRedirectWhy {
    match value {
        wire::SignInRedirectWhy::NotWaiting => domain::SignInRedirectWhy::NotWaiting,
        wire::SignInRedirectWhy::Address => domain::SignInRedirectWhy::Address,
        wire::SignInRedirectWhy::State => domain::SignInRedirectWhy::State,
        wire::SignInRedirectWhy::TooMany => domain::SignInRedirectWhy::TooMany,
    }
}

pub(super) fn encode_hold_owner(value: &domain::HoldOwner, _limits: &wire::Limits) -> wire::HoldOwner {
    match value {
        domain::HoldOwner::Party { person } => wire::HoldOwner::Party(wire::HoldOwnerParty { person: *person }),
        domain::HoldOwner::Task { task } => wire::HoldOwner::Task(wire::HoldOwnerTask { task: *task }),
    }
}
pub(super) fn decode_hold_owner(value: wire::HoldOwner) -> domain::HoldOwner {
    match value {
        wire::HoldOwner::Party(value) => domain::HoldOwner::Party { person: value.person },
        wire::HoldOwner::Task(value) => domain::HoldOwner::Task { task: value.task },
    }
}

fn encode_ceiling_holder(value: domain::CeilingHolder, _limits: &wire::Limits) -> wire::CeilingHolder {
    match value {
        domain::CeilingHolder::Rules => wire::CeilingHolder::Rules,
        domain::CeilingHolder::Policy => wire::CeilingHolder::Policy,
        domain::CeilingHolder::Role(role) => wire::CeilingHolder::Role(wire::CeilingRole { role }),
    }
}
fn decode_ceiling_holder(value: &wire::CeilingHolder) -> domain::CeilingHolder {
    match value {
        wire::CeilingHolder::Rules => domain::CeilingHolder::Rules,
        wire::CeilingHolder::Policy => domain::CeilingHolder::Policy,
        wire::CeilingHolder::Role(row) => domain::CeilingHolder::Role(row.role),
    }
}
fn encode_ceiling_dimension(value: domain::CeilingDimension) -> wire::CeilingDimension {
    match value {
        domain::CeilingDimension::PeriodSpend => wire::CeilingDimension::PeriodSpend,
        domain::CeilingDimension::Spend => wire::CeilingDimension::Spend,
        domain::CeilingDimension::Tools => wire::CeilingDimension::Tools,
        domain::CeilingDimension::Tasks => wire::CeilingDimension::Tasks,
        domain::CeilingDimension::Depth => wire::CeilingDimension::Depth,
        domain::CeilingDimension::Executors => wire::CeilingDimension::Executors,
        domain::CeilingDimension::Grants => wire::CeilingDimension::Grants,
        domain::CeilingDimension::Deadline => wire::CeilingDimension::Deadline,
        domain::CeilingDimension::Notes => wire::CeilingDimension::Notes,
    }
}
fn decode_ceiling_dimension(value: &wire::CeilingDimension) -> domain::CeilingDimension {
    match value {
        wire::CeilingDimension::PeriodSpend => domain::CeilingDimension::PeriodSpend,
        wire::CeilingDimension::Spend => domain::CeilingDimension::Spend,
        wire::CeilingDimension::Tools => domain::CeilingDimension::Tools,
        wire::CeilingDimension::Tasks => domain::CeilingDimension::Tasks,
        wire::CeilingDimension::Depth => domain::CeilingDimension::Depth,
        wire::CeilingDimension::Executors => domain::CeilingDimension::Executors,
        wire::CeilingDimension::Grants => domain::CeilingDimension::Grants,
        wire::CeilingDimension::Deadline => domain::CeilingDimension::Deadline,
        wire::CeilingDimension::Notes => domain::CeilingDimension::Notes,
    }
}

/// Translate the document refusal facts without checking the settings again.
fn encode_setting_detail(value: domain::SettingDetail, _limits: &wire::Limits) -> wire::SettingDetail {
    wire::SettingDetail {
        member: match value.member {
            domain::SettingMember::Documents => wire::SettingMember::Documents,
            domain::SettingMember::NameSegments => wire::SettingMember::NameSegments,
            domain::SettingMember::NameSegmentBytes => wire::SettingMember::NameSegmentBytes,
            domain::SettingMember::Path => wire::SettingMember::Path,
            domain::SettingMember::Heading => wire::SettingMember::Heading,
            domain::SettingMember::Connector => wire::SettingMember::Connector,
            domain::SettingMember::StoredPolicy => wire::SettingMember::StoredPolicy,
        },
        rule: match value.rule {
            domain::SettingRule::Maximum => wire::SettingRule::Maximum,
            domain::SettingRule::Minimum => wire::SettingRule::Minimum,
            domain::SettingRule::Registered => wire::SettingRule::Registered,
        },
        document: value.document,
        segment: value.segment,
        supplied: value.supplied,
        bound: value.bound,
    }
}
/// Translate the document refusal facts without checking the settings again.
fn decode_setting_detail(value: &wire::SettingDetail) -> domain::SettingDetail {
    domain::SettingDetail {
        member: match value.member {
            wire::SettingMember::Documents => domain::SettingMember::Documents,
            wire::SettingMember::NameSegments => domain::SettingMember::NameSegments,
            wire::SettingMember::NameSegmentBytes => domain::SettingMember::NameSegmentBytes,
            wire::SettingMember::Path => domain::SettingMember::Path,
            wire::SettingMember::Heading => domain::SettingMember::Heading,
            wire::SettingMember::Connector => domain::SettingMember::Connector,
            wire::SettingMember::StoredPolicy => domain::SettingMember::StoredPolicy,
        },
        rule: match value.rule {
            wire::SettingRule::Maximum => domain::SettingRule::Maximum,
            wire::SettingRule::Minimum => domain::SettingRule::Minimum,
            wire::SettingRule::Registered => domain::SettingRule::Registered,
        },
        document: value.document,
        segment: value.segment,
        supplied: value.supplied,
        bound: value.bound,
    }
}

fn encode_malformed_settings(
    setting: domain::Setting,
    fault: domain::SettingFault,
    detail: Option<domain::SettingDetail>,
    limits: &wire::Limits,
) -> wire::Malformed {
    wire::Malformed::Settings(wire::MalformedSettings {
        setting: encode_setting(setting, limits),
        fault: encode_setting_fault(fault),
        detail: match detail {
            Some(value) => Some(encode_setting_detail(value, limits)),
            None => None,
        },
    })
}
fn decode_malformed_settings(value: wire::MalformedSettings) -> domain::Malformed {
    domain::Malformed::Settings {
        setting: decode_setting(&value.setting),
        fault: decode_setting_fault(&value.fault),
        detail: match value.detail {
            Some(value) => Some(decode_setting_detail(&value)),
            None => None,
        },
    }
}
