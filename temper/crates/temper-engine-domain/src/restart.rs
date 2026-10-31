//! Store adapters for the core-selected restart script (jig's domain/root.md, 8).
use crate::archive::{
    Waiting, goal_failed, goal_loaded, input_failed, input_loaded, note_failed, note_loaded, transcript_failed,
    transcript_loaded, transcript_unread, waiting,
};
use crate::boundary::{Request, Work};
use crate::domain::{Domain, close, emit, save};
use crate::limits::{Limits, core_limits, environment_core};
use crate::route::{route_core_requests, route_into};
use crate::{Decision, Delivery, Key, Range, Record, loads};
use alloc::boxed::Box;
use jig_domain_core::start::{Action, Phase, SweepInput};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Token};
use temper_engine_domain_forge as forge;

/// Store terminals always open a decision, including the cold restore route.
pub(crate) fn load_decision(domain: &mut Domain, limits: &Limits) -> Decision {
    domain.settled = None;
    Decision::decide(&mut domain.journal, &limits.journal)
}

pub(crate) fn accept_load(domain: &mut Domain, env: &Env<Limits>, decision: Option<Decision>) {
    if let Some(decision) = decision {
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.core.counters,
            &env.limits.journal,
            decision,
            false,
        )
        .expect("read-only store terminal was admitted before changing its owner");
    }
}

pub(crate) fn request_load(
    domain: &mut Domain,
    waiter: Token,
    range: Range,
    after: Option<Key>,
    out: &mut Queue<Request>,
) {
    let mut load_out = Queue::with_capacity(1);
    let most = match &range {
        Range::Core(
            crate::CoreRange::Deployment
            | crate::CoreRange::RequesterWord { .. }
            | crate::CoreRange::TaskResult { .. }
            | crate::CoreRange::EndedRead { .. }
            | crate::CoreRange::EscalationDecision { .. }
            | crate::CoreRange::ProposalDecision { .. }
            | crate::CoreRange::TaskProposal { .. }
            | crate::CoreRange::Notes(jig_domain_notes::Range::Entry { .. }),
        ) => 1,
        Range::Core(
            crate::CoreRange::Calls
            | crate::CoreRange::Tasks
            | crate::CoreRange::Settings
            | crate::CoreRange::Accounts
            | crate::CoreRange::People
            | crate::CoreRange::Sweep
            | crate::CoreRange::RunProofs
            | crate::CoreRange::Turns { .. }
            | crate::CoreRange::TaskTranscript { .. },
        )
        | Range::Census(_)
        | Range::Forge => domain.limits.loads.rows,
        Range::Core(crate::CoreRange::Notes(jig_domain_notes::Range::Lines { .. })) => {
            domain.limits.loads.rows.min(domain.limits.notes.load_rows)
        }
    };
    if loads::begin(&mut domain.loads, waiter, range.clone(), after, most, &mut load_out).is_none() {
        match range {
            Range::Core(crate::CoreRange::RequesterWord { .. }) => {
                goal_failed(domain, waiter);
                return;
            }
            Range::Core(crate::CoreRange::TaskTranscript { .. }) => {
                transcript_failed(domain, waiter);
                return;
            }
            Range::Core(
                crate::CoreRange::ProposalDecision { .. }
                | crate::CoreRange::TaskProposal { .. }
                | crate::CoreRange::EscalationDecision { .. }
                | crate::CoreRange::Calls
                | crate::CoreRange::Deployment
                | crate::CoreRange::Tasks
                | crate::CoreRange::RunProofs
                | crate::CoreRange::Settings
                | crate::CoreRange::Accounts
                | crate::CoreRange::People
                | crate::CoreRange::Sweep
                | crate::CoreRange::TaskResult { .. }
                | crate::CoreRange::EndedRead { .. }
                | crate::CoreRange::Turns { .. },
            )
            | Range::Census(_)
            | Range::Forge => {}
            Range::Core(crate::CoreRange::Notes(_)) => {
                note_failed(domain, waiter);
                return;
            }
        }
        match range {
            Range::Core(crate::CoreRange::ProposalDecision { .. } | crate::CoreRange::TaskProposal { .. }) => {
                domain.work.push(Work::ProposalFailed { waiter });
                return;
            }
            Range::Core(crate::CoreRange::EscalationDecision { .. }) => {
                domain.work.push(Work::EscalationFailed { waiter });
                return;
            }
            Range::Core(
                crate::CoreRange::Calls
                | crate::CoreRange::Deployment
                | crate::CoreRange::Tasks
                | crate::CoreRange::Settings
                | crate::CoreRange::Accounts
                | crate::CoreRange::People
                | crate::CoreRange::Sweep
                | crate::CoreRange::RunProofs
                | crate::CoreRange::Turns { .. }
                | crate::CoreRange::TaskTranscript { .. }
                | crate::CoreRange::RequesterWord { .. }
                | crate::CoreRange::TaskResult { .. }
                | crate::CoreRange::EndedRead { .. },
            )
            | Range::Census(_)
            | Range::Forge => {
                unreachable!("startup/result load room reserved")
            }
            Range::Core(crate::CoreRange::Notes(_)) => unreachable!("notes load failure handled above"),
        }
    }
    match load_out.pop().expect("load issued") {
        loads::Request::Load { owner, range, after, most, bytes } => {
            out.push(Request::Store(crate::StoreRequest::Load { owner, range, after, most, bytes }));
        }
        loads::Request::Loaded { .. } | loads::Request::Unloaded { .. } => unreachable!("begin only issues IO"),
    }
}

