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
