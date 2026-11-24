//! Core-owned held-recipient selection and restore validation
//! (domain/engine/README.md, section 3, and domain/engine/vocabulary.md, section 1.4).

use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Queue;

use crate::{Core, Limits, translate};

fn fallback(core: &Core, project: u32) -> Option<tasks::EscalationHolder> {
    // The policy names the escalation role; this slice resolves that role here.
    let role = core.authority.policy(project)?.escalation_role?;
    Some(tasks::EscalationHolder::Role { project, role })
}

fn covers(core: &Core, limits: &Limits, project: u32, person: u64, role: Option<people::Role>) -> bool {
    let Some(role) = role else { return false };
    let pool = tasks::Funder::Pool { project, person, period: core.period(project) };
    let numbers = match core.tasks.funding(pool) {
        Some(record) => record.numbers,
        None => tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
    };
    let Some(needed) = authority::needs(&authority::Action::Escalate { release: None }) else { return false };
    authority::covers(
        &core.authority,
        &needed,
        &authority::Holder::Person {
            project,
            role: role.number(),
            proposal: authority::ProposalKind::Escalation,
            pool: translate::authority_numbers(numbers),
            tasks_left: limits.tasks.tree_tasks,
        },
        0,
    )
}

fn supported(core: &Core, task: &tasks::TaskRecord) -> bool {
    let Some(fallback) = fallback(core, task.project) else { return false };
    match &task.escalation {
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Role { .. }, entry, .. } => {
            // An old selector can be rechecked, but its project must remain the chat's.
            *entry <= core.counters.deployment().messages
                && match fallback {
                    tasks::EscalationHolder::Role { project, .. } => project == task.project,
                    tasks::EscalationHolder::Task(_) | tasks::EscalationHolder::Person(_) => false,
                }
        }
        tasks::Escalation::Rejected { by: tasks::EscalationDecider::Person(person), .. } => {
            *person <= core.counters.deployment().people
        }
        tasks::Escalation::Rejected { by: tasks::EscalationDecider::Task(task), .. } => {
            *task <= core.counters.deployment().tasks
        }
        tasks::Escalation::Waiting { entry, .. } => *entry <= core.counters.deployment().messages,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } => true,
    }
}

/// Pure recipient selection for startup/live routing and candidate-roster
/// preflight; preserves a final-role holder.
fn recipient(
    core: &Core,
    limits: &Limits,
    context: &tasks::EscalationContext,
    requester_role: Option<people::Role>,
) -> Option<tasks::EscalationHolder> {
    recipient_after(core, limits, context, requester_role, None)
}

/// A job's requesting procedure hears its hold through its next step
/// (jig/docs/design/domain/engine/executors.md, 5).
fn job_requester(core: &Core, task: u64, requester: &tasks::TaskRecord) -> bool {
    let Some(job) = core.tasks.task(task) else { return false };
    match job.executor {
        tasks::Executor::Job { .. } if job.requester == tasks::Party::Task(requester.number) => {}
        tasks::Executor::Job { .. }
        | tasks::Executor::Agent { .. }
        | tasks::Executor::Procedure { .. }
        | tasks::Executor::Person(_) => return false,
    }
    match requester.executor {
        tasks::Executor::Procedure { .. } => {}
        tasks::Executor::Agent { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => return false,
    }
    match requester.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) => true,
        tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    }
}

/// A standing agent's requester ends it, but it still decides its delegates' holds.
fn standing_requester(row: &tasks::TaskRecord) -> bool {
    if row.flags.ended_by != tasks::EndedBy::Standing || !row.waiting_on.is_empty() {
        return false;
    }
    for credit in &row.questions {
        if credit.asked == row.requester {
            return false;
        }
    }
    match (row.executor, &row.phase) {
        (tasks::Executor::Agent { .. }, tasks::Phase::Waiting | tasks::Phase::Active(_)) => true,
        (
            tasks::Executor::Agent { .. },
            tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_),
        )
        | (
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. },
            tasks::Phase::Waiting
            | tasks::Phase::Active(_)
            | tasks::Phase::Held { .. }
            | tasks::Phase::Closing(_)
            | tasks::Phase::Ended(_),
        ) => false,
    }
}

fn recipient_after(
    core: &Core,
    limits: &Limits,
    context: &tasks::EscalationContext,
    requester_role: Option<people::Role>,
    after: Option<tasks::EscalationHolder>,
) -> Option<tasks::EscalationHolder> {
    if let tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Role { .. }, .. } = context.escalation {
        return fallback(core, context.project);
    }
    route(
        core,
        limits,
        &Route {
            task: context.task,
            project: context.project,
            requester: context.requester,
            immediate: context.immediate,
        },
        requester_role,
        after,
    )
}

struct Route {
    task: u64,
    project: u32,
    requester: u64,
    immediate: tasks::Party,
}

