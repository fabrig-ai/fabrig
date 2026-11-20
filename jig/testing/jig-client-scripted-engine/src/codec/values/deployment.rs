//! Wire projections of owner deployment facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, pieces, w};
use super::wire_account_list;
use super::{
    wire_account_fact, wire_account_holder, wire_account_source, wire_account_use, wire_admission_limit,
    wire_document_text, wire_funder, wire_payer_readiness, wire_proposer, wire_provider_offer, wire_requester,
    wire_seen_through, wire_task_link,
};
use skein_lib::List;
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
        c::HelloRefusal::Agent { why } => {
            w::HelloRefusal::Agent(w::HelloRefusalAgent { why: wire_agent_shortfall(why, limits)? })
        }
        c::HelloRefusal::JobKinds => w::HelloRefusal::JobKinds,
        c::HelloRefusal::ReservedChannel => w::HelloRefusal::ReservedChannel,
        c::HelloRefusal::Position { worker } => {
            w::HelloRefusal::Position(w::HelloRefusalPosition { worker: *(worker) })
        }
        c::HelloRefusal::Workers { limit } => w::HelloRefusal::Workers(w::HelloRefusalWorkers { limit: *(limit) }),
        c::HelloRefusal::StopBound { grace } => {
            w::HelloRefusal::StopBound(w::HelloRefusalStopBound { grace: *(grace) })
        }
    })
}

pub(in crate::codec) fn wire_agent_shortfall(
    value: &c::AgentShortfall,
    _limits: &w::Limits,
) -> Result<w::AgentShortfall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AgentShortfall::Model { charter, model } => {
            w::AgentShortfall::Model(w::AgentShortfallModel { charter: *(charter), model: *(model) })
        }
        c::AgentShortfall::Window { charter, model } => {
            w::AgentShortfall::Window(w::AgentShortfallWindow { charter: *(charter), model: *(model) })
        }
        c::AgentShortfall::Output { charter, model } => {
            w::AgentShortfall::Output(w::AgentShortfallOutput { charter: *(charter), model: *(model) })
        }
        c::AgentShortfall::Turns { charter } => w::AgentShortfall::Turns(w::AgentShortfallTurns { charter: *charter }),
        c::AgentShortfall::Spend => w::AgentShortfall::Spend,
        c::AgentShortfall::Time { charter } => w::AgentShortfall::Time(w::AgentShortfallTime { charter: *charter }),
        c::AgentShortfall::Waiting { charter } => {
            w::AgentShortfall::Waiting(w::AgentShortfallWaiting { charter: *charter })
        }
        c::AgentShortfall::Checks { charter } => {
            w::AgentShortfall::Checks(w::AgentShortfallChecks { charter: *charter })
        }
        c::AgentShortfall::Message => w::AgentShortfall::Message,
        c::AgentShortfall::Inbox => w::AgentShortfall::Inbox,
        c::AgentShortfall::Unacknowledged => w::AgentShortfall::Unacknowledged,
    })
}

