//! Logical names and the tasks a world made for them (`temper/docs/design/testing/workloads.md`, 3.2).

use std::collections::BTreeMap;

#[derive(Default)]
pub struct Names {
    tasks: BTreeMap<String, u64>,
    logical: BTreeMap<u64, String>,
}

impl Names {
    #[must_use]
    pub fn new() -> Names {
        Self::default()
    }

    /// Refuses a second task for a name, or a second name for a task.
    pub fn name(&mut self, logical: &str, task: u64) -> Result<(), String> {
        if let Some(previous) = self.tasks.get(logical)
            && *previous != task
        {
            return Err(format!("name {logical}: already task {previous}, not {task}"));
        }
        if let Some(previous) = self.logical.get(&task)
            && previous != logical
        {
            return Err(format!("task {task}: already named {previous}, not {logical}"));
        }
        self.tasks.insert(logical.to_owned(), task);
        self.logical.insert(task, logical.to_owned());
        Ok(())
    }

    #[must_use]
    pub fn task(&self, logical: &str) -> Option<u64> {
        self.tasks.get(logical).copied()
    }

    #[must_use]
    pub fn logical(&self, task: u64) -> Option<&str> {
        self.logical.get(&task).map(String::as_str)
    }
}
