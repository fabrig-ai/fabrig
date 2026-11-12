//! Wire projections of refusals facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, w};
use super::{
    wire_account_why, wire_admission_limit, wire_chip_executor, wire_llm_why, wire_malformed, wire_refusal_full,
    wire_refusal_stale, wire_revision, wire_task_status,
};
pub(in crate::codec) fn wire_refusal(value: &c::Refusal, limits: &w::Limits) -> Result<w::Refusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Refusal::SignInRedirect { why } => w::Refusal::SignInRedirect(w::RefusalSignInRedirect::new(
            limits,
            w::RefusalSignInRedirectParts { why: wire_sign_in_redirect_why(why, limits)? },
        )?),
        c::Refusal::Judged { at, setting, why } => w::Refusal::Judged(w::RefusalJudged::new(
            limits,
            w::RefusalJudgedParts {
                at: *at,
                setting: super::settings::wire_setting(setting, limits)?,
                why: why.bytes().into(),
            },
        )?),
        c::Refusal::Forgotten { oldest } => {
            w::Refusal::Forgotten(w::RefusalForgotten::new(limits, w::RefusalForgottenParts { oldest: *oldest })?)
        }
        c::Refusal::Stale(value) => w::Refusal::Stale(wire_refusal_stale(value, limits)?),
        c::Refusal::Full(value) => w::Refusal::Full(wire_refusal_full(value, limits)?),
        c::Refusal::AccountMove { why } => w::Refusal::AccountMove(w::RefusalAccountMove::new(
            limits,
            w::RefusalAccountMoveParts { why: wire_account_move_why(why, limits)? },
        )?),
        c::Refusal::LiveTasks { tasks, more } => w::Refusal::LiveTasks(w::RefusalLiveTasks::new(
            limits,
            w::RefusalLiveTasksParts { tasks: tasks.clone(), more: *more },
        )?),
        c::Refusal::Circle { tasks, cut } => w::Refusal::Circle(w::RefusalCircle::new(
            limits,
            w::RefusalCircleParts {
                tasks: list((tasks).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                cut: *(cut),
            },
        )?),
        c::Refusal::Dependency { dependency, how } => w::Refusal::Dependency(w::RefusalDependency::new(
            limits,
            w::RefusalDependencyParts { dependency: *(dependency), how: wire_task_status(how, limits)? },
        )?),
        c::Refusal::Kept { pending } => w::Refusal::Kept(w::RefusalKept::new(
            limits,
            w::RefusalKeptParts { pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))? },
        )?),
        c::Refusal::Account { account, why } => w::Refusal::Account(w::RefusalAccount::new(
            limits,
            w::RefusalAccountParts { account: *(account), why: wire_account_why(why, limits)? },
        )?),
        c::Refusal::NoEscalationHolder => w::Refusal::NoEscalationHolder,
        c::Refusal::Provider => w::Refusal::Provider,
        c::Refusal::Malformed { at, why } => w::Refusal::Malformed(w::RefusalMalformed::new(
            limits,
            w::RefusalMalformedParts { at: *(at), why: wire_malformed(why, limits)? },
        )?),
        c::Refusal::Role { held } => w::Refusal::Role(w::RefusalRole::new(
            limits,
            w::RefusalRoleParts { held: (held).as_ref().map(|item| wire_held_role(item, limits)).transpose()? },
        )?),
        c::Refusal::Taken { by } => {
            w::Refusal::Taken(w::RefusalTaken::new(limits, w::RefusalTakenParts { by: *(by) })?)
        }
        c::Refusal::NotWaiting => w::Refusal::NotWaiting,
        c::Refusal::NoFurther => w::Refusal::NoFurther,
        c::Refusal::Authority { lacking, proposable } => w::Refusal::Authority(w::RefusalAuthority::new(
            limits,
            w::RefusalAuthorityParts {
                lacking: wire_lacking(lacking, limits)?,
                proposable: (proposable).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::Refusal::Limit { which } => {
            w::Refusal::Limit(w::RefusalLimit::new(limits, w::RefusalLimitParts { which: wire_limit(which, limits)? })?)
        }
        c::Refusal::Moved { now, what } => w::Refusal::Moved(w::RefusalMoved::new(
            limits,
            w::RefusalMovedParts { now: wire_revision(now, limits)?, what: wire_moved(what, limits)? },
        )?),
        c::Refusal::Ended { how } => {
            w::Refusal::Ended(w::RefusalEnded::new(limits, w::RefusalEndedParts { how: facts::ending(how, limits)? })?)
        }
        c::Refusal::Unknown => w::Refusal::Unknown,
        c::Refusal::NameTaken => w::Refusal::NameTaken,
        c::Refusal::NotOffered => w::Refusal::NotOffered,
        c::Refusal::ProgramNameReserved => w::Refusal::ProgramNameReserved,
        c::Refusal::KeyConflict => w::Refusal::KeyConflict,
        c::Refusal::Llm { model, why } => w::Refusal::Llm(w::RefusalLlm::new(
            limits,
            w::RefusalLlmParts { model: bytes::copy_of((model).bytes()), why: wire_llm_why(why, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_sign_in_redirect_why(
    value: &c::SignInRedirectWhy,
    _limits: &w::Limits,
) -> Result<w::SignInRedirectWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SignInRedirectWhy::NotWaiting => w::SignInRedirectWhy::NotWaiting,
        c::SignInRedirectWhy::Address => w::SignInRedirectWhy::Address,
        c::SignInRedirectWhy::State => w::SignInRedirectWhy::State,
        c::SignInRedirectWhy::TooMany => w::SignInRedirectWhy::TooMany,
    })
}

pub(in crate::codec) fn wire_account_move_why(
    value: &c::AccountMoveWhy,
    _limits: &w::Limits,
) -> Result<w::AccountMoveWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountMoveWhy::NotSignedIn => w::AccountMoveWhy::NotSignedIn,
        c::AccountMoveWhy::NotOurs => w::AccountMoveWhy::NotOurs,
        c::AccountMoveWhy::Held => w::AccountMoveWhy::Held,
        c::AccountMoveWhy::Cancelled => w::AccountMoveWhy::Cancelled,
        c::AccountMoveWhy::Moving => w::AccountMoveWhy::Moving,
    })
}

pub(in crate::codec) fn wire_held_role(value: &c::HeldRole, limits: &w::Limits) -> Result<w::HeldRole, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HeldRole::Owner => w::HeldRole::Owner,
        c::HeldRole::Maintainer => w::HeldRole::Maintainer,
        c::HeldRole::Member => w::HeldRole::Member,
        c::HeldRole::Observer => w::HeldRole::Observer,
        c::HeldRole::Policy { role } => {
            w::HeldRole::Policy(w::HeldRolePolicy::new(limits, w::HeldRolePolicyParts { role: *(role) })?)
        }
    })
}