pub(crate) fn load_outputs(
    domain: &mut Domain,
    env: &Env<Limits>,
    load_out: &mut Queue<loads::Request>,
    mut texts: Option<crate::TranscriptTexts>,
    decision: &mut Option<Decision>,
    out: &mut Queue<Request>,
) {
    for _ in 0..load_out.len() {
        match load_out.pop().expect("load terminal count") {
            loads::Request::Loaded { waiter, rows, next, cut } => {
                if waiter == Token::new(u64::MAX) {
                    if cut.is_some() {
                        stop_load(domain, loads::Failure::Bytes);
                        out.push(Request::Stop);
                    } else {
                        startup_page(domain, env, rows, next, decision.take().expect("admitted restore page"), out);
                    }
                    return;
                }
                match waiting(domain, waiter) {
                    Some(Waiting::Notes) => note_loaded(domain, waiter, rows, next, cut),
                    Some(Waiting::GoalWords) if cut.is_none() && next.is_none() => {
                        goal_loaded(domain, waiter, rows, out);
                    }
                    Some(Waiting::GoalWords) => goal_failed(domain, waiter),
                    Some(Waiting::Input) if cut.is_none() => input_loaded(domain, waiter, rows, next, out),
                    Some(Waiting::Input) => input_failed(domain, waiter),
                    Some(Waiting::Transcript) => transcript_loaded(domain, waiter, rows, texts.take(), next, cut, out),
                    Some(Waiting::Ended) if cut.is_none() && next.is_none() => {
                        domain.work.push(Work::EndedLoaded { waiter, rows });
                    }
                    Some(Waiting::Ended) => domain.work.push(Work::EndedFailed { waiter }),
                    Some(Waiting::TaskProposal) => domain.work.push(Work::TaskProposalLoaded {
                        waiter,
                        rows: if cut.is_none() && next.is_none() { Some(rows) } else { None },
                    }),
                    Some(Waiting::Proposal) if cut.is_none() && next.is_none() => {
                        domain.work.push(Work::ProposalLoaded { waiter, rows });
                    }
                    Some(Waiting::Proposal) => domain.work.push(Work::ProposalFailed { waiter }),
                    Some(Waiting::Escalation) if cut.is_none() => {
                        domain.work.push(Work::EscalationLoaded { waiter, rows });
                    }
                    Some(Waiting::Escalation) => domain.work.push(Work::EscalationFailed { waiter }),
                    None => {}
                }
            }
            loads::Request::Unloaded { waiter, failure } => {
                if waiter == Token::new(u64::MAX) {
                    stop_load(domain, failure);
                    out.push(Request::Stop);
                    return;
                }
                match waiting(domain, waiter) {
                    Some(Waiting::Notes) => note_failed(domain, waiter),
                    Some(Waiting::Input) => input_failed(domain, waiter),
                    Some(Waiting::GoalWords) => goal_failed(domain, waiter),
                    Some(Waiting::Transcript) => transcript_unread(domain, waiter),
                    Some(Waiting::Ended) => domain.work.push(Work::EndedFailed { waiter }),
                    Some(Waiting::Proposal) => domain.work.push(Work::ProposalFailed { waiter }),
                    Some(Waiting::TaskProposal) => domain.work.push(Work::TaskProposalLoaded { waiter, rows: None }),
                    Some(Waiting::Escalation) => domain.work.push(Work::EscalationFailed { waiter }),
                    None => {}
                }
            }
            loads::Request::Load { .. } => unreachable!("terminal methods never issue IO"),
        }
    }
}

