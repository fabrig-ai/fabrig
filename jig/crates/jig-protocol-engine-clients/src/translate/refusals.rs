//! Sparse entrance facts without invented names, words or bounds (client/domain/contract/answers.md, 1).
use crate::generated::client as mapped;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_people as p;
use skein_lib::Writer;
pub(crate) fn refusal(value: p::Refusal, limits: &w::Limits) -> Result<w::Refusal, w::Problem> {
    Ok(match value {
        p::Refusal::Forgotten { oldest } => w::Refusal::Forgotten(w::RefusalForgotten { oldest }),
        p::Refusal::NoTerminal => w::Refusal::NoTerminal,
        p::Refusal::LoginUnderWay => w::Refusal::LoginUnderWay,
        p::Refusal::NotLending => w::Refusal::NotLending,
        p::Refusal::NameTaken => w::Refusal::NameTaken,
        p::Refusal::Role { held } => {
            let held = match held {
                Some(value) => Some(mapped::held_role_to_record(&value)),
                None => None,
            };
            w::Refusal::Role(w::RefusalRole { held })
        }
        p::Refusal::Authority { lacking, proposable } => w::Refusal::Authority(w::RefusalAuthority {
            lacking: lack(lacking),
            proposable: match proposable {
                Some(p::Accepter::Role(_)) => Some(true),
                Some(p::Accepter::Nobody) => Some(false),
                None => None,
            },
        }),
        p::Refusal::Limit { which } => {
            let which = w::Limit::Admission(w::LimitAdmission { which: limit(which) });
            w::Refusal::Limit(w::RefusalLimit { which })
        }
        p::Refusal::Moved { now } => moved(now),
        p::Refusal::Ended { how } => w::Refusal::Ended(w::RefusalEnded { how: ending(how, limits)? }),
        p::Refusal::Taken { by } => w::Refusal::Taken(w::RefusalTaken { by }),
        p::Refusal::Judged { at, setting, why } => {
            w::Refusal::Judged(w::RefusalJudged { at, setting: encode_setting(setting), why })
        }
        p::Refusal::Stale { newest } => w::Refusal::Stale(w::RefusalStale { newest }),
        p::Refusal::Full { revision, tasks, more } => w::Refusal::Full(w::RefusalFull {
            revision,
            tasks: names(&tasks, limits.refusal_full_tasks, w::paths::REFUSAL_FULL_TASKS)?,
            more,
        }),
        p::Refusal::NotWellFormed { at, why } => w::Refusal::Malformed(w::RefusalMalformed {
            at,
            why: match why {
                p::Malformed::Settings { setting, fault, detail } => encode_malformed_settings(setting, fault, detail),
                p::Malformed::SettingsUnchanged => w::Malformed::SettingsUnchanged,
                p::Malformed::Running => w::Malformed::Running,
                p::Malformed::Name(rule) => {
                    w::Malformed::Name(w::MalformedName { rule: mapped::name_rule_to_record(&rule) })
                }
                p::Malformed::AsAllowedTwice => w::Malformed::TwoAsAllowed,
                p::Malformed::Conversation => w::Malformed::ConversationNotAgent,

                p::Malformed::EndedBy => w::Malformed::EndedBy,
                p::Malformed::Cycle => w::Malformed::Cycle,
                p::Malformed::ProposedAsAllowed => w::Malformed::ProposedAsAllowed,
            },
        }),
        p::Refusal::LiveTasks { tasks, more } => w::Refusal::LiveTasks(w::RefusalLiveTasks {
            tasks: names(&tasks, limits.refusal_live_tasks_tasks, w::paths::REFUSAL_LIVE_TASKS_TASKS)?,
            more,
        }),
        p::Refusal::Circle { tasks, cut } => w::Refusal::Circle(w::RefusalCircle {
            tasks: names(&tasks, limits.refusal_circle_tasks, w::paths::REFUSAL_CIRCLE_TASKS)?,
            cut,
        }),
        p::Refusal::Kept { pending } => w::Refusal::Kept(w::RefusalKept {
            pending: names(&pending, limits.refusal_kept_pending, w::paths::REFUSAL_KEPT_PENDING)?,
        }),
        p::Refusal::Unlinked { worker } => w::Refusal::Unlinked(w::RefusalUnlinked { worker }),
        p::Refusal::Concluded { at } => {
            w::Refusal::Concluded(w::RefusalConcluded { at: skein_codec::convert::wall_to_record(&at) })
        }
        p::Refusal::Dependency { dependency, how } => w::Refusal::Dependency(w::RefusalDependency {
            dependency,
            how: match how {
                p::DependencyStatus::Done => w::TaskStatus::Done,
                p::DependencyStatus::Failed => w::TaskStatus::Failed,
                p::DependencyStatus::Cancelled => w::TaskStatus::Cancelled,
            },
        }),
        p::Refusal::NotWaiting => w::Refusal::NotWaiting,
        p::Refusal::NoFurther => w::Refusal::NoFurther,
        p::Refusal::Unknown => w::Refusal::Unknown,
        p::Refusal::SignInRedirect { why } => {
            w::Refusal::SignInRedirect(w::RefusalSignInRedirect { why: mapped::sign_in_redirect_why_to_record(&why) })
        }
        p::Refusal::AccountMove { why } => {
            w::Refusal::AccountMove(w::RefusalAccountMove { why: mapped::account_move_why_to_record(&why) })
        }
        p::Refusal::Account { account, why } => {
            w::Refusal::Account(w::RefusalAccount { account, why: mapped::account_why_to_record(&why) })
        }
        p::Refusal::Provider => w::Refusal::Provider,
        p::Refusal::NotOffered => w::Refusal::NotOffered,
        p::Refusal::ProgramNameReserved => w::Refusal::ProgramNameReserved,
        p::Refusal::NoEscalationHolder => w::Refusal::NoEscalationHolder,
        p::Refusal::KeyConflict => w::Refusal::KeyConflict,
        // Only a made or amended task's refusal names a model, which the answer names by the
        // configured models (answers.rs); an offer's admission is people's own check.
        p::Refusal::Llm { .. } => {
            return Err(w::Problem { path: w::paths::REFUSAL_LLM_MODEL, reason: skein_codec::Reason::Bound });
        }
    })
}
fn lack(value: p::Lack) -> w::Lacking {
    match value {
        p::Lack::Ceiling { holder, dimension, value, asked } => w::Lacking::Ceiling(w::LackingCeiling {
            holder: mapped::ceiling_holder_to_record(&holder),
            dimension: mapped::ceiling_dimension_to_record(&dimension),
            value,
            asked,
        }),
        p::Lack::Approver => w::Lacking::Approver,
        p::Lack::Funding => w::Lacking::Funding,
        p::Lack::Tool => w::Lacking::Tool,
        p::Lack::Scope => w::Lacking::Scope,
        p::Lack::Models => w::Lacking::Models,
        p::Lack::HoldOwner { hold, owner } => {
            w::Lacking::HoldOwner(w::LackingHoldOwner { hold, owner: mapped::hold_owner_to_record(&owner) })
        }
        p::Lack::Reference => w::Lacking::Reference,
        p::Lack::ResourceAccess => w::Lacking::ResourceAccess,
        p::Lack::ResourceUnavailable => w::Lacking::ResourceUnavailable,
        p::Lack::Shape => w::Lacking::Shape,
        p::Lack::Account => w::Lacking::Account,
        p::Lack::Writer => w::Lacking::Writer,
        p::Lack::Budget { needs, left } => w::Lacking::Budget(w::LackingBudget { needed: needs, left }),
        p::Lack::Grant { kind } => w::Lacking::GrantKind(w::LackingGrantKind { kind }),
        p::Lack::TaskKind { .. } => w::Lacking::TaskKind(w::LackingTaskKind { executor: None }),
        p::Lack::Depth { most } => w::Lacking::Depth(w::LackingDepth { needed: None, allowed: most }),
        p::Lack::Deadline => w::Lacking::Deadline(w::LackingDeadline { asked: None, latest: None }),
        p::Lack::Tasks { left } => w::Lacking::Tasks(w::LackingTasks { needed: None, left }),
        p::Lack::Again { required } => w::Lacking::Again(w::LackingAgain { required }),
        p::Lack::Required { connector, kind } => w::Lacking::Required(w::LackingRequired { connector, kind }),
        p::Lack::Failed { connector, kind } => w::Lacking::Failed(w::LackingFailed { connector, kind }),
        p::Lack::Unguarded { connector, kind } => w::Lacking::Unguarded(w::LackingUnguarded { connector, kind }),
    }
}
pub(super) fn limit(value: p::LimitKind) -> w::AdmissionLimit {
    mapped::admission_limit_to_record(&value)
}
fn moved(value: p::Moved) -> w::Refusal {
    let (revision, what) = match value {
        p::Moved::Hold { revision, reason } => {
            let value = hold(reason);
            let reason = Some(w::CurrentHold::Reason(w::CurrentHoldReason { value }));
            (revision, w::Moved::Hold(w::MovedHold { reason }))
        }
        p::Moved::PlacedHold { revision } => (revision, w::Moved::PlacedHold(w::MovedPlacedHold { revision })),
        p::Moved::Spec { revision } => (revision, w::Moved::Spec),
        p::Moved::Person { revision } => (revision, w::Moved::PersonTask),
        p::Moved::Note { revision } => (u64::from(revision), w::Moved::Note),
    };
    let now = w::Revision { value: revision };
    w::Refusal::Moved(w::RefusalMoved { now, what })
}
fn hold(value: p::HoldReason) -> w::HoldCategory {
    match value {
        p::HoldReason::Stalled => w::HoldCategory::Stalled,
        p::HoldReason::Stopped => w::HoldCategory::Stopped,
        p::HoldReason::Drift => w::HoldCategory::Drift,
        p::HoldReason::Effects => w::HoldCategory::Effects,
        p::HoldReason::EffectFailed => w::HoldCategory::EffectFailed,
        p::HoldReason::Budget => w::HoldCategory::Budget,
        p::HoldReason::Deadline => w::HoldCategory::Deadline,
        p::HoldReason::HoldsWaited => w::HoldCategory::HoldsWaited,
        p::HoldReason::ResourceUnavailable => w::HoldCategory::ResourceUnavailable,
        p::HoldReason::ResourceAccess => w::HoldCategory::ResourceAccess,
        p::HoldReason::Procedure => w::HoldCategory::Procedure,
        p::HoldReason::Circle => w::HoldCategory::Circle,
        p::HoldReason::Dependency { task } => w::HoldCategory::Dependency(w::HoldCategoryDependency { task }),
        p::HoldReason::Placed { hold } => w::HoldCategory::Placed(w::HoldCategoryPlaced { hold }),
        p::HoldReason::Uncertain { entry } => w::HoldCategory::Uncertain(w::HoldCategoryUncertain { entry }),
        p::HoldReason::PoolLost { pool } => w::HoldCategory::PoolLost(w::HoldCategoryPoolLost { pool }),
        p::HoldReason::Exceeded(value) => w::HoldCategory::Exceeded(w::Exceeded {
            bound: match value.bound {
                p::Bound::Reasoning => w::Bound::Reasoning,
                p::Bound::Metadata => w::Bound::Metadata,
                p::Bound::Items => w::Bound::Items,
                p::Bound::Answer => w::Bound::Answer,
                p::Bound::Tools => w::Bound::Tools,
                p::Bound::HistoryItems => w::Bound::HistoryItems,
                p::Bound::History => w::Bound::History,
                p::Bound::Window => w::Bound::Window,
                p::Bound::Output => w::Bound::Output,
            },
            value: value.value,
        }),
        p::HoldReason::Failures { class } => {
            let class = match class {
                p::FailureClass::Transient => w::HoldFailure::Transient,
                p::FailureClass::Permanent => w::HoldFailure::Permanent,
                p::FailureClass::Run => w::HoldFailure::Run,
                p::FailureClass::Agent => w::HoldFailure::Agent,
                p::FailureClass::Lost => w::HoldFailure::Lost,
                p::FailureClass::Invalid => w::HoldFailure::Invalid,
                p::FailureClass::Infrastructure => w::HoldFailure::Infrastructure,
            };
            w::HoldCategory::Failures(w::HoldCategoryFailures { class })
        }
    }
}
fn actor(value: p::Party) -> w::Actor {
    match value {
        p::Party::Person(value) => w::Actor::Party(w::ActorParty { value }),
        p::Party::Task(value) => w::Actor::Task(w::ActorTask { value }),
        p::Party::Deployment { project } => w::Actor::ProjectDeployment(w::ActorProjectDeployment { project }),
    }
}
fn ending(value: p::Ending, limits: &w::Limits) -> Result<w::Ending, w::Problem> {
    let value = match value {
        p::Ending::Done => w::EndingFields::Done,
        p::Ending::Failed => w::EndingFields::Failed(w::EndingFailedFields { reason: None }),
        p::Ending::Cancelled { by, at } => w::EndingFields::Cancelled(w::EndingCancelledFields {
            by: actor(by),
            reason: None,
            at: Some(skein_codec::convert::wall_to_record(&at)),
        }),
        p::Ending::Decided { by, at } => w::EndingFields::Decided(w::EndingDecidedFields {
            by,
            party: None,
            how: None,
            at: skein_codec::convert::wall_to_record(&at),
        }),
        p::Ending::Deleted => w::EndingFields::Deleted,
    };
    let row = w::Fact::Ending(w::EndingRow { parent: None, value });
    let size = row
        .measure(limits)?
        .checked_add(4)
        .ok_or(w::Problem { path: w::paths::ENDING_ROWS, reason: skein_codec::Reason::Bound })?;
    if size > limits.ending_rows {
        return Err(w::Problem { path: w::paths::ENDING_ROWS, reason: skein_codec::Reason::Bound });
    }
    let mut rows = Writer::new(skein_lib::length::usize_of(size));
    rows.put(&row.measure(limits)?.to_be_bytes()).expect("measured piece length");
    row.write(limits, &mut rows).expect("measured ending row");
    Ok(w::Ending { rows: rows.finish() })
}

