//! Core-owned standing and policy decisions for a person's task amendment.

pub use jig_domain_core_vocabulary::admission::amendments::*;

use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

use crate::{Core, translate};

impl Core {
    /// An address supplied from outside the core names a held identity
    /// (jig/docs/design/domain/engine/README.md, 5.1).
    pub(crate) fn amendment_person(&self, amendment: &tasks::Amendment) -> Result<(), tasks::Refusal> {
        if let Some(tasks::AddressChange { address: tasks::PersonAddress::Person(person), .. }) = amendment.addressed
            && self.people.identity(person).is_none()
        {
            return Err(tasks::Refusal::Unknown);
        }
        Ok(())
    }

    /// A raw or accepted amendment names a held person before account admission.
    pub(crate) fn admit_amendment(
        &self,
        task: u64,
        amendment: &tasks::Amendment,
        funder: Option<tasks::Funder>,
    ) -> Result<(), tasks::Refusal> {
        self.amendment_person(amendment)?;
        self.amendment_account(task, amendment, funder)
    }

    /// The models a task keeps through an amendment of its authority, which names none; none for
    /// an unknown task, which the amendment's admission refuses (domain/authority.md, section 3).
    #[must_use]
    pub(crate) fn kept_models(&self, task: u64) -> tasks::Models {
        match self.tasks.task(task) {
            Some(row) => row.authority.models,
            None => tasks::Models::NONE,
        }
    }

    /// A task-origin amendment's authority keeps its target's models, as it names none
    /// (domain/authority.md, section 3).
    pub(crate) fn keep_models(&self, target: u64, amendment: &mut tasks::Amendment) {
        if let Some(authority) = &mut amendment.authority {
            authority.models = self.kept_models(target);
        }
    }

    /// A proposed action keeps the models its proposer names none of: a batch's members take the
    /// proposer's, an amendment or a widening keeps its task's, and an effect's needs name none
    /// (domain/authority.md, sections 3 and 5).
    pub(crate) fn keep_proposal_models(&self, proposer: u64, action: &mut tasks::ProposalAction) {
        match action {
            tasks::ProposalAction::Batch(batch) => {
                let models = self.kept_models(proposer);
                for member in batch.iter_mut() {
                    member.authority.models = models;
                }
            }
            tasks::ProposalAction::Amend { task, amendment, .. } => self.keep_models(*task, amendment),
            tasks::ProposalAction::Widen { task, authority } => authority.models = self.kept_models(*task),
            tasks::ProposalAction::Settings { .. }
            | tasks::ProposalAction::Effect { .. }
            | tasks::ProposalAction::ProcedureEffect { .. }
            | tasks::ProposalAction::ReleasePlaced { .. }
            | tasks::ProposalAction::Release { .. } => {}
        }
    }

