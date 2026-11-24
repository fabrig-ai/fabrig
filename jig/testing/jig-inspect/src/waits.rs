//! Map every lifecycle variant, and every reason an idle agent has to wait.
use jig_domain_core::Core;
use jig_domain_tasks::{
    self as task, Active, Domain, Escalation, EscalationHolder, Executor, Hold, HoldSource, Party, Phase,
    ProposalHolder, ProposalKind, ProposalState, Stage, SubscriptionKind, TaskRecord,
};
use skein_lib::{Deadlines, Time};
use skein_world::inspect::{Inspection, Wait, Wake};

pub(crate) fn alarm(alarms: &Deadlines<u64>, number: u64) -> Option<Time> {
    alarms.iter().find_map(|(key, due)| (key == number).then_some(due))
}

pub(crate) fn task_wake(domain: &Domain, number: u64) -> Wake {
    // Ended rows leave the live arena. A remaining reference to one is not a live wake.
    if domain.task(number).is_none_or(|row| matches!(row.phase, Phase::Ended(_))) {
        Wake::Ended(format!("task {number}"))
    } else {
        Wake::Waiter(format!("task {number}"))
    }
}

fn party(domain: &Domain, party: Party) -> Wake {
    match party {
        Party::Task(number) => task_wake(domain, number),
        Party::Person(number) => Wake::Party(format!("person {number}")),
        Party::Deployment { project } => Wake::Party(format!("deployment of project {project}")),
    }
}

fn emit(domain: &Domain, core: Option<&Core>, row: &TaskRecord, mut wait: Wait, into: &mut Inspection) {
    let retry = match row.executor {
        Executor::Person(_) => None, // This alarm is the person's stall notice, not an authority deadline.
        Executor::Agent { .. } | Executor::Procedure { .. } | Executor::Job { .. } => {
            alarm(domain.inspection_alarms(), row.number)
        }
    };
    let bound = core.and_then(|core| core.inspection_wait_bound(row.number));
    for due in [retry, alarm(domain.inspection_grace_alarms(), row.number), bound].into_iter().flatten() {
        let timer = Wake::Timer(due);
        if !wait.wakes.contains(&timer) {
            wait.wakes.push(timer);
        }
        wait.bound = Some(wait.bound.map_or(due, |old| old.min(due)));
    }
    if wait.bound.is_some() && core.is_some_and(|core| core.inspection_timer_withheld(jig_domain_core::Timer::Tasks)) {
        wait.bound = None;
        wait.wakes.retain(|wake| !matches!(wake, Wake::Timer(_)));
        wait.wakes.push(Wake::Party("the next start".into()));
    }
    into.wait(wait);
}

fn wait(row: &TaskRecord, kind: &str, mut wakes: Vec<Wake>, bound: Option<Time>, required: bool) -> Wait {
    if let Some(due) = bound {
        let timer = Wake::Timer(due);
        if !wakes.contains(&timer) {
            wakes.push(timer);
        }
    }
    Wait {
        waiter: format!("task {}", row.number),
        kind: if required && bound.is_none() { format!("{kind}, with no deadline armed") } else { kind.into() },
        wakes,
        bound,
    }
}

pub(crate) fn task(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    match &row.phase {
        Phase::Waiting => {
            if row.waiting_on.is_empty() {
                queued(domain, core, row, into);
            } else {
                dependencies(domain, core, row, into);
            }
        }
        Phase::Active(active) => {
            dependencies(domain, core, row, into);
            proposal(domain, core, row, into);
            match active {
                Active::Idle => idle(domain, core, row, into),
                Active::Due => {
                    if row.suspension.is_some() {
                        emit(
                            domain,
                            core,
                            row,
                            wait(row, "suspended", vec![], alarm(domain.inspection_alarms(), row.number), true),
                            into,
                        );
                    } else if let Executor::Person(_) = row.executor {
                        person(domain, core, row, into);
                    }
                }
                Active::Preparing | Active::Claimed { .. } | Active::Running { .. } => {}
                Active::BackingOff { .. } => emit(
                    domain,
                    core,
                    row,
                    wait(row, "backing off", vec![], alarm(domain.inspection_alarms(), row.number), true),
                    into,
                ),
            }
        }
        Phase::Closing(closing) => match closing.stage {
            Stage::Run { .. } => {
                let wake = core.and_then(|core| core.inspection_placement(row.number)).map_or_else(
                    || Wake::Party(format!("host of task {}", row.number)),
                    |host| Wake::Party(format!("host {}", host.raw())),
                );
                emit(domain, core, row, wait(row, "closing on its run", vec![wake], None, false), into);
            }
            Stage::Delegates => {
                delegates(domain, core, row, into);
                proposal(domain, core, row, into);
            }
            Stage::Effects | Stage::Releases => {
                let mut wakes = Vec::new();
                if let Some(core) = core {
                    for ((number, connector), ()) in core.inspection_closing_connectors() {
                        if *number == row.number {
                            wakes.push(Wake::Party(format!("connector {connector}")));
                        }
                    }
                }
                if wakes.is_empty() {
                    wakes.push(Wake::Party(format!("connectors of task {}", row.number)));
                }
                let kind =
                    if closing.stage == Stage::Effects { "closing on its effects" } else { "closing on its releases" };
                emit(domain, core, row, wait(row, kind, wakes, None, false), into);
            }
            Stage::Settled => emit(domain, core, row, wait(row, "closing settled", vec![], None, false), into),
        },
        Phase::Held { why, source, .. } => {
            held(domain, core, row, *why, source.as_deref(), into);
            if let Some(hold) = row.placed
                && !matches!(why, Hold::Placed { .. })
            {
                held(domain, core, row, Hold::Placed { hold }, None, into);
            }
        }
        Phase::Ended(_) => {}
    }
}

