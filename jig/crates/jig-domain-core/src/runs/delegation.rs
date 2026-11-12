//! Generic delegate executor and symbolic-grant decisions owned by the core
//! (domain/engine/README.md, 4.4, and domain/engine/settings.md, 3).
#![cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::result_large_err,
        reason = "bounded owner refusal keeps exact revision and capacity facts inline instead of allocating on refusal"
    )
)]

pub use jig_domain_core_vocabulary::runs::delegation::*;

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Wall};

use crate::{CallKey, CallPart, Core, translate};

impl Core {
    /// Admit one procedure's delegated batch against its current task authority.
    pub fn procedure_batch_admit(
        &self,
        limits: &tasks::Limits,
        task: u64,
        batch: &[Delegate],
        decision: Wall,
    ) -> Result<(), crate::StepRefusal> {
        let Some(context) = self.tasks.delegation(task) else {
            return Err(crate::routing::effect::procedure::problem(task, tasks::Refusal::Batch));
        };
        if batch.is_empty() || batch.len() > skein_lib::length::usize_of(limits.batch) {
            return Err(crate::routing::effect::procedure::problem(task, tasks::Refusal::Batch));
        }
        let mut asked = List::with_capacity(limits.batch);
        for member in batch {
            if missing_closer(member.flags, &member.authority).is_some() {
                return Err(crate::StepRefusal::Denied {
                    answer: authority::Answer::Refuse,
                    findings: Box::new([authority::Finding::Executor { source: authority::Source::Task }]),
                });
            }
            if let tasks::Executor::Person(tasks::PersonAddress::Person(person)) = member.executor
                && self.people.identity(person).is_none()
            {
                return Err(crate::routing::effect::procedure::problem(task, tasks::Refusal::Unknown));
            }
            if !member.spec.inputs.is_empty() {
                return Err(crate::routing::effect::procedure::problem(task, tasks::Refusal::Batch));
            }
            let Some(executor) = self.delegation_executor(context.project, member.executor) else {
                return Err(crate::routing::effect::procedure::problem(task, tasks::Refusal::Batch));
            };
            let Some(symbolic) = symbolic_grants(&member.symbolic_grants, self.authority.limits().grants) else {
                return Err(crate::routing::effect::procedure::problem(task, tasks::Refusal::Batch));
            };
            asked
                .push(authority::Delegate {
                    executor,
                    authority: crate::admission::deadlines::authority(&member.authority, member.deadline, decision),
                    symbolic,
                })
                .expect("bounded procedure batch");
        }
        let mut findings = Queue::uncapped();
        let checked = self.connector_batch_admit(
            context.project,
            &context.authority,
            context.numbers,
            context.tasks_left,
            asked.into_boxed(),
            &mut findings,
        );
        if checked.answer == authority::Answer::Allow {
            return Ok(());
        }
        let mut reasons = List::with_capacity(findings.len());
        for _ in 0..findings.len() {
            reasons.push(findings.pop().expect("finding count")).expect("finding bound");
        }
        Err(crate::StepRefusal::Denied { answer: checked.answer, findings: reasons.into_boxed() })
    }
    /// A delegate's models are its creator's, as a batch names none; a procedure's delegates take
    /// those it keeps, though it names none itself (domain/authority.md, sections 3 and 5).
    pub(crate) fn take_models(&self, task: u64, batch: &mut [Delegate]) {
        let models = self.kept_models(task);
        for member in batch {
            member.authority.models = models;
        }
    }
    /// A delegate naming no deadline ends with its creator's: its
    /// deadline is never later (domain/authority.md, 5 and 7).
    pub(crate) fn take_deadline(&self, task: u64, batch: &mut [Delegate]) {
        let Some(context) = self.tasks.delegation(task) else { return };
        for member in batch {
            if member.deadline == tasks::Deadline::Named && member.authority.budget.deadline.is_none() {
                member.authority.budget.deadline = context.authority.budget.deadline;
            }
        }
    }
    /// Check a named task delegation at `decision`, before any retained
    /// input load or connector handoff.
    pub fn delegate_preflight(
        &self,
        limits: &tasks::Limits,
        key: CallKey,
        batch: &[Delegate],
        decision: Wall,
    ) -> Result<(), CallPart> {
        if !self.current_proof(key.task, key.attempt) {
            return Err(delegate_refused(Some(key.task), tasks::Refusal::State));
        }
        let Some(context) = self.tasks.delegation(key.task) else {
            return Err(delegate_refused(Some(key.task), tasks::Refusal::Unknown));
        };
        if batch.is_empty() || batch.len() > skein_lib::length::usize_of(limits.batch) {
            return Err(delegate_refused(None, tasks::Refusal::Batch));
        }
        let mut asked = List::with_capacity(limits.batch);
        for member in batch {
            if missing_closer(member.flags, &member.authority).is_some() {
                return Err(CallPart::DelegationDenied {
                    answer: authority::Answer::Refuse,
                    findings: Box::new([authority::Finding::Executor { source: authority::Source::Task }]),
                });
            }
            match member.executor {
                tasks::Executor::Job { .. } => return Err(delegate_refused(None, tasks::Refusal::Executor)),
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => {}
            }
            if let tasks::Executor::Person(tasks::PersonAddress::Person(person)) = member.executor
                && self.people.identity(person).is_none()
            {
                return Err(delegate_refused(None, tasks::Refusal::Unknown));
            }
            if let tasks::Executor::Agent { charter } = member.executor
                && self.charter(charter).is_none()
            {
                return Err(delegate_refused(None, tasks::Refusal::Executor));
            }
            if let Err(why) = super::account_preference::executor(member.executor, member.llm.as_ref()) {
                return Err(delegate_refused(None, why));
            }
            if let Some(choice) = &member.llm
                && choice.account.is_some()
            {
                let payer =
                    self.tasks.payer(key.task).ok_or(delegate_refused(Some(key.task), tasks::Refusal::Funding))?;
                if let Err(why) = self.account_preference(Some(choice), payer, context.project, None) {
                    return Err(delegate_refused(None, why));
                }
            }
            let Some(executor) = self.delegation_executor(context.project, member.executor) else {
                return Err(delegate_refused(None, tasks::Refusal::Executor));
            };
            let Some(symbolic) = symbolic_grants(&member.symbolic_grants, self.authority.limits().grants) else {
                return Err(delegate_refused(None, tasks::Refusal::AuthorityShape));
            };
            let mut authority = crate::admission::deadlines::authority(&member.authority, member.deadline, decision);
            if member.deadline == tasks::Deadline::Named && authority.budget.deadline.is_none() {
                authority.budget.deadline = context.authority.budget.deadline;
            }
            asked.push(authority::Delegate { executor, authority, symbolic }).expect("bounded delegation request");
        }
        let mut findings = Queue::uncapped();
        let checked = self.connector_batch_admit(
            context.project,
            &context.authority,
            context.numbers,
            context.tasks_left,
            asked.into_boxed(),
            &mut findings,
        );
        if checked.answer != authority::Answer::Allow {
            let mut found = List::with_capacity(findings.len());
            for _ in 0..findings.len() {
                found.push(findings.pop().expect("finding count")).expect("finding bound");
            }
            return Err(CallPart::DelegationDenied { answer: checked.answer, findings: found.into_boxed() });
        }
        Ok(())
    }