pub(crate) fn startup_page(
    domain: &mut Domain,
    env: &Env<Limits>,
    rows: Box<[Record]>,
    next: Option<Key>,
    mut decision: Decision,
    out: &mut Queue<Request>,
) {
    let Some(range) = domain.startup_range.clone() else { return };
    if let Range::Census(_) = range {
        census_page(domain, env, rows, next, decision, out);
        return;
    }
    if range == Range::Core(crate::CoreRange::Sweep) {
        for row in rows {
            domain.sweep_rows.push(row);
        }
        domain.sweep_page = true;
        domain.sweep_next = next;
        sweep(domain, env, &mut decision);
        route_into(domain, env, &mut decision);
        close(domain, env, decision, out);
        return;
    }
    for row in rows {
        restore_page_row(domain, env, row);
        route_into(domain, env, &mut decision);
        if domain.start_stop().is_some() || domain.core.start_failure().is_some() {
            if domain.start_stop().is_none() {
                domain.stop_core();
            }
            return;
        }
    }
    if range == Range::Core(crate::CoreRange::Deployment) {
        domain.core_header_loaded = true;
        close(domain, env, decision, out);
        let following = match next {
            Some(after) => (range, Some(after)),
            None => (Range::Core(crate::CoreRange::Settings), None),
        };
        domain.startup_range = Some(following.0.clone());
        request_load(domain, Token::new(u64::MAX), following.0, following.1, out);
        return;
    }
    if let Some(after) = next {
        emit(&mut decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: Some(after) });
        close(domain, env, decision, out);
        return;
    }
    let following = match range {
        Range::Core(crate::CoreRange::Deployment) => Some(Range::Core(crate::CoreRange::Settings)),
        Range::Core(crate::CoreRange::Settings) => Some(Range::Core(crate::CoreRange::Accounts)),
        Range::Core(crate::CoreRange::Accounts) => Some(Range::Core(crate::CoreRange::People)),
        Range::Core(crate::CoreRange::People) => Some(Range::Core(crate::CoreRange::Tasks)),
        Range::Core(crate::CoreRange::Tasks) => Some(Range::Core(crate::CoreRange::RunProofs)),
        Range::Core(crate::CoreRange::RunProofs) => Some(Range::Core(crate::CoreRange::Calls)),
        Range::Forge | Range::Census(_) | Range::Core(crate::CoreRange::Calls) => None,
        Range::Core(crate::CoreRange::Sweep) => unreachable!("sweep page handled above"),
        Range::Core(
            crate::CoreRange::EscalationDecision { .. }
            | crate::CoreRange::ProposalDecision { .. }
            | crate::CoreRange::TaskProposal { .. }
            | crate::CoreRange::Turns { .. }
            | crate::CoreRange::TaskTranscript { .. }
            | crate::CoreRange::RequesterWord { .. }
            | crate::CoreRange::TaskResult { .. }
            | crate::CoreRange::EndedRead { .. }
            | crate::CoreRange::Notes(_),
        ) => unreachable!("startup range"),
    };
    if let Some(range) = following {
        domain.startup_range = Some(range.clone());
        emit(&mut decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: None });
    } else {
        domain.startup_range = None;
        let phase = if range == Range::Forge {
            Phase::RestoreConnector { connector: domain.config.forge_connector }
        } else {
            Phase::LoadCore
        };
        domain.work.push(Work::StartDone(phase));
    }
    route_into(domain, env, &mut decision);
    close(domain, env, decision, out);
}

