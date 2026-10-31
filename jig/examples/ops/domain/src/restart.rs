//! Copy the core's script. A connector reports when it has completed
//! reads or settlement; the root never advances it by guessing (domain/root.md, 8).
use crate::domain::{Work, held};
use crate::{Domain, Limits, Output, Range, Record, Store, Write};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_ops_domain_connectors::Connector;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::{Decision, Env, Queue};

/// Translate the core's phases without selecting their successors.
pub(crate) fn route(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    action: core::start::Action,
    work: &mut Queue<Work>,
) {
    use core::start::{Action, Phase};
    match action {
        Action::Refused(_) => domain.start.state = crate::domain::StartState::Stopped,
        Action::Idle => {}
        Action::Commit(continuation) => {
            domain.start.waiting = true;
            held(decision, Output::ToChild(crate::Released::Start(continuation)));
        }
        Action::Step(phase) => {
            #[cfg(test)]
            domain.start_phases.push(phase).expect("the start's finite phase list");
            match phase {
                Phase::LoadCore => held(decision, Output::Load { phase, range: Range::Core }),
                Phase::LoadInbox => {
                    let census = domain.core.census(&env.limits.core);
                    held(decision, Output::Load { phase, range: Range::Census(census) });
                }
                Phase::RestoreConnector { connector } => {
                    let range = match jig_ops_domain_connectors::connector(connector) {
                        Some(Connector::Infrastructure) => Range::Infrastructure,
                        Some(Connector::Observability) => Range::Observability,
                        None => {
                            domain.start.state = crate::domain::StartState::Stopped;
                            return;
                        }
                    };
                    held(decision, Output::Load { phase, range });
                }
                Phase::Link | Phase::Wires => work.push(Work::StartDone(phase)),
                Phase::Sweep => {
                    if domain.start.sweep_loaded {
                        sweep(domain, env, decision, work);
                    } else {
                        domain.start.sweep_loaded = true;
                        held(decision, Output::Load { phase, range: Range::People });
                    }
                }
                Phase::Switch => held(decision, Output::Switch),
                Phase::AdoptRuns => {
                    work.push(Work::StartDone(phase));
                }
                Phase::ReadAfresh { connector } => match jig_ops_domain_connectors::connector(connector) {
                    Some(Connector::Infrastructure) => {
                        work.push(Work::Infrastructure(infrastructure::Event::ReadAfresh));
                    }
                    Some(Connector::Observability) => work.push(Work::Observability(observability::Event::ReadAfresh)),
                    None => domain.start.state = crate::domain::StartState::Stopped,
                },
                Phase::SettleOutbox { connector } => match jig_ops_domain_connectors::connector(connector) {
                    Some(Connector::Infrastructure) => work.push(Work::Infrastructure(infrastructure::Event::Restart)),
                    Some(Connector::Observability) => work.push(Work::Observability(observability::Event::Restart)),
                    None => domain.start.state = crate::domain::StartState::Stopped,
                },
                Phase::Open => {
                    work.push(Work::Infrastructure(infrastructure::Event::SendCloses));
                    work.push(Work::StartDone(phase));
                }
            }
        }
    }
}

pub(crate) fn done(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    phase: core::start::Phase,
    work: &mut Queue<Work>,
) {
    let mut out = Queue::uncapped();
    let action =
        domain.core.start_done(&Env { now: env.now, wall: env.wall, limits: env.limits.core }, phase, &mut out);
    if phase == core::start::Phase::Wires {
        match &action {
            core::start::Action::Commit(_) => crate::domain::write(domain, decision, Write::Wires),
            core::start::Action::Refused(_) => {}
            core::start::Action::Step(_) | core::start::Action::Idle => {
                unreachable!("Wires needs a durable continuation")
            }
        }
    }
    if domain.core.start_opened()
        && let Some(bytes) = domain.start.fill.take()
    {
        work.push(Work::Core(core::Event::Fill { bytes }));
    }
    crate::route::core_out(domain, env, decision, out, work);
    work.push(Work::Start(action));
}