pub(in crate::codec) fn wire_deployment(
    value: &c::Deployment,
    limits: &w::Limits,
) -> Result<w::Deployment, w::Problem> {
    let mut worker_settings = List::with_capacity(value.worker_settings.len());
    for row in &value.worker_settings {
        worker_settings
            .push(w::WorkerState {
                number: row.number,
                name: row.name.clone(),
                places: row.places,
                digest: row.digest,
                withdrawn: row.withdrawn,
                linked: row.linked,
                since: row.since.map(skein_lib::Wall::as_nanos),
            })
            .expect("worker row count");
    }
    Ok(w::Deployment {
        worker_settings,
        cut: value.cut,
        connectors_cut: value.connectors_cut,
        providers_cut: value.providers_cut,
        readiness_cut: value.readiness_cut,
        store: wire_store_fill(&value.store, limits)?,
        census: value.census.as_deref().map(|value| wire_store_census(value, limits)).transpose()?,
        hosts: pieces(value.hosts.iter().map(|item| wire_host(item, limits)), limits)?,
        attempts: pieces(value.attempts.iter().map(|item| wire_host_run(item, limits)), limits)?,
        accounts: pieces(value.accounts.iter().map(|item| wire_account(item, limits)), limits)?,
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
        task_room_refusals: list(value.task_room_refusals.iter().map(|item| wire_task_capacity_refusal(item, limits)))?,
        task_room_refusals_forgotten: value.task_room_refusals_forgotten,
        connectors: list(value.connectors.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
        providers: pieces(value.providers.iter().map(|item| wire_provider_offer(item, limits)), limits)?,
        key_bytes: value.key_bytes,
        readiness: pieces(value.readiness.iter().map(|item| wire_payer_readiness(item, limits)), limits)?,
    })
}

fn wire_store_census(value: &c::StoreCensus, _limits: &w::Limits) -> Result<w::StoreCensus, w::Problem> {
    let keyspaces = list(value.keyspaces.iter().map(|row| {
        let heads = list(row.heads.iter().map(|head| Ok(w::CensusHead { head: head.head, rows: head.rows })))?;
        Ok(w::CensusKeyspace {
            prefix: row.prefix.clone(),
            name: row.name.as_bytes().into(),
            rows: row.rows,
            keys: row.keys,
            values: row.values,
            heads,
            puts: row.puts,
            erases: row.erases,
            written: row.written,
        })
    }))?;
    Ok(w::StoreCensus { commit: value.commit, since: value.since, keyspaces })
}

pub(in crate::codec) fn wire_store_fill(value: &c::StoreFill, _limits: &w::Limits) -> Result<w::StoreFill, w::Problem> {
    Ok(w::StoreFill { bytes: value.bytes, mark: value.mark })
}

pub(in crate::codec) fn wire_host(value: &c::Host, limits: &w::Limits) -> Result<w::Host, w::Problem> {
    Ok(w::Host {
        awaiting_logins: value.awaiting_logins,
        draining: value.draining,
        connected: value.connected,
        cut: value.cut,
        name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
        rule: wire_host_rule(&value.rule, limits)?,
        programs: pieces(value.programs.iter().map(|item| wire_program_login(item, limits)), limits)?,
        channel: value.channel,
        kind: wire_host_kind(&value.kind, limits)?,
        slots: value.slots,
        used: value.used,
        free: value.free,
        grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        grace_remaining: value.grace_remaining.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        runs: pieces(value.runs.iter().map(|item| wire_host_run(item, limits)), limits)?,
    })
}

pub(in crate::codec) fn wire_host_rule(value: &c::HostRule, _limits: &w::Limits) -> Result<w::HostRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HostRule::Open => w::HostRule::Open,
        c::HostRule::Person(value) => w::HostRule::Person(w::HostPerson { party: *(value) }),
        c::HostRule::Several => w::HostRule::Several,
        c::HostRule::Unknown => w::HostRule::Unknown,
    })
}

pub(in crate::codec) fn wire_program_login(
    value: &c::ProgramLogin,
    limits: &w::Limits,
) -> Result<w::ProgramLogin, w::Problem> {
    Ok(w::ProgramLogin {
        name: bytes::copy_of(&value.name),
        state: value.state.as_ref().map(|item| wire_program_state(item, limits)).transpose()?,
        sign_out: value.sign_out.as_ref().map(|item| wire_program_sign_out(item, limits)).transpose()?,
        cleanup: value.cleanup,
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_program_state(
    value: &c::ProgramState,
    limits: &w::Limits,
) -> Result<w::ProgramState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProgramState::Usable => w::ProgramState::Usable,
        c::ProgramState::NotSignedIn => w::ProgramState::NotSignedIn,
        c::ProgramState::SignedInOtherwise { method } => {
            w::ProgramState::SignedInOtherwise(w::ProgramOtherwise { method: bytes::copy_of(method) })
        }
        c::ProgramState::Failed { reason, detail } => w::ProgramState::Failed(w::ProgramFailed {
            reason: wire_program_failure(reason, limits)?,
            detail: bytes::copy_of(detail),
        }),
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
        c::ProgramSignOut::Failed { reason, detail } => w::ProgramSignOut::Failed(w::ProgramFailed {
            reason: wire_program_failure(reason, limits)?,
            detail: bytes::copy_of(detail),
        }),
    })
}