/// Translate only the phase selected by the core (temper/docs/design/domain/root.md, 9.1).
pub(crate) fn start_action(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, action: Action) {
    if domain.core.stopping() || domain.start_stop().is_some() {
        return;
    }
    match action {
        Action::Idle => {}
        Action::Refused(failure) => {
            domain.stop_start(crate::StartStop::Core { phase: domain.start_phase(), failure });
        }
        Action::Commit(continuation) => emit(decision, &env.limits, Delivery::Start(continuation)),
        Action::Step(phase) => {
            #[cfg(test)]
            domain.start_phases.push(phase);
            match phase {
                Phase::LoadCore => start_load(domain, env, decision, Range::Core(crate::CoreRange::Deployment)),
                Phase::LoadInbox => {
                    let census = domain.core.census(&core_limits(&env.limits));
                    start_load(domain, env, decision, Range::Census(census));
                }
                Phase::RestoreConnector { connector } => {
                    assert_eq!(connector, domain.config.forge_connector, "configured forge connector");
                    start_load(domain, env, decision, Range::Forge);
                }
                Phase::Link => domain.work.push(Work::ForgeRestored),
                Phase::Sweep => sweep(domain, env, decision),
                Phase::Switch => {
                    if domain.start_pending {
                        assert!(crate::domain::now(domain, Request::Store(crate::StoreRequest::Switch)), "switch door");
                    } else {
                        domain.work.push(Work::StartDone(phase));
                    }
                }
                Phase::Wires => {
                    save(decision, &env.limits, crate::Write::Wires);
                    domain.work.push(Work::StartDone(phase));
                }
                Phase::AdoptRuns => {
                    if let Some(bytes) = domain.startup_fill.take() {
                        domain.work.push(Work::Core(jig_domain_core::Event::Fill { bytes }));
                    }
                    for _ in 0..domain.core.adopted.len() {
                        domain.work.push(Work::Fleet(domain.core.adopted.pop().expect("restored claims")));
                    }
                    domain.work.push(Work::Fleet(jig_domain_fleet::Event::Loaded));
                    domain.work.push(Work::StartDone(phase));
                }
                Phase::ReadAfresh { connector } => {
                    assert_eq!(connector, domain.config.forge_connector, "configured forge connector");
                    domain.work.push(Work::Forge(forge::Event::ReadAfresh));
                }
                Phase::SettleOutbox { connector } => {
                    assert_eq!(connector, domain.config.forge_connector, "configured forge connector");
                    domain.work.push(Work::Forge(forge::Event::SettleOutbox));
                }
                Phase::Open => {
                    for index in 0..domain.core.projects.len() {
                        let project = *domain.core.projects.get(index).expect("bounded core project");
                        let period = domain.core.period(project);
                        if let Some(row) =
                            domain.core.tasks.funding(jig_domain_tasks::Funder::Period { project, period })
                            && let Some(times) = row.period
                        {
                            crate::route::calendar::route(
                                domain,
                                env,
                                temper_engine_domain_calendar::Event::Restored {
                                    project,
                                    period,
                                    began: times.began,
                                    next: times.next,
                                },
                                None,
                            );
                        }
                    }
                    domain.work.push(Work::Forge(forge::Event::SendDemoActs));
                    domain.work.push(Work::StartDone(phase));
                }
            }
        }
    }
}

fn start_load(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, range: Range) {
    domain.startup_range = Some(range.clone());
    emit(decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: None });
}

fn sweep(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    if !domain.sweep_page {
        start_load(domain, env, decision, Range::Core(crate::CoreRange::Sweep));
        return;
    }
    while let Some(row) = domain.sweep_rows.pop() {
        let Record::Core(jig_domain_core::Record::People(row)) = row else { unreachable!("sweep loads people's rows") };
        let mut output = Queue::uncapped();
        let action = domain.core.start_sweep(&environment_core(env), SweepInput::Record(row), &mut output);
        route_core_requests(domain, env, decision, jig_domain_core::Requests::Out(output));
        match action {
            Action::Idle => {}
            action @ (Action::Commit(_) | Action::Refused(_) | Action::Step(_)) => {
                start_action(domain, env, decision, action);
                return;
            }
        }
    }
    domain.sweep_page = false;
    if let Some(after) = domain.sweep_next.take() {
        emit(
            decision,
            &env.limits,
            Delivery::Load {
                waiter: Token::new(u64::MAX),
                range: Range::Core(crate::CoreRange::Sweep),
                after: Some(after),
            },
        );
    } else {
        domain.startup_range = None;
        let mut output = Queue::uncapped();
        let action = domain.core.start_sweep(&environment_core(env), SweepInput::Finished, &mut output);
        route_core_requests(domain, env, decision, jig_domain_core::Requests::Out(output));
        domain.work.push(Work::Start(action));
    }
}

pub(crate) fn connector_start_done(domain: &mut Domain, stage: jig_domain_core::connector::RestartStage) {
    let connector = domain.config.forge_connector;
    let phase = match stage {
        jig_domain_core::connector::RestartStage::Restored => Phase::Link,
        jig_domain_core::connector::RestartStage::ReadAfresh => Phase::ReadAfresh { connector },
        jig_domain_core::connector::RestartStage::Settled => Phase::SettleOutbox { connector },
    };
    domain.work.push(Work::StartDone(phase));
}

