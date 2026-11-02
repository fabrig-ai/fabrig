//! Wire projections of owner deployment facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, pieces, w};
use super::{
    wire_account_holder, wire_account_source, wire_account_use, wire_admission_limit, wire_document_text, wire_funder,
    wire_payer_readiness, wire_proposer, wire_provider_offer, wire_requester, wire_seen_through, wire_task_link,
};
pub(in crate::codec) fn wire_task_source_place(
    value: &c::AccountPlace,
    _limits: &w::Limits,
) -> Result<w::TaskSourcePlace, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountPlace::Directory => w::TaskSourcePlace::Directory,
        c::AccountPlace::File => w::TaskSourcePlace::File,
    })
}

pub(in crate::codec) fn wire_task_source_unread(
    value: &c::AccountUnread,
    _limits: &w::Limits,
) -> Result<w::TaskSourceUnread, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountUnread::Unreadable => w::TaskSourceUnread::Unreadable,
        c::AccountUnread::Refused => w::TaskSourceUnread::Refused,
        c::AccountUnread::Failed => w::TaskSourceUnread::Failed,
        c::AccountUnread::Stalled => w::TaskSourceUnread::Stalled,
        c::AccountUnread::TooLarge => w::TaskSourceUnread::TooLarge,
    })
}

pub(in crate::codec) fn wire_host_kind(value: &c::HostKind, _limits: &w::Limits) -> Result<w::HostKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HostKind::Worker => w::HostKind::Worker,
        c::HostKind::Engine => w::HostKind::Engine,
    })
}

pub(in crate::codec) fn wire_hello_refusal(
    value: &c::HelloRefusal,
    limits: &w::Limits,
) -> Result<w::HelloRefusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HelloRefusal::Agent { why } => w::HelloRefusal::Agent(w::HelloRefusalAgent::new(
            limits,
            w::HelloRefusalAgentParts { why: wire_agent_shortfall(why, limits)? },
        )?),
        c::HelloRefusal::JobKinds => w::HelloRefusal::JobKinds,
        c::HelloRefusal::ReservedChannel => w::HelloRefusal::ReservedChannel,
        c::HelloRefusal::Position { worker } => w::HelloRefusal::Position(w::HelloRefusalPosition::new(
            limits,
            w::HelloRefusalPositionParts { worker: *(worker) },
        )?),
        c::HelloRefusal::Workers { limit } => w::HelloRefusal::Workers(w::HelloRefusalWorkers::new(
            limits,
            w::HelloRefusalWorkersParts { limit: *(limit) },
        )?),
        c::HelloRefusal::StopBound { grace } => w::HelloRefusal::StopBound(w::HelloRefusalStopBound::new(
            limits,
            w::HelloRefusalStopBoundParts { grace: *(grace) },
        )?),
    })
}

pub(in crate::codec) fn wire_agent_shortfall(
    value: &c::AgentShortfall,
    limits: &w::Limits,
) -> Result<w::AgentShortfall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AgentShortfall::Model { charter, model } => w::AgentShortfall::Model(w::AgentShortfallModel::new(
            limits,
            w::AgentShortfallModelParts { charter: *(charter), model: *(model) },
        )?),
        c::AgentShortfall::Window { charter, model } => w::AgentShortfall::Window(w::AgentShortfallWindow::new(
            limits,
            w::AgentShortfallWindowParts { charter: *(charter), model: *(model) },
        )?),
        c::AgentShortfall::Output { charter, model } => w::AgentShortfall::Output(w::AgentShortfallOutput::new(
            limits,
            w::AgentShortfallOutputParts { charter: *(charter), model: *(model) },
        )?),
        c::AgentShortfall::Turns => w::AgentShortfall::Turns,
        c::AgentShortfall::Spend => w::AgentShortfall::Spend,
        c::AgentShortfall::Time => w::AgentShortfall::Time,
        c::AgentShortfall::Waiting => w::AgentShortfall::Waiting,
        c::AgentShortfall::Checks => w::AgentShortfall::Checks,
        c::AgentShortfall::Message => w::AgentShortfall::Message,
        c::AgentShortfall::Inbox => w::AgentShortfall::Inbox,
        c::AgentShortfall::Unacknowledged => w::AgentShortfall::Unacknowledged,
    })
}