pub(in crate::codec) fn wire_host_run(value: &c::HostRun, limits: &w::Limits) -> Result<w::HostRun, w::Problem> {
    Ok(w::HostRun {
        task: wire_task_link(&value.task, limits)?,
        attempt: value.attempt,
        kept: value.kept,
        listed: value.listed,
        state: wire_attempt_state(&value.state, limits)?,
        location: wire_run_location(&value.location, limits)?,
        grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
    })
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
        c::RunLocation::Connected { channel, kind } => w::RunLocation::Connected(w::RunLocationConnected {
            channel: *(channel),
            kind: wire_host_kind(kind, limits)?,
        }),
        c::RunLocation::Adrift { channel, until, remaining } => w::RunLocation::Adrift(w::RunLocationAdrift {
            channel: (channel).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            remaining: (remaining).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        }),
    })
}

pub(in crate::codec) fn wire_account(value: &c::Account, limits: &w::Limits) -> Result<w::Account, w::Problem> {
    Ok(w::Account {
        list: value.list.as_ref().map(|list| wire_account_list(list, limits)).transpose()?,
        facts: list(value.facts.iter().map(|fact| wire_account_fact(fact, limits)))?,
        plan: value.plan,
        usable: value.usable,
        expiring: value.expiring,
        attention: value.attention,
        cut: value.cut,
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
        sources: pieces(value.sources.iter().map(|item| wire_account_source(item, limits)), limits)?,
        generation: value.generation.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        state: wire_account_state(&value.state, limits)?,
        spent_until: value.spent_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        runs: pieces(value.runs.iter().map(|item| facts::chip(item, limits)), limits)?,
        waiting: pieces(value.waiting.iter().map(|item| facts::chip(item, limits)), limits)?,
        provider_name: value.provider_name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
        manage_usage: value.manage_usage.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
    })
}

pub(in crate::codec) fn wire_account_state(
    value: &c::AccountState,
    limits: &w::Limits,
) -> Result<w::AccountState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountState::Asking => w::AccountState::Asking,
        c::AccountState::Granted => w::AccountState::Granted,
        c::AccountState::Later { until } => w::AccountState::Later(w::AccountStateLater {
            until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        }),
        c::AccountState::SignInAgain => w::AccountState::SignInAgain,
        c::AccountState::Misconfigured => w::AccountState::Misconfigured,
        c::AccountState::Unloaded { at, why } => w::AccountState::Unloaded(w::AccountStateUnloaded {
            at: wire_account_place(at, limits)?,
            why: wire_account_unread(why, limits)?,
        }),
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

pub(in crate::codec) fn wire_capacity(value: &c::Capacity, _limits: &w::Limits) -> Result<w::Capacity, w::Problem> {
    Ok(w::Capacity { used: value.used, limit: value.limit })
}

pub(in crate::codec) fn wire_project_capacity(
    value: &c::ProjectCapacity,
    limits: &w::Limits,
) -> Result<w::ProjectCapacity, w::Problem> {
    Ok(w::ProjectCapacity {
        project: value.project,
        tasks: wire_capacity(&value.tasks, limits)?,
        proposals: value.proposals,
    })
}

pub(in crate::codec) fn wire_hello_refusal_line(
    value: &c::HelloRefusalLine,
    limits: &w::Limits,
) -> Result<w::HelloRefusalLine, w::Problem> {
    Ok(w::HelloRefusalLine {
        channel: value.channel,
        at: value.at.as_nanos(),
        slots: value.slots,
        stop_bound: value.stop_bound,
        why: wire_hello_refusal(&value.why, limits)?,
    })
}