fn names(values: &[u64], bound: u32, path: w::Path) -> Result<skein_lib::List<u64>, w::Problem> {
    let mut rows = skein_lib::List::with_capacity(bound);
    for number in values {
        if rows.push(*number).is_err() {
            return Err(w::Problem { path, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(rows)
}

fn encode_setting(value: p::Setting) -> w::Setting {
    match value {
        jig_domain_people::Setting::ProjectPolicy { project } => {
            w::Setting::ProjectPolicy(w::ProjectPolicySetting { project })
        }
        p::Setting::Charter(number) => w::Setting::Charter(w::SettingNumber { number }),
        p::Setting::Template(number) => w::Setting::Template(w::SettingNumber { number }),
        p::Setting::Policy(number) => w::Setting::Policy(w::SettingNumber { number }),
        p::Setting::Model(number) => w::Setting::Model(w::SettingNumber { number }),
        p::Setting::Worker(number) => w::Setting::Worker(w::SettingNumber { number }),
        p::Setting::Provider(number) => w::Setting::Provider(w::SettingNumber { number }),
        p::Setting::Rules => w::Setting::Rules,
        p::Setting::Limits => w::Setting::Limits,
        p::Setting::Application { kind, scope, number } => {
            w::Setting::Application(w::ApplicationSetting { kind, scope, number })
        }
    }
}
fn encode_setting_fault(value: p::SettingFault) -> w::SettingFault {
    match value {
        p::SettingFault::Shape => w::SettingFault::Shape,
        p::SettingFault::Count => w::SettingFault::Count,
        p::SettingFault::Reference => w::SettingFault::Reference,
        p::SettingFault::Rules => w::SettingFault::Rules,
        p::SettingFault::Budget => w::SettingFault::Budget,
        p::SettingFault::Grace => w::SettingFault::Grace,
        p::SettingFault::Owner => w::SettingFault::Owner,
        p::SettingFault::Judged => w::SettingFault::Judged,
        p::SettingFault::Linked => w::SettingFault::Linked,
        p::SettingFault::Accounts => w::SettingFault::Accounts,
        p::SettingFault::Range => w::SettingFault::Range,
    }
}

/// Translate the document refusal facts without checking the settings again.
fn encode_setting_detail(value: p::SettingDetail) -> w::SettingDetail {
    w::SettingDetail {
        member: match value.member {
            p::SettingMember::Documents => w::SettingMember::Documents,
            p::SettingMember::NameSegments => w::SettingMember::NameSegments,
            p::SettingMember::NameSegmentBytes => w::SettingMember::NameSegmentBytes,
            p::SettingMember::Path => w::SettingMember::Path,
            p::SettingMember::Heading => w::SettingMember::Heading,
            p::SettingMember::Connector => w::SettingMember::Connector,
            p::SettingMember::StoredPolicy => w::SettingMember::StoredPolicy,
        },
        rule: match value.rule {
            p::SettingRule::Maximum => w::SettingRule::Maximum,
            p::SettingRule::Minimum => w::SettingRule::Minimum,
            p::SettingRule::Registered => w::SettingRule::Registered,
        },
        document: value.document,
        segment: value.segment,
        supplied: value.supplied,
        bound: value.bound,
    }
}

fn encode_malformed_settings(
    setting: p::Setting,
    fault: p::SettingFault,
    detail: Option<p::SettingDetail>,
) -> w::Malformed {
    w::Malformed::Settings(w::MalformedSettings {
        setting: encode_setting(setting),
        fault: encode_setting_fault(fault),
        detail: match detail {
            Some(value) => Some(encode_setting_detail(value)),
            None => None,
        },
    })
}
