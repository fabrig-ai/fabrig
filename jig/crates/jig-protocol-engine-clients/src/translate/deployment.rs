//! Whole live document facts (protocol/client/wire.md, 1, and protocol/client/README.md, 5).
use super::{chips, owner, pieces, refusals, task, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_accounts as accounts;
use jig_domain_core_vocabulary::documents as d;
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
pub(super) fn shortfall(value: fleet::AgentShortfall) -> w::AgentShortfall {
    match value {
        fleet::AgentShortfall::Model { charter, model } => {
            w::AgentShortfall::Model(w::AgentShortfallModel { charter, model })
        }
        fleet::AgentShortfall::Window { charter, model } => {
            w::AgentShortfall::Window(w::AgentShortfallWindow { charter, model })
        }
        fleet::AgentShortfall::Output { charter, model } => {
            w::AgentShortfall::Output(w::AgentShortfallOutput { charter, model })
        }
        fleet::AgentShortfall::Turns { charter } => w::AgentShortfall::Turns(w::AgentShortfallTurns { charter }),
        fleet::AgentShortfall::Spend => w::AgentShortfall::Spend,
        fleet::AgentShortfall::Time { charter } => w::AgentShortfall::Time(w::AgentShortfallTime { charter }),
        fleet::AgentShortfall::Waiting { charter } => w::AgentShortfall::Waiting(w::AgentShortfallWaiting { charter }),
        fleet::AgentShortfall::Checks { charter } => w::AgentShortfall::Checks(w::AgentShortfallChecks { charter }),
        fleet::AgentShortfall::Message => w::AgentShortfall::Message,
        fleet::AgentShortfall::Inbox => w::AgentShortfall::Inbox,
        fleet::AgentShortfall::Unacknowledged => w::AgentShortfall::Unacknowledged,
    }
}
pub(super) fn hello(value: fleet::HelloRefusal) -> w::HelloRefusal {
    match value {
        fleet::HelloRefusal::Agent { why } => w::HelloRefusal::Agent(w::HelloRefusalAgent { why: shortfall(why) }),
        fleet::HelloRefusal::JobKinds => w::HelloRefusal::JobKinds,
        fleet::HelloRefusal::ReservedChannel => w::HelloRefusal::ReservedChannel,
        fleet::HelloRefusal::Workers { limit } => w::HelloRefusal::Workers(w::HelloRefusalWorkers { limit }),
        fleet::HelloRefusal::Position { worker } => w::HelloRefusal::Position(w::HelloRefusalPosition { worker }),
        fleet::HelloRefusal::StopBound { grace } => w::HelloRefusal::StopBound(w::HelloRefusalStopBound { grace }),
    }
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
pub(crate) fn account_state(value: d::AccountState) -> w::AccountState {
    match value {
        d::AccountState::Granted => w::AccountState::Granted,
        d::AccountState::Asking => w::AccountState::Asking,
        d::AccountState::SignInAgain => w::AccountState::SignInAgain,
        d::AccountState::Misconfigured => w::AccountState::Misconfigured,
        d::AccountState::Cancelled => w::AccountState::Cancelled,
        d::AccountState::Moving => w::AccountState::Moving,
        d::AccountState::Closing => w::AccountState::Closing,
        d::AccountState::Later { until } => w::AccountState::Later(w::AccountStateLater {
            until: match until {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        }),
        d::AccountState::Unloaded { at, why } => {
            w::AccountState::Unloaded(w::AccountStateUnloaded { at: account_place(at), why: account_unread(why) })
        }
    }
}
fn person_action(value: people::CapacityAction) -> w::PersonCapacityAction {
    match value {
        people::CapacityAction::AccountSignIn => w::PersonCapacityAction::AccountSignIn,
        people::CapacityAction::AccountSignOut => w::PersonCapacityAction::AccountSignOut,
        people::CapacityAction::AccountChange => w::PersonCapacityAction::AccountChange,
        people::CapacityAction::Create => w::PersonCapacityAction::Create,
        people::CapacityAction::Compound => w::PersonCapacityAction::Compound,
        people::CapacityAction::Service => w::PersonCapacityAction::Service,
        people::CapacityAction::Adoption => w::PersonCapacityAction::Adoption,
        people::CapacityAction::Roles => w::PersonCapacityAction::Roles,
        people::CapacityAction::Policy => w::PersonCapacityAction::Policy,
        people::CapacityAction::Application { connector } => {
            w::PersonCapacityAction::Application(w::PersonCapacityActionApplication { connector })
        }
        people::CapacityAction::Note { name } => w::PersonCapacityAction::Note(w::PersonCapacityActionNote { name }),
        people::CapacityAction::Pool { person } => {
            w::PersonCapacityAction::Pool(w::PersonCapacityActionPool { person })
        }
        people::CapacityAction::Escalation { task } => {
            w::PersonCapacityAction::Escalation(w::PersonCapacityActionEscalation { task })
        }
        people::CapacityAction::Words { task } => w::PersonCapacityAction::Words(w::PersonCapacityActionWords { task }),
        people::CapacityAction::Question { task, question } => {
            w::PersonCapacityAction::Question(w::PersonCapacityActionQuestion { task, question })
        }
        people::CapacityAction::Amend { task } => w::PersonCapacityAction::Amend(w::PersonCapacityActionAmend { task }),
        people::CapacityAction::Move { task } => {
            w::PersonCapacityAction::MoveTask(w::PersonCapacityActionMove { task })
        }
        people::CapacityAction::TakePerson { task } => {
            w::PersonCapacityAction::TakePerson(w::PersonCapacityActionTakePerson { task })
        }
        people::CapacityAction::HandBackPerson { task } => {
            w::PersonCapacityAction::HandBackPerson(w::PersonCapacityActionHandBackPerson { task })
        }
        people::CapacityAction::AnswerPerson { task } => {
            w::PersonCapacityAction::AnswerPerson(w::PersonCapacityActionAnswerPerson { task })
        }
        people::CapacityAction::Stop { task } => w::PersonCapacityAction::Stop(w::PersonCapacityActionStop { task }),
        people::CapacityAction::Conclude { task } => {
            w::PersonCapacityAction::Conclude(w::PersonCapacityActionConclude { task })
        }
        people::CapacityAction::Cancel { task } => {
            w::PersonCapacityAction::Cancel(w::PersonCapacityActionCancel { task })
        }
        people::CapacityAction::Lift { hold } => w::PersonCapacityAction::Lift(w::PersonCapacityActionLift { hold }),
        people::CapacityAction::Release { task } => {
            w::PersonCapacityAction::Release(w::PersonCapacityActionRelease { task })
        }
        people::CapacityAction::Proposal { proposer, number } => {
            w::PersonCapacityAction::Proposal(w::PersonCapacityActionProposal {
                proposer: task::proposal(&d::ProposalRef { proposer, number }).proposer,
                number,
            })
        }
        people::CapacityAction::MarkSeen { seen } => {
            w::PersonCapacityAction::MarkSeen(w::PersonCapacityActionMarkSeen { seen: seen_record(seen) })
        }
    }
}
fn seen_record(value: people::Seen) -> w::SeenThrough {
    match value {
        people::Seen::Results { through } => w::SeenThrough::Results(w::SeenResults { through }),
        people::Seen::Conversation { task, through } => {
            w::SeenThrough::Conversation(w::SeenConversation { task, through })
        }
    }
}
fn person_cause(value: people::CapacityCause) -> w::PersonCapacityCause {
    match value {
        people::CapacityCause::Requests { limit } => {
            w::PersonCapacityCause::Requests(w::PersonCapacityCauseRequests { limit })
        }
        people::CapacityCause::Pending { limit } => {
            w::PersonCapacityCause::Pending(w::PersonCapacityCausePending { limit })
        }
        people::CapacityCause::Waiters { limit } => {
            w::PersonCapacityCause::Waiters(w::PersonCapacityCauseWaiters { limit })
        }
        people::CapacityCause::Limit { which } => {
            w::PersonCapacityCause::Limit(w::PersonCapacityCauseLimit { which: refusals::limit(which) })
        }
    }
}
fn task_action(value: tasks::CapacityAction) -> w::TaskCapacityAction {
    match value {
        tasks::CapacityAction::Place { hold } => w::TaskCapacityAction::Place(w::TaskCapacityActionPlace { hold }),
        tasks::CapacityAction::Release => w::TaskCapacityAction::Release,
        tasks::CapacityAction::Move => w::TaskCapacityAction::MoveTask,
        tasks::CapacityAction::Batch { first, members } => {
            w::TaskCapacityAction::Batch(w::TaskCapacityActionBatch { first, members })
        }
        tasks::CapacityAction::Proposal { number } => {
            w::TaskCapacityAction::Proposal(w::TaskCapacityActionProposal { number })
        }
        tasks::CapacityAction::PersonProposal { number } => {
            w::TaskCapacityAction::PersonProposal(w::TaskCapacityActionPersonProposal { number })
        }
        tasks::CapacityAction::Message { task, number } => {
            w::TaskCapacityAction::Message(w::TaskCapacityActionMessage { task, number })
        }
        tasks::CapacityAction::Introduce { left, right } => {
            w::TaskCapacityAction::Introduce(w::TaskCapacityActionIntroduce { left, right })
        }
        tasks::CapacityAction::Subscription { number } => {
            w::TaskCapacityAction::Subscription(w::TaskCapacityActionSubscription { number })
        }
        tasks::CapacityAction::Effect { entry } => w::TaskCapacityAction::Effect(w::TaskCapacityActionEffect { entry }),
        tasks::CapacityAction::Claim { attempt } => {
            w::TaskCapacityAction::Claim(w::TaskCapacityActionClaim { attempt })
        }
        tasks::CapacityAction::Source { funder } => {
            w::TaskCapacityAction::Source(w::TaskCapacityActionSource { funder: owner::funder(funder) })
        }
    }
}
fn task_cause(value: tasks::CapacityCause) -> w::TaskCapacityCause {
    match value {
        tasks::CapacityCause::PlacedHolds { limit } => {
            w::TaskCapacityCause::PlacedHolds(w::TaskCapacityCausePlacedHolds { limit })
        }
        tasks::CapacityCause::ProjectPlacedHolds { limit } => {
            w::TaskCapacityCause::ProjectPlacedHolds(w::TaskCapacityCauseProjectPlacedHolds { limit })
        }
        tasks::CapacityCause::PartyPlacedHolds { limit } => {
            w::TaskCapacityCause::PartyPlacedHolds(w::TaskCapacityCausePartyPlacedHolds { limit })
        }
        tasks::CapacityCause::Exemptions { limit } => {
            w::TaskCapacityCause::Exemptions(w::TaskCapacityCauseExemptions { limit })
        }
        tasks::CapacityCause::Tasks { limit } => w::TaskCapacityCause::Tasks(w::TaskCapacityCauseTasks { limit }),
        tasks::CapacityCause::CreationFlights { limit } => {
            w::TaskCapacityCause::CreationFlights(w::TaskCapacityCauseCreationFlights { limit })
        }
        tasks::CapacityCause::ResourceReports { limit } => {
            w::TaskCapacityCause::ResourceReports(w::TaskCapacityCauseResourceReports { limit })
        }
        tasks::CapacityCause::ProjectTasks { limit } => {
            w::TaskCapacityCause::ProjectTasks(w::TaskCapacityCauseProjectTasks { limit })
        }
        tasks::CapacityCause::Tree { limit } => w::TaskCapacityCause::Tree(w::TaskCapacityCauseTree { limit }),
        tasks::CapacityCause::Delegates { limit } => {
            w::TaskCapacityCause::Delegates(w::TaskCapacityCauseDelegates { limit })
        }
        tasks::CapacityCause::Holdings { limit } => {
            w::TaskCapacityCause::Holdings(w::TaskCapacityCauseHoldings { limit })
        }
        tasks::CapacityCause::HoldWaiters { limit } => {
            w::TaskCapacityCause::HoldWaiters(w::TaskCapacityCauseHoldWaiters { limit })
        }
        tasks::CapacityCause::Proposals { limit } => {
            w::TaskCapacityCause::Proposals(w::TaskCapacityCauseProposals { limit })
        }
        tasks::CapacityCause::References { limit } => {
            w::TaskCapacityCause::References(w::TaskCapacityCauseReferences { limit })
        }
        tasks::CapacityCause::Subscriptions { limit } => {
            w::TaskCapacityCause::Subscriptions(w::TaskCapacityCauseSubscriptions { limit })
        }
        tasks::CapacityCause::Funders { limit } => w::TaskCapacityCause::Funders(w::TaskCapacityCauseFunders { limit }),
        tasks::CapacityCause::Writers { limit } => w::TaskCapacityCause::Writers(w::TaskCapacityCauseWriters { limit }),
        tasks::CapacityCause::Inbox { task, messages, bytes } => {
            w::TaskCapacityCause::Inbox(w::TaskCapacityCauseInbox { task, messages, bytes })
        }
    }
}
#[must_use]
pub fn capacity(value: d::Capacity) -> w::Capacity {
    w::Capacity { used: value.used, limit: value.limit }
}
fn location(value: d::RunLocation) -> w::RunLocation {
    match value {
        d::RunLocation::None => w::RunLocation::None,
        d::RunLocation::Connected { channel, kind } => {
            w::RunLocation::Connected(w::RunLocationConnected { channel, kind: host_kind(kind) })
        }
        d::RunLocation::Adrift { channel, until, remaining } => w::RunLocation::Adrift(w::RunLocationAdrift {
            channel,
            until: match until {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            remaining,
        }),
    }
}
pub fn run(value: &d::HostRun, limits: &w::Limits) -> Result<w::HostRun, w::Problem> {
    Ok(w::HostRun {
        task: task::link(&value.task, limits)?,
        attempt: value.attempt,
        kept: value.kept,
        listed: value.listed,
        state: attempt(value.state),
        location: location(value.location),
        grace_until: match value.grace_until {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
    })
}
fn host_rule(value: jig_domain_core_vocabulary::HostRule) -> w::HostRule {
    use jig_domain_core_vocabulary::HostRule;
    match value {
        HostRule::Open => w::HostRule::Open,
        HostRule::Person(party) => w::HostRule::Person(w::HostPerson { party }),
        HostRule::Several => w::HostRule::Several,
        HostRule::Unknown => w::HostRule::Unknown,
    }
}
pub fn host(value: &d::Host, limits: &w::Limits) -> Result<w::Host, w::Problem> {
    let mut runs = List::with_capacity(value.runs.len());
    for row in &value.runs {
        runs.push(run(row, limits)?).expect("supplied rows fit");
    }
    let runs = run_pieces(&runs, limits.host_runs, w::paths::HOST_RUNS, limits)?;

    Ok(w::Host {
        awaiting_logins: value.awaiting_logins,
        draining: value.draining,
        connected: value.connected,
        cut: value.cut,
        name: value.name.clone(),
        rule: host_rule(value.rule),
        programs: super::programs::logins(&value.programs, limits, limits.host_programs, w::paths::HOST_PROGRAMS)?,
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
    })
}
pub fn account(value: &d::Account, limits: &w::Limits) -> Result<w::Account, w::Problem> {
    let mut runs = List::with_capacity(value.runs.len());
    for row in &value.runs {
        runs.push(chips::chip(row, limits)?).expect("supplied rows fit");
    }
    let runs = chip_pieces(&runs, limits.account_runs, w::paths::ACCOUNT_RUNS, limits)?;
    let mut waiting = List::with_capacity(value.waiting.len());
    for row in &value.waiting {
        waiting.push(chips::chip(row, limits)?).expect("supplied rows fit");
    }
    let waiting = chip_pieces(&waiting, limits.account_waiting, w::paths::ACCOUNT_WAITING, limits)?;

    Ok(w::Account {
        list: super::accounts::list(value.list.as_ref()),
        facts: super::accounts::facts(&value.facts),
        number: value.number,
        place: value.place,
        uses: super::accounts::uses(value.uses.as_ref()),
        provider: value.provider,
        provider_name: match &value.provider_name {
            Some(name) => Some(owner::text(name)),
            None => None,
        },
        manage_usage: value.manage_usage.clone(),
        display: owner::text(&value.display),
        holder: super::accounts::holder(value.holder),
        sources: super::accounts::source_pieces(
            &value.sources,
            limits,
            limits.account_sources,
            w::paths::ACCOUNT_SOURCES,
        )?,
        generation: value.generation,
        state: account_state(value.state),
        spent_until: match value.spent_until {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        runs,
        waiting,
        plan: value.plan,
        usable: value.usable,
        expiring: value.expiring,
        attention: value.attention,
        cut: value.cut,
    })
}
#[must_use]
pub fn person_refusal(value: &people::CapacityRefusal) -> w::PersonCapacityRefusal {
    w::PersonCapacityRefusal {
        key: w::RequestKey { person: value.key.person, key: value.key.key },
        project: value.project,
        action: person_action(value.action),
        at: value.at.as_nanos(),
        cause: person_cause(value.cause),
    }
}
#[must_use]
pub fn task_refusal(value: &tasks::CapacityRefusal) -> w::TaskCapacityRefusal {
    w::TaskCapacityRefusal {
        request: w::TaskCapacityRequest {
            project: value.request.project,
            by: owner::requester(value.request.by),
            action: task_action(value.request.action),
            cause: task_cause(value.request.cause),
        },
        at: value.at.as_nanos(),
    }
}
#[expect(clippy::too_many_lines, reason = "one field-for-field deployment document translation")]
pub fn document(value: &d::Deployment, limits: &w::Limits) -> Result<w::Deployment, w::Problem> {
    let mut hosts = List::with_capacity(value.hosts.len());
    for row in &value.hosts {
        hosts.push(host(row, limits)?).expect("supplied rows fit");
    }
    let hosts = host_pieces(&hosts, limits.deployment_hosts, w::paths::DEPLOYMENT_HOSTS, limits)?;
    let mut attempts = List::with_capacity(value.attempts.len());
    for row in &value.attempts {
        attempts.push(run(row, limits)?).expect("supplied rows fit");
    }
    let attempts = run_pieces(&attempts, limits.deployment_attempts, w::paths::DEPLOYMENT_ATTEMPTS, limits)?;
    let mut accounts = List::with_capacity(value.accounts.len());
    for row in &value.accounts {
        accounts.push(account(row, limits)?).expect("supplied rows fit");
    }
    let accounts = account_pieces(&accounts, limits.deployment_accounts, w::paths::DEPLOYMENT_ACCOUNTS, limits)?;
    let mut projects = List::with_capacity(value.projects.len());
    for row in &value.projects {
        projects
            .push(w::ProjectCapacity { project: row.project, tasks: capacity(row.tasks), proposals: row.proposals })
            .expect("supplied rows fit");
    }
    let mut hello_refusals = List::with_capacity(value.hello_refusals.len());
    for row in &value.hello_refusals {
        hello_refusals
            .push(w::HelloRefusalLine {
                channel: row.channel,
                at: row.at.as_nanos(),
                slots: row.slots,
                stop_bound: row.stop_bound,
                why: hello(row.why),
            })
            .expect("supplied rows fit");
    }
    let mut person_room_refusals = List::with_capacity(value.person_room_refusals.len());
    for row in &value.person_room_refusals {
        person_room_refusals.push(person_refusal(row)).expect("supplied rows fit");
    }
    let mut task_room_refusals = List::with_capacity(value.task_room_refusals.len());
    for row in &value.task_room_refusals {
        task_room_refusals.push(task_refusal(row)).expect("supplied rows fit");
    }
    let mut connectors = List::with_capacity(value.connectors.len());
    for row in &value.connectors {
        connectors.push(*row).expect("supplied rows fit");
    }

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
                pause: match row.pause {
                    jig_domain_link::PauseState::Taking => w::WorkerPause::Taking,
                    jig_domain_link::PauseState::Paused => w::WorkerPause::Paused,
                    jig_domain_link::PauseState::Draining => w::WorkerPause::Draining,
                    jig_domain_link::PauseState::Drained => w::WorkerPause::Drained,
                },
                since: match row.since {
                    Some(at) => Some(at.as_nanos()),
                    None => None,
                },
            })
            .expect("worker row count");
    }
    Ok(w::Deployment {
        worker_settings,
        cut: value.cut,
        connectors_cut: value.connectors_cut,
        providers_cut: value.providers_cut,
        readiness_cut: value.readiness_cut,
        providers: super::catalogue::offers(&value.providers, limits.deployment_providers, limits)?,
        key_bytes: value.key_bytes,
        readiness: super::catalogue::readiness(&value.readiness, limits.deployment_readiness, limits)?,
        store: w::StoreFill { bytes: value.store.bytes, mark: value.store.mark },
        census: match &value.census {
            Some(value) => Some(store_census(value)),
            None => None,
        },
        hosts,
        attempts,
        accounts,
        tasks: capacity(value.tasks),
        workers: capacity(value.workers),
        attempts_capacity: capacity(value.attempts_capacity),
        waiting_attempts: value.waiting_attempts,
        proposals: capacity(value.proposals),
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
    })
}
fn store_census(value: &d::StoreCensus) -> w::StoreCensus {
    let mut keyspaces = List::with_capacity(value.keyspaces.len());
    for row in &value.keyspaces {
        let mut heads = List::with_capacity(row.heads.len());
        for head in &row.heads {
            heads.push(w::CensusHead { head: head.head, rows: head.rows }).expect("supplied heads fit");
        }
        keyspaces
            .push(w::CensusKeyspace {
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
            .expect("supplied keyspaces fit");
    }
    w::StoreCensus { commit: value.commit, since: value.since, keyspaces }
}
fn chip_pieces(rows: &List<w::Chip>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn run_pieces(rows: &List<w::HostRun>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn host_pieces(rows: &List<w::Host>, bound: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn account_pieces(
    rows: &List<w::Account>,
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
    pieces(&encoded, bound, path)
}