pub(in crate::codec) fn wire_deployment(
    value: &c::Deployment,
    limits: &w::Limits,
) -> Result<w::Deployment, w::Problem> {
    w::Deployment::new(
        limits,
        w::DeploymentParts {
            skein_bools: [value.cut, value.connectors_cut, value.providers_cut, value.readiness_cut],
            store: wire_store_fill(&value.store, limits)?,
            hosts: pieces(value.hosts.iter().map(|item| wire_host(item, limits)), w::Host::measure, w::Host::encode)?,
            attempts: pieces(
                value.attempts.iter().map(|item| wire_host_run(item, limits)),
                w::HostRun::measure,
                w::HostRun::encode,
            )?,
            accounts: pieces(
                value.accounts.iter().map(|item| wire_account(item, limits)),
                w::Account::measure,
                w::Account::encode,
            )?,
            tasks: wire_capacity(&value.tasks, limits)?,
            workers: wire_capacity(&value.workers, limits)?,
            attempts_capacity: wire_capacity(&value.attempts_capacity, limits)?,
            waiting_attempts: value.waiting_attempts,
            proposals: wire_capacity(&value.proposals, limits)?,
            projects: list(value.projects.iter().map(|item| wire_project_capacity(item, limits)))?,
            grace_bound: value.grace_bound,
            hello_refusals: list(value.hello_refusals.iter().map(|item| wire_hello_refusal_line(item, limits)))?,
            hosts_forgotten: value.hosts_forgotten,
            hello_refusals_forgotten: value.hello_refusals_forgotten,
            person_room_refusals: list(
                value.person_room_refusals.iter().map(|item| wire_person_capacity_refusal(item, limits)),
            )?,
            person_room_refusals_forgotten: value.person_room_refusals_forgotten,
            person_answer_retention: value.person_answer_retention,
            task_room_refusals: list(
                value.task_room_refusals.iter().map(|item| wire_task_capacity_refusal(item, limits)),
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

pub(in crate::codec) fn wire_host(value: &c::Host, limits: &w::Limits) -> Result<w::Host, w::Problem> {
    w::Host::new(
        limits,
        w::HostParts {
            skein_bools: [value.awaiting_logins, value.draining, value.connected, value.cut],
            name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
            rule: wire_host_rule(&value.rule, limits)?,
            programs: pieces(
                value.programs.iter().map(|item| wire_program_login(item, limits)),
                w::ProgramLogin::measure,
                w::ProgramLogin::encode,
            )?,
            channel: value.channel,
            kind: wire_host_kind(&value.kind, limits)?,
            slots: value.slots,
            used: value.used,
            free: value.free,
            grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            grace_remaining: value.grace_remaining.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            runs: pieces(
                value.runs.iter().map(|item| wire_host_run(item, limits)),
                w::HostRun::measure,
                w::HostRun::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_host_rule(value: &c::HostRule, limits: &w::Limits) -> Result<w::HostRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HostRule::Open => w::HostRule::Open,
        c::HostRule::Person(value) => {
            w::HostRule::Person(w::HostPerson::new(limits, w::HostPersonParts { party: *(value) })?)
        }
        c::HostRule::Several => w::HostRule::Several,
        c::HostRule::Unknown => w::HostRule::Unknown,
    })
}

pub(in crate::codec) fn wire_program_login(
    value: &c::ProgramLogin,
    limits: &w::Limits,
) -> Result<w::ProgramLogin, w::Problem> {
    w::ProgramLogin::new(
        limits,
        w::ProgramLoginParts {
            name: bytes::copy_of(&value.name),
            state: value.state.as_ref().map(|item| wire_program_state(item, limits)).transpose()?,
            sign_out: value.sign_out.as_ref().map(|item| wire_program_sign_out(item, limits)).transpose()?,
            cleanup: value.cleanup,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_program_state(
    value: &c::ProgramState,
    limits: &w::Limits,
) -> Result<w::ProgramState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProgramState::Usable => w::ProgramState::Usable,
        c::ProgramState::NotSignedIn => w::ProgramState::NotSignedIn,
        c::ProgramState::SignedInOtherwise { method } => w::ProgramState::SignedInOtherwise(w::ProgramOtherwise::new(
            limits,
            w::ProgramOtherwiseParts { method: bytes::copy_of(method) },
        )?),
        c::ProgramState::Failed { reason, detail } => w::ProgramState::Failed(w::ProgramFailed::new(
            limits,
            w::ProgramFailedParts { reason: wire_program_failure(reason, limits)?, detail: bytes::copy_of(detail) },
        )?),
    })
}

pub(in crate::codec) fn wire_program_failure(
    value: &c::ProgramFailure,
    _limits: &w::Limits,
) -> Result<w::ProgramFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProgramFailure::Missing => w::ProgramFailure::Missing,
        c::ProgramFailure::Deadline => w::ProgramFailure::Deadline,
        c::ProgramFailure::Unreadable => w::ProgramFailure::Unreadable,
        c::ProgramFailure::Exited => w::ProgramFailure::Exited,
        c::ProgramFailure::Unconfigured => w::ProgramFailure::Unconfigured,
        c::ProgramFailure::Unknown => w::ProgramFailure::Unknown,
        c::ProgramFailure::Closed => w::ProgramFailure::Closed,
    })
}

pub(in crate::codec) fn wire_program_sign_out(
    value: &c::ProgramSignOut,
    limits: &w::Limits,
) -> Result<w::ProgramSignOut, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProgramSignOut::SignedOut => w::ProgramSignOut::SignedOut,
        c::ProgramSignOut::Failed { reason, detail } => w::ProgramSignOut::Failed(w::ProgramFailed::new(
            limits,
            w::ProgramFailedParts { reason: wire_program_failure(reason, limits)?, detail: bytes::copy_of(detail) },
        )?),
    })
}

