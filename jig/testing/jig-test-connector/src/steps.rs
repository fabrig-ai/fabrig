//! Translate the fake procedure program into the contract's atomic step.
use crate as connector;
use alloc::boxed::Box;
use jig_domain_contract::ops as tasks;
use jig_domain_contract::procedures as core;
use skein_lib::{Decimal, List, Writer};

/// The part of the start context the fake program uses when it chooses its step.
#[derive(Debug)]
pub(crate) struct StepContext {
    pub(crate) task: u64,
    pub(crate) authority: tasks::Authority,
    pub(crate) numbers: tasks::Numbers,
}

pub(crate) fn translate(
    context: &StepContext,
    step: u64,
    number: u16,
    decision: connector::StepDecision,
) -> core::ProcedureStep {
    let task = context.task;
    let end = match decision {
        connector::StepDecision::Finish { result: connector::ProcedureResult::Succeeded } => {
            core::StepEnd::Result(tasks::TaskResult::Report { words: Box::new([]) })
        }
        connector::StepDecision::Finish { result: connector::ProcedureResult::Local { code } } => {
            core::StepEnd::Result(tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: number,
                kind: code,
                resource: 0,
                words: Box::new([]),
            })
        }
        connector::StepDecision::Wait { .. } | connector::StepDecision::Stall => core::StepEnd::Wait,
        connector::StepDecision::Hold { .. } => core::StepEnd::Hold(tasks::Hold::Effects),
        connector::StepDecision::Amend { task, revision, words, priority } => {
            return amendment(task, revision, words, priority);
        }
        connector::StepDecision::Cancel { task, grace } => {
            return core::ProcedureStep {
                cancels: Box::new([core::StepCancel {
                    task,
                    reason: b"procedure cancelled delegate".as_slice().into(),
                    grace: Some(grace),
                }]),
                ..core::ProcedureStep::new(core::StepEnd::Wait)
            };
        }
        connector::StepDecision::Message { task, words } => {
            return core::ProcedureStep {
                messages: Box::new([core::StepMessage {
                    notice: None,
                    target: task,
                    kind: tasks::MessageKind::Words,
                    words: Decimal::of(words).as_bytes().into(),
                }]),
                ..core::ProcedureStep::new(core::StepEnd::Wait)
            };
        }
        connector::StepDecision::Propose { spend } => {
            let mut authority = context.authority.clone();
            authority.budget.spend = authority.budget.spend.saturating_add(spend);
            return core::ProcedureStep {
                proposals: Box::new([tasks::ProposalAction::Widen { task, authority }]),
                ..core::ProcedureStep::new(core::StepEnd::Wait)
            };
        }
        connector::StepDecision::Job { kind, spec, deadline } => return job(context, kind, spec, deadline),
        connector::StepDecision::Delegate { kinds } => {
            let capacity = u32::try_from(kinds.len()).expect("bounded program");
            let mut batch = List::with_capacity(capacity);
            let budget = context
                .numbers
                .budget
                .saturating_sub(context.numbers.spent)
                .saturating_sub(context.numbers.spent_below)
                .saturating_sub(context.numbers.reserved)
                .checked_div(u64::from(capacity))
                .unwrap_or(0);
            for kind in kinds {
                let mut authority = context.authority.clone();
                authority.budget.spend = budget;
                authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
                batch
                    .push(core::Delegate {
                        deadline: tasks::Deadline::Named,
                        label: 0,
                        flags: tasks::Flags::PLAIN,
                        tracked: None,
                        budget: core::Budget::Named,
                        executor: tasks::Executor::Agent { charter: u32::from(kind) },
                        spec: specification(words(task, step)),
                        contract: tasks::Contract::Report { words: 128 },
                        authority,
                        llm: None,
                        symbolic_grants: Box::new([]),
                        dependencies: Box::new([]),
                        wake: Some(tasks::WakePolicy::DEFAULT),
                    })
                    .expect("one delegate per program kind");
            }
            return core::ProcedureStep { batch: batch.into_boxed(), ..core::ProcedureStep::new(core::StepEnd::Wait) };
        }
        connector::StepDecision::Effect(_) => unreachable!("effects start through their own port"),
    };
    core::ProcedureStep::new(end)
}