pub(in crate::codec) fn wire_lacking(value: &c::Lacking, limits: &w::Limits) -> Result<w::Lacking, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Lacking::HoldOwner { hold, owner } => w::Lacking::HoldOwner(w::LackingHoldOwner::new(
            limits,
            w::LackingHoldOwnerParts { hold: *(hold), owner: wire_hold_owner(owner, limits)? },
        )?),
        c::Lacking::Ceiling { holder, dimension, value, asked } => w::Lacking::Ceiling(w::LackingCeiling::new(
            limits,
            w::LackingCeilingParts {
                holder: ceiling_holder(*holder, limits)?,
                dimension: ceiling_dimension(*dimension),
                value: *value,
                asked: *asked,
            },
        )?),
        c::Lacking::Approver => w::Lacking::Approver,
        c::Lacking::Budget { needed, left } => w::Lacking::Budget(w::LackingBudget::new(
            limits,
            w::LackingBudgetParts { needed: *(needed), left: *(left) },
        )?),
        c::Lacking::GrantKind { kind } => {
            w::Lacking::GrantKind(w::LackingGrantKind::new(limits, w::LackingGrantKindParts { kind: *(kind) })?)
        }
        c::Lacking::Funding => w::Lacking::Funding,
        c::Lacking::Tool => w::Lacking::Tool,
        c::Lacking::Scope => w::Lacking::Scope,
        c::Lacking::Models => w::Lacking::Models,
        c::Lacking::Reference => w::Lacking::Reference,
        c::Lacking::ResourceAccess => w::Lacking::ResourceAccess,
        c::Lacking::ResourceUnavailable => w::Lacking::ResourceUnavailable,
        c::Lacking::Shape => w::Lacking::Shape,
        c::Lacking::Again { required } => {
            w::Lacking::Again(w::LackingAgain::new(limits, w::LackingAgainParts { required: *(required) })?)
        }
        c::Lacking::Account => w::Lacking::Account,
        c::Lacking::Writer => w::Lacking::Writer,
        c::Lacking::Required { connector, kind } => w::Lacking::Required(w::LackingRequired::new(
            limits,
            w::LackingRequiredParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Lacking::Failed { connector, kind } => w::Lacking::Failed(w::LackingFailed::new(
            limits,
            w::LackingFailedParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Lacking::Unguarded { connector, kind } => w::Lacking::Unguarded(w::LackingUnguarded::new(
            limits,
            w::LackingUnguardedParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Lacking::TaskKind { executor } => w::Lacking::TaskKind(w::LackingTaskKind::new(
            limits,
            w::LackingTaskKindParts {
                executor: (executor).as_ref().map(|item| wire_chip_executor(item, limits)).transpose()?,
            },
        )?),
        c::Lacking::Depth { needed, allowed } => w::Lacking::Depth(w::LackingDepth::new(
            limits,
            w::LackingDepthParts {
                needed: (needed).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                allowed: *(allowed),
            },
        )?),
        c::Lacking::Deadline { asked, latest } => w::Lacking::Deadline(w::LackingDeadline::new(
            limits,
            w::LackingDeadlineParts {
                asked: (asked).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                latest: (latest).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::Lacking::Tasks { needed, left } => w::Lacking::Tasks(w::LackingTasks::new(
            limits,
            w::LackingTasksParts {
                needed: (needed).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                left: *(left),
            },
        )?),
    })
}

pub(in crate::codec) fn wire_hold_owner(value: &c::HoldOwner, limits: &w::Limits) -> Result<w::HoldOwner, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldOwner::Party { person } => {
            w::HoldOwner::Party(w::HoldOwnerParty::new(limits, w::HoldOwnerPartyParts { person: *(person) })?)
        }
        c::HoldOwner::Task { task } => {
            w::HoldOwner::Task(w::HoldOwnerTask::new(limits, w::HoldOwnerTaskParts { task: *(task) })?)
        }
    })
}

pub(in crate::codec) fn wire_limit(value: &c::Limit, limits: &w::Limits) -> Result<w::Limit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Limit::Admission { which } => w::Limit::Admission(w::LimitAdmission::new(
            limits,
            w::LimitAdmissionParts { which: wire_admission_limit(which, limits)? },
        )?),
        c::Limit::LiveTasks { project, limit } => w::Limit::LiveTasks(w::LimitLiveTasks::new(
            limits,
            w::LimitLiveTasksParts { project: *(project), limit: *(limit) },
        )?),
        c::Limit::Inbox { task, limit } => {
            w::Limit::Inbox(w::LimitInbox::new(limits, w::LimitInboxParts { task: *(task), limit: *(limit) })?)
        }
        c::Limit::Words { limit } => {
            w::Limit::Words(w::LimitWords::new(limits, w::LimitWordsParts { limit: *(limit) })?)
        }
        c::Limit::Fields => w::Limit::Fields,
        c::Limit::Records => w::Limit::Records,
        c::Limit::Requests => w::Limit::Requests,
        c::Limit::Reads => w::Limit::Reads,
        c::Limit::Watches => w::Limit::Watches,
        c::Limit::Objects => w::Limit::Objects,
    })
}