fn route(
    core: &Core,
    limits: &Limits,
    context: &Route,
    requester_role: Option<people::Role>,
    after: Option<tasks::EscalationHolder>,
) -> Option<tasks::EscalationHolder> {
    let needed = authority::needs(&authority::Action::Escalate { release: None })?;
    let mut above = context.immediate;
    let mut distance = 1_u32;
    let mut passed = after.is_none();
    for _ in 0..limits.tasks.depth.saturating_add(1) {
        let parent = match above {
            tasks::Party::Task(parent) => parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        };
        let Some((holder, deciding, tasks_left)) = core.tasks.delegation_row(parent) else {
            return fallback(core, context.project);
        };
        if passed
            && (deciding || standing_requester(holder) || job_requester(core, context.task, holder))
            && holder.project == context.project
            && translate::task_holder_covers(&core.authority, &needed, holder, tasks_left, distance)
        {
            return Some(tasks::EscalationHolder::Task(parent));
        }
        if after == Some(tasks::EscalationHolder::Task(parent)) {
            passed = true;
        }
        above = holder.requester;
        distance = distance.checked_add(1)?;
    }
    if passed && context.requester != 0 && covers(core, limits, context.project, context.requester, requester_role) {
        Some(tasks::EscalationHolder::Person(context.requester))
    } else {
        fallback(core, context.project)
    }
}

/// Read the route a hold on this row would use, even before it has a hold.
pub(crate) fn wait_holder(
    core: &Core,
    limits: &Limits,
    row: &tasks::TaskRecord,
    after: Option<tasks::EscalationHolder>,
) -> Option<tasks::EscalationHolder> {
    if let tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Role { .. }, .. } = row.escalation {
        return fallback(core, row.project);
    }
    let root = if row.root == row.number { row } else { core.tasks.task(row.root)? };
    let requester = match root.requester {
        tasks::Party::Person(person) => person,
        tasks::Party::Deployment { .. } => 0,
        tasks::Party::Task(_) => return None,
    };
    let context = Route { task: row.number, project: row.project, requester, immediate: row.requester };
    route(core, limits, &context, core.people.role(requester, row.project), after)
}

impl Core {
    /// Whether a person may see this held decision context.
    #[must_use]
    pub fn escalation_visible(
        &self,
        context: &tasks::EscalationContext,
        person: u64,
        role: Option<people::Role>,
    ) -> bool {
        if person == context.requester {
            return true;
        }
        let Some(role) = role else { return false };
        match self.escalation_fallback(context.project) {
            Some(tasks::EscalationHolder::Role { role: selected, .. }) => role.number() == selected,
            Some(tasks::EscalationHolder::Task(_) | tasks::EscalationHolder::Person(_)) | None => false,
        }
    }

    /// Whether the exact current held route addresses this person.
    #[must_use]
    pub fn escalation_standing(context: &tasks::EscalationContext, person: u64, role: Option<people::Role>) -> bool {
        match &context.escalation {
            tasks::Escalation::Waiting { holder, .. } => match holder {
                tasks::EscalationHolder::Task(_) => false,
                tasks::EscalationHolder::Person(number) => person == *number,
                tasks::EscalationHolder::Role { project, role: selected } => {
                    *project == context.project
                        && match role {
                            Some(role) => role.number() == *selected,
                            None => false,
                        }
                }
            },
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Rejected { .. } => false,
        }
    }

    /// Recheck a person's current funding and authority before releasing a
    /// held task's escalation.
    pub fn escalation_release_allowed(
        &self,
        context: &tasks::EscalationContext,
        person: u64,
        role: Option<people::Role>,
        tasks_left: u32,
    ) -> Result<(), people::Refusal> {
        let Some(role) = role else { return Err(people::Refusal::Role { held: None }) };
        let pool = tasks::Funder::Pool { project: context.project, person, period: self.period(context.project) };
        let numbers = match self.tasks.funding(pool) {
            Some(record) => record.numbers,
            None => tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        };
        let mut findings = Queue::uncapped();
        let ask = authority::PersonAsk {
            project: context.project,
            role: role.number(),
            pool: translate::authority_numbers(numbers),
            tasks_left,
            request: authority::PersonRequest::Accept(authority::Action::Escalate { release: None }),
        };
        let checked = authority::check_request(&self.authority, &ask, &mut findings);
        if checked.answer == authority::Answer::Allow {
            Ok(())
        } else {
            Err(self.request_refusal(&ask, checked.answer, &mut findings))
        }
    }

    /// Select the project's configured fallback role for a held decision.
    #[must_use]
    pub fn escalation_fallback(&self, project: u32) -> Option<tasks::EscalationHolder> {
        fallback(self, project)
    }

    /// Check whether a party can hold the escalation at this route.
    #[must_use]
    pub fn escalation_covers(
        &self,
        limits: &Limits,
        context: &tasks::EscalationContext,
        person: u64,
        role: Option<people::Role>,
    ) -> bool {
        covers(self, limits, context.project, person, role)
    }

    /// Validate one restored task's held-recipient shape.
    #[must_use]
    pub fn escalation_supported(&self, task: &tasks::TaskRecord) -> bool {
        supported(self, task)
    }

    /// Select the current holder of a waiting escalation.
    #[must_use]
    pub fn escalation_recipient(
        &self,
        limits: &Limits,
        context: &tasks::EscalationContext,
        requester_role: Option<people::Role>,
    ) -> Option<tasks::EscalationHolder> {
        recipient(self, limits, context, requester_role)
    }

    /// Select the next holder after one current holder stalls.
    #[must_use]
    pub fn escalation_recipient_after(
        &self,
        limits: &Limits,
        context: &tasks::EscalationContext,
        requester_role: Option<people::Role>,
        after: Option<tasks::EscalationHolder>,
    ) -> Option<tasks::EscalationHolder> {
        recipient_after(self, limits, context, requester_role, after)
    }
}