pub(in crate::codec) fn wire_person_capacity_refusal(
    value: &c::PersonCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::PersonCapacityRefusal, w::Problem> {
    Ok(w::PersonCapacityRefusal {
        key: wire_request_key(&value.key, limits)?,
        project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        action: wire_person_capacity_action(&value.action, limits)?,
        at: value.at.as_nanos(),
        cause: wire_person_capacity_cause(&value.cause, limits)?,
    })
}

pub(in crate::codec) fn wire_request_key(
    value: &c::RequestKey,
    _limits: &w::Limits,
) -> Result<w::RequestKey, w::Problem> {
    Ok(w::RequestKey { person: value.person, key: value.key })
}

pub(in crate::codec) fn wire_person_capacity_action(
    value: &c::PersonCapacityAction,
    limits: &w::Limits,
) -> Result<w::PersonCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PersonCapacityAction::Lift { hold } => {
            w::PersonCapacityAction::Lift(w::PersonCapacityActionLift { hold: *(hold) })
        }
        c::PersonCapacityAction::Create => w::PersonCapacityAction::Create,
        c::PersonCapacityAction::Compound => w::PersonCapacityAction::Compound,
        c::PersonCapacityAction::Application { connector } => {
            w::PersonCapacityAction::Application(w::PersonCapacityActionApplication { connector: *(connector) })
        }
        c::PersonCapacityAction::Note { name } => {
            w::PersonCapacityAction::Note(w::PersonCapacityActionNote { name: *(name) })
        }
        c::PersonCapacityAction::Service => w::PersonCapacityAction::Service,
        c::PersonCapacityAction::Adoption => w::PersonCapacityAction::Adoption,
        c::PersonCapacityAction::Roles => w::PersonCapacityAction::Roles,
        c::PersonCapacityAction::Policy => w::PersonCapacityAction::Policy,
        c::PersonCapacityAction::Pool { person } => {
            w::PersonCapacityAction::Pool(w::PersonCapacityActionPool { person: *(person) })
        }
        c::PersonCapacityAction::Escalation { task } => {
            w::PersonCapacityAction::Escalation(w::PersonCapacityActionEscalation { task: *(task) })
        }
        c::PersonCapacityAction::Proposal { proposer, number } => {
            w::PersonCapacityAction::Proposal(w::PersonCapacityActionProposal {
                proposer: wire_proposer(proposer, limits)?,
                number: *(number),
            })
        }
        c::PersonCapacityAction::Words { task } => {
            w::PersonCapacityAction::Words(w::PersonCapacityActionWords { task: *(task) })
        }
        c::PersonCapacityAction::Question { task, question } => {
            w::PersonCapacityAction::Question(w::PersonCapacityActionQuestion { task: *(task), question: *(question) })
        }
        c::PersonCapacityAction::Amend { task } => {
            w::PersonCapacityAction::Amend(w::PersonCapacityActionAmend { task: *(task) })
        }
        c::PersonCapacityAction::Move { task } => {
            w::PersonCapacityAction::MoveTask(w::PersonCapacityActionMove { task: *(task) })
        }
        c::PersonCapacityAction::TakePerson { task } => {
            w::PersonCapacityAction::TakePerson(w::PersonCapacityActionTakePerson { task: *(task) })
        }
        c::PersonCapacityAction::HandBackPerson { task } => {
            w::PersonCapacityAction::HandBackPerson(w::PersonCapacityActionHandBackPerson { task: *(task) })
        }
        c::PersonCapacityAction::AnswerPerson { task } => {
            w::PersonCapacityAction::AnswerPerson(w::PersonCapacityActionAnswerPerson { task: *(task) })
        }
        c::PersonCapacityAction::Stop { task } => {
            w::PersonCapacityAction::Stop(w::PersonCapacityActionStop { task: *(task) })
        }
        c::PersonCapacityAction::Conclude { task } => {
            w::PersonCapacityAction::Conclude(w::PersonCapacityActionConclude { task: *task })
        }
        c::PersonCapacityAction::Cancel { task } => {
            w::PersonCapacityAction::Cancel(w::PersonCapacityActionCancel { task: *(task) })
        }
        c::PersonCapacityAction::Release { task } => {
            w::PersonCapacityAction::Release(w::PersonCapacityActionRelease { task: *(task) })
        }
        c::PersonCapacityAction::MarkSeen { seen } => {
            w::PersonCapacityAction::MarkSeen(w::PersonCapacityActionMarkSeen {
                seen: wire_seen_through(seen, limits)?,
            })
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
        c::PersonCapacityCause::Requests { limit } => {
            w::PersonCapacityCause::Requests(w::PersonCapacityCauseRequests { limit: *(limit) })
        }
        c::PersonCapacityCause::Pending { limit } => {
            w::PersonCapacityCause::Pending(w::PersonCapacityCausePending { limit: *(limit) })
        }
        c::PersonCapacityCause::Waiters { limit } => {
            w::PersonCapacityCause::Waiters(w::PersonCapacityCauseWaiters { limit: *(limit) })
        }
        c::PersonCapacityCause::Limit { which } => {
            w::PersonCapacityCause::Limit(w::PersonCapacityCauseLimit { which: wire_admission_limit(which, limits)? })
        }
    })
}