pub(in crate::codec) fn wire_moved(value: &c::Moved, limits: &w::Limits) -> Result<w::Moved, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Moved::PlacedHold { revision } => w::Moved::PlacedHold(w::MovedPlacedHold::new(
            limits,
            w::MovedPlacedHoldParts { revision: (revision).value },
        )?),
        c::Moved::Hold { reason } => w::Moved::Hold(w::MovedHold::new(
            limits,
            w::MovedHoldParts { reason: (reason).as_ref().map(|item| wire_current_hold(item, limits)).transpose()? },
        )?),
        c::Moved::Spec => w::Moved::Spec,
        c::Moved::PersonTask => w::Moved::PersonTask,
        c::Moved::Note => w::Moved::Note,
        c::Moved::Waiting => w::Moved::Waiting,
    })
}

pub(in crate::codec) fn wire_current_hold(
    value: &c::CurrentHold,
    limits: &w::Limits,
) -> Result<w::CurrentHold, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::CurrentHold::Document(value) => w::CurrentHold::Document(w::CurrentHoldDocument::new(
            limits,
            w::CurrentHoldDocumentParts { value: facts::hold_reason(value, limits)? },
        )?),
        c::CurrentHold::Reason(value) => w::CurrentHold::Reason(w::CurrentHoldReason::new(
            limits,
            w::CurrentHoldReasonParts { value: wire_hold_category(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_hold_category(
    value: &c::HoldCategory,
    limits: &w::Limits,
) -> Result<w::HoldCategory, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldCategory::Exceeded(value) => w::HoldCategory::Exceeded(wire_exceeded(value, limits)?),
        c::HoldCategory::Failures { class } => w::HoldCategory::Failures(w::HoldCategoryFailures::new(
            limits,
            w::HoldCategoryFailuresParts { class: wire_hold_failure(class, limits)? },
        )?),
        c::HoldCategory::Stalled => w::HoldCategory::Stalled,
        c::HoldCategory::Dependency { task } => w::HoldCategory::Dependency(w::HoldCategoryDependency::new(
            limits,
            w::HoldCategoryDependencyParts { task: *(task) },
        )?),
        c::HoldCategory::Stopped => w::HoldCategory::Stopped,
        c::HoldCategory::Placed { hold } => {
            w::HoldCategory::Placed(w::HoldCategoryPlaced::new(limits, w::HoldCategoryPlacedParts { hold: *(hold) })?)
        }
        c::HoldCategory::Drift => w::HoldCategory::Drift,
        c::HoldCategory::Effects => w::HoldCategory::Effects,
        c::HoldCategory::EffectFailed => w::HoldCategory::EffectFailed,
        c::HoldCategory::Uncertain { entry } => w::HoldCategory::Uncertain(w::HoldCategoryUncertain::new(
            limits,
            w::HoldCategoryUncertainParts { entry: *(entry) },
        )?),
        c::HoldCategory::Budget => w::HoldCategory::Budget,
        c::HoldCategory::Deadline => w::HoldCategory::Deadline,
        c::HoldCategory::HoldsWaited => w::HoldCategory::HoldsWaited,
        c::HoldCategory::Circle => w::HoldCategory::Circle,
        c::HoldCategory::ResourceUnavailable => w::HoldCategory::ResourceUnavailable,
        c::HoldCategory::ResourceAccess => w::HoldCategory::ResourceAccess,
        c::HoldCategory::PoolLost { pool } => w::HoldCategory::PoolLost(w::HoldCategoryPoolLost::new(
            limits,
            w::HoldCategoryPoolLostParts { pool: *(pool) },
        )?),
        c::HoldCategory::Procedure => w::HoldCategory::Procedure,
    })
}

pub(in crate::codec) fn wire_exceeded(value: &c::Exceeded, limits: &w::Limits) -> Result<w::Exceeded, w::Problem> {
    w::Exceeded::new(limits, w::ExceededParts { bound: wire_bound(&value.bound, limits)?, value: value.value })
}

pub(in crate::codec) fn wire_bound(value: &c::Bound, _limits: &w::Limits) -> Result<w::Bound, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Bound::Reasoning => w::Bound::Reasoning,
        c::Bound::Metadata => w::Bound::Metadata,
        c::Bound::Items => w::Bound::Items,
        c::Bound::Answer => w::Bound::Answer,
        c::Bound::Tools => w::Bound::Tools,
        c::Bound::HistoryItems => w::Bound::HistoryItems,
        c::Bound::History => w::Bound::History,
        c::Bound::Window => w::Bound::Window,
        c::Bound::Output => w::Bound::Output,
    })
}

