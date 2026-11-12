//! Deployment facts and store fill (client/domain/pages.md, 7; protocol/store/README.md, 6.2).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

pub use crate::below::documents::deployment::*;
use crate::{
    Core, Limits,
    documents::{Missing, TaskLink, Window, inbox_deployment::line_bound},
};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Time, Wall};

/// Clock supplied by the decision caller; deployment timers are volatile
/// monotonic facts and become wall times only against this injected pair.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DocumentClock {
    pub now: Time,
    pub wall: Wall,
}

pub(super) fn wall_deadline(clock: Option<DocumentClock>, at: Time) -> Option<Wall> {
    let clock = clock?;
    if at >= clock.now {
        Some(Wall::from_nanos(clock.wall.as_nanos().saturating_add(at.saturating_since(clock.now).as_nanos())))
    } else {
        Some(Wall::from_nanos(clock.wall.as_nanos().saturating_sub(clock.now.saturating_since(at).as_nanos())))
    }
}
pub(super) fn account_state(state: accounts::State) -> AccountState {
    match state {
        accounts::State::Moving => AccountState::Moving,
        accounts::State::Asking => AccountState::Asking,
        accounts::State::Granted => AccountState::Granted,
        accounts::State::Later { until } | accounts::State::Limited { until: Some(until) } => {
            AccountState::Later { until: Some(until.wall) }
        }
        accounts::State::Limited { until: None } => AccountState::Later { until: None },
        accounts::State::Refused | accounts::State::NeedsSignIn => AccountState::SignInAgain,
        accounts::State::Misconfigured => AccountState::Misconfigured,
        accounts::State::Unloaded { at, why } => AccountState::Unloaded { at, why },
        accounts::State::Cancelled => AccountState::Cancelled,
        accounts::State::Closing => AccountState::Closing,
    }
}
fn host_run(
    core: &Core,
    party: u64,
    limits: &Limits,
    clock: Option<DocumentClock>,
    value: &fleet::AttemptView,
) -> Result<HostRun, Missing> {
    let location = match value.placement {
        fleet::Placement::None => RunLocation::None,
        fleet::Placement::Connected { channel, kind } => RunLocation::Connected { channel: channel.raw(), kind },
        fleet::Placement::Adrift { channel, until } => RunLocation::Adrift {
            channel: match channel {
                Some(value) => Some(value.raw()),
                None => None,
            },
            until: wall_deadline(clock, until),
            remaining: match clock {
                Some(clock) => Some(until.saturating_since(clock.now)),
                None => None,
            },
        },
    };
    Ok(HostRun {
        task: match core.tasks.task(value.run.raw()) {
            Some(row) => TaskLink::Live(super::small::chip_with_core(core, limits, party, row)?),
            None => TaskLink::Stored { number: value.run.raw(), status: None },
        },
        attempt: value.attempt.raw(),
        kept: value.kept,
        listed: value.listed,
        state: value.state,
        location,
        grace_until: match value.until {
            Some(until) => wall_deadline(clock, until),
            None => None,
        },
    })
}
#[expect(
    clippy::too_many_lines,
    reason = "one deployment document gathers bounded child-owned host, account and capacity facts"
)]
pub(super) fn deployment(
    core: &Core,
    limits: &Limits,
    party: u64,
    clock: Option<DocumentClock>,
) -> Result<Deployment, Missing> {
    if core.people.role(party, people::DEPLOYMENT).is_none() {
        return Err(Missing::NotAllowed);
    }
    let source_hosts = core.fleet.host_views();
    let source_runs = core.fleet.attempt_views();
    let mut hosts = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    for value in source_hosts.iter().take(line_bound(limits)) {
        let mut runs = List::with_capacity(limits.documents.lines);
        let mut host_cut = false;
        for run in &source_runs {
            match run.placement {
                fleet::Placement::Connected { channel, .. }
                | fleet::Placement::Adrift { channel: Some(channel), .. }
                    if channel == value.channel =>
                {
                    if runs.room() > 0 {
                        runs.push(host_run(core, party, limits, clock, run)?).expect("bounded host runs");
                    } else {
                        host_cut = true;
                    }
                }
                fleet::Placement::None | fleet::Placement::Connected { .. } | fleet::Placement::Adrift { .. } => {}
            }
        }
        let name = match value.worker {
            Some(worker) => match core.fleet.worker_name(worker) {
                Some(name) => Some(skein_lib::bytes::copy_of(name)),
                None => None,
            },
            None => None,
        };
        let (programs, programs_cut) = super::programs::logins(core, &limits.documents, value.channel, None);
        hosts
            .push(Host {
                name,
                awaiting_logins: value.kind == fleet::HostKind::Worker && core.fleet.awaiting_logins(value.channel),
                rule: core.host_rule(fleet::PlacementConstraint { channel: value.channel, kind: value.kind }),
                programs,
                channel: value.channel.raw(),
                kind: value.kind,
                slots: value.slots,
                used: value.used,
                free: value.slots.checked_sub(value.used).expect("host occupants fit slots"),
                draining: value.draining,
                connected: value.connected,
                grace_until: match value.grace_until {
                    Some(at) => wall_deadline(clock, at),
                    None => None,
                },
                grace_remaining: match value.grace_until {
                    Some(at) => match clock {
                        Some(value) => Some(at.saturating_since(value.now)),
                        None => None,
                    },
                    None => None,
                },
                runs,
                cut: host_cut || programs_cut,
            })
            .expect("bounded hosts");
    }
    cut |= skein_lib::length::usize_of(source_hosts.len()) > line_bound(limits);
    cut |= core.fleet.host_history_lost() != 0 || core.fleet.hello_history_lost() != 0;
    let mut hello_refusals = List::with_capacity(limits.documents.lines);
    let refused = core.fleet.hello_refusals();
    for value in refused.iter().take(line_bound(limits)) {
        hello_refusals
            .push(HelloRefusalLine {
                channel: value.channel.raw(),
                at: value.at,
                slots: value.slots,
                stop_bound: value.stop_bound,
                why: value.why,
            })
            .expect("bounded hello refusals");
    }
    cut |= skein_lib::length::usize_of(refused.len()) > line_bound(limits);
    let mut attempts = List::with_capacity(limits.documents.lines);
    for value in source_runs.iter().take(line_bound(limits)) {
        attempts.push(host_run(core, party, limits, clock, value)?).expect("bounded attempt rows");
    }
    cut |= skein_lib::length::usize_of(source_runs.len()) > line_bound(limits);
    let rows = core.tasks.view_tasks();
    let mut accounts = List::with_capacity(limits.documents.lines);
    let source_accounts = core.accounts.account_views();
    let mut after = None;
    for _ in 0..source_accounts.len() {
        let Some((place, number)) =
            super::accounts::next_account(core, &source_accounts, accounts::Holder::Deployment, after)
        else {
            break;
        };
        after = Some((place, number));
        let account = core.accounts.account(number).expect("a selected deployment account");
        let mut selected = None;
        for fact in &source_accounts {
            if fact.account == number {
                selected = Some(*fact);
            }
        }
        let value = selected.expect("a selected account view");
        if accounts.room() == 0 {
            cut = true;
            continue;
        }
        let sources = super::accounts::source_lines(core, limits, account);
        let mut runs = List::with_capacity(limits.documents.lines);
        let mut waiting = List::with_capacity(limits.documents.lines);
        let mut account_cut = sources.len() < account.sources().len();
        for row in &rows {
            let row = core.tasks.task(row.number).expect("live view task");
            let mut running = false;
            if let Some(proof) = core.proofs.get(&row.number)
                && proof.terminal.is_none()
            {
                for binding in &proof.accounts {
                    if binding.account == value.account {
                        running = true;
                    }
                }
            }
            let needs = match row.executor {
                tasks::Executor::Agent { .. }
                    if (row.phase == tasks::Phase::Active(tasks::Active::Due)
                        || row.phase == tasks::Phase::Active(tasks::Active::Preparing)) =>
                {
                    if let Err(crate::RunWait::Account { account: Some(number), .. }) = core.run_choice(row.number) {
                        number == value.account
                    } else {
                        false
                    }
                }
                tasks::Executor::Agent { .. }
                | tasks::Executor::Person(_)
                | tasks::Executor::Procedure { .. }
                | tasks::Executor::Job { .. } => false,
            };
            if running {
                if runs.room() > 0 {
                    runs.push(super::small::chip_with_core(core, limits, party, row)?).expect("bounded account runs");
                } else {
                    account_cut = true;
                }
            }
            if needs {
                if waiting.room() > 0 {
                    waiting
                        .push(super::small::chip_with_core(core, limits, party, row)?)
                        .expect("bounded waiting agents");
                } else {
                    account_cut = true;
                }
            }
        }
        accounts
            .push(Account {
                list: super::accounts::list(core, value.account),
                number: value.account,
                place: account.place(),
                uses: super::accounts::uses(core, account),
                plan: account.plan(),
                provider: Some(account.provider()),
                provider_name: super::accounts::provider_name(core, account, limits.documents.text_bytes),
                manage_usage: super::accounts::manage_usage(core, account),
                display: super::small::text(account.display(), limits.documents.text_bytes),
                holder: account.holder(),
                sources,
                facts: super::accounts::fact_lines(
                    core,
                    number,
                    people::account_holder_reads(&core.people, party, people::AccountHolder::Deployment),
                ),
                generation: value.generation,
                state: account_state(value.state),
                usable: super::accounts::account_usable(core, account),
                expiring: value.expiring,
                attention: value.attention,
                spent_until: match value.spent_until {
                    Some(until) => Some(until.wall),
                    None => None,
                },
                runs,
                waiting,
                cut: account_cut,
            })
            .expect("bounded accounts");
    }
    let mut proposal_count = 0_u32;
    let mut projects = List::with_capacity(limits.documents.lines);
    for project in &core.projects {
        let mut live = 0_u32;
        let mut proposed = 0_u32;
        for row in &rows {
            if row.project == *project {
                live = live.checked_add(1).expect("bounded live tasks");
                let row = core.tasks.task(row.number).expect("live task view");
                if let Some(value) = &row.proposal {
                    match value.state {
                        tasks::ProposalState::Pending { .. } => {
                            proposed = proposed.checked_add(1).expect("one proposal per live task");
                        }
                        tasks::ProposalState::Accepted { .. }
                        | tasks::ProposalState::Rejected { .. }
                        | tasks::ProposalState::Withdrawn => {}
                    }
                }
            }
        }
        for (_, value) in core.tasks.person_proposals() {
            if value.project == *project {
                proposed = proposed.checked_add(1).expect("bounded person proposals");
            }
        }
        proposal_count = proposal_count.checked_add(proposed).expect("bounded proposal census");
        if projects.room() > 0 {
            projects
                .push(ProjectCapacity {
                    project: *project,
                    tasks: Capacity { used: live, limit: limits.tasks.project_tasks },
                    proposals: proposed,
                })
                .expect("bounded project capacities");
        } else {
            cut = true;
        }
    }
    let refused = core.people.capacity_refusals();
    let mut person_room_refusals = List::with_capacity(limits.documents.lines);
    for value in refused.iter().take(line_bound(limits)) {
        person_room_refusals.push(*value).expect("bounded person room refusals");
    }
    cut |= skein_lib::length::usize_of(refused.len()) > line_bound(limits);
    cut |= core.people.capacity_refusals_lost() != 0;
    let refused_tasks = core.tasks.capacity_refusals();
    let mut task_room_refusals = List::with_capacity(limits.documents.lines);
    for value in refused_tasks.as_slice().iter().rev().take(line_bound(limits)) {
        task_room_refusals.push(*value).expect("bounded task room refusals");
    }
    cut |= skein_lib::length::usize_of(refused_tasks.len()) > line_bound(limits);
    cut |= core.tasks.capacity_refusals_lost() != 0;
    let mut connectors = List::with_capacity(limits.documents.lines);
    for connector in core.connectors.iter().take(line_bound(limits)) {
        connectors.push(*connector).expect("bounded configured connector references");
    }
    let connectors_cut = core.connectors.len() > line_bound(limits);
    cut |= connectors_cut;
    let (providers, providers_cut) = super::catalogue::offers(core, &limits.documents)?;
    let (readiness, readiness_cut) = super::catalogue::rows(core, limits, party)?;
    Ok(Deployment {
        providers,
        providers_cut,
        readiness,
        readiness_cut,
        key_bytes: core.settings.key_bytes,
        store: StoreFill { bytes: core.fill, mark: limits.fill_mark },
        hosts,
        attempts,
        accounts,
        tasks: Capacity { used: u32::try_from(rows.len()).expect("bounded live rows"), limit: limits.tasks.tasks },
        workers: Capacity { used: core.fleet.workers(), limit: limits.fleet.workers },
        attempts_capacity: Capacity {
            used: core.fleet.attempts(),
            limit: limits
                .fleet
                .attempts
                .checked_add(limits.fleet.workers.checked_mul(limits.fleet.slots).ok_or(Missing::TooLarge)?)
                .ok_or(Missing::TooLarge)?,
        },
        waiting_attempts: core.fleet.waiting(),
        proposals: Capacity {
            used: proposal_count,
            limit: limits.tasks.tasks.checked_mul(2).ok_or(Missing::TooLarge)?,
        },
        projects,
        grace_bound: limits.fleet.grace,
        hello_refusals,
        hosts_forgotten: core.fleet.host_history_lost(),
        hello_refusals_forgotten: core.fleet.hello_history_lost(),
        person_room_refusals,
        person_room_refusals_forgotten: core.people.capacity_refusals_lost(),
        person_answer_retention: limits.people.request_retention,
        task_room_refusals,
        task_room_refusals_forgotten: core.tasks.capacity_refusals_lost(),
        connectors,
        connectors_cut,
        cut,
    })
}
/// Construct a document with the decision caller's injected clock. Other
/// subjects use their owners' wall facts; fleet grace is projected with this clock.
pub fn make_with_clock(
    core: &Core,
    limits: &Limits,
    party: u64,
    subject: &super::Subject,
    window: &Window,
    clock: DocumentClock,
) -> Result<super::Document, Missing> {
    match subject {
        super::Subject::Accounts { party: owner } => {
            Ok(super::Document::Accounts(Box::new(super::accounts::make(core, limits, party, *owner, window)?)))
        }
        super::Subject::Deployment => {
            core.people.identity(party).ok_or(Missing::Unknown)?;
            if super::document_bound(&limits.documents).is_none() || window.rows > limits.documents.rows {
                return Err(Missing::TooLarge);
            }
            Ok(super::Document::Deployment(Box::new(deployment(core, limits, party, Some(clock))?)))
        }
        super::Subject::Settings
        | super::Subject::Frame { .. }
        | super::Subject::Task { .. }
        | super::Subject::Tree { .. }
        | super::Subject::Chips { .. }
        | super::Subject::Inbox { .. }
        | super::Subject::Query { .. }
        | super::Subject::Run { .. }
        | super::Subject::Proposal { .. }
        | super::Subject::Escalation { .. }
        | super::Subject::Question { .. }
        | super::Subject::PersonTask { .. }
        | super::Subject::Result { .. }
        | super::Subject::Project { .. }
        | super::Subject::Note { .. }
        | super::Subject::Application { .. } => super::make(core, limits, party, subject, window, clock.wall),
    }
}