fn queued(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    let mut wakes = Vec::new();
    for ((name, number), ()) in domain.inspection_waiters() {
        if *number == row.number && super::tables::blocked(domain, name) {
            for ((held, holder), ()) in domain.inspection_holders() {
                if held == name {
                    let wake = task_wake(domain, *holder);
                    if !wakes.contains(&wake) {
                        wakes.push(wake);
                    }
                }
            }
        }
    }
    emit(
        domain,
        core,
        row,
        wait(row, "in a resource's queue", wakes, alarm(domain.inspection_hold_alarms(), row.number), true),
        into,
    );
}

fn dependencies(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    if !row.waiting_on.is_empty() {
        emit(
            domain,
            core,
            row,
            wait(
                row,
                "on its dependencies",
                row.waiting_on.iter().map(|n| task_wake(domain, *n)).collect(),
                None,
                false,
            ),
            into,
        );
    }
}

fn delegates(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    if !row.delegates.is_empty() {
        emit(
            domain,
            core,
            row,
            wait(row, "on its delegates", row.delegates.iter().map(|n| task_wake(domain, *n)).collect(), None, false),
            into,
        );
    }
}

fn person(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    if let Executor::Person(address) = row.executor {
        let wake = match row.taken_by.map_or(address, task::PersonAddress::Person) {
            task::PersonAddress::Person(number) => Wake::Party(format!("person {number}")),
            task::PersonAddress::Role(role) => Wake::Party(format!("role {role} of project {}", row.project)),
        };
        let mut wakes = vec![wake];
        if let Some(due) = alarm(domain.inspection_alarms(), row.number) {
            wakes.push(Wake::Timer(due));
        }
        emit(domain, core, row, wait(row, "on a person's answer", wakes, None, false), into);
    }
}

fn idle(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    match row.executor {
        Executor::Person(_) => person(domain, core, row, into),
        Executor::Procedure { .. } | Executor::Job { .. } => {}
        Executor::Agent { .. } => {
            let before = into.waits().len();
            if task::own_question(row).is_some() {
                emit(
                    domain,
                    core,
                    row,
                    wait(row, "on its requester", vec![party(domain, row.requester)], None, false),
                    into,
                );
            }
            delegates(domain, core, row, into);
            for subscription in &row.subscriptions {
                let (kind, wakes, due, required) = match subscription.kind {
                    SubscriptionKind::Task { target, .. } => {
                        ("on a watch", vec![task_wake(domain, target)], None, false)
                    }
                    SubscriptionKind::Topic { connector, .. } => {
                        ("on a watch", vec![Wake::Party(format!("connector {connector}"))], None, false)
                    }
                    SubscriptionKind::Timer { .. } => {
                        ("on a timer", vec![], alarm(domain.inspection_timers(), subscription.number), true)
                    }
                };
                emit(domain, core, row, wait(row, kind, wakes, due, required), into);
            }
            if row.flags.conversation
                || matches!(row.flags.ended_by, task::EndedBy::Requester | task::EndedBy::Standing)
            {
                emit(
                    domain,
                    core,
                    row,
                    wait(row, "on its party", vec![party(domain, row.requester)], None, false),
                    into,
                );
            }
            if let Some(due) = alarm(domain.inspection_wakes(), row.number) {
                emit(domain, core, row, wait(row, "paced", vec![], Some(due), true), into);
            }
            if before == into.waits().len()
                && row.waiting_on.is_empty()
                && !row
                    .proposal
                    .as_ref()
                    .is_some_and(|proposal| matches!(proposal.state, ProposalState::Pending { .. }))
            {
                emit(domain, core, row, wait(row, "idle with nothing left to await", vec![], None, false), into);
            }
        }
    }
}

