//! Translate task-owned forge holdings without routing reports (domain/forge/branches.md, 1–2).
use alloc::boxed::Box;
use forge::routing::{branch_what, hub_name, include_repository, tree_branch, write_holding};
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge as forge;

pub const GOAL_BRANCH_SYMBOL: u16 = 1;
pub const CHANGE_CODE: u32 = 2;
pub const GOAL_CLOSER_CODE: u32 = 6;

#[derive(Debug)]
pub struct Holdings<'a> {
    pub forge: &'a forge::Domain,
    pub tasks: &'a tasks::Domain,
    pub connector: u16,
    pub forge_limits: &'a forge::Limits,
    pub task_limits: &'a tasks::Limits,
}

impl Holdings<'_> {
    #[expect(
        clippy::too_many_arguments,
        reason = "translate the same task and symbolic holdings for both making paths"
    )]
    pub fn prepare(
        &self,
        project: u32,
        root: u64,
        number: u64,
        executor: tasks::Executor,
        spec: &tasks::Spec,
        parent: Option<u64>,
        tracked: bool,
        symbols: &[u16],
        names: &mut List<forge::Name>,
    ) -> Result<Box<[tasks::Holding]>, Option<jig_domain_core::HoldingsRefusal>> {
        let mut holdings = List::with_capacity(self.task_limits.holdings);
        let mut reports = List::with_capacity(self.task_limits.holdings);
        for &symbol in symbols {
            if symbol != GOAL_BRANCH_SYMBOL || !tracked {
                return Err(Some(jig_domain_core::HoldingsRefusal::Symbol { symbol }));
            }
            let home = self.forge.home(project).ok_or(Some(jig_domain_core::HoldingsRefusal::Symbol { symbol }))?;
            let branch = tree_branch(home, root, b'g', number, None, self.forge_limits.name_bytes).ok_or(None)?;
            let name = forge::Name {
                forge: home.provider.forge,
                repository: home.provider.repository,
                what: branch_what(&branch, self.forge_limits.name_bytes).ok_or(None)?,
            };
            let holding = write_holding(self.connector, &name, self.task_limits).ok_or(None)?;
            holdings.push(holding).ok().ok_or(None)?;
            reports.push(name).ok().ok_or(None)?;
        }
        let implicit = self.implicit_task_holdings(project, root, number, executor, spec, parent, names)?;
        for holding in implicit {
            holdings.push(holding).ok().ok_or(None)?;
        }
        for name in reports.into_boxed() {
            names.push(name).expect("reports bounded by holdings");
        }
        Ok(holdings.into_boxed())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "task identity and specification arrive from separate admitted carriers"
    )]
    fn implicit_task_holdings(
        &self,
        project: u32,
        root: u64,
        number: u64,
        executor: tasks::Executor,
        spec: &tasks::Spec,
        parent: Option<u64>,
        names: &mut List<forge::Name>,
    ) -> Result<Box<[tasks::Holding]>, Option<jig_domain_core::HoldingsRefusal>> {
        let child_of_change = match parent {
            Some(task) => self.forge.change(task).is_some(),
            None => false,
        };
        let eligible = match executor {
            tasks::Executor::Person(_) | tasks::Executor::Job { .. } => false,
            tasks::Executor::Procedure { connector, code } => {
                connector == self.connector && (code == 1 || code == CHANGE_CODE)
            }
            tasks::Executor::Agent { .. } => !child_of_change,
        };
        if !eligible {
            return Ok(Box::new([]));
        }
        let mut selected = List::with_capacity(self.forge_limits.resources_per_task);
        if let tasks::Executor::Procedure { connector, code: 1 | CHANGE_CODE } = executor
            && connector == self.connector
        {
            let mut repository = None;
            let mut chosen = None;
            let mut base = None;
            let mut requester_branch = false;
            for parameter in &spec.parameters {
                match parameter {
                    tasks::Parameter::Resource { name: 1, connector, resource } if *connector == self.connector => {
                        repository = self.forge.repository_tag(project, *resource);
                    }
                    tasks::Parameter::Bytes { name: 2, value } => base = Some(value),
                    tasks::Parameter::Bytes { name: 3, value } => chosen = Some(value.clone()),
                    tasks::Parameter::Number { name: 11, value: 1 } => requester_branch = true,
                    tasks::Parameter::Number { .. }
                    | tasks::Parameter::Bytes { .. }
                    | tasks::Parameter::Resource { .. } => {}
                }
            }
            let repository = repository.ok_or(None)?;
            if let Some(base) = base {
                self.check_landing_holder(&repository, base, parent)?;
            }
            if requester_branch {
                return Ok(Box::new([]));
            }
            let branch = match chosen {
                Some(branch) => branch,
                None => tree_branch(&repository, root, b'c', number, None, self.forge_limits.name_bytes).ok_or(None)?,
            };
            let name = forge::Name {
                forge: repository.provider.forge,
                repository: repository.provider.repository,
                what: branch_what(&branch, self.forge_limits.name_bytes).ok_or(None)?,
            };
            let holding = write_holding(self.connector, &name, self.task_limits).ok_or(None)?;
            names.push(name).ok().ok_or(None)?;
            return Ok(Box::new([holding]));
        }
        if let Some(home) = self.forge.home(project) {
            include_repository(&mut selected, home).ok_or(None)?;
        }
        for parameter in &spec.parameters {
            if let tasks::Parameter::Resource { connector, resource, .. } = parameter
                && *connector == self.connector
            {
                let repository = self.forge.repository_tag(project, *resource).ok_or(None)?;
                include_repository(&mut selected, &repository).ok_or(None)?;
            }
        }
        let mut holdings = List::with_capacity(self.task_limits.holdings);
        let mut reports = List::with_capacity(self.task_limits.holdings);
        for repository in &selected {
            if repository.role == forge::Role::Context || !repository.kinds.push {
                continue;
            }
            let branch = tree_branch(repository, root, b'r', number, None, self.forge_limits.name_bytes).ok_or(None)?;
            let name = forge::Name {
                forge: repository.provider.forge,
                repository: repository.provider.repository,
                what: branch_what(&branch, self.forge_limits.name_bytes).ok_or(None)?,
            };
            let holding = write_holding(self.connector, &name, self.task_limits).ok_or(None)?;
            holdings.push(holding).ok().ok_or(None)?;
            reports.push(name).ok().ok_or(None)?;
        }
        for name in reports.into_boxed() {
            names.push(name).expect("reports bounded by holdings");
        }
        Ok(holdings.into_boxed())
    }

    /// A landing into a held branch belongs to its holder's requester subtree
    /// (temper/docs/design/domain/forge/branches.md, 1).
    fn check_landing_holder(
        &self,
        repository: &forge::Repository,
        base: &[u8],
        parent: Option<u64>,
    ) -> Result<(), Option<jig_domain_core::HoldingsRefusal>> {
        let name = forge::Name {
            forge: repository.provider.forge,
            repository: repository.provider.repository,
            what: branch_what(base, self.forge_limits.name_bytes).ok_or(None)?,
        };
        let resource = hub_name(self.connector, &name, self.task_limits).ok_or(None)?;
        if let Some(holder) = self.tasks.holder(&resource) {
            let inside = match parent {
                Some(requester) if requester == holder => true,
                Some(requester) => match self.tasks.task(requester) {
                    Some(task) => in_subtree(self.tasks, task, holder),
                    None => false,
                },
                None => false,
            };
            if !inside {
                return Err(Some(jig_domain_core::HoldingsRefusal::Held { holder }));
            }
        }
        Ok(())
    }
}

/// Whether `task` is in `goal`'s subtree below it: its requesters, task by
/// task, reach `goal` (temper/docs/design/domain/forge/topics.md, 1.2).
#[must_use]
pub fn in_subtree(tasks: &tasks::Domain, task: &tasks::TaskRecord, goal: u64) -> bool {
    let mut current = task;
    for _ in 0..task.depth {
        let tasks::Party::Task(number) = current.requester else { return false };
        if number == goal {
            return true;
        }
        let Some(parent) = tasks.task(number) else { return false };
        current = parent;
    }
    false
}
