//! Admission of goals and recurring work (`jig/docs/design/domain/engine.md`, 4.4).
use crate::{Core, translate};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Queue;

impl Core {
    /// Whether a configured recurring template is novel and affordable.
    #[must_use]
    pub fn recurring_admit(&self, project: u32, given: &tasks::Authority, template: &tasks::RecurringTemplate) -> bool {
        for member in &template.batch {
            match member.executor {
                tasks::Executor::Person(_) => return false,
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => {}
            }
        }
        for task in self.tasks.recurring_tasks(project) {
            if self.tasks.recurring_template(task).expect("identified recurring task").key == template.key {
                return false;
            }
        }
        let Some(policy) = self.authority.policy(project) else { return false };
        authority::at_most(&translate::tasks_authority(given), &policy.ceiling, &self.authority.rules().implies)
            && self.period_budget(project) <= policy.period_spend
    }

    /// Check the current holder's standing on one person-origin goal.
    pub fn goal_standing(
        &self,
        proposal: &tasks::PersonProposal,
        project: u32,
        role: Option<people::Role>,
    ) -> Result<(), people::Refusal> {
        if proposal.action.is_settings() {
            return if project == 0 && role == Some(people::Role::Owner) {
                Ok(())
            } else {
                Err(people::Refusal::Role { held: role })
            };
        }
        if proposal.project != project || role.is_none() {
            return Err(people::Refusal::Unknown);
        }
        let named = role.expect("checked role");
        let Some(policy) = self.authority.role(project, named.number()) else {
            return Err(people::Refusal::Unknown);
        };
        let kind = match &proposal.action {
            tasks::PersonAction::Settings { .. } => unreachable!("settings standing checked separately"),
            tasks::PersonAction::Batch(_) => authority::ProposalKind::Batch,
            tasks::PersonAction::Amend { .. } => authority::ProposalKind::Amend,
        };
        if !policy.decides.allows(kind) {
            return Err(people::Refusal::Unknown);
        }
        Ok(())
    }

    /// Recheck authentic pool funding before a goal proposal is accepted.
    pub fn goal_accept_allowed(
        &self,
        proposal: &tasks::PersonProposal,
        project: u32,
        person: u64,
        role: people::Role,
        tasks_left: u32,
        decision: skein_lib::Wall,
    ) -> Result<tasks::Funder, people::Refusal> {
        let (source, numbers) = self.person_pool(project, person);
        match &proposal.action {
            tasks::PersonAction::Settings { .. } => return Ok(source),
            tasks::PersonAction::Batch(batch) => {
                if let Err(why) = self.funded_accounts(batch, source) {
                    return Err(self.account_refusal(why));
                }
            }
            tasks::PersonAction::Amend { task, amendment, .. } => {
                if let Err(why) = self.amendment_account(*task, amendment, Some(source)) {
                    return Err(self.account_refusal(why));
                }
            }
        }
        let action = match &proposal.action {
            tasks::PersonAction::Settings { .. } => return Ok(source),
            tasks::PersonAction::Batch(batch) => {
                let mut members =
                    skein_lib::List::with_capacity(u32::try_from(batch.len()).expect("bounded party batch"));
                for member in batch {
                    members
                        .push(authority::Delegate {
                            executor: self
                                .delegation_executor(project, member.executor)
                                .ok_or(people::Refusal::Unknown)?,
                            authority: super::deadlines::authority(&member.authority, member.deadline, decision),
                            symbolic: Box::new([]),
                        })
                        .expect("bounded party batch");
                }
                authority::Action::Batch(members.into_boxed())
            }
            tasks::PersonAction::Amend { task, amendment, .. } => {
                let row = self.tasks.task(*task).ok_or(people::Refusal::Unknown)?;
                authority::Action::Amend(translate::tasks_authority(
                    amendment.authority.as_ref().unwrap_or(&row.authority),
                ))
            }
        };
        let mut findings = Queue::uncapped();
        let ask = authority::PersonAsk {
            project,
            role: role.number(),
            pool: translate::authority_numbers(numbers),
            tasks_left,
            request: authority::PersonRequest::Accept(action),
        };
        let checked = authority::check_request(&self.authority, &ask, &mut findings);
        if checked.answer == authority::Answer::Allow {
            Ok(source)
        } else {
            Err(self.request_refusal(&ask, checked.answer, &mut findings))
        }
    }
}
