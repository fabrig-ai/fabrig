//! Wire projections of owner deployment facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, pieces, w};
use super::{
    wire_admission_limit, wire_document_text, wire_funder, wire_owner_account_holder, wire_owner_account_source,
    wire_owner_account_use, wire_owner_requester, wire_owner_task_link, wire_payer_readiness, wire_proposer,
    wire_provider_offer, wire_seen_through,
};
pub(in crate::codec) fn wire_task_source_place(
    value: &c::OwnerAccountPlace,
    _limits: &w::Limits,
) -> Result<w::TaskSourcePlace, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountPlace::Directory => w::TaskSourcePlace::Directory,
        c::OwnerAccountPlace::File => w::TaskSourcePlace::File,
    })
}

pub(in crate::codec) fn wire_task_source_unread(
    value: &c::OwnerAccountUnread,
    _limits: &w::Limits,
) -> Result<w::TaskSourceUnread, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountUnread::Unreadable => w::TaskSourceUnread::Unreadable,
        c::OwnerAccountUnread::Refused => w::TaskSourceUnread::Refused,
        c::OwnerAccountUnread::Failed => w::TaskSourceUnread::Failed,
        c::OwnerAccountUnread::Stalled => w::TaskSourceUnread::Stalled,
        c::OwnerAccountUnread::TooLarge => w::TaskSourceUnread::TooLarge,
    })
}

pub(in crate::codec) fn wire_owner_host_kind(
    value: &c::OwnerHostKind,
    _limits: &w::Limits,
) -> Result<w::OwnerHostKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHostKind::Worker => w::OwnerHostKind::Worker,
        c::OwnerHostKind::Engine => w::OwnerHostKind::Engine,
    })
}

pub(in crate::codec) fn wire_owner_hello_refusal(
    value: &c::OwnerHelloRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerHelloRefusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHelloRefusal::Agent { why } => w::OwnerHelloRefusal::Agent(w::OwnerHelloRefusalAgent::new(
            limits,
            w::OwnerHelloRefusalAgentParts { why: wire_owner_agent_shortfall(why, limits)? },
        )?),
        c::OwnerHelloRefusal::ReservedChannel => w::OwnerHelloRefusal::ReservedChannel,
        c::OwnerHelloRefusal::Position { worker } => w::OwnerHelloRefusal::Position(w::OwnerHelloRefusalPosition::new(
            limits,
            w::OwnerHelloRefusalPositionParts { worker: *(worker) },
        )?),
        c::OwnerHelloRefusal::Workers { limit } => w::OwnerHelloRefusal::Workers(w::OwnerHelloRefusalWorkers::new(
            limits,
            w::OwnerHelloRefusalWorkersParts { limit: *(limit) },
        )?),
        c::OwnerHelloRefusal::StopBound { grace } => w::OwnerHelloRefusal::StopBound(
            w::OwnerHelloRefusalStopBound::new(limits, w::OwnerHelloRefusalStopBoundParts { grace: *(grace) })?,
        ),
    })
}

pub(in crate::codec) fn wire_owner_agent_shortfall(
    value: &c::OwnerAgentShortfall,
    limits: &w::Limits,
) -> Result<w::OwnerAgentShortfall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAgentShortfall::Model { charter, model } => {
            w::OwnerAgentShortfall::Model(w::OwnerAgentShortfallModel::new(
                limits,
                w::OwnerAgentShortfallModelParts { charter: *(charter), model: *(model) },
            )?)
        }
        c::OwnerAgentShortfall::Window { charter, model } => {
            w::OwnerAgentShortfall::Window(w::OwnerAgentShortfallWindow::new(
                limits,
                w::OwnerAgentShortfallWindowParts { charter: *(charter), model: *(model) },
            )?)
        }
        c::OwnerAgentShortfall::Output { charter, model } => {
            w::OwnerAgentShortfall::Output(w::OwnerAgentShortfallOutput::new(
                limits,
                w::OwnerAgentShortfallOutputParts { charter: *(charter), model: *(model) },
            )?)
        }
        c::OwnerAgentShortfall::Turns => w::OwnerAgentShortfall::Turns,
        c::OwnerAgentShortfall::Spend => w::OwnerAgentShortfall::Spend,
        c::OwnerAgentShortfall::Time => w::OwnerAgentShortfall::Time,
        c::OwnerAgentShortfall::Waiting => w::OwnerAgentShortfall::Waiting,
        c::OwnerAgentShortfall::Checks => w::OwnerAgentShortfall::Checks,
        c::OwnerAgentShortfall::Message => w::OwnerAgentShortfall::Message,
        c::OwnerAgentShortfall::Inbox => w::OwnerAgentShortfall::Inbox,
        c::OwnerAgentShortfall::Unacknowledged => w::OwnerAgentShortfall::Unacknowledged,
    })
}