    /// Admit a task-origin amendment, including ancestry and widening.
    pub fn task_amend_admit(
        &self,
        caller: u64,
        target: u64,
        amendment: &tasks::Amendment,
        depth: u32,
        limits: &tasks::Limits,
    ) -> Result<bool, TaskAmendDenied> {
        if amendment.release.is_some() {
            return Err(TaskAmendDenied::Refused { task: target, why: tasks::Refusal::Read });
        }
        let Some(holder) = self.tasks.delegation(caller) else {
            return Err(TaskAmendDenied::Refused { task: caller, why: tasks::Refusal::Unknown });
        };
        let Some(current) = self.tasks.delegation(target) else {
            return Err(TaskAmendDenied::Refused { task: target, why: tasks::Refusal::Unknown });
        };
        let mut next = target;
        let mut descendant = false;
        for _ in 0..=depth {
            if next == caller {
                descendant = true;
                break;
            }
            let Some(context) = self.tasks.delegation(next) else { break };
            match context.requester {
                tasks::Party::Task(parent) => next = parent,
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
            }
        }
        if target == caller || current.project != holder.project || !descendant {
            return Err(TaskAmendDenied::Refused { task: target, why: tasks::Refusal::Reference });
        }
        let row = self.tasks.task(target).expect("delegation found target");
        if let Err(why) = self.amendment_person(amendment) {
            return Err(TaskAmendDenied::Refused { task: target, why });
        }
        if let Err(why) = tasks::address_admit(row, &amendment.addressed) {
            return Err(TaskAmendDenied::Refused { task: target, why });
        }
        if let Err(why) = tasks::instruction_admit(limits, row.executor, &amendment.instructions) {
            return Err(TaskAmendDenied::Refused { task: target, why });
        }
        if let Err(why) = self.amendment_account(target, amendment, None) {
            return Err(TaskAmendDenied::Refused { task: target, why });
        }
        let Some(after) = &amendment.authority else { return Ok(false) };
        let before = translate::tasks_authority(&current.authority);
        let after = translate::tasks_authority(after);
        let implies = &self.authority.rules().implies;
        let stop_run = !authority::at_most(&before, &after, implies);
        if !authority::at_most(&after, &before, implies) {
            let hard = match self.authority.policy(current.project) {
                Some(policy) => {
                    authority::at_most(&after, &policy.ceiling, implies)
                        && authority::at_most(&after, &self.authority.rules().ceiling, implies)
                }
                None => false,
            };
            let answer = if hard {
                if authority::at_most(&after, &translate::tasks_authority(&holder.authority), implies) {
                    authority::Answer::Allow
                } else {
                    authority::Answer::Propose
                }
            } else {
                authority::Answer::Refuse
            };
            if answer != authority::Answer::Allow {
                return Err(TaskAmendDenied::Denied(answer));
            }
        }
        Ok(stop_run)
    }

    /// A priority alone does not compare the task's specification revision.
    pub(crate) fn party_amend_revision(
        &self,
        task: u64,
        revision: u64,
        amendment: &tasks::Amendment,
    ) -> Result<(), people::Refusal> {
        Self::party_amend_revision_in(task, revision, amendment, &tasks::CompoundAdmission::new(&self.tasks, 0))
    }
    pub(crate) fn party_amend_revision_in(
        task: u64,
        revision: u64,
        amendment: &tasks::Amendment,
        admission: &tasks::CompoundAdmission<'_>,
    ) -> Result<(), people::Refusal> {
        let tracked = match admission.task(task) {
            Some(row) => row.tracked.is_some(),
            None => false,
        };
        let priority = match amendment.tracked {
            Some(tasks::Tracking::Tracked { .. }) => true,
            Some(tasks::Tracking::Untracked) | None => false,
        };
        if priority
            && tracked
            && amendment.addressed.is_none()
            && amendment.instructions.is_none()
            && amendment.spec.is_none()
            && amendment.wake.is_none()
            && amendment.dependencies.is_none()
            && amendment.authority.is_none()
            && amendment.llm.is_none()
        {
            Ok(())
        } else {
            Self::spec_revision_in(task, revision, admission)
        }
    }

    /// Return whether to stop a current run and whether widening must be
    /// offered to a policy holder as a proposal.
    pub(crate) fn amend_admit(
        &self,
        party: super::compound::PartyTask,
        amendment: &tasks::Amendment,
        depth: u32,
        limits: &tasks::Limits,
    ) -> Result<bool, people::Refusal> {
        crate::acts::bounds::amendment_admit(self, party, amendment, depth, limits)
    }
}

fn wake_rule(value: people::WakeRule) -> tasks::WakeRule {
    match value {
        people::WakeRule::Never => tasks::WakeRule::Never,
        people::WakeRule::Immediate => tasks::WakeRule::Immediate,
        people::WakeRule::Batch { count, age } => tasks::WakeRule::Batch { count, age },
        people::WakeRule::Paced { span } => tasks::WakeRule::Paced { span },
    }
}