pub(in crate::codec) fn wire_host_run(value: &c::HostRun, limits: &w::Limits) -> Result<w::HostRun, w::Problem> {
    w::HostRun::new(
        limits,
        w::HostRunParts {
            task: wire_task_link(&value.task, limits)?,
            attempt: value.attempt,
            kept: value.kept,
            listed: value.listed,
            state: wire_attempt_state(&value.state, limits)?,
            location: wire_run_location(&value.location, limits)?,
            grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_attempt_state(
    value: &c::AttemptState,
    _limits: &w::Limits,
) -> Result<w::AttemptState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AttemptState::Waiting => w::AttemptState::Waiting,
        c::AttemptState::Adopted => w::AttemptState::Adopted,
        c::AttemptState::Claimed => w::AttemptState::Claimed,
        c::AttemptState::Cancelled => w::AttemptState::Cancelled,
        c::AttemptState::Handed => w::AttemptState::Handed,
        c::AttemptState::Acknowledged => w::AttemptState::Acknowledged,
        c::AttemptState::Stray => w::AttemptState::Stray,
        c::AttemptState::Kept => w::AttemptState::Kept,
        c::AttemptState::Fenced => w::AttemptState::Fenced,
        c::AttemptState::Closed => w::AttemptState::Closed,
    })
}