pub(in crate::codec) fn wire_owner_deployment(
    value: &c::OwnerDeployment,
    limits: &w::Limits,
) -> Result<w::OwnerDeployment, w::Problem> {
    w::OwnerDeployment::new(
        limits,
        w::OwnerDeploymentParts {
            skein_bools: [value.cut, value.connectors_cut, value.providers_cut, value.readiness_cut],
            store: wire_store_fill(&value.store, limits)?,
            hosts: pieces(
                value.hosts.iter().map(|item| wire_owner_host(item, limits)),
                w::OwnerHost::measure,
                w::OwnerHost::encode,
            )?,
            attempts: pieces(
                value.attempts.iter().map(|item| wire_owner_host_run(item, limits)),
                w::OwnerHostRun::measure,
                w::OwnerHostRun::encode,
            )?,
            accounts: pieces(
                value.accounts.iter().map(|item| wire_owner_account(item, limits)),
                w::OwnerAccount::measure,
                w::OwnerAccount::encode,
            )?,
            tasks: wire_owner_capacity(&value.tasks, limits)?,
            workers: wire_owner_capacity(&value.workers, limits)?,
            attempts_capacity: wire_owner_capacity(&value.attempts_capacity, limits)?,
            waiting_attempts: value.waiting_attempts,
            proposals: wire_owner_capacity(&value.proposals, limits)?,
            projects: list(value.projects.iter().map(|item| wire_owner_project_capacity(item, limits)))?,
            grace_bound: value.grace_bound,
            hello_refusals: list(value.hello_refusals.iter().map(|item| wire_owner_hello_refusal_line(item, limits)))?,
            hosts_forgotten: value.hosts_forgotten,
            hello_refusals_forgotten: value.hello_refusals_forgotten,
            person_room_refusals: list(
                value.person_room_refusals.iter().map(|item| wire_owner_person_capacity_refusal(item, limits)),
            )?,
            person_room_refusals_forgotten: value.person_room_refusals_forgotten,
            person_answer_retention: value.person_answer_retention,
            task_room_refusals: list(
                value.task_room_refusals.iter().map(|item| wire_owner_task_capacity_refusal(item, limits)),
            )?,
            task_room_refusals_forgotten: value.task_room_refusals_forgotten,
            connectors: list(value.connectors.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            providers: pieces(
                value.providers.iter().map(|item| wire_provider_offer(item, limits)),
                w::ProviderOffer::measure,
                w::ProviderOffer::encode,
            )?,
            key_bytes: value.key_bytes,
            readiness: pieces(
                value.readiness.iter().map(|item| wire_payer_readiness(item, limits)),
                w::PayerReadiness::measure,
                w::PayerReadiness::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_store_fill(value: &c::StoreFill, limits: &w::Limits) -> Result<w::StoreFill, w::Problem> {
    w::StoreFill::new(limits, w::StoreFillParts { bytes: value.bytes, mark: value.mark })
}

pub(in crate::codec) fn wire_owner_host(value: &c::OwnerHost, limits: &w::Limits) -> Result<w::OwnerHost, w::Problem> {
    w::OwnerHost::new(
        limits,
        w::OwnerHostParts {
            skein_bools: [value.awaiting_logins, value.draining, value.connected, value.cut],
            name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
            rule: wire_owner_host_rule(&value.rule, limits)?,
            programs: pieces(
                value.programs.iter().map(|item| wire_owner_program_login(item, limits)),
                w::OwnerProgramLogin::measure,
                w::OwnerProgramLogin::encode,
            )?,
            channel: value.channel,
            kind: wire_owner_host_kind(&value.kind, limits)?,
            slots: value.slots,
            used: value.used,
            free: value.free,
            grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            grace_remaining: value.grace_remaining.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            runs: pieces(
                value.runs.iter().map(|item| wire_owner_host_run(item, limits)),
                w::OwnerHostRun::measure,
                w::OwnerHostRun::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_owner_host_rule(
    value: &c::HostRule,
    limits: &w::Limits,
) -> Result<w::OwnerHostRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HostRule::Open => w::OwnerHostRule::Open,
        c::HostRule::Person(value) => {
            w::OwnerHostRule::Person(w::OwnerHostPerson::new(limits, w::OwnerHostPersonParts { party: *(value) })?)
        }
        c::HostRule::Several => w::OwnerHostRule::Several,
        c::HostRule::Unknown => w::OwnerHostRule::Unknown,
    })
}

pub(in crate::codec) fn wire_owner_program_login(
    value: &c::OwnerProgramLogin,
    limits: &w::Limits,
) -> Result<w::OwnerProgramLogin, w::Problem> {
    w::OwnerProgramLogin::new(
        limits,
        w::OwnerProgramLoginParts {
            name: bytes::copy_of(&value.name),
            state: value.state.as_ref().map(|item| wire_owner_program_state(item, limits)).transpose()?,
            sign_out: value.sign_out.as_ref().map(|item| wire_owner_program_sign_out(item, limits)).transpose()?,
            cleanup: value.cleanup,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_owner_program_state(
    value: &c::OwnerProgramState,
    limits: &w::Limits,
) -> Result<w::OwnerProgramState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProgramState::Usable => w::OwnerProgramState::Usable,
        c::OwnerProgramState::NotSignedIn => w::OwnerProgramState::NotSignedIn,
        c::OwnerProgramState::SignedInOtherwise { method } => w::OwnerProgramState::SignedInOtherwise(
            w::OwnerProgramOtherwise::new(limits, w::OwnerProgramOtherwiseParts { method: bytes::copy_of(method) })?,
        ),
        c::OwnerProgramState::Failed { reason, detail } => w::OwnerProgramState::Failed(w::OwnerProgramFailed::new(
            limits,
            w::OwnerProgramFailedParts {
                reason: wire_owner_program_failure(reason, limits)?,
                detail: bytes::copy_of(detail),
            },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_program_failure(
    value: &c::OwnerProgramFailure,
    _limits: &w::Limits,
) -> Result<w::OwnerProgramFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProgramFailure::Missing => w::OwnerProgramFailure::Missing,
        c::OwnerProgramFailure::Deadline => w::OwnerProgramFailure::Deadline,
        c::OwnerProgramFailure::Unreadable => w::OwnerProgramFailure::Unreadable,
        c::OwnerProgramFailure::Exited => w::OwnerProgramFailure::Exited,
        c::OwnerProgramFailure::Unconfigured => w::OwnerProgramFailure::Unconfigured,
        c::OwnerProgramFailure::Unknown => w::OwnerProgramFailure::Unknown,
        c::OwnerProgramFailure::Closed => w::OwnerProgramFailure::Closed,
    })
}

pub(in crate::codec) fn wire_owner_program_sign_out(
    value: &c::OwnerProgramSignOut,
    limits: &w::Limits,
) -> Result<w::OwnerProgramSignOut, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProgramSignOut::SignedOut => w::OwnerProgramSignOut::SignedOut,
        c::OwnerProgramSignOut::Failed { reason, detail } => {
            w::OwnerProgramSignOut::Failed(w::OwnerProgramFailed::new(
                limits,
                w::OwnerProgramFailedParts {
                    reason: wire_owner_program_failure(reason, limits)?,
                    detail: bytes::copy_of(detail),
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_host_run(
    value: &c::OwnerHostRun,
    limits: &w::Limits,
) -> Result<w::OwnerHostRun, w::Problem> {
    w::OwnerHostRun::new(
        limits,
        w::OwnerHostRunParts {
            task: wire_owner_task_link(&value.task, limits)?,
            attempt: value.attempt,
            kept: value.kept,
            listed: value.listed,
            state: wire_owner_attempt_state(&value.state, limits)?,
            location: wire_owner_run_location(&value.location, limits)?,
            grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_attempt_state(
    value: &c::OwnerAttemptState,
    _limits: &w::Limits,
) -> Result<w::OwnerAttemptState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAttemptState::Waiting => w::OwnerAttemptState::Waiting,
        c::OwnerAttemptState::Adopted => w::OwnerAttemptState::Adopted,
        c::OwnerAttemptState::Claimed => w::OwnerAttemptState::Claimed,
        c::OwnerAttemptState::Cancelled => w::OwnerAttemptState::Cancelled,
        c::OwnerAttemptState::Handed => w::OwnerAttemptState::Handed,
        c::OwnerAttemptState::Acknowledged => w::OwnerAttemptState::Acknowledged,
        c::OwnerAttemptState::Stray => w::OwnerAttemptState::Stray,
        c::OwnerAttemptState::Kept => w::OwnerAttemptState::Kept,
        c::OwnerAttemptState::Fenced => w::OwnerAttemptState::Fenced,
        c::OwnerAttemptState::Closed => w::OwnerAttemptState::Closed,
    })
}

pub(in crate::codec) fn wire_owner_run_location(
    value: &c::OwnerRunLocation,
    limits: &w::Limits,
) -> Result<w::OwnerRunLocation, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRunLocation::None => w::OwnerRunLocation::None,
        c::OwnerRunLocation::Connected { channel, kind } => {
            w::OwnerRunLocation::Connected(w::OwnerRunLocationConnected::new(
                limits,
                w::OwnerRunLocationConnectedParts { channel: *(channel), kind: wire_owner_host_kind(kind, limits)? },
            )?)
        }
        c::OwnerRunLocation::Adrift { channel, until, remaining } => {
            w::OwnerRunLocation::Adrift(w::OwnerRunLocationAdrift::new(
                limits,
                w::OwnerRunLocationAdriftParts {
                    channel: (channel).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                    remaining: (remaining).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_account(
    value: &c::OwnerAccount,
    limits: &w::Limits,
) -> Result<w::OwnerAccount, w::Problem> {
    w::OwnerAccount::new(
        limits,
        w::OwnerAccountParts {
            skein_bools: [value.plan, value.usable, value.expiring, value.attention, value.cut],
            number: value.number,
            place: value.place,
            uses: value
                .uses
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_owner_account_use(item, limits))))
                .transpose()?,
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            display: wire_document_text(&value.display, limits)?,
            holder: wire_owner_account_holder(&value.holder, limits)?,
            sources: pieces(
                value.sources.iter().map(|item| wire_owner_account_source(item, limits)),
                w::OwnerAccountSource::measure,
                w::OwnerAccountSource::encode,
            )?,
            generation: value.generation.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            state: wire_owner_account_state(&value.state, limits)?,
            spent_until: value.spent_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            runs: pieces(value.runs.iter().map(|item| facts::chip(item, limits)), w::Chip::measure, w::Chip::encode)?,
            waiting: pieces(
                value.waiting.iter().map(|item| facts::chip(item, limits)),
                w::Chip::measure,
                w::Chip::encode,
            )?,
            provider_name: value.provider_name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            manage_usage: value
                .manage_usage
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of(item)))
                .transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_account_state(
    value: &c::OwnerAccountState,
    limits: &w::Limits,
) -> Result<w::OwnerAccountState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountState::Asking => w::OwnerAccountState::Asking,
        c::OwnerAccountState::Granted => w::OwnerAccountState::Granted,
        c::OwnerAccountState::Later { until } => w::OwnerAccountState::Later(w::OwnerAccountStateLater::new(
            limits,
            w::OwnerAccountStateLaterParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::OwnerAccountState::SignInAgain => w::OwnerAccountState::SignInAgain,
        c::OwnerAccountState::Misconfigured => w::OwnerAccountState::Misconfigured,
        c::OwnerAccountState::Unloaded { at, why } => {
            w::OwnerAccountState::Unloaded(w::OwnerAccountStateUnloaded::new(
                limits,
                w::OwnerAccountStateUnloadedParts {
                    at: wire_owner_account_place(at, limits)?,
                    why: wire_owner_account_unread(why, limits)?,
                },
            )?)
        }
        c::OwnerAccountState::Cancelled => w::OwnerAccountState::Cancelled,
        c::OwnerAccountState::Closing => w::OwnerAccountState::Closing,
        c::OwnerAccountState::Moving => w::OwnerAccountState::Moving,
    })
}

pub(in crate::codec) fn wire_owner_account_place(
    value: &c::OwnerAccountPlace,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountPlace, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountPlace::Directory => w::OwnerAccountPlace::Directory,
        c::OwnerAccountPlace::File => w::OwnerAccountPlace::File,
    })
}

pub(in crate::codec) fn wire_owner_account_unread(
    value: &c::OwnerAccountUnread,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountUnread, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountUnread::Unreadable => w::OwnerAccountUnread::Unreadable,
        c::OwnerAccountUnread::Refused => w::OwnerAccountUnread::Refused,
        c::OwnerAccountUnread::Failed => w::OwnerAccountUnread::Failed,
        c::OwnerAccountUnread::Stalled => w::OwnerAccountUnread::Stalled,
        c::OwnerAccountUnread::TooLarge => w::OwnerAccountUnread::TooLarge,
    })
}

pub(in crate::codec) fn wire_owner_capacity(
    value: &c::OwnerCapacity,
    limits: &w::Limits,
) -> Result<w::OwnerCapacity, w::Problem> {
    w::OwnerCapacity::new(limits, w::OwnerCapacityParts { used: value.used, limit: value.limit })
}

pub(in crate::codec) fn wire_owner_project_capacity(
    value: &c::OwnerProjectCapacity,
    limits: &w::Limits,
) -> Result<w::OwnerProjectCapacity, w::Problem> {
    w::OwnerProjectCapacity::new(
        limits,
        w::OwnerProjectCapacityParts {
            project: value.project,
            tasks: wire_owner_capacity(&value.tasks, limits)?,
            proposals: value.proposals,
        },
    )
}

pub(in crate::codec) fn wire_owner_hello_refusal_line(
    value: &c::OwnerHelloRefusalLine,
    limits: &w::Limits,
) -> Result<w::OwnerHelloRefusalLine, w::Problem> {
    w::OwnerHelloRefusalLine::new(
        limits,
        w::OwnerHelloRefusalLineParts {
            channel: value.channel,
            at: value.at.as_nanos(),
            slots: value.slots,
            stop_bound: value.stop_bound,
            why: wire_owner_hello_refusal(&value.why, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_person_capacity_refusal(
    value: &c::OwnerPersonCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerPersonCapacityRefusal, w::Problem> {
    w::OwnerPersonCapacityRefusal::new(
        limits,
        w::OwnerPersonCapacityRefusalParts {
            key: wire_owner_request_key(&value.key, limits)?,
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            action: wire_owner_person_capacity_action(&value.action, limits)?,
            at: value.at.as_nanos(),
            cause: wire_owner_person_capacity_cause(&value.cause, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_request_key(
    value: &c::OwnerRequestKey,
    limits: &w::Limits,
) -> Result<w::OwnerRequestKey, w::Problem> {
    w::OwnerRequestKey::new(limits, w::OwnerRequestKeyParts { person: value.person, key: value.key })
}

pub(in crate::codec) fn wire_owner_person_capacity_action(
    value: &c::OwnerPersonCapacityAction,
    limits: &w::Limits,
) -> Result<w::OwnerPersonCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPersonCapacityAction::Lift { hold } => w::OwnerPersonCapacityAction::Lift(
            w::OwnerPersonCapacityActionLift::new(limits, w::OwnerPersonCapacityActionLiftParts { hold: *(hold) })?,
        ),
        c::OwnerPersonCapacityAction::Create => w::OwnerPersonCapacityAction::Create,
        c::OwnerPersonCapacityAction::Compound => w::OwnerPersonCapacityAction::Compound,
        c::OwnerPersonCapacityAction::Application { connector } => {
            w::OwnerPersonCapacityAction::Application(w::OwnerPersonCapacityActionApplication::new(
                limits,
                w::OwnerPersonCapacityActionApplicationParts { connector: *(connector) },
            )?)
        }
        c::OwnerPersonCapacityAction::Note { name } => w::OwnerPersonCapacityAction::Note(
            w::OwnerPersonCapacityActionNote::new(limits, w::OwnerPersonCapacityActionNoteParts { name: *(name) })?,
        ),
        c::OwnerPersonCapacityAction::Service => w::OwnerPersonCapacityAction::Service,
        c::OwnerPersonCapacityAction::Adoption => w::OwnerPersonCapacityAction::Adoption,
        c::OwnerPersonCapacityAction::Roles => w::OwnerPersonCapacityAction::Roles,
        c::OwnerPersonCapacityAction::Policy => w::OwnerPersonCapacityAction::Policy,
        c::OwnerPersonCapacityAction::Pool { person } => w::OwnerPersonCapacityAction::Pool(
            w::OwnerPersonCapacityActionPool::new(limits, w::OwnerPersonCapacityActionPoolParts { person: *(person) })?,
        ),
        c::OwnerPersonCapacityAction::Escalation { task } => {
            w::OwnerPersonCapacityAction::Escalation(w::OwnerPersonCapacityActionEscalation::new(
                limits,
                w::OwnerPersonCapacityActionEscalationParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::Proposal { proposer, number } => {
            w::OwnerPersonCapacityAction::Proposal(w::OwnerPersonCapacityActionProposal::new(
                limits,
                w::OwnerPersonCapacityActionProposalParts {
                    proposer: wire_proposer(proposer, limits)?,
                    number: *(number),
                },
            )?)
        }
        c::OwnerPersonCapacityAction::Words { task } => w::OwnerPersonCapacityAction::Words(
            w::OwnerPersonCapacityActionWords::new(limits, w::OwnerPersonCapacityActionWordsParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Question { task, question } => {
            w::OwnerPersonCapacityAction::Question(w::OwnerPersonCapacityActionQuestion::new(
                limits,
                w::OwnerPersonCapacityActionQuestionParts { task: *(task), question: *(question) },
            )?)
        }
        c::OwnerPersonCapacityAction::Amend { task } => w::OwnerPersonCapacityAction::Amend(
            w::OwnerPersonCapacityActionAmend::new(limits, w::OwnerPersonCapacityActionAmendParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Move { task } => w::OwnerPersonCapacityAction::MoveTask(
            w::OwnerPersonCapacityActionMove::new(limits, w::OwnerPersonCapacityActionMoveParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::TakePerson { task } => {
            w::OwnerPersonCapacityAction::TakePerson(w::OwnerPersonCapacityActionTakePerson::new(
                limits,
                w::OwnerPersonCapacityActionTakePersonParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::HandBackPerson { task } => {
            w::OwnerPersonCapacityAction::HandBackPerson(w::OwnerPersonCapacityActionHandBackPerson::new(
                limits,
                w::OwnerPersonCapacityActionHandBackPersonParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::AnswerPerson { task } => {
            w::OwnerPersonCapacityAction::AnswerPerson(w::OwnerPersonCapacityActionAnswerPerson::new(
                limits,
                w::OwnerPersonCapacityActionAnswerPersonParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::Stop { task } => w::OwnerPersonCapacityAction::Stop(
            w::OwnerPersonCapacityActionStop::new(limits, w::OwnerPersonCapacityActionStopParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Cancel { task } => w::OwnerPersonCapacityAction::Cancel(
            w::OwnerPersonCapacityActionCancel::new(limits, w::OwnerPersonCapacityActionCancelParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Release { task } => {
            w::OwnerPersonCapacityAction::Release(w::OwnerPersonCapacityActionRelease::new(
                limits,
                w::OwnerPersonCapacityActionReleaseParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::MarkSeen { seen } => {
            w::OwnerPersonCapacityAction::MarkSeen(w::OwnerPersonCapacityActionMarkSeen::new(
                limits,
                w::OwnerPersonCapacityActionMarkSeenParts { seen: wire_seen_through(seen, limits)? },
            )?)
        }
        c::OwnerPersonCapacityAction::AccountSignIn => w::OwnerPersonCapacityAction::AccountSignIn,
        c::OwnerPersonCapacityAction::AccountSignOut => w::OwnerPersonCapacityAction::AccountSignOut,
        c::OwnerPersonCapacityAction::AccountChange => w::OwnerPersonCapacityAction::AccountChange,
    })
}

pub(in crate::codec) fn wire_owner_person_capacity_cause(
    value: &c::OwnerPersonCapacityCause,
    limits: &w::Limits,
) -> Result<w::OwnerPersonCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPersonCapacityCause::Requests { limit } => {
            w::OwnerPersonCapacityCause::Requests(w::OwnerPersonCapacityCauseRequests::new(
                limits,
                w::OwnerPersonCapacityCauseRequestsParts { limit: *(limit) },
            )?)
        }
        c::OwnerPersonCapacityCause::Pending { limit } => {
            w::OwnerPersonCapacityCause::Pending(w::OwnerPersonCapacityCausePending::new(
                limits,
                w::OwnerPersonCapacityCausePendingParts { limit: *(limit) },
            )?)
        }
        c::OwnerPersonCapacityCause::Waiters { limit } => {
            w::OwnerPersonCapacityCause::Waiters(w::OwnerPersonCapacityCauseWaiters::new(
                limits,
                w::OwnerPersonCapacityCauseWaitersParts { limit: *(limit) },
            )?)
        }
        c::OwnerPersonCapacityCause::Limit { which } => {
            w::OwnerPersonCapacityCause::Limit(w::OwnerPersonCapacityCauseLimit::new(
                limits,
                w::OwnerPersonCapacityCauseLimitParts { which: wire_admission_limit(which, limits)? },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_task_capacity_refusal(
    value: &c::OwnerTaskCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityRefusal, w::Problem> {
    w::OwnerTaskCapacityRefusal::new(
        limits,
        w::OwnerTaskCapacityRefusalParts {
            request: wire_owner_task_capacity_request(&value.request, limits)?,
            at: value.at.as_nanos(),
        },
    )
}

pub(in crate::codec) fn wire_owner_task_capacity_request(
    value: &c::OwnerTaskCapacityRequest,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityRequest, w::Problem> {
    w::OwnerTaskCapacityRequest::new(
        limits,
        w::OwnerTaskCapacityRequestParts {
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            by: wire_owner_requester(&value.by, limits)?,
            action: wire_owner_task_capacity_action(&value.action, limits)?,
            cause: wire_owner_task_capacity_cause(&value.cause, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_task_capacity_action(
    value: &c::OwnerTaskCapacityAction,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskCapacityAction::Batch { first, members } => {
            w::OwnerTaskCapacityAction::Batch(w::OwnerTaskCapacityActionBatch::new(
                limits,
                w::OwnerTaskCapacityActionBatchParts {
                    first: (first).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    members: *(members),
                },
            )?)
        }
        c::OwnerTaskCapacityAction::Proposal { number } => {
            w::OwnerTaskCapacityAction::Proposal(w::OwnerTaskCapacityActionProposal::new(
                limits,
                w::OwnerTaskCapacityActionProposalParts { number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::PersonProposal { number } => {
            w::OwnerTaskCapacityAction::PersonProposal(w::OwnerTaskCapacityActionPersonProposal::new(
                limits,
                w::OwnerTaskCapacityActionPersonProposalParts { number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::Message { task, number } => {
            w::OwnerTaskCapacityAction::Message(w::OwnerTaskCapacityActionMessage::new(
                limits,
                w::OwnerTaskCapacityActionMessageParts { task: *(task), number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::Source { funder } => {
            w::OwnerTaskCapacityAction::Source(w::OwnerTaskCapacityActionSource::new(
                limits,
                w::OwnerTaskCapacityActionSourceParts { funder: wire_funder(funder, limits)? },
            )?)
        }
        c::OwnerTaskCapacityAction::Introduce { left, right } => {
            w::OwnerTaskCapacityAction::Introduce(w::OwnerTaskCapacityActionIntroduce::new(
                limits,
                w::OwnerTaskCapacityActionIntroduceParts { left: *(left), right: *(right) },
            )?)
        }
        c::OwnerTaskCapacityAction::Subscription { number } => {
            w::OwnerTaskCapacityAction::Subscription(w::OwnerTaskCapacityActionSubscription::new(
                limits,
                w::OwnerTaskCapacityActionSubscriptionParts { number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::Effect { entry } => w::OwnerTaskCapacityAction::Effect(
            w::OwnerTaskCapacityActionEffect::new(limits, w::OwnerTaskCapacityActionEffectParts { entry: *(entry) })?,
        ),
        c::OwnerTaskCapacityAction::Claim { attempt } => w::OwnerTaskCapacityAction::Claim(
            w::OwnerTaskCapacityActionClaim::new(limits, w::OwnerTaskCapacityActionClaimParts { attempt: *(attempt) })?,
        ),
        c::OwnerTaskCapacityAction::Release => w::OwnerTaskCapacityAction::Release,
        c::OwnerTaskCapacityAction::Move => w::OwnerTaskCapacityAction::MoveTask,
        c::OwnerTaskCapacityAction::Place { hold } => w::OwnerTaskCapacityAction::Place(
            w::OwnerTaskCapacityActionPlace::new(limits, w::OwnerTaskCapacityActionPlaceParts { hold: *(hold) })?,
        ),
    })
}

pub(in crate::codec) fn wire_owner_task_capacity_cause(
    value: &c::OwnerTaskCapacityCause,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskCapacityCause::Tasks { limit } => w::OwnerTaskCapacityCause::Tasks(
            w::OwnerTaskCapacityCauseTasks::new(limits, w::OwnerTaskCapacityCauseTasksParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::CreationFlights { limit } => {
            w::OwnerTaskCapacityCause::CreationFlights(w::OwnerTaskCapacityCauseCreationFlights::new(
                limits,
                w::OwnerTaskCapacityCauseCreationFlightsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::ResourceReports { limit } => {
            w::OwnerTaskCapacityCause::ResourceReports(w::OwnerTaskCapacityCauseResourceReports::new(
                limits,
                w::OwnerTaskCapacityCauseResourceReportsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::ProjectTasks { limit } => {
            w::OwnerTaskCapacityCause::ProjectTasks(w::OwnerTaskCapacityCauseProjectTasks::new(
                limits,
                w::OwnerTaskCapacityCauseProjectTasksParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Tree { limit } => w::OwnerTaskCapacityCause::Tree(
            w::OwnerTaskCapacityCauseTree::new(limits, w::OwnerTaskCapacityCauseTreeParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::Delegates { limit } => {
            w::OwnerTaskCapacityCause::Delegates(w::OwnerTaskCapacityCauseDelegates::new(
                limits,
                w::OwnerTaskCapacityCauseDelegatesParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Inbox { task, messages, bytes } => {
            w::OwnerTaskCapacityCause::Inbox(w::OwnerTaskCapacityCauseInbox::new(
                limits,
                w::OwnerTaskCapacityCauseInboxParts { task: *(task), messages: *(messages), bytes: *(bytes) },
            )?)
        }
        c::OwnerTaskCapacityCause::Holdings { limit } => w::OwnerTaskCapacityCause::Holdings(
            w::OwnerTaskCapacityCauseHoldings::new(limits, w::OwnerTaskCapacityCauseHoldingsParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::HoldWaiters { limit } => {
            w::OwnerTaskCapacityCause::HoldWaiters(w::OwnerTaskCapacityCauseHoldWaiters::new(
                limits,
                w::OwnerTaskCapacityCauseHoldWaitersParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Proposals { limit } => {
            w::OwnerTaskCapacityCause::Proposals(w::OwnerTaskCapacityCauseProposals::new(
                limits,
                w::OwnerTaskCapacityCauseProposalsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::References { limit } => {
            w::OwnerTaskCapacityCause::References(w::OwnerTaskCapacityCauseReferences::new(
                limits,
                w::OwnerTaskCapacityCauseReferencesParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Subscriptions { limit } => {
            w::OwnerTaskCapacityCause::Subscriptions(w::OwnerTaskCapacityCauseSubscriptions::new(
                limits,
                w::OwnerTaskCapacityCauseSubscriptionsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Funders { limit } => w::OwnerTaskCapacityCause::Funders(
            w::OwnerTaskCapacityCauseFunders::new(limits, w::OwnerTaskCapacityCauseFundersParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::Writers { limit } => w::OwnerTaskCapacityCause::Writers(
            w::OwnerTaskCapacityCauseWriters::new(limits, w::OwnerTaskCapacityCauseWritersParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::PlacedHolds { limit } => {
            w::OwnerTaskCapacityCause::PlacedHolds(w::OwnerTaskCapacityCausePlacedHolds::new(
                limits,
                w::OwnerTaskCapacityCausePlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::ProjectPlacedHolds { limit } => {
            w::OwnerTaskCapacityCause::ProjectPlacedHolds(w::OwnerTaskCapacityCauseProjectPlacedHolds::new(
                limits,
                w::OwnerTaskCapacityCauseProjectPlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::PartyPlacedHolds { limit } => {
            w::OwnerTaskCapacityCause::PartyPlacedHolds(w::OwnerTaskCapacityCausePartyPlacedHolds::new(
                limits,
                w::OwnerTaskCapacityCausePartyPlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Exemptions { limit } => {
            w::OwnerTaskCapacityCause::Exemptions(w::OwnerTaskCapacityCauseExemptions::new(
                limits,
                w::OwnerTaskCapacityCauseExemptionsParts { limit: *(limit) },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_program_host(
    value: &c::OwnerProgramHost,
    limits: &w::Limits,
) -> Result<w::OwnerProgramHost, w::Problem> {
    w::OwnerProgramHost::new(
        limits,
        w::OwnerProgramHostParts {
            name: bytes::copy_of(&value.name),
            channel: value.channel,
            connected: value.connected,
            awaiting_logins: value.awaiting_logins,
            programs: pieces(
                value.programs.iter().map(|item| wire_owner_program_login(item, limits)),
                w::OwnerProgramLogin::measure,
                w::OwnerProgramLogin::encode,
            )?,
            cut: value.cut,
        },
    )
}