pub(in crate::codec) fn wire_task_capacity_refusal(
    value: &c::TaskCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::TaskCapacityRefusal, w::Problem> {
    Ok(w::TaskCapacityRefusal { request: wire_task_capacity_request(&value.request, limits)?, at: value.at.as_nanos() })
}

pub(in crate::codec) fn wire_task_capacity_request(
    value: &c::TaskCapacityRequest,
    limits: &w::Limits,
) -> Result<w::TaskCapacityRequest, w::Problem> {
    Ok(w::TaskCapacityRequest {
        project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        by: wire_requester(&value.by, limits)?,
        action: wire_task_capacity_action(&value.action, limits)?,
        cause: wire_task_capacity_cause(&value.cause, limits)?,
    })
}

pub(in crate::codec) fn wire_task_capacity_action(
    value: &c::TaskCapacityAction,
    limits: &w::Limits,
) -> Result<w::TaskCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskCapacityAction::Batch { first, members } => w::TaskCapacityAction::Batch(w::TaskCapacityActionBatch {
            first: (first).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            members: *(members),
        }),
        c::TaskCapacityAction::Proposal { number } => {
            w::TaskCapacityAction::Proposal(w::TaskCapacityActionProposal { number: *(number) })
        }
        c::TaskCapacityAction::PersonProposal { number } => {
            w::TaskCapacityAction::PersonProposal(w::TaskCapacityActionPersonProposal { number: *(number) })
        }
        c::TaskCapacityAction::Message { task, number } => {
            w::TaskCapacityAction::Message(w::TaskCapacityActionMessage { task: *(task), number: *(number) })
        }
        c::TaskCapacityAction::Source { funder } => {
            w::TaskCapacityAction::Source(w::TaskCapacityActionSource { funder: wire_funder(funder, limits)? })
        }
        c::TaskCapacityAction::Introduce { left, right } => {
            w::TaskCapacityAction::Introduce(w::TaskCapacityActionIntroduce { left: *(left), right: *(right) })
        }
        c::TaskCapacityAction::Subscription { number } => {
            w::TaskCapacityAction::Subscription(w::TaskCapacityActionSubscription { number: *(number) })
        }
        c::TaskCapacityAction::Effect { entry } => {
            w::TaskCapacityAction::Effect(w::TaskCapacityActionEffect { entry: *(entry) })
        }
        c::TaskCapacityAction::Claim { attempt } => {
            w::TaskCapacityAction::Claim(w::TaskCapacityActionClaim { attempt: *(attempt) })
        }
        c::TaskCapacityAction::Release => w::TaskCapacityAction::Release,
        c::TaskCapacityAction::Move => w::TaskCapacityAction::MoveTask,
        c::TaskCapacityAction::Place { hold } => {
            w::TaskCapacityAction::Place(w::TaskCapacityActionPlace { hold: *(hold) })
        }
    })
}