pub(in crate::codec) fn wire_run_location(
    value: &c::RunLocation,
    limits: &w::Limits,
) -> Result<w::RunLocation, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunLocation::None => w::RunLocation::None,
        c::RunLocation::Connected { channel, kind } => w::RunLocation::Connected(w::RunLocationConnected::new(
            limits,
            w::RunLocationConnectedParts { channel: *(channel), kind: wire_host_kind(kind, limits)? },
        )?),
        c::RunLocation::Adrift { channel, until, remaining } => w::RunLocation::Adrift(w::RunLocationAdrift::new(
            limits,
            w::RunLocationAdriftParts {
                channel: (channel).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                remaining: (remaining).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_account(value: &c::Account, limits: &w::Limits) -> Result<w::Account, w::Problem> {
    w::Account::new(
        limits,
        w::AccountParts {
            skein_bools: [value.plan, value.usable, value.expiring, value.attention, value.cut],
            number: value.number,
            place: value.place,
            uses: value
                .uses
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_account_use(item, limits))))
                .transpose()?,
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            display: wire_document_text(&value.display, limits)?,
            holder: wire_account_holder(&value.holder, limits)?,
            sources: pieces(
                value.sources.iter().map(|item| wire_account_source(item, limits)),
                w::AccountSource::measure,
                w::AccountSource::encode,
            )?,
            generation: value.generation.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            state: wire_account_state(&value.state, limits)?,
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

pub(in crate::codec) fn wire_account_state(
    value: &c::AccountState,
    limits: &w::Limits,
) -> Result<w::AccountState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountState::Asking => w::AccountState::Asking,
        c::AccountState::Granted => w::AccountState::Granted,
        c::AccountState::Later { until } => w::AccountState::Later(w::AccountStateLater::new(
            limits,
            w::AccountStateLaterParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::AccountState::SignInAgain => w::AccountState::SignInAgain,
        c::AccountState::Misconfigured => w::AccountState::Misconfigured,
        c::AccountState::Unloaded { at, why } => w::AccountState::Unloaded(w::AccountStateUnloaded::new(
            limits,
            w::AccountStateUnloadedParts {
                at: wire_account_place(at, limits)?,
                why: wire_account_unread(why, limits)?,
            },
        )?),
        c::AccountState::Cancelled => w::AccountState::Cancelled,
        c::AccountState::Closing => w::AccountState::Closing,
        c::AccountState::Moving => w::AccountState::Moving,
    })
}

pub(in crate::codec) fn wire_account_place(
    value: &c::AccountPlace,
    _limits: &w::Limits,
) -> Result<w::AccountPlace, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountPlace::Directory => w::AccountPlace::Directory,
        c::AccountPlace::File => w::AccountPlace::File,
    })
}

pub(in crate::codec) fn wire_account_unread(
    value: &c::AccountUnread,
    _limits: &w::Limits,
) -> Result<w::AccountUnread, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountUnread::Unreadable => w::AccountUnread::Unreadable,
        c::AccountUnread::Refused => w::AccountUnread::Refused,
        c::AccountUnread::Failed => w::AccountUnread::Failed,
        c::AccountUnread::Stalled => w::AccountUnread::Stalled,
        c::AccountUnread::TooLarge => w::AccountUnread::TooLarge,
    })
}

pub(in crate::codec) fn wire_capacity(value: &c::Capacity, limits: &w::Limits) -> Result<w::Capacity, w::Problem> {
    w::Capacity::new(limits, w::CapacityParts { used: value.used, limit: value.limit })
}

pub(in crate::codec) fn wire_project_capacity(
    value: &c::ProjectCapacity,
    limits: &w::Limits,
) -> Result<w::ProjectCapacity, w::Problem> {
    w::ProjectCapacity::new(
        limits,
        w::ProjectCapacityParts {
            project: value.project,
            tasks: wire_capacity(&value.tasks, limits)?,
            proposals: value.proposals,
        },
    )
}