pub(crate) fn refused(why: core::StepRefusal) -> connector::StepRefusal {
    match why {
        core::StepRefusal::Moved { now: tasks::party::Moved::Spec { revision }, .. } => {
            connector::StepRefusal::Moved { revision }
        }
        core::StepRefusal::Refused(problem) if problem.why == tasks::Refusal::Kept => connector::StepRefusal::Kept {
            pending: match problem.blocked_by {
                Some(pending) => pending,
                None => Box::new([]),
            },
        },
        core::StepRefusal::Moved { .. } | core::StepRefusal::Refused(_) | core::StepRefusal::Denied { .. } => {
            connector::StepRefusal::Refused
        }
    }
}

fn words(task: u64, step: u64) -> Box<[u8]> {
    let task = Decimal::of(task);
    let step = Decimal::of(step);
    let mut words = Writer::new(
        16_usize
            .checked_add(task.as_bytes().len())
            .expect("bounded task label")
            .checked_add(step.as_bytes().len())
            .expect("bounded step label"),
    );
    words.put(b"procedure ").expect("bounded label");
    words.put(task.as_bytes()).expect("bounded task number");
    words.put(b" step ").expect("bounded label");
    words.put(step.as_bytes()).expect("bounded step number");
    words.finish()
}

fn amendment(task: u64, revision: u64, words: u64, priority: Option<u32>) -> core::ProcedureStep {
    let count = Decimal::of(words);
    let mut text = Writer::new(8_usize.checked_add(count.as_bytes().len()).expect("bounded spec"));
    text.put(b"amended ").expect("bounded spec");
    text.put(count.as_bytes()).expect("bounded word count");
    let mut tracked = None;
    if let Some(priority) = priority {
        tracked = Some(tasks::Tracking::Tracked { priority });
    }
    core::ProcedureStep {
        amendments: Box::new([core::StepAmendment {
            task,
            revision,
            amendment: tasks::Amendment {
                spec: if priority.is_some() { None } else { Some(specification(text.finish())) },
                tracked,
                addressed: None,
                instructions: None,
                release: None,
                wake: None,
                dependencies: None,
                authority: None,
                llm: None,
                reason: Box::new([]),
            },
        }]),
        ..core::ProcedureStep::new(core::StepEnd::Wait)
    }
}

/// Translate a committed procedure delivery into its connector's input.
#[must_use]
pub fn procedure_input(context: &tasks::RunContext, code: u16, resource: connector::Path) -> connector::Event {
    let mut added = List::with_capacity(u32::try_from(context.added.len()).expect("bounded dependencies"));
    for value in &context.added {
        let by = match value.by {
            tasks::Party::Person(person) => connector::application_core::Party::Person(person),
            tasks::Party::Task(task) => connector::application_core::Party::Task(task),
            tasks::Party::Deployment { project } => connector::application_core::Party::Deployment { project },
        };
        added.push(connector::Added { task: value.task, at: value.at, by }).expect("one row per dependency");
    }
    connector::Event::Procedure {
        project: context.project,
        task: context.task,
        number: code,
        resource,
        signal: connector::ProcedureSignal::Activate,
        pending: context.pending.clone(),
        added: added.into_boxed(),
    }
}

fn specification(words: Box<[u8]>) -> tasks::Spec {
    tasks::Spec { job: None, asks: false, words, parameters: Box::new([]), inputs: Box::new([]) }
}

/// A job delegate's opaque spec and empty child delegation (jig/docs/design/domain/engine/executors.md, 5).
fn job(context: &StepContext, kind: u32, spec: Box<[u8]>, deadline: skein_lib::Duration) -> core::ProcedureStep {
    let mut authority = context.authority.clone();
    authority.budget.spend = 0;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    core::ProcedureStep {
        batch: Box::new([core::Delegate {
            deadline: tasks::Deadline::Named,
            label: 0,
            flags: tasks::Flags::PLAIN,
            tracked: None,
            budget: core::Budget::Named,
            executor: tasks::Executor::Job { kind },
            spec: tasks::Spec {
                words: Box::new([]),
                parameters: Box::new([]),
                inputs: Box::new([]),
                asks: false,
                job: Some(Box::new(tasks::JobSpec { bytes: spec, deadline })),
            },
            contract: tasks::Contract::Job { bytes: 0 },
            authority,
            llm: None,
            symbolic_grants: Box::new([]),
            dependencies: Box::new([]),
            wake: Some(tasks::WakePolicy::DEFAULT),
        }]),
        ..core::ProcedureStep::new(core::StepEnd::Wait)
    }
}
