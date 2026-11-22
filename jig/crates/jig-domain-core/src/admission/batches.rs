//! A batch names only people held by the core before any task child changes
//! (jig/docs/design/domain/engine/README.md, 5.1).
#![cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::result_large_err,
        reason = "batch admission keeps the task child's exact refusal without allocation"
    )
)]
use crate::Core;
use jig_domain_tasks as tasks;

impl Core {
    /// Read-only admission for an application Make, after its resource reports
    /// and without a party's held dependency loads (domain/authority.md, 6.3).
    pub fn check_make(&self, limits: &crate::Limits, make: &tasks::Event) -> Result<(), tasks::Refusal> {
        let tasks::Event::Make { creator, maker, batch, places, .. } = make else {
            return Err(tasks::Refusal::Batch);
        };
        let mut batch = batch.clone();
        if let Err(problem) = self.make_room(limits, *creator, &batch) {
            return Err(problem.why);
        }
        if let Err(problem) = self.admit_batch(*creator, &mut batch) {
            return Err(problem.why);
        }
        crate::effects::resources::check_make(self, &limits.tasks, *creator, &mut batch)?;
        match self.tasks.check_make(&limits.tasks, *creator, *maker, &batch, places) {
            Ok(()) => Ok(()),
            Err(problem) => Err(problem.why),
        }
    }

    /// Give a preflighted application Make an internal reply right rather than
    /// treating its task number as a pending continuation's token.
    #[must_use]
    pub fn bind_seeded_goal(&mut self, make: tasks::Event) -> tasks::Event {
        let tasks::Event::Make { creator: tasks::Party::Task(_), .. } = &make else {
            unreachable!("a seeded goal is a task's making");
        };
        let token = self.internal(crate::routing::flight::Internal::Admission);
        crate::routing::flight::bind(make, token)
    }

    pub(crate) fn make_room(
        &self,
        limits: &crate::Limits,
        creator: tasks::Party,
        batch: &[tasks::New],
    ) -> Result<(), tasks::Problem> {
        let mut tracked = 0_u32;
        for row in batch {
            if row.tracked.is_some() {
                tracked = tracked.saturating_add(1);
            }
        }
        if tracked > limits.tasks.tasks.saturating_sub(self.projections.len()) {
            let capacity =
                tasks::Capacity::batch(creator, batch, None, tasks::CapacityCause::Tasks { limit: limits.tasks.tasks });
            return Err(tasks::Problem::capacity(None, tasks::Refusal::Live, capacity));
        }
        Ok(())
    }

    pub(crate) fn admit_batch(&self, creator: tasks::Party, batch: &mut [tasks::New]) -> Result<(), tasks::Problem> {
        if let tasks::Party::Person(person) = creator
            && self.people.identity(person).is_none()
        {
            let task = match batch.first() {
                Some(row) => Some(row.number),
                None => None,
            };
            return Err(tasks::Problem::new(task, tasks::Refusal::Unknown));
        }
        self.batch_people(batch)?;
        self.resolve_llm(batch)
    }

    pub(crate) fn batch_people(&self, batch: &[tasks::New]) -> Result<(), tasks::Problem> {
        for row in batch {
            if let tasks::Executor::Person(tasks::PersonAddress::Person(person)) = row.executor
                && self.people.identity(person).is_none()
            {
                return Err(tasks::Problem::new(Some(row.number), tasks::Refusal::Unknown));
            }
            if let tasks::Funder::Pool { person, .. } = row.funder
                && self.people.identity(person).is_none()
            {
                return Err(tasks::Problem::new(Some(row.number), tasks::Refusal::Unknown));
            }
            if let Some(template) = &row.recurring {
                self.batch_people(&template.batch)?;
            }
        }
        Ok(())
    }
}