pub(in crate::codec) fn wire_hold_failure(
    value: &c::HoldFailure,
    _limits: &w::Limits,
) -> Result<w::HoldFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldFailure::Transient => w::HoldFailure::Transient,
        c::HoldFailure::Permanent => w::HoldFailure::Permanent,
        c::HoldFailure::Run => w::HoldFailure::Run,
        c::HoldFailure::Agent => w::HoldFailure::Agent,
        c::HoldFailure::Lost => w::HoldFailure::Lost,
        c::HoldFailure::Invalid => w::HoldFailure::Invalid,
    })
}

fn ceiling_holder(value: c::CeilingHolder, limits: &w::Limits) -> Result<w::CeilingHolder, w::Problem> {
    Ok(match value {
        c::CeilingHolder::Rules => w::CeilingHolder::Rules,
        c::CeilingHolder::Policy => w::CeilingHolder::Policy,
        c::CeilingHolder::Role(role) => {
            w::CeilingHolder::Role(w::CeilingRole::new(limits, w::CeilingRoleParts { role })?)
        }
    })
}
fn ceiling_dimension(value: c::CeilingDimension) -> w::CeilingDimension {
    match value {
        c::CeilingDimension::PeriodSpend => w::CeilingDimension::PeriodSpend,
        c::CeilingDimension::Spend => w::CeilingDimension::Spend,
        c::CeilingDimension::Tools => w::CeilingDimension::Tools,
        c::CeilingDimension::Tasks => w::CeilingDimension::Tasks,
        c::CeilingDimension::Depth => w::CeilingDimension::Depth,
        c::CeilingDimension::Executors => w::CeilingDimension::Executors,
        c::CeilingDimension::Grants => w::CeilingDimension::Grants,
        c::CeilingDimension::Deadline => w::CeilingDimension::Deadline,
        c::CeilingDimension::Notes => w::CeilingDimension::Notes,
    }
}
