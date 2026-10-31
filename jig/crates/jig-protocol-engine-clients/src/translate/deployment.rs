//! Whole live document facts (protocol/client.md, 3 and 7).
use super::{chips, owner, pieces, refusals, task, w};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_core::documents as d;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Writer};
pub(super) fn host_kind(value: fleet::HostKind) -> w::HostKind {
    match value {
        fleet::HostKind::Worker => w::HostKind::Worker,
        fleet::HostKind::Engine => w::HostKind::Engine,
    }
}
fn attempt(value: fleet::AttemptState) -> w::AttemptState {
    match value {
        fleet::AttemptState::Waiting => w::AttemptState::Waiting,
        fleet::AttemptState::Adopted => w::AttemptState::Adopted,
        fleet::AttemptState::Claimed => w::AttemptState::Claimed,
        fleet::AttemptState::Cancelled => w::AttemptState::Cancelled,
        fleet::AttemptState::Handed => w::AttemptState::Handed,
        fleet::AttemptState::Acknowledged => w::AttemptState::Acknowledged,
        fleet::AttemptState::Stray => w::AttemptState::Stray,
        fleet::AttemptState::Kept => w::AttemptState::Kept,
        fleet::AttemptState::Fenced => w::AttemptState::Fenced,
        fleet::AttemptState::Closed => w::AttemptState::Closed,
    }
}
// The current client wire reports these dimensions without their charter numbers.
fn shortfall(value: fleet::AgentShortfall, limits: &w::Limits) -> Result<w::AgentShortfall, w::Problem> {
    Ok(match value {
        fleet::AgentShortfall::Model { charter, model } => w::AgentShortfall::Model(w::AgentShortfallModel::new(
            limits,
            w::AgentShortfallModelParts { charter, model },
        )?),
        fleet::AgentShortfall::Window { charter, model } => w::AgentShortfall::Window(w::AgentShortfallWindow::new(
            limits,
            w::AgentShortfallWindowParts { charter, model },
        )?),
        fleet::AgentShortfall::Output { charter, model } => w::AgentShortfall::Output(w::AgentShortfallOutput::new(
            limits,
            w::AgentShortfallOutputParts { charter, model },
        )?),
        fleet::AgentShortfall::Turns { charter: _ } => w::AgentShortfall::Turns,
        fleet::AgentShortfall::Spend => w::AgentShortfall::Spend,
        fleet::AgentShortfall::Time { charter: _ } => w::AgentShortfall::Time,
        fleet::AgentShortfall::Waiting { charter: _ } => w::AgentShortfall::Waiting,
        fleet::AgentShortfall::Checks { charter: _ } => w::AgentShortfall::Checks,
        fleet::AgentShortfall::Message => w::AgentShortfall::Message,
        fleet::AgentShortfall::Inbox => w::AgentShortfall::Inbox,
        fleet::AgentShortfall::Unacknowledged => w::AgentShortfall::Unacknowledged,
    })
}
pub(super) fn hello(value: fleet::HelloRefusal, limits: &w::Limits) -> Result<w::HelloRefusal, w::Problem> {
    Ok(match value {
        fleet::HelloRefusal::Agent { why } => w::HelloRefusal::Agent(w::HelloRefusalAgent::new(
            limits,
            w::HelloRefusalAgentParts { why: shortfall(why, limits)? },
        )?),
        fleet::HelloRefusal::ReservedChannel => w::HelloRefusal::ReservedChannel,
        fleet::HelloRefusal::Workers { limit } => {
            w::HelloRefusal::Workers(w::HelloRefusalWorkers::new(limits, w::HelloRefusalWorkersParts { limit })?)
        }
        fleet::HelloRefusal::Position { worker } => {
            w::HelloRefusal::Position(w::HelloRefusalPosition::new(limits, w::HelloRefusalPositionParts { worker })?)
        }
        fleet::HelloRefusal::StopBound { grace } => {
            w::HelloRefusal::StopBound(w::HelloRefusalStopBound::new(limits, w::HelloRefusalStopBoundParts { grace })?)
        }
    })
}
pub(crate) fn account_place(value: accounts::Place) -> w::AccountPlace {
    match value {
        accounts::Place::Directory => w::AccountPlace::Directory,
        accounts::Place::File => w::AccountPlace::File,
    }
}
pub(crate) fn account_unread(value: accounts::Unread) -> w::AccountUnread {
    match value {
        accounts::Unread::Unreadable => w::AccountUnread::Unreadable,
        accounts::Unread::Refused => w::AccountUnread::Refused,
        accounts::Unread::Failed => w::AccountUnread::Failed,
        accounts::Unread::Stalled => w::AccountUnread::Stalled,
        accounts::Unread::TooLarge => w::AccountUnread::TooLarge,
    }
}
pub(crate) fn account_state(value: d::AccountState, limits: &w::Limits) -> Result<w::AccountState, w::Problem> {
    Ok(match value {
        d::AccountState::Granted => w::AccountState::Granted,
        d::AccountState::Asking => w::AccountState::Asking,
        d::AccountState::SignInAgain => w::AccountState::SignInAgain,
        d::AccountState::Misconfigured => w::AccountState::Misconfigured,
        d::AccountState::Cancelled => w::AccountState::Cancelled,
        d::AccountState::Moving => w::AccountState::Moving,
        d::AccountState::Closing => w::AccountState::Closing,
        d::AccountState::Later { until } => w::AccountState::Later(w::AccountStateLater::new(
            limits,
            w::AccountStateLaterParts {
                until: match until {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
            },
        )?),
        d::AccountState::Unloaded { at, why } => w::AccountState::Unloaded(w::AccountStateUnloaded::new(
            limits,
            w::AccountStateUnloadedParts { at: account_place(at), why: account_unread(why) },
        )?),
    })
}
fn person_action(value: people::CapacityAction, limits: &w::Limits) -> Result<w::PersonCapacityAction, w::Problem> {
    Ok(match value {
        people::CapacityAction::AccountSignIn => w::PersonCapacityAction::AccountSignIn,
        people::CapacityAction::AccountSignOut => w::PersonCapacityAction::AccountSignOut,
        people::CapacityAction::AccountChange => w::PersonCapacityAction::AccountChange,
        people::CapacityAction::Create => w::PersonCapacityAction::Create,
        people::CapacityAction::Compound => w::PersonCapacityAction::Compound,
        people::CapacityAction::Service => w::PersonCapacityAction::Service,
        people::CapacityAction::Adoption => w::PersonCapacityAction::Adoption,
        people::CapacityAction::Roles => w::PersonCapacityAction::Roles,
        people::CapacityAction::Policy => w::PersonCapacityAction::Policy,
        people::CapacityAction::Application { connector } => w::PersonCapacityAction::Application(
            w::PersonCapacityActionApplication::new(limits, w::PersonCapacityActionApplicationParts { connector })?,
        ),
        people::CapacityAction::Note { name } => w::PersonCapacityAction::Note(w::PersonCapacityActionNote::new(
            limits,
            w::PersonCapacityActionNoteParts { name },
        )?),
        people::CapacityAction::Pool { person } => w::PersonCapacityAction::Pool(w::PersonCapacityActionPool::new(
            limits,
            w::PersonCapacityActionPoolParts { person },
        )?),
        people::CapacityAction::Escalation { task } => w::PersonCapacityAction::Escalation(
            w::PersonCapacityActionEscalation::new(limits, w::PersonCapacityActionEscalationParts { task })?,
        ),
        people::CapacityAction::Words { task } => w::PersonCapacityAction::Words(w::PersonCapacityActionWords::new(
            limits,
            w::PersonCapacityActionWordsParts { task },
        )?),
        people::CapacityAction::Question { task, question } => w::PersonCapacityAction::Question(
            w::PersonCapacityActionQuestion::new(limits, w::PersonCapacityActionQuestionParts { task, question })?,
        ),
        people::CapacityAction::Amend { task } => w::PersonCapacityAction::Amend(w::PersonCapacityActionAmend::new(
            limits,
            w::PersonCapacityActionAmendParts { task },
        )?),
        people::CapacityAction::Move { task } => w::PersonCapacityAction::MoveTask(w::PersonCapacityActionMove::new(
            limits,
            w::PersonCapacityActionMoveParts { task },
        )?),
        people::CapacityAction::TakePerson { task } => w::PersonCapacityAction::TakePerson(
            w::PersonCapacityActionTakePerson::new(limits, w::PersonCapacityActionTakePersonParts { task })?,
        ),
        people::CapacityAction::HandBackPerson { task } => w::PersonCapacityAction::HandBackPerson(
            w::PersonCapacityActionHandBackPerson::new(limits, w::PersonCapacityActionHandBackPersonParts { task })?,
        ),
        people::CapacityAction::AnswerPerson { task } => w::PersonCapacityAction::AnswerPerson(
            w::PersonCapacityActionAnswerPerson::new(limits, w::PersonCapacityActionAnswerPersonParts { task })?,
        ),
        people::CapacityAction::Stop { task } => w::PersonCapacityAction::Stop(w::PersonCapacityActionStop::new(
            limits,
            w::PersonCapacityActionStopParts { task },
        )?),
        people::CapacityAction::Cancel { task } => w::PersonCapacityAction::Cancel(w::PersonCapacityActionCancel::new(
            limits,
            w::PersonCapacityActionCancelParts { task },
        )?),
        people::CapacityAction::Lift { hold } => w::PersonCapacityAction::Lift(w::PersonCapacityActionLift::new(
            limits,
            w::PersonCapacityActionLiftParts { hold },
        )?),
        people::CapacityAction::Release { task } => w::PersonCapacityAction::Release(
            w::PersonCapacityActionRelease::new(limits, w::PersonCapacityActionReleaseParts { task })?,
        ),
        people::CapacityAction::Proposal { proposer, number } => {
            w::PersonCapacityAction::Proposal(w::PersonCapacityActionProposal::new(
                limits,
                w::PersonCapacityActionProposalParts {
                    proposer: task::proposal(&d::ProposalRef { proposer, number }, limits)?.proposer().clone(),
                    number,
                },
            )?)
        }
        people::CapacityAction::MarkSeen { seen } => {
            w::PersonCapacityAction::MarkSeen(w::PersonCapacityActionMarkSeen::new(
                limits,
                w::PersonCapacityActionMarkSeenParts { seen: seen_record(seen, limits)? },
            )?)
        }
    })
}
fn seen_record(value: people::Seen, limits: &w::Limits) -> Result<w::SeenThrough, w::Problem> {
    Ok(match value {
        people::Seen::Results { through } => {
            w::SeenThrough::Results(w::SeenResults::new(limits, w::SeenResultsParts { through })?)
        }
        people::Seen::Conversation { task, through } => {
            w::SeenThrough::Conversation(w::SeenConversation::new(limits, w::SeenConversationParts { task, through })?)
        }
    })
}
fn person_cause(value: people::CapacityCause, limits: &w::Limits) -> Result<w::PersonCapacityCause, w::Problem> {
    Ok(match value {
        people::CapacityCause::Requests { limit } => w::PersonCapacityCause::Requests(
            w::PersonCapacityCauseRequests::new(limits, w::PersonCapacityCauseRequestsParts { limit })?,
        ),
        people::CapacityCause::Pending { limit } => w::PersonCapacityCause::Pending(
            w::PersonCapacityCausePending::new(limits, w::PersonCapacityCausePendingParts { limit })?,
        ),
        people::CapacityCause::Waiters { limit } => w::PersonCapacityCause::Waiters(
            w::PersonCapacityCauseWaiters::new(limits, w::PersonCapacityCauseWaitersParts { limit })?,
        ),
        people::CapacityCause::Limit { which } => w::PersonCapacityCause::Limit(w::PersonCapacityCauseLimit::new(
            limits,
            w::PersonCapacityCauseLimitParts { which: refusals::limit(which) },
        )?),
    })
}
fn task_action(value: tasks::CapacityAction, limits: &w::Limits) -> Result<w::TaskCapacityAction, w::Problem> {
    Ok(match value {
        tasks::CapacityAction::Place { hold } => w::TaskCapacityAction::Place(w::TaskCapacityActionPlace::new(
            limits,
            w::TaskCapacityActionPlaceParts { hold },
        )?),
        tasks::CapacityAction::Release => w::TaskCapacityAction::Release,
        tasks::CapacityAction::Move => w::TaskCapacityAction::MoveTask,
        tasks::CapacityAction::Batch { first, members } => w::TaskCapacityAction::Batch(
            w::TaskCapacityActionBatch::new(limits, w::TaskCapacityActionBatchParts { first, members })?,
        ),
        tasks::CapacityAction::Proposal { number } => w::TaskCapacityAction::Proposal(
            w::TaskCapacityActionProposal::new(limits, w::TaskCapacityActionProposalParts { number })?,
        ),
        tasks::CapacityAction::PersonProposal { number } => w::TaskCapacityAction::PersonProposal(
            w::TaskCapacityActionPersonProposal::new(limits, w::TaskCapacityActionPersonProposalParts { number })?,
        ),
        tasks::CapacityAction::Message { task, number } => w::TaskCapacityAction::Message(
            w::TaskCapacityActionMessage::new(limits, w::TaskCapacityActionMessageParts { task, number })?,
        ),
        tasks::CapacityAction::Introduce { left, right } => w::TaskCapacityAction::Introduce(
            w::TaskCapacityActionIntroduce::new(limits, w::TaskCapacityActionIntroduceParts { left, right })?,
        ),
        tasks::CapacityAction::Subscription { number } => w::TaskCapacityAction::Subscription(
            w::TaskCapacityActionSubscription::new(limits, w::TaskCapacityActionSubscriptionParts { number })?,
        ),
        tasks::CapacityAction::Effect { entry } => w::TaskCapacityAction::Effect(w::TaskCapacityActionEffect::new(
            limits,
            w::TaskCapacityActionEffectParts { entry },
        )?),
        tasks::CapacityAction::Claim { attempt } => w::TaskCapacityAction::Claim(w::TaskCapacityActionClaim::new(
            limits,
            w::TaskCapacityActionClaimParts { attempt },
        )?),
        tasks::CapacityAction::Source { funder } => w::TaskCapacityAction::Source(w::TaskCapacityActionSource::new(
            limits,
            w::TaskCapacityActionSourceParts { funder: owner::funder(funder, limits)? },
        )?),
    })
}
fn task_cause(value: tasks::CapacityCause, limits: &w::Limits) -> Result<w::TaskCapacityCause, w::Problem> {
    Ok(match value {
        tasks::CapacityCause::PlacedHolds { limit } => w::TaskCapacityCause::PlacedHolds(
            w::TaskCapacityCausePlacedHolds::new(limits, w::TaskCapacityCausePlacedHoldsParts { limit })?,
        ),
        tasks::CapacityCause::ProjectPlacedHolds { limit } => w::TaskCapacityCause::ProjectPlacedHolds(
            w::TaskCapacityCauseProjectPlacedHolds::new(limits, w::TaskCapacityCauseProjectPlacedHoldsParts { limit })?,
        ),
        tasks::CapacityCause::PartyPlacedHolds { limit } => w::TaskCapacityCause::PartyPlacedHolds(
            w::TaskCapacityCausePartyPlacedHolds::new(limits, w::TaskCapacityCausePartyPlacedHoldsParts { limit })?,
        ),
        tasks::CapacityCause::Exemptions { limit } => w::TaskCapacityCause::Exemptions(
            w::TaskCapacityCauseExemptions::new(limits, w::TaskCapacityCauseExemptionsParts { limit })?,
        ),
        tasks::CapacityCause::Tasks { limit } => w::TaskCapacityCause::Tasks(w::TaskCapacityCauseTasks::new(
            limits,
            w::TaskCapacityCauseTasksParts { limit },
        )?),
        tasks::CapacityCause::CreationFlights { limit } => w::TaskCapacityCause::CreationFlights(
            w::TaskCapacityCauseCreationFlights::new(limits, w::TaskCapacityCauseCreationFlightsParts { limit })?,
        ),
        tasks::CapacityCause::ResourceReports { limit } => w::TaskCapacityCause::ResourceReports(
            w::TaskCapacityCauseResourceReports::new(limits, w::TaskCapacityCauseResourceReportsParts { limit })?,
        ),
        tasks::CapacityCause::ProjectTasks { limit } => w::TaskCapacityCause::ProjectTasks(
            w::TaskCapacityCauseProjectTasks::new(limits, w::TaskCapacityCauseProjectTasksParts { limit })?,
        ),
        tasks::CapacityCause::Tree { limit } => {
            w::TaskCapacityCause::Tree(w::TaskCapacityCauseTree::new(limits, w::TaskCapacityCauseTreeParts { limit })?)
        }
        tasks::CapacityCause::Delegates { limit } => w::TaskCapacityCause::Delegates(
            w::TaskCapacityCauseDelegates::new(limits, w::TaskCapacityCauseDelegatesParts { limit })?,
        ),
        tasks::CapacityCause::Holdings { limit } => w::TaskCapacityCause::Holdings(w::TaskCapacityCauseHoldings::new(
            limits,
            w::TaskCapacityCauseHoldingsParts { limit },
        )?),
        tasks::CapacityCause::HoldWaiters { limit } => w::TaskCapacityCause::HoldWaiters(
            w::TaskCapacityCauseHoldWaiters::new(limits, w::TaskCapacityCauseHoldWaitersParts { limit })?,
        ),
        tasks::CapacityCause::Proposals { limit } => w::TaskCapacityCause::Proposals(
            w::TaskCapacityCauseProposals::new(limits, w::TaskCapacityCauseProposalsParts { limit })?,
        ),
        tasks::CapacityCause::References { limit } => w::TaskCapacityCause::References(
            w::TaskCapacityCauseReferences::new(limits, w::TaskCapacityCauseReferencesParts { limit })?,
        ),
        tasks::CapacityCause::Subscriptions { limit } => w::TaskCapacityCause::Subscriptions(
            w::TaskCapacityCauseSubscriptions::new(limits, w::TaskCapacityCauseSubscriptionsParts { limit })?,
        ),
        tasks::CapacityCause::Funders { limit } => w::TaskCapacityCause::Funders(w::TaskCapacityCauseFunders::new(
            limits,
            w::TaskCapacityCauseFundersParts { limit },
        )?),
        tasks::CapacityCause::Writers { limit } => w::TaskCapacityCause::Writers(w::TaskCapacityCauseWriters::new(
            limits,
            w::TaskCapacityCauseWritersParts { limit },
        )?),
        tasks::CapacityCause::Inbox { task, messages, bytes } => w::TaskCapacityCause::Inbox(
            w::TaskCapacityCauseInbox::new(limits, w::TaskCapacityCauseInboxParts { task, messages, bytes })?,
        ),
    })
}
pub fn capacity(value: d::Capacity, limits: &w::Limits) -> Result<w::Capacity, w::Problem> {
    w::Capacity::new(limits, w::CapacityParts { used: value.used, limit: value.limit })
}
fn location(value: d::RunLocation, limits: &w::Limits) -> Result<w::RunLocation, w::Problem> {
    Ok(match value {
        d::RunLocation::None => w::RunLocation::None,
        d::RunLocation::Connected { channel, kind } => w::RunLocation::Connected(w::RunLocationConnected::new(
            limits,
            w::RunLocationConnectedParts { channel, kind: host_kind(kind) },
        )?),
        d::RunLocation::Adrift { channel, until, remaining } => w::RunLocation::Adrift(w::RunLocationAdrift::new(
            limits,
            w::RunLocationAdriftParts {
                channel,
                until: match until {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
                remaining,
            },
        )?),
    })
}
pub fn run(value: &d::HostRun, limits: &w::Limits) -> Result<w::HostRun, w::Problem> {
    w::HostRun::new(
        limits,
        w::HostRunParts {
            task: task::link(&value.task, limits)?,
            attempt: value.attempt,
            kept: value.kept,
            listed: value.listed,
            state: attempt(value.state),
            location: location(value.location, limits)?,
            grace_until: match value.grace_until {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        },
    )
}
fn host_rule(value: jig_domain_core::HostRule, limits: &w::Limits) -> Result<w::HostRule, w::Problem> {
    use jig_domain_core::HostRule;
    Ok(match value {
        HostRule::Open => w::HostRule::Open,
        HostRule::Person(party) => w::HostRule::Person(w::HostPerson::new(limits, w::HostPersonParts { party })?),
        HostRule::Several => w::HostRule::Several,
        HostRule::Unknown => w::HostRule::Unknown,
    })
}
pub fn host(value: &d::Host, limits: &w::Limits) -> Result<w::Host, w::Problem> {
    let mut runs = List::with_capacity(value.runs.len());
    for row in &value.runs {
        runs.push(run(row, limits)?).expect("supplied rows fit");
    }
    let runs = run_pieces(&runs, limits.host_runs, w::Path::HostRuns)?;

    w::Host::new(
        limits,
        w::HostParts {
            skein_bools: [value.awaiting_logins, value.draining, value.connected, value.cut],
            name: value.name.clone(),
            rule: host_rule(value.rule, limits)?,
            programs: super::programs::logins(&value.programs, limits, limits.host_programs, w::Path::HostPrograms)?,
            channel: value.channel,
            kind: host_kind(value.kind),
            slots: value.slots,
            used: value.used,
            free: value.free,
            grace_until: match value.grace_until {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            grace_remaining: value.grace_remaining,
            runs,
        },
    )
}
pub fn account(value: &d::Account, limits: &w::Limits) -> Result<w::Account, w::Problem> {
    let mut runs = List::with_capacity(value.runs.len());
    for row in &value.runs {
        runs.push(chips::chip(row, limits)?).expect("supplied rows fit");
    }
    let runs = chip_pieces(&runs, limits.account_runs, w::Path::AccountRuns)?;
    let mut waiting = List::with_capacity(value.waiting.len());
    for row in &value.waiting {
        waiting.push(chips::chip(row, limits)?).expect("supplied rows fit");
    }
    let waiting = chip_pieces(&waiting, limits.account_waiting, w::Path::AccountWaiting)?;

    w::Account::new(
        limits,
        w::AccountParts {
            number: value.number,
            place: value.place,
            uses: super::accounts::uses(value.uses.as_ref(), limits)?,
            provider: value.provider,
            provider_name: match &value.provider_name {
                Some(name) => Some(owner::text(name, limits)?),
                None => None,
            },
            manage_usage: value.manage_usage.clone(),
            display: owner::text(&value.display, limits)?,
            holder: super::accounts::holder(value.holder, limits)?,
            sources: super::accounts::source_pieces(
                &value.sources,
                limits,
                limits.account_sources,
                w::Path::AccountSources,
            )?,
            generation: value.generation,
            state: account_state(value.state, limits)?,
            spent_until: match value.spent_until {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            runs,
            waiting,
            skein_bools: [value.plan, value.usable, value.expiring, value.attention, value.cut],
        },
    )
}
pub fn person_refusal(
    value: &people::CapacityRefusal,
    limits: &w::Limits,
) -> Result<w::PersonCapacityRefusal, w::Problem> {
    w::PersonCapacityRefusal::new(
        limits,
        w::PersonCapacityRefusalParts {
            key: w::RequestKey::new(limits, w::RequestKeyParts { person: value.key.person, key: value.key.key })?,
            project: value.project,
            action: person_action(value.action, limits)?,
            at: value.at.as_nanos(),
            cause: person_cause(value.cause, limits)?,
        },
    )
}
pub fn task_refusal(value: &tasks::CapacityRefusal, limits: &w::Limits) -> Result<w::TaskCapacityRefusal, w::Problem> {
    w::TaskCapacityRefusal::new(
        limits,
        w::TaskCapacityRefusalParts {
            request: w::TaskCapacityRequest::new(
                limits,
                w::TaskCapacityRequestParts {
                    project: value.request.project,
                    by: owner::requester(value.request.by, limits)?,
                    action: task_action(value.request.action, limits)?,
                    cause: task_cause(value.request.cause, limits)?,
                },
            )?,
            at: value.at.as_nanos(),
        },
    )
}
pub fn document(value: &d::Deployment, limits: &w::Limits) -> Result<w::Deployment, w::Problem> {
    let mut hosts = List::with_capacity(value.hosts.len());
    for row in &value.hosts {
        hosts.push(host(row, limits)?).expect("supplied rows fit");
    }
    let hosts = host_pieces(&hosts, limits.deployment_hosts, w::Path::DeploymentHosts)?;
    let mut attempts = List::with_capacity(value.attempts.len());
    for row in &value.attempts {
        attempts.push(run(row, limits)?).expect("supplied rows fit");
    }
    let attempts = run_pieces(&attempts, limits.deployment_attempts, w::Path::DeploymentAttempts)?;
    let mut accounts = List::with_capacity(value.accounts.len());
    for row in &value.accounts {
        accounts.push(account(row, limits)?).expect("supplied rows fit");
    }
    let accounts = account_pieces(&accounts, limits.deployment_accounts, w::Path::DeploymentAccounts)?;
    let mut projects = List::with_capacity(value.projects.len());
    for row in &value.projects {
        projects
            .push(w::ProjectCapacity::new(
                limits,
                w::ProjectCapacityParts {
                    project: row.project,
                    tasks: capacity(row.tasks, limits)?,
                    proposals: row.proposals,
                },
            )?)
            .expect("supplied rows fit");
    }
    let mut hello_refusals = List::with_capacity(value.hello_refusals.len());
    for row in &value.hello_refusals {
        hello_refusals
            .push(w::HelloRefusalLine::new(
                limits,
                w::HelloRefusalLineParts {
                    channel: row.channel,
                    at: row.at.as_nanos(),
                    slots: row.slots,
                    stop_bound: row.stop_bound,
                    why: hello(row.why, limits)?,
                },
            )?)
            .expect("supplied rows fit");
    }
    let mut person_room_refusals = List::with_capacity(value.person_room_refusals.len());
    for row in &value.person_room_refusals {
        person_room_refusals.push(person_refusal(row, limits)?).expect("supplied rows fit");
    }
    let mut task_room_refusals = List::with_capacity(value.task_room_refusals.len());
    for row in &value.task_room_refusals {
        task_room_refusals.push(task_refusal(row, limits)?).expect("supplied rows fit");
    }
    let mut connectors = List::with_capacity(value.connectors.len());
    for row in &value.connectors {
        connectors.push(*row).expect("supplied rows fit");
    }

    w::Deployment::new(
        limits,
        w::DeploymentParts {
            skein_bools: [value.cut, value.connectors_cut, value.providers_cut, value.readiness_cut],
            providers: super::catalogue::offers(&value.providers, limits.deployment_providers, limits)?,
            key_bytes: value.key_bytes,
            readiness: super::catalogue::readiness(&value.readiness, limits.deployment_readiness, limits)?,
            store: w::StoreFill::new(limits, w::StoreFillParts { bytes: value.store.bytes, mark: value.store.mark })?,
            hosts,
            attempts,
            accounts,
            tasks: capacity(value.tasks, limits)?,
            workers: capacity(value.workers, limits)?,
            attempts_capacity: capacity(value.attempts_capacity, limits)?,
            waiting_attempts: value.waiting_attempts,
            proposals: capacity(value.proposals, limits)?,
            projects,
            grace_bound: value.grace_bound,
            hello_refusals,
            hosts_forgotten: value.hosts_forgotten,
            hello_refusals_forgotten: value.hello_refusals_forgotten,
            person_room_refusals,
            person_room_refusals_forgotten: value.person_room_refusals_forgotten,
            person_answer_retention: value.person_answer_retention,
            task_room_refusals,
            task_room_refusals_forgotten: value.task_room_refusals_forgotten,
            connectors,
        },
    )
}
fn chip_pieces(rows: &List<w::Chip>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn run_pieces(rows: &List<w::HostRun>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn host_pieces(rows: &List<w::Host>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn account_pieces(rows: &List<w::Account>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