pub(crate) fn default_wake(core: &Core, executor: &tasks::Executor) -> tasks::WakePolicy {
    match executor {
        tasks::Executor::Agent { charter } => match core.charter(*charter) {
            Some(charter) => charter.wake,
            None => tasks::WakePolicy::DEFAULT,
        },
        tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => {
            tasks::WakePolicy::DEFAULT
        }
    }
}

pub(crate) fn wake(value: people::WakePolicy) -> tasks::WakePolicy {
    tasks::WakePolicy {
        words: wake_rule(value.words),
        notices: wake_rule(value.notices),
        news: wake_rule(value.news),
        results: match value.results {
            people::ResultsWake::Never => tasks::ResultsWake::Never,
            people::ResultsWake::Each => tasks::ResultsWake::Each,
            people::ResultsWake::LastOrFailure => tasks::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}

pub(crate) fn spec(value: people::Spec, limits: &tasks::Limits) -> Option<tasks::Spec> {
    let mut parameters = List::with_capacity(limits.parameters);
    for parameter in value.parameters {
        let translated = match parameter {
            people::Parameter::Number { name, value } => tasks::Parameter::Number { name, value },
            people::Parameter::Bytes { name, value } => tasks::Parameter::Bytes { name, value },
            people::Parameter::Resource { name, connector, resource } => {
                tasks::Parameter::Resource { name, connector, resource }
            }
        };
        parameters.push(translated).ok()?;
    }
    Some(tasks::Spec {
        job: None,
        asks: value.asks,
        words: value.words,
        parameters: parameters.into_boxed(),
        inputs: value.inputs,
    })
}

/// A party's given authority in the task vocabulary, with the models the task takes: a party names
/// none, so a created task takes its creator's role's and an amended one keeps its own
/// (domain/authority.md, sections 3 and 6).
pub(crate) fn granted(
    value: people::Authority,
    models: tasks::Models,
    limits: &tasks::Limits,
) -> Option<tasks::Authority> {
    let grants_limit = skein_lib::length::usize_of(limits.authority_grants);
    let segments_limit = skein_lib::length::usize_of(limits.authority_segments);
    if value.grants.len() > grants_limit
        || value.note_resources.len() > grants_limit
        || value.delegation.kinds.len() > skein_lib::length::usize_of(limits.executor_kinds)
    {
        return None;
    }
    for grant in &value.grants {
        if grant.pattern.segments.len() > segments_limit {
            return None;
        }
    }
    for scope in &value.note_resources {
        if scope.pattern.segments.len() > segments_limit {
            return None;
        }
    }
    let inherit_models = value.models.is_none();
    let mut given = translate::task_authority(&translate::people_authority(value)?);
    if inherit_models {
        given.models = models;
    }
    Some(given)
}

/// A party's amendment in the task vocabulary; a changed authority keeps the task's `models`.
pub(crate) fn translate(
    value: people::Amendment,
    models: tasks::Models,
    limits: &tasks::Limits,
) -> Option<tasks::Amendment> {
    Some(tasks::Amendment {
        tracked: match value.tracked {
            Some(people::Tracking::Tracked { priority }) => Some(tasks::Tracking::Tracked { priority }),
            Some(people::Tracking::Untracked) => Some(tasks::Tracking::Untracked),
            None => None,
        },
        addressed: match value.addressed {
            Some(change) => Some(tasks::AddressChange {
                revision: change.revision,
                address: match change.address {
                    people::PersonAddress::Person(person) => tasks::PersonAddress::Person(person),
                    people::PersonAddress::Role(role) => tasks::PersonAddress::Role(role),
                },
            }),
            None => None,
        },
        release: None,
        instructions: value.instructions,
        spec: match value.spec {
            Some(specification) => Some(spec(specification, limits)?),
            None => None,
        },
        wake: match value.wake {
            Some(policy) => Some(wake(policy)),
            None => None,
        },
        dependencies: value.dependencies,
        authority: match value.authority {
            Some(authority) => Some(granted(authority, models, limits)?),
            None => None,
        },
        llm: crate::runs::llm::choice(value.llm.as_ref()),
        reason: value.reason,
    })
}