pub(in crate::codec) fn wire_hello_refusal_line(
    value: &c::HelloRefusalLine,
    limits: &w::Limits,
) -> Result<w::HelloRefusalLine, w::Problem> {
    w::HelloRefusalLine::new(
        limits,
        w::HelloRefusalLineParts {
            channel: value.channel,
            at: value.at.as_nanos(),
            slots: value.slots,
            stop_bound: value.stop_bound,
            why: wire_hello_refusal(&value.why, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_person_capacity_refusal(
    value: &c::PersonCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::PersonCapacityRefusal, w::Problem> {
    w::PersonCapacityRefusal::new(
        limits,
        w::PersonCapacityRefusalParts {
            key: wire_request_key(&value.key, limits)?,
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            action: wire_person_capacity_action(&value.action, limits)?,
            at: value.at.as_nanos(),
            cause: wire_person_capacity_cause(&value.cause, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_request_key(
    value: &c::RequestKey,
    limits: &w::Limits,
) -> Result<w::RequestKey, w::Problem> {
    w::RequestKey::new(limits, w::RequestKeyParts { person: value.person, key: value.key })
}

pub(in crate::codec) fn wire_person_capacity_action(
    value: &c::PersonCapacityAction,
    limits: &w::Limits,
) -> Result<w::PersonCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PersonCapacityAction::Lift { hold } => w::PersonCapacityAction::Lift(w::PersonCapacityActionLift::new(
            limits,
            w::PersonCapacityActionLiftParts { hold: *(hold) },
        )?),
        c::PersonCapacityAction::Create => w::PersonCapacityAction::Create,
        c::PersonCapacityAction::Compound => w::PersonCapacityAction::Compound,
        c::PersonCapacityAction::Application { connector } => {
            w::PersonCapacityAction::Application(w::PersonCapacityActionApplication::new(
                limits,
                w::PersonCapacityActionApplicationParts { connector: *(connector) },
            )?)
        }
        c::PersonCapacityAction::Note { name } => w::PersonCapacityAction::Note(w::PersonCapacityActionNote::new(
            limits,
            w::PersonCapacityActionNoteParts { name: *(name) },
        )?),
        c::PersonCapacityAction::Service => w::PersonCapacityAction::Service,
        c::PersonCapacityAction::Adoption => w::PersonCapacityAction::Adoption,
        c::PersonCapacityAction::Roles => w::PersonCapacityAction::Roles,
        c::PersonCapacityAction::Policy => w::PersonCapacityAction::Policy,
        c::PersonCapacityAction::Pool { person } => w::PersonCapacityAction::Pool(w::PersonCapacityActionPool::new(
            limits,
            w::PersonCapacityActionPoolParts { person: *(person) },
        )?),
        c::PersonCapacityAction::Escalation { task } => w::PersonCapacityAction::Escalation(
            w::PersonCapacityActionEscalation::new(limits, w::PersonCapacityActionEscalationParts { task: *(task) })?,
        ),
        c::PersonCapacityAction::Proposal { proposer, number } => {
            w::PersonCapacityAction::Proposal(w::PersonCapacityActionProposal::new(
                limits,
                w::PersonCapacityActionProposalParts { proposer: wire_proposer(proposer, limits)?, number: *(number) },
            )?)
        }
        c::PersonCapacityAction::Words { task } => w::PersonCapacityAction::Words(w::PersonCapacityActionWords::new(
            limits,
            w::PersonCapacityActionWordsParts { task: *(task) },
        )?),
        c::PersonCapacityAction::Question { task, question } => {
            w::PersonCapacityAction::Question(w::PersonCapacityActionQuestion::new(
                limits,
                w::PersonCapacityActionQuestionParts { task: *(task), question: *(question) },
            )?)
        }
        c::PersonCapacityAction::Amend { task } => w::PersonCapacityAction::Amend(w::PersonCapacityActionAmend::new(
            limits,
            w::PersonCapacityActionAmendParts { task: *(task) },
        )?),
        c::PersonCapacityAction::Move { task } => w::PersonCapacityAction::MoveTask(w::PersonCapacityActionMove::new(
            limits,
            w::PersonCapacityActionMoveParts { task: *(task) },
        )?),
        c::PersonCapacityAction::TakePerson { task } => w::PersonCapacityAction::TakePerson(
            w::PersonCapacityActionTakePerson::new(limits, w::PersonCapacityActionTakePersonParts { task: *(task) })?,
        ),
        c::PersonCapacityAction::HandBackPerson { task } => {
            w::PersonCapacityAction::HandBackPerson(w::PersonCapacityActionHandBackPerson::new(
                limits,
                w::PersonCapacityActionHandBackPersonParts { task: *(task) },
            )?)
        }
        c::PersonCapacityAction::AnswerPerson { task } => {
            w::PersonCapacityAction::AnswerPerson(w::PersonCapacityActionAnswerPerson::new(
                limits,
                w::PersonCapacityActionAnswerPersonParts { task: *(task) },
            )?)
        }
        c::PersonCapacityAction::Stop { task } => w::PersonCapacityAction::Stop(w::PersonCapacityActionStop::new(
            limits,
            w::PersonCapacityActionStopParts { task: *(task) },
        )?),
        c::PersonCapacityAction::Cancel { task } => w::PersonCapacityAction::Cancel(
            w::PersonCapacityActionCancel::new(limits, w::PersonCapacityActionCancelParts { task: *(task) })?,
        ),
        c::PersonCapacityAction::Release { task } => w::PersonCapacityAction::Release(
            w::PersonCapacityActionRelease::new(limits, w::PersonCapacityActionReleaseParts { task: *(task) })?,
        ),
        c::PersonCapacityAction::MarkSeen { seen } => {
            w::PersonCapacityAction::MarkSeen(w::PersonCapacityActionMarkSeen::new(
                limits,
                w::PersonCapacityActionMarkSeenParts { seen: wire_seen_through(seen, limits)? },
            )?)
        }
        c::PersonCapacityAction::AccountSignIn => w::PersonCapacityAction::AccountSignIn,
        c::PersonCapacityAction::AccountSignOut => w::PersonCapacityAction::AccountSignOut,
        c::PersonCapacityAction::AccountChange => w::PersonCapacityAction::AccountChange,
    })
}

pub(in crate::codec) fn wire_person_capacity_cause(
    value: &c::PersonCapacityCause,
    limits: &w::Limits,
) -> Result<w::PersonCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PersonCapacityCause::Requests { limit } => w::PersonCapacityCause::Requests(
            w::PersonCapacityCauseRequests::new(limits, w::PersonCapacityCauseRequestsParts { limit: *(limit) })?,
        ),
        c::PersonCapacityCause::Pending { limit } => w::PersonCapacityCause::Pending(
            w::PersonCapacityCausePending::new(limits, w::PersonCapacityCausePendingParts { limit: *(limit) })?,
        ),
        c::PersonCapacityCause::Waiters { limit } => w::PersonCapacityCause::Waiters(
            w::PersonCapacityCauseWaiters::new(limits, w::PersonCapacityCauseWaitersParts { limit: *(limit) })?,
        ),
        c::PersonCapacityCause::Limit { which } => w::PersonCapacityCause::Limit(w::PersonCapacityCauseLimit::new(
            limits,
            w::PersonCapacityCauseLimitParts { which: wire_admission_limit(which, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_task_capacity_refusal(
    value: &c::TaskCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::TaskCapacityRefusal, w::Problem> {
    w::TaskCapacityRefusal::new(
        limits,
        w::TaskCapacityRefusalParts {
            request: wire_task_capacity_request(&value.request, limits)?,
            at: value.at.as_nanos(),
        },
    )
}

pub(in crate::codec) fn wire_task_capacity_request(
    value: &c::TaskCapacityRequest,
    limits: &w::Limits,
) -> Result<w::TaskCapacityRequest, w::Problem> {
    w::TaskCapacityRequest::new(
        limits,
        w::TaskCapacityRequestParts {
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            by: wire_requester(&value.by, limits)?,
            action: wire_task_capacity_action(&value.action, limits)?,
            cause: wire_task_capacity_cause(&value.cause, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_task_capacity_action(
    value: &c::TaskCapacityAction,
    limits: &w::Limits,
) -> Result<w::TaskCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskCapacityAction::Batch { first, members } => {
            w::TaskCapacityAction::Batch(w::TaskCapacityActionBatch::new(
                limits,
                w::TaskCapacityActionBatchParts {
                    first: (first).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    members: *(members),
                },
            )?)
        }
        c::TaskCapacityAction::Proposal { number } => w::TaskCapacityAction::Proposal(
            w::TaskCapacityActionProposal::new(limits, w::TaskCapacityActionProposalParts { number: *(number) })?,
        ),
        c::TaskCapacityAction::PersonProposal { number } => {
            w::TaskCapacityAction::PersonProposal(w::TaskCapacityActionPersonProposal::new(
                limits,
                w::TaskCapacityActionPersonProposalParts { number: *(number) },
            )?)
        }
        c::TaskCapacityAction::Message { task, number } => {
            w::TaskCapacityAction::Message(w::TaskCapacityActionMessage::new(
                limits,
                w::TaskCapacityActionMessageParts { task: *(task), number: *(number) },
            )?)
        }
        c::TaskCapacityAction::Source { funder } => w::TaskCapacityAction::Source(w::TaskCapacityActionSource::new(
            limits,
            w::TaskCapacityActionSourceParts { funder: wire_funder(funder, limits)? },
        )?),
        c::TaskCapacityAction::Introduce { left, right } => {
            w::TaskCapacityAction::Introduce(w::TaskCapacityActionIntroduce::new(
                limits,
                w::TaskCapacityActionIntroduceParts { left: *(left), right: *(right) },
            )?)
        }
        c::TaskCapacityAction::Subscription { number } => {
            w::TaskCapacityAction::Subscription(w::TaskCapacityActionSubscription::new(
                limits,
                w::TaskCapacityActionSubscriptionParts { number: *(number) },
            )?)
        }
        c::TaskCapacityAction::Effect { entry } => w::TaskCapacityAction::Effect(w::TaskCapacityActionEffect::new(
            limits,
            w::TaskCapacityActionEffectParts { entry: *(entry) },
        )?),
        c::TaskCapacityAction::Claim { attempt } => w::TaskCapacityAction::Claim(w::TaskCapacityActionClaim::new(
            limits,
            w::TaskCapacityActionClaimParts { attempt: *(attempt) },
        )?),
        c::TaskCapacityAction::Release => w::TaskCapacityAction::Release,
        c::TaskCapacityAction::Move => w::TaskCapacityAction::MoveTask,
        c::TaskCapacityAction::Place { hold } => w::TaskCapacityAction::Place(w::TaskCapacityActionPlace::new(
            limits,
            w::TaskCapacityActionPlaceParts { hold: *(hold) },
        )?),
    })
}

pub(in crate::codec) fn wire_task_capacity_cause(
    value: &c::TaskCapacityCause,
    limits: &w::Limits,
) -> Result<w::TaskCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskCapacityCause::Tasks { limit } => w::TaskCapacityCause::Tasks(w::TaskCapacityCauseTasks::new(
            limits,
            w::TaskCapacityCauseTasksParts { limit: *(limit) },
        )?),
        c::TaskCapacityCause::CreationFlights { limit } => {
            w::TaskCapacityCause::CreationFlights(w::TaskCapacityCauseCreationFlights::new(
                limits,
                w::TaskCapacityCauseCreationFlightsParts { limit: *(limit) },
            )?)
        }
        c::TaskCapacityCause::ResourceReports { limit } => {
            w::TaskCapacityCause::ResourceReports(w::TaskCapacityCauseResourceReports::new(
                limits,
                w::TaskCapacityCauseResourceReportsParts { limit: *(limit) },
            )?)
        }
        c::TaskCapacityCause::ProjectTasks { limit } => w::TaskCapacityCause::ProjectTasks(
            w::TaskCapacityCauseProjectTasks::new(limits, w::TaskCapacityCauseProjectTasksParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::Tree { limit } => w::TaskCapacityCause::Tree(w::TaskCapacityCauseTree::new(
            limits,
            w::TaskCapacityCauseTreeParts { limit: *(limit) },
        )?),
        c::TaskCapacityCause::Delegates { limit } => w::TaskCapacityCause::Delegates(
            w::TaskCapacityCauseDelegates::new(limits, w::TaskCapacityCauseDelegatesParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::Inbox { task, messages, bytes } => {
            w::TaskCapacityCause::Inbox(w::TaskCapacityCauseInbox::new(
                limits,
                w::TaskCapacityCauseInboxParts { task: *(task), messages: *(messages), bytes: *(bytes) },
            )?)
        }
        c::TaskCapacityCause::Holdings { limit } => w::TaskCapacityCause::Holdings(w::TaskCapacityCauseHoldings::new(
            limits,
            w::TaskCapacityCauseHoldingsParts { limit: *(limit) },
        )?),
        c::TaskCapacityCause::HoldWaiters { limit } => w::TaskCapacityCause::HoldWaiters(
            w::TaskCapacityCauseHoldWaiters::new(limits, w::TaskCapacityCauseHoldWaitersParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::Proposals { limit } => w::TaskCapacityCause::Proposals(
            w::TaskCapacityCauseProposals::new(limits, w::TaskCapacityCauseProposalsParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::References { limit } => w::TaskCapacityCause::References(
            w::TaskCapacityCauseReferences::new(limits, w::TaskCapacityCauseReferencesParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::Subscriptions { limit } => w::TaskCapacityCause::Subscriptions(
            w::TaskCapacityCauseSubscriptions::new(limits, w::TaskCapacityCauseSubscriptionsParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::Funders { limit } => w::TaskCapacityCause::Funders(w::TaskCapacityCauseFunders::new(
            limits,
            w::TaskCapacityCauseFundersParts { limit: *(limit) },
        )?),
        c::TaskCapacityCause::Writers { limit } => w::TaskCapacityCause::Writers(w::TaskCapacityCauseWriters::new(
            limits,
            w::TaskCapacityCauseWritersParts { limit: *(limit) },
        )?),
        c::TaskCapacityCause::PlacedHolds { limit } => w::TaskCapacityCause::PlacedHolds(
            w::TaskCapacityCausePlacedHolds::new(limits, w::TaskCapacityCausePlacedHoldsParts { limit: *(limit) })?,
        ),
        c::TaskCapacityCause::ProjectPlacedHolds { limit } => {
            w::TaskCapacityCause::ProjectPlacedHolds(w::TaskCapacityCauseProjectPlacedHolds::new(
                limits,
                w::TaskCapacityCauseProjectPlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::TaskCapacityCause::PartyPlacedHolds { limit } => {
            w::TaskCapacityCause::PartyPlacedHolds(w::TaskCapacityCausePartyPlacedHolds::new(
                limits,
                w::TaskCapacityCausePartyPlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::TaskCapacityCause::Exemptions { limit } => w::TaskCapacityCause::Exemptions(
            w::TaskCapacityCauseExemptions::new(limits, w::TaskCapacityCauseExemptionsParts { limit: *(limit) })?,
        ),
    })
}

pub(in crate::codec) fn wire_program_host(
    value: &c::ProgramHost,
    limits: &w::Limits,
) -> Result<w::ProgramHost, w::Problem> {
    w::ProgramHost::new(
        limits,
        w::ProgramHostParts {
            name: bytes::copy_of(&value.name),
            channel: value.channel,
            connected: value.connected,
            awaiting_logins: value.awaiting_logins,
            programs: pieces(
                value.programs.iter().map(|item| wire_program_login(item, limits)),
                w::ProgramLogin::measure,
                w::ProgramLogin::encode,
            )?,
            cut: value.cut,
        },
    )
}
