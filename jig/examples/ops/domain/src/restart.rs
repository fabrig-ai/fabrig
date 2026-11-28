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
                    let census = domain.engine.core_mut().census(&env.limits.core);
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
                Phase::Switch => {
                    if domain.rehearsing {
                        domain.rehearsed = true;
                    } else {
                        held(decision, Output::Switch);
                    }
                }
                Phase::AdoptRuns => {
                    work.push(Work::StartDone(phase));
                }
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
    let action = domain.engine.core_mut().start_done(&env.with(env.limits.core), phase, &mut out);
    if phase == core::start::Phase::Wires {
        match &action {
            core::start::Action::Commit(_) => crate::domain::write(domain, decision, Write::Wires),
            core::start::Action::Refused(_) => {}
            core::start::Action::Step(_) | core::start::Action::Idle => {
                unreachable!("Wires needs a durable continuation")
            }
        }
    }
    if domain.engine.core().start_opened()
        && let Some(bytes) = domain.start.fill.take()
    {
        work.push(Work::Store(core::engine::store::In::Fill { bytes }));
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
        let action = domain.engine.core_mut().start_sweep(&env.with(env.limits.core), input, &mut out);
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
            if domain.engine.core().stopping() {
                return;
            }
            work.push(Work::StartDone(core::start::Phase::Switch));
        }
        Store::Committed { .. } | Store::Failed { .. } => unreachable!("store terminals belong to the journal"),
        Store::Loaded(event) => work.push(Work::Core(event)),
        Store::Engine(answer) => work.push(Work::Store(answer)),
        Store::Restored(phase) => {
            if phase == core::start::Phase::Sweep {
                domain.start.sweep.push(core::start::SweepInput::Finished);
                sweep(domain, env, decision, work);
            } else {
                work.push(Work::StartDone(phase));
            }
        }
        Store::Restore(record) => {
            if domain.start.state == crate::domain::StartState::Stopped
                || domain.engine.core().start_failure().is_some()
            {
                return;
            }
            let valid = match record {
                Record::Core(row) => match domain.engine.core_mut().restore(&env.with(env.limits.core), row) {
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
                },
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
                assert!(domain.engine.core().start_failure().is_some(), "a refused row records its start failure");
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
                valid &= domain.engine.core_mut().restore_resource_role(
                    &env.with(env.limits.core),
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
