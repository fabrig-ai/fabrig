//! Core-owned funding admission for person-origin goals
//! (domain/engine.md, section 4.4; domain/authority.md, section 9).

use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::ReplyTo;

use crate::Core;

#[derive(Debug)]
pub struct CreateStart {
    /// Person pool charged by a direct start or later accepted proposal.
    pub source: tasks::Funder,
    /// Authority bounded to this goal's requested budget.
    pub authority: authority::Authority,
}

impl Core {
    /// Ask the task child to open a missing period or person pool before a
    /// goal starts; both requests join the same decision as the goal.
    #[must_use]
    pub fn open_person_pool(&mut self, project: u32, person: u64) -> (Option<tasks::Event>, Option<tasks::Event>) {
        let period = tasks::Funder::Period { project, period: self.period(project) };
        let open_period = if self.tasks.funding(period).is_none() {
            Some(tasks::Event::OpenPeriod {
                reply_to: ReplyTo::new(self.internal(crate::routing::flight::Internal::Delegate)),
                project,
                period: self.period(project),
                budget: self.period_budget(project),
            })
        } else {
            None
        };
        let (pool, _) = self.person_pool(project, person);
        let carve_pool = if self.tasks.funding(pool).is_none() {
            Some(tasks::Event::CarvePool {
                reply_to: ReplyTo::new(self.internal(crate::routing::flight::Internal::Delegate)),
                project,
                person,
                period: self.period(project),
                budget: self.pool_budget(project, person),
            })
        } else {
            None
        };
        (open_period, carve_pool)
    }

    /// Read the person's current pool, including their role's unopened allotment.
    #[must_use]
    pub fn person_pool(&self, project: u32, person: u64) -> (tasks::Funder, tasks::Numbers) {
        let source = tasks::Funder::Pool { project, person, period: self.period(project) };
        let numbers = match self.tasks.funding(source) {
            Some(row) => row.numbers,
            None => tasks::Numbers { budget: self.pool_budget(project, person), spent: 0, spent_below: 0, reserved: 0 },
        };
        (source, numbers)
    }
}