fn policy(project: u32, kind: ProposalKind) -> Wake {
    let kind = match kind {
        ProposalKind::Settings => "settings",
        ProposalKind::Batch => "batch",
        ProposalKind::Effect => "effect",
        ProposalKind::Amend => "amend",
        ProposalKind::Widen => "widen",
        ProposalKind::Release => "release",
    };
    Wake::Party(format!("policy {kind} of project {project}"))
}

fn proposal(domain: &Domain, core: Option<&Core>, row: &TaskRecord, into: &mut Inspection) {
    let Some(proposal) = &row.proposal else { return };
    if let ProposalState::Pending { holder, .. } = proposal.state {
        let (wake, required) = match holder {
            ProposalHolder::Task(number) => (task_wake(domain, number), true),
            ProposalHolder::Person(number) => (Wake::Party(format!("person {number}")), true),
            ProposalHolder::Policy { project, kind } => (policy(project, kind), false),
        };
        let bound = alarm(domain.inspection_proposal_alarms(), row.number);
        let spent = bound.is_none()
            && core.is_some_and(|core| !core.inspection_proposal_has_further(row.number, proposal.number));
        emit(domain, core, row, wait(row, "on its proposal's decision", vec![wake], bound, required && !spent), into);
    }
}

fn held(
    domain: &Domain,
    core: Option<&Core>,
    row: &TaskRecord,
    why: Hold,
    source: Option<&HoldSource>,
    into: &mut Inspection,
) {
    let kind = match why {
        Hold::Exceeded(_) => "held: exceeded",
        Hold::Failures(_) => "held: failures",
        Hold::Stalled => "held: stalled",
        Hold::Dependency(_) => "held: dependency",
        Hold::Stopped => "held: stopped",
        Hold::Placed { .. } => "held: placed",
        Hold::Drift => "held: drift",
        Hold::Effects => "held: effects",
        Hold::EffectFailed => "held: effect failed",
        Hold::Uncertain { .. } => "held: uncertain",
        Hold::Budget => "held: budget",
        Hold::Deadline => "held: deadline",
        Hold::HoldsWaited => "held: holds waited",
        Hold::ResourceUnavailable => "held: resource unavailable",
        Hold::ResourceAccess => "held: resource access",
        Hold::PoolLost { .. } => "held: pool lost",
        Hold::Procedure => "held: procedure",
        Hold::Circle => "held: circle",
        Hold::Waited => "held: wait past its bound",
    };
    let mut wakes = Vec::new();
    if let Hold::Placed { hold } = why {
        if let Some(placed) = domain.placed_hold(hold) {
            wakes.push(match placed.owner {
                task::Owner::Party { person } => Wake::Party(format!("person {person}")),
                task::Owner::Task { task } => task_wake(domain, task),
            });
        }
        emit(domain, core, row, wait(row, kind, wakes, None, false), into);
        return;
    }
    let mut required = false;
    let kind = match row.escalation {
        Escalation::Waiting { holder, .. } => {
            wakes.push(match holder {
                EscalationHolder::Task(number) => {
                    required = true;
                    task_wake(domain, number)
                }
                EscalationHolder::Person(number) => {
                    required = true;
                    Wake::Party(format!("person {number}"))
                }
                EscalationHolder::Role { project, role } => Wake::Party(format!("role {role} of project {project}")),
            });
            kind.to_owned()
        }
        Escalation::Unheld { .. } | Escalation::Routing { .. } if row.project == 0 => {
            wakes.push(party(domain, row.requester));
            kind.to_owned()
        }
        Escalation::Unheld { .. } => format!("{kind}, unheld escalation"),
        Escalation::Routing { .. } => format!("{kind}, routing escalation"),
        Escalation::Rejected { by, .. } => {
            wakes.push(match by {
                task::EscalationDecider::Task(number) => task_wake(domain, number),
                task::EscalationDecider::Person(number) => Wake::Party(format!("person {number}")),
            });
            kind.to_owned()
        }
    };
    match source {
        Some(HoldSource::Circle(circle)) => {
            for number in &circle.tasks {
                if *number != row.number {
                    wakes.push(task_wake(domain, *number));
                }
            }
        }
        Some(HoldSource::Resource(name)) => wakes.push(Wake::Party(format!("connector {}", name.connector))),
        Some(HoldSource::Effect { connector, .. } | HoldSource::Account { connector, .. }) => {
            wakes.push(Wake::Party(format!("connector {connector}")));
        }
        Some(HoldSource::Preparation { .. } | HoldSource::Wait { .. }) | None => {}
    }
    emit(
        domain,
        core,
        row,
        wait(row, &kind, wakes, alarm(domain.inspection_escalation_alarms(), row.number), required),
        into,
    );
}