pub(in crate::codec) fn wire_task_capacity_cause(
    value: &c::TaskCapacityCause,
    _limits: &w::Limits,
) -> Result<w::TaskCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskCapacityCause::Tasks { limit } => {
            w::TaskCapacityCause::Tasks(w::TaskCapacityCauseTasks { limit: *(limit) })
        }
        c::TaskCapacityCause::CreationFlights { limit } => {
            w::TaskCapacityCause::CreationFlights(w::TaskCapacityCauseCreationFlights { limit: *(limit) })
        }
        c::TaskCapacityCause::ResourceReports { limit } => {
            w::TaskCapacityCause::ResourceReports(w::TaskCapacityCauseResourceReports { limit: *(limit) })
        }
        c::TaskCapacityCause::ProjectTasks { limit } => {
            w::TaskCapacityCause::ProjectTasks(w::TaskCapacityCauseProjectTasks { limit: *(limit) })
        }
        c::TaskCapacityCause::Tree { limit } => {
            w::TaskCapacityCause::Tree(w::TaskCapacityCauseTree { limit: *(limit) })
        }
        c::TaskCapacityCause::Delegates { limit } => {
            w::TaskCapacityCause::Delegates(w::TaskCapacityCauseDelegates { limit: *(limit) })
        }
        c::TaskCapacityCause::Inbox { task, messages, bytes } => {
            w::TaskCapacityCause::Inbox(w::TaskCapacityCauseInbox {
                task: *(task),
                messages: *(messages),
                bytes: *(bytes),
            })
        }
        c::TaskCapacityCause::Holdings { limit } => {
            w::TaskCapacityCause::Holdings(w::TaskCapacityCauseHoldings { limit: *(limit) })
        }
        c::TaskCapacityCause::HoldWaiters { limit } => {
            w::TaskCapacityCause::HoldWaiters(w::TaskCapacityCauseHoldWaiters { limit: *(limit) })
        }
        c::TaskCapacityCause::Proposals { limit } => {
            w::TaskCapacityCause::Proposals(w::TaskCapacityCauseProposals { limit: *(limit) })
        }
        c::TaskCapacityCause::References { limit } => {
            w::TaskCapacityCause::References(w::TaskCapacityCauseReferences { limit: *(limit) })
        }
        c::TaskCapacityCause::Subscriptions { limit } => {
            w::TaskCapacityCause::Subscriptions(w::TaskCapacityCauseSubscriptions { limit: *(limit) })
        }
        c::TaskCapacityCause::Funders { limit } => {
            w::TaskCapacityCause::Funders(w::TaskCapacityCauseFunders { limit: *(limit) })
        }
        c::TaskCapacityCause::Writers { limit } => {
            w::TaskCapacityCause::Writers(w::TaskCapacityCauseWriters { limit: *(limit) })
        }
        c::TaskCapacityCause::PlacedHolds { limit } => {
            w::TaskCapacityCause::PlacedHolds(w::TaskCapacityCausePlacedHolds { limit: *(limit) })
        }
        c::TaskCapacityCause::ProjectPlacedHolds { limit } => {
            w::TaskCapacityCause::ProjectPlacedHolds(w::TaskCapacityCauseProjectPlacedHolds { limit: *(limit) })
        }
        c::TaskCapacityCause::PartyPlacedHolds { limit } => {
            w::TaskCapacityCause::PartyPlacedHolds(w::TaskCapacityCausePartyPlacedHolds { limit: *(limit) })
        }
        c::TaskCapacityCause::Exemptions { limit } => {
            w::TaskCapacityCause::Exemptions(w::TaskCapacityCauseExemptions { limit: *(limit) })
        }
    })
}

pub(in crate::codec) fn wire_program_host(
    value: &c::ProgramHost,
    limits: &w::Limits,
) -> Result<w::ProgramHost, w::Problem> {
    Ok(w::ProgramHost {
        name: bytes::copy_of(&value.name),
        channel: value.channel,
        connected: value.connected,
        awaiting_logins: value.awaiting_logins,
        programs: pieces(value.programs.iter().map(|item| wire_program_login(item, limits)), limits)?,
        cut: value.cut,
    })
}
