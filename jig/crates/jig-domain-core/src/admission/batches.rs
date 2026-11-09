//! A batch names only people held by the core before any task child changes
//! (jig/docs/design/domain/engine.md, 5.1).
use crate::Core;
use jig_domain_tasks as tasks;

impl Core {
    pub(crate) fn admit_batch(&self, creator: tasks::Party, batch: &mut [tasks::New]) -> Result<(), tasks::Problem> {
        if let tasks::Party::Person(person) = creator
            && self.people.identity(person).is_none()
        {
            return Err(tasks::Problem::new(batch.first().map(|row| row.number), tasks::Refusal::Unknown));
        }
        self.batch_people(batch)?;
        self.resolve_llm(batch)
    }

    fn batch_people(&self, batch: &[tasks::New]) -> Result<(), tasks::Problem> {
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