    /// Name each retained input once before the root starts its bounded result loads.
    pub fn delegate_inputs(
        &self,
        limits: &tasks::Limits,
        key: CallKey,
        batch: &[Delegate],
    ) -> Result<(u32, Box<[u64]>), CallPart> {
        let Some(context) = self.tasks.delegation(key.task) else {
            return Err(delegate_refused(Some(key.task), tasks::Refusal::Unknown));
        };
        let capacity =
            batch.len().checked_mul(skein_lib::length::usize_of(limits.inputs)).expect("bounded batch input count");
        let mut ids = List::with_capacity(u32::try_from(capacity).expect("bounded input IDs"));
        for member in batch {
            if member.spec.inputs.len() > skein_lib::length::usize_of(limits.inputs) {
                return Err(delegate_refused(None, tasks::Refusal::Inputs));
            }
            for &input in &member.spec.inputs {
                let mut known = false;
                for &id in &ids {
                    if id == input {
                        known = true;
                    }
                }
                if !known {
                    ids.push(input).expect("bounded input ID count");
                }
            }
        }
        Ok((context.project, ids.into_boxed()))
    }

    /// Resolve a proposed executor against the current party roles.
    #[must_use]
    pub fn delegation_executor(&self, project: u32, executor: tasks::Executor) -> Option<authority::Executor> {
        match executor {
            tasks::Executor::Job { kind } => Some(authority::Executor::Job(kind)),
            tasks::Executor::Agent { charter } => Some(authority::Executor::Charter(charter)),
            tasks::Executor::Procedure { code, .. } => Some(authority::Executor::Procedure(code)),
            tasks::Executor::Person(tasks::PersonAddress::Role(role)) => Some(authority::Executor::Role(role)),
            tasks::Executor::Person(tasks::PersonAddress::Person(person)) => {
                let holding = self.people.role(person, project)?;
                Some(authority::Executor::Role(holding.number()))
            }
        }
    }
}

