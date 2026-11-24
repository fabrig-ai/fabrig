//! Whole live document facts (protocol/client/wire.md, 1, and protocol/client/README.md, 5).
use super::{chips, owner, pieces, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Writer};
pub(super) fn host_kind(value: fleet::HostKind) -> w::HostKind {
    mapped::host_kind_to_record(&value)
}
#[must_use]
pub fn capacity(value: d::Capacity) -> w::Capacity {
    mapped::capacity_to_record(&value)
}
pub fn run(value: &d::HostRun, limits: &w::Limits) -> Result<w::HostRun, w::Problem> {
    mapped::host_run_to_record(value, &(&[], limits))
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
        rule: mapped::host_rule_to_record(&value.rule),
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
        state: mapped::account_state_to_record(&value.state),
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
    mapped::person_capacity_refusal_to_record(value)
}
#[must_use]
pub fn task_refusal(value: &tasks::CapacityRefusal) -> w::TaskCapacityRefusal {
    mapped::task_capacity_refusal_to_record(value)
}
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
        projects.push(mapped::project_capacity_to_record(row)).expect("supplied rows fit");
    }
    let mut hello_refusals = List::with_capacity(value.hello_refusals.len());
    for row in &value.hello_refusals {
        hello_refusals.push(mapped::hello_refusal_line_to_record(row)).expect("supplied rows fit");
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
        worker_settings.push(mapped::worker_state_to_record(row)).expect("worker row count");
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
        store: mapped::store_fill_to_record(&value.store),
        census: match &value.census {
            Some(value) => Some(mapped::store_census_to_record(value)),
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