/// A page may arrive while an earlier expiry erase waits for its commit.
fn sweep(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision<Write, Output>, work: &mut Queue<Work>) {
    for _ in 0..domain.start.sweep.len() {
        if domain.start.waiting {
            break;
        }
        let input = domain.start.sweep.pop().expect("queued sweep page rows");
        let mut out = Queue::uncapped();
        let action =
            domain.core.start_sweep(&Env { now: env.now, wall: env.wall, limits: env.limits.core }, input, &mut out);
        crate::route::core_out(domain, env, decision, out, work);
        match action {
            core::start::Action::Idle => {}
            action @ (core::start::Action::Step(_)
            | core::start::Action::Commit(_)
            | core::start::Action::Refused(_)) => {
                domain.start.waiting = match action {
                    core::start::Action::Commit(_) => true,
                    core::start::Action::Step(_) | core::start::Action::Refused(_) => false,
                    core::start::Action::Idle => unreachable!("Idle continues the sweep"),
                };
                work.push(Work::Start(action));
                break;
            }
        }
    }
}

pub(crate) fn store(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Output>,
    store: Store,
    work: &mut Queue<Work>,
) {
    match store {
        Store::Switched => {
            if domain.core.stopping() {
                return;
            }
            work.push(Work::StartDone(core::start::Phase::Switch));
        }
        Store::Committed { .. } | Store::Failed { .. } => unreachable!("store terminals belong to the journal"),
        Store::Loaded(event) => work.push(Work::Core(event)),
        Store::Restored(phase) => {
            if phase == core::start::Phase::Sweep {
                domain.start.sweep.push(core::start::SweepInput::Finished);
                sweep(domain, env, decision, work);
            } else {
                work.push(Work::StartDone(phase));
            }
        }
        Store::Transcript { task, rows, done } => {
            // A message can invalidate preparation while this store read is
            // in flight. Its old answer belongs to no current preparation.
            if !domain.core.transcripts.contains_key(&task) {
                return;
            }
            for (row, text) in rows {
                if !domain.core.append_transcript(task, row, text) {
                    work.push(Work::Core(core::Event::PreparationFailed { task }));
                    return;
                }
            }
            if done {
                let event = if domain.core.transcript_readable(task) {
                    core::Event::StartBrief { task }
                } else {
                    core::Event::TranscriptUnread { task }
                };
                work.push(Work::Core(event));
            }
        }
        Store::Restore(record) => {
            if domain.start.state == crate::domain::StartState::Stopped || domain.core.start_failure().is_some() {
                return;
            }
            let valid = match record {
                Record::Core(row) => {
                    match domain.core.restore(&Env { now: env.now, wall: env.wall, limits: env.limits.core }, row) {
                        core::Restore::Deployment { commits } => {
                            domain.restored_commits = Some(commits);
                            true
                        }
                        core::Restore::Kept => true,
                        core::Restore::Refused => false,
                        core::Restore::Swept => {
                            domain.start.sweep.push(core::start::SweepInput::Next);
                            sweep(domain, env, decision, work);
                            true
                        }
                    }
                }
                Record::Infrastructure(record) => {
                    let valid = restore_roles(domain, env, &record);
                    if valid {
                        work.push(Work::Infrastructure(infrastructure::Event::Restore { record }));
                    }
                    valid
                }
                Record::Observability(record) => {
                    work.push(Work::Observability(observability::Event::Restore { record }));
                    true
                }
            };
            if !valid {
                assert!(domain.core.start_failure().is_some(), "a refused row records its start failure");
            }
        }
    }
}

/// Preserve connector-owned project adoption before the core admits live claims.
fn restore_roles(domain: &mut Domain, env: &Env<Limits>, record: &infrastructure::Record) -> bool {
    let mut valid = true;
    match record {
        infrastructure::Record::Rely { project, resources, .. } => {
            for named in resources {
                valid &= domain.core.restore_resource_role(
                    &Env { now: env.now, wall: env.wall, limits: env.limits.core },
                    *project,
                    tasks::Name {
                        connector: jig_ops_domain_connectors::number(Connector::Infrastructure),
                        path: named.resource.segments(),
                    },
                    crate::translate::resource_role(named.role),
                );
            }
        }
        infrastructure::Record::Closing { .. }
        | infrastructure::Record::Drift(_)
        | infrastructure::Record::Procedure(_)
        | infrastructure::Record::Proposal { .. }
        | infrastructure::Record::Outbox(_)
        | infrastructure::Record::EnvironmentEnd { .. }
        | infrastructure::Record::ApplicationAnswer { .. }
        | infrastructure::Record::Made { .. } => {}
    }
    valid
}