pub(crate) fn connector_restarting(domain: &Domain) -> bool {
    match domain.start_phase() {
        Some(Phase::ReadAfresh { .. } | Phase::SettleOutbox { .. }) => true,
        Some(
            Phase::LoadCore
            | Phase::LoadInbox
            | Phase::RestoreConnector { .. }
            | Phase::Link
            | Phase::Sweep
            | Phase::Switch
            | Phase::Wires
            | Phase::AdoptRuns
            | Phase::Open,
        )
        | None => false,
    }
}

fn stop_load(domain: &mut Domain, failure: loads::Failure) {
    let range = loads::kind(domain.startup_range.as_ref().expect("startup range in flight"));
    domain.stop_start(crate::StartStop::Load(loads::RestoreFailure { range, failure }));
}

/// Rebuild child state and transient proof correlations before linking
/// (temper/docs/design/domain/root.md, 9.1). Archive history stays outside the live map.
pub(crate) fn restore_page_row(domain: &mut Domain, env: &Env<Limits>, row: Record) {
    match row {
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(row))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::Settings(row), &core_limits(&env.limits))
                != jig_domain_core::Restored::Live
            {
                domain.stop_core();
            }
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::Call(record), &core_limits(&env.limits))
                != jig_domain_core::Restored::Live
            {
                domain.stop_core();
            }
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(deployment))) => {
            if !domain.forge.bind_deployment(deployment.id, &env.limits.forge) {
                domain.stop_start(crate::StartStop::Binding);
                return;
            }
            match domain
                .core
                .restore_core(jig_domain_core::CoreRecord::Deployment(deployment), &core_limits(&env.limits))
            {
                jig_domain_core::Restored::Deployment { commits } => {
                    domain.core_header_loaded = true;
                    domain.restored_commits = Some(commits);
                }
                jig_domain_core::Restored::Live
                | jig_domain_core::Restored::Archive
                | jig_domain_core::Restored::Rejected => {
                    unreachable!("deployment restore result")
                }
            }
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Projection(row))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::Projection(row), &core_limits(&env.limits))
                == jig_domain_core::Restored::Rejected
            {
                domain.stop_core();
            }
        }
        Record::Core(jig_domain_core::Record::People(people::Stored::Policy { project, name, value })) => {
            if !domain.core.restore_policy(&core_limits(&domain.limits), project, name, value) {
                domain.stop_core();
            }
        }
        Record::Core(jig_domain_core::Record::People(record)) => {
            domain.work.push(Work::People(people::Event::Restore { record }));
        }
        Record::Core(jig_domain_core::Record::Accounts(record)) => {
            domain
                .work
                .push(Work::Core(jig_domain_core::Event::Account(jig_domain_accounts::Event::Restore { record })));
        }
        Record::Forge { row } => {
            domain.work.push(Work::Forge(forge::Event::Restore { record: *row }));
        }
        Record::Core(jig_domain_core::Record::Tasks(record)) => {
            if !domain.core.restore_task_row(&record) {
                domain.stop_core();
                return;
            }
            domain.work.push(Work::Tasks(tasks::Event::Restore { record }));
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::RunProof(proof), &core_limits(&env.limits))
                != jig_domain_core::Restored::Live
            {
                domain.stop_core();
            }
        }
        Record::Core(jig_domain_core::Record::Notes(_)) => unreachable!("notes are loaded on demand, outside startup"),
        Record::Core(jig_domain_core::Record::Core(
            jig_domain_core::CoreRecord::Turn(_)
            | jig_domain_core::CoreRecord::Terminal(_)
            | jig_domain_core::CoreRecord::EscalationDecision(_)
            | jig_domain_core::CoreRecord::ProposalDecision(_),
        )) => {
            unreachable!("startup excludes archive families")
        }
    }
}

/// Rebuild the derived inbox from the census's ended owners, after people's
/// read positions restored (jig's `protocol/store.md`, 7.2).
fn census_page(
    domain: &mut Domain,
    env: &Env<Limits>,
    rows: Box<[Record]>,
    next: Option<Key>,
    mut decision: Decision,
    out: &mut Queue<Request>,
) {
    for row in rows {
        let restored = match row {
            Record::Core(row) => domain.core.restore_inbox_row(&core_limits(&env.limits), &row),
            Record::Forge { .. } => false,
        };
        if !restored {
            domain.stop_core();
            out.push(Request::Stop);
            return;
        }
    }
    let range = domain.startup_range.clone().expect("census range in flight");
    if let Some(after) = next {
        emit(&mut decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: Some(after) });
        close(domain, env, decision, out);
        return;
    }
    domain.startup_range = None;
    domain.work.push(Work::StartDone(Phase::LoadInbox));
    route_into(domain, env, &mut decision);
    close(domain, env, decision, out);
}
