//! Seeds for root-translated repair work and person tasks
//! (jig/docs/design/domain/engine/vocabulary.md, 2.5).

use alloc::boxed::Box;
pub use jig_domain_contract::effects::{EffectDescription, EffectForm, EffectRefusal, OutboxOutcome, Recovery};
pub use jig_domain_contract::lifecycle::{AdoptedParty, AdoptionRole, RestartStage};
pub use jig_domain_contract::ops::SeededGoal;
pub use jig_domain_contract::resources::{HoldKind, ResourceRole};
pub use jig_domain_contract::settings::{DefaultState, DroppedDefault, Judged};
use jig_domain_tasks as tasks;
use skein_lib::Token;

/// A person's task in no project (jig/docs/design/domain/authority.md, 6.1).
#[derive(Hash, Debug)]
pub struct PersonSeed {
    pub number: u64,
    pub funder: tasks::Funder,
    pub authority: tasks::Authority,
    pub opens: (Option<tasks::Event>, Option<tasks::Event>),
}

impl PersonSeed {
    /// The models its authority allows (jig/docs/design/domain/authority.md, 6.1).
    #[must_use]
    pub const fn models(&self) -> tasks::Models {
        self.authority.models
    }

    /// The allocated task number.
    #[must_use]
    pub const fn number(&self) -> u64 {
        self.number
    }
}

/// Core-allocated connector work, funded by its project, a person's pool or its requesting task
/// (jig/docs/design/domain/authority.md, 6–6.3).
#[derive(Hash, Debug)]
pub struct RepairSeed {
    pub number: u64,
    pub project: u32,
    pub funder: tasks::Funder,
    /// Period and pool opens, in that order, in the decision that makes the task.
    pub opens: (Option<tasks::Event>, Option<tasks::Event>),
    pub authority: tasks::Authority,
    pub budget: u64,
}

impl RepairSeed {
    /// The allocated task number for root-owned connector holding translation.
    #[must_use]
    pub const fn number(&self) -> u64 {
        self.number
    }

    /// Make an application-narrowed goal with its task's reservation, never cutting its budget
    /// to fit (jig/docs/design/domain/authority.md, 6.3).
    pub fn goal(
        self,
        task: u64,
        context: &tasks::DelegationContext,
        goal: SeededGoal,
    ) -> Result<tasks::Event, tasks::Refusal> {
        if context.tasks_left == 0 {
            return Err(tasks::Refusal::Tree);
        }
        if context.authority.delegation.depth == 0 {
            return Err(tasks::Refusal::Depth);
        }
        let mut authority = goal.authority;
        authority.delegation.depth = authority.delegation.depth.checked_sub(1).ok_or(tasks::Refusal::Depth)?;
        let available = context
            .numbers
            .budget
            .saturating_sub(context.numbers.spent)
            .saturating_sub(context.numbers.spent_below)
            .saturating_sub(context.numbers.reserved);
        if available < goal.budget {
            return Err(tasks::Refusal::Funding);
        }
        authority.budget.spend = goal.budget;
        Ok(tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(Token::new(self.number)),
            creator: tasks::Party::Task(task),
            batch: Box::new([tasks::New {
                charter_scope: tasks::CharterScope::Project,
                number: self.number,
                project: self.project,
                executor: tasks::Executor::Agent { charter: goal.charter },
                spec: goal.spec,
                contract: goal.contract,
                numbers: tasks::Numbers { budget: goal.budget, spent: 0, spent_below: 0, reserved: 0 },
                authority,
                funder: tasks::Funder::Task(task),
                deadline: tasks::Deadline::Named,
                label: goal.label,
                flags: goal.flags,
                llm: goal.llm,
                dependencies: Box::new([]),
                holdings: goal.holdings,
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: goal.tracked,
            }]),
        })
    }

    /// Root-made procedure delegate, checking its requester's room, depth and funding,
    /// without requiring an agent-selectable delegation kind (jig/docs/design/domain/tasks/lifecycle-endings.md, 3).
    pub fn delegate(
        self,
        task: u64,
        context: &tasks::DelegationContext,
        connector: u16,
        code: u32,
        spec: tasks::Spec,
        contract: tasks::Contract,
    ) -> Result<tasks::Event, tasks::Refusal> {
        if context.tasks_left == 0 {
            return Err(tasks::Refusal::Tree);
        }
        let mut authority = context.authority.clone();
        authority.delegation.depth = authority.delegation.depth.checked_sub(1).ok_or(tasks::Refusal::Depth)?;
        authority.budget.spend = self.budget.min(
            context
                .numbers
                .budget
                .saturating_sub(context.numbers.spent)
                .saturating_sub(context.numbers.spent_below)
                .saturating_sub(context.numbers.reserved),
        );
        if authority.budget.spend == 0 {
            return Err(tasks::Refusal::Funding);
        }
        Ok(tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(Token::new(self.number)),
            creator: tasks::Party::Task(task),
            batch: Box::new([tasks::New {
                charter_scope: tasks::CharterScope::Project,
                number: self.number,
                project: self.project,
                executor: tasks::Executor::Procedure { connector, code },
                spec,
                contract,
                numbers: tasks::Numbers { budget: authority.budget.spend, spent: 0, spent_below: 0, reserved: 0 },
                authority,
                funder: tasks::Funder::Task(task),
                deadline: tasks::Deadline::Named,
                label: 0,
                flags: tasks::Flags::PLAIN,
                llm: None,
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        })
    }
}