/// A closer is a procedure the task's own delegation must name (jig/docs/design/domain/tasks/lifecycle.md, 11).
pub(crate) fn missing_closer(flags: tasks::Flags, authority: &tasks::Authority) -> Option<tasks::Closer> {
    match flags.ended_by {
        tasks::EndedBy::Result | tasks::EndedBy::Requester | tasks::EndedBy::Standing => None,
        tasks::EndedBy::Closer(closer) => {
            if authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Procedure(closer.code)) {
                None
            } else {
                Some(closer)
            }
        }
    }
}

fn delegate_refused(task: Option<u64>, why: tasks::Refusal) -> CallPart {
    CallPart::DelegationRefused(tasks::Problem { task, why, blocked_by: None, capacity: None })
}

/// Translate open grant patterns for a delegate whose number is not yet fixed.
#[must_use]
pub fn symbolic_grants(grants: &[tasks::Grant], limit: u32) -> Option<Box<[authority::Grant]>> {
    let mut result = List::with_capacity(limit);
    for grant in grants {
        let last = match &grant.pattern.last {
            tasks::Last::Open(prefix) => authority::Last::Open(prefix.clone()),
            tasks::Last::Exact(_) => return None,
        };
        result
            .push(authority::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: authority::Pattern { segments: grant.pattern.segments.clone(), last },
            })
            .ok()?;
    }
    Some(result.into_boxed())
}

/// Narrow a delegate's symbolic grants with its newly assigned task number.
#[must_use]
pub fn resolved_delegate_authority(
    base: &tasks::Authority,
    symbolic: &[tasks::Grant],
    task: u64,
    limits: &authority::Limits,
) -> Option<tasks::Authority> {
    let symbols = symbolic_grants(symbolic, limits.grants)?;
    let resolved = authority::resolve_task_grants(&symbols, task, limits)?;
    let mut authority = translate::tasks_authority(base);
    let mut grants = List::with_capacity(limits.grants);
    for grant in &authority.grants {
        grants.push(grant.clone()).ok()?;
    }
    for grant in resolved {
        grants.push(grant).ok()?;
    }
    authority.grants = grants.into_boxed();
    Some(translate::task_authority(&authority))
}

/// One delegation waiting for the task child's durable creation reply.
pub type Delegating = (CallKey, Box<[tasks::Stub]>, Option<jig_domain_people::Given>);

/// A procedure's job takes its application's bounds (jig/docs/design/domain/engine/executors.md, 5).
pub(crate) fn job_member(jobs: &[jig_domain_link::JobBound], member: &mut Delegate) -> Result<(), tasks::Refusal> {
    let tasks::Executor::Job { kind } = member.executor else { return Ok(()) };
    let mut declared = None;
    for bound in jobs {
        if bound.kind.0 == kind {
            declared = Some(bound);
            break;
        }
    }
    let bound = declared.ok_or(tasks::Refusal::Executor)?;
    let spec = member.spec.job.as_ref().ok_or(tasks::Refusal::Spec)?;
    if spec.bytes.len() > skein_lib::length::usize_of(bound.spec) || spec.deadline > bound.deadline {
        return Err(tasks::Refusal::Spec);
    }
    if member.llm.is_some() {
        return Err(tasks::Refusal::Executor);
    }
    member.contract = tasks::Contract::Job { bytes: bound.result };
    Ok(())
}
